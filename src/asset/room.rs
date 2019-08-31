use crate::asset::{assert_ver, Asset, AssetDataError};
use crate::byteio::{ReadBytes, ReadString, WriteBytes, WriteString};
use std::io::{self, Seek, SeekFrom};

pub const VERSION: u32 = 541;

pub struct Room {
    /// The asset name present in GML and the editor.
    pub name: String,

    /// The default window title when this room is loaded.
    pub caption: String,

    /// The size of the room in pixels.
    pub width: u32,
    pub height: u32,

    /// The frames per second the room runs at.
    pub speed: u32,

    /// Whether the room contents will persist after loading a different room.
    pub persistent: bool,

    /// The background colour the room gets cleared to every frame before drawing.
    /// Unused if clear_screen is true.
    /// TODO: Colour type?
    pub bg_colour: u32,

    /// Whether to clear the screen inbetween frames.
    pub clear_screen: bool,

    /// The GML source executed when the room is created,
    pub creation_code: String,

    pub backgrounds: Vec<Background>,

    pub views_enabled: bool,

    pub views: Vec<View>,

    pub tiles: Vec<Tile>,
}

pub struct Background {
    pub visible_on_start: bool,

    /// If this is true then it's actually a foreground and not a background.
    /// Incredible design! Thank you Mark Overmars!
    pub is_foreground: bool,

    pub source_bg: i32, // TODO: Background asset ID type
    pub xoffset: i32,
    pub yoffset: i32,
    pub tile_horz: bool,
    pub tile_vert: bool,
    pub hspeed: i32,
    pub vspeed: i32,
    pub stretch: bool,
}

pub struct Instance {
    pub x: i32,
    pub y: i32,
    pub object: i32, // TODO: Object asset ID type
    pub id: i32,     // TODO: Instance ID type
    pub creation_code: String,
}

pub struct Tile {
    x: i32,
    y: i32,
    source_bg: i32, // TODO: Background asset ID type
    tile_x: bool,
    tile_y: bool,
    width: u32,
    height: u32,
    depth: i32,
    id: i32, // TODO: Instance ID type
}

pub struct View {
    pub visible: bool,
    pub source_x: i32,
    pub source_y: i32,
    pub source_w: u32,
    pub source_h: u32,
    pub port_x: i32,
    pub port_y: i32,
    pub port_w: u32,
    pub port_h: u32,
    pub following: ViewFollowData,
}

pub struct ViewFollowData {
    pub hborder: i32,
    pub vborder: i32,
    pub hspeed: i32,
    pub vspeed: i32,
    pub target: i32, // TODO: Instance ID type
}

impl Asset for Room {
    fn deserialize<B>(bytes: B, strict: bool, _version: u32) -> Result<Self, AssetDataError>
    where
        B: AsRef<[u8]>,
        Self: Sized
    {
        let mut reader = io::Cursor::new(bytes.as_ref());
        let name = reader.read_pas_string()?;

        if strict {
            let version = reader.read_u32_le()?;
            assert_ver(version, VERSION)?;
        } else {
            reader.seek(SeekFrom::Current(4))?;
        }

        let caption = reader.read_pas_string()?;
        let width = reader.read_u32_le()?;
        let height = reader.read_u32_le()?;
        let speed = reader.read_u32_le()?;
        let persistent = reader.read_u32_le()? != 0;
        let bg_colour = reader.read_u32_le()?;
        let clear_screen = reader.read_u32_le()? != 0;
        let creation_code = reader.read_pas_string()?;

        let background_count = reader.read_u32_le()? as usize;
        let mut backgrounds = Vec::with_capacity(background_count);
        for _ in 0..background_count {
            backgrounds.push(Background {
                visible_on_start: reader.read_u32_le()? != 0,
                is_foreground: reader.read_u32_le()? != 0,
                source_bg: reader.read_i32_le()?,
                xoffset: reader.read_i32_le()?,
                yoffset: reader.read_i32_le()?,
                tile_horz: reader.read_u32_le()? != 0,
                tile_vert: reader.read_u32_le()? != 0,
                hspeed: reader.read_i32_le()?,
                vspeed: reader.read_i32_le()?,
                stretch: reader.read_u32_le()? != 0,
            });
        }

        let views_enabled = reader.read_u32_le()? != 0;
        let view_count = reader.read_u32_le()? as usize;
        let mut views = Vec::with_capacity(view_count);
        for _ in 0..view_count {
            views.push(View {
                visible: reader.read_u32_le()? != 0,
                source_x: reader.read_i32_le()?,
                source_y: reader.read_i32_le()?,
                source_w: reader.read_u32_le()?,
                source_h: reader.read_u32_le()?,
                port_x: reader.read_i32_le()?,
                port_y: reader.read_i32_le()?,
                port_w: reader.read_u32_le()?,
                port_h: reader.read_u32_le()?,
                following: ViewFollowData {
                    hborder: reader.read_i32_le()?,
                    vborder: reader.read_i32_le()?,
                    hspeed: reader.read_i32_le()?,
                    vspeed: reader.read_i32_le()?,
                    target: reader.read_i32_le()?,
                },
            });
        }

        let instance_count = reader.read_u32_le()? as usize;
        let mut instances = Vec::with_capacity(instance_count);
        for _ in 0..instance_count {
            instances.push(self::Instance {
                x: reader.read_i32_le()?,
                y: reader.read_i32_le()?,
                object: reader.read_i32_le()?,
                id: reader.read_i32_le()?,
                creation_code: reader.read_pas_string()?,
            });
        }

        let tile_count = reader.read_u32_le()? as usize;
        let mut tiles = Vec::with_capacity(tile_count);
        for _ in 0..tile_count {
            tiles.push(self::Tile {
                x: reader.read_i32_le()?,
                y: reader.read_i32_le()?,
                source_bg: reader.read_i32_le()?,
                tile_x: reader.read_u32_le()? != 0,
                tile_y: reader.read_u32_le()? != 0,
                width: reader.read_u32_le()?,
                height: reader.read_u32_le()?,
                depth: reader.read_i32_le()?,
                id: reader.read_i32_le()?,
            });
        }

        Ok(Room {
            name,
            caption,
            width,
            height,
            speed,
            persistent,
            bg_colour,
            clear_screen,
            creation_code,
            backgrounds,
            views_enabled,
            views,
            tiles,
        })
    }

    fn serialize<W>(&self, writer: &mut W) -> io::Result<usize>
    where
        W: io::Write
    {
        panic!("unimplemented");
    }
}