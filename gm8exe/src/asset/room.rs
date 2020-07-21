use crate::{
    asset::{assert_ver, Asset, AssetDataError, PascalString, ReadPascalString, WritePascalString},
    colour::Colour,
    def::ID,
    GameVersion,
};

use minio::{ReadPrimitives, WritePrimitives};
use std::io::{self, Seek, SeekFrom};

pub const VERSION: u32 = 541;

pub struct Room {
    /// The asset name present in GML and the editor.
    pub name: PascalString,

    /// The default window title when this room is loaded.
    pub caption: PascalString,

    /// The width of the room in pixels.
    pub width: u32,

    /// The height of the room in pixels.
    pub height: u32,

    /// The frames per second the room runs at.
    pub speed: u32,

    /// Whether the room contents will persist after loading a different room.
    pub persistent: bool,

    /// The background colour the room gets cleared to every frame before drawing.
    /// Unused if clear_screen is true.
    pub bg_colour: Colour,

    /// Whether to clear the screen inbetween frames.
    pub clear_screen: bool,

    /// The GML source executed when the room is created,
    pub creation_code: PascalString,

    pub backgrounds: Vec<Background>,

    pub views_enabled: bool,

    pub views: Vec<View>,

    pub instances: Vec<Instance>,

    pub tiles: Vec<Tile>,
}

pub struct Background {
    pub visible_on_start: bool,

    /// If this is true then it's actually a foreground and not a background.
    /// Incredible design! Thank you Mark Overmars!
    pub is_foreground: bool,

    pub source_bg: ID,
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
    pub object: ID,
    pub id: ID,
    pub creation_code: PascalString,
}

pub struct Tile {
    pub x: i32,
    pub y: i32,
    pub source_bg: ID,
    pub tile_x: u32,
    pub tile_y: u32,
    pub width: u32,
    pub height: u32,
    pub depth: i32,
    pub id: ID,
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
    pub target: ID,
}

impl Asset for Room {
    fn deserialize<B>(bytes: B, strict: bool, _version: GameVersion) -> Result<Self, AssetDataError>
    where
        B: AsRef<[u8]>,
        Self: Sized,
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
        let bg_colour = reader.read_u32_le()?.into();
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
                tile_x: reader.read_u32_le()?,
                tile_y: reader.read_u32_le()?,
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
            instances,
            tiles,
        })
    }

    fn serialize<W>(&self, writer: &mut W) -> io::Result<usize>
    where
        W: io::Write,
    {
        let mut result = writer.write_pas_string(&self.name)?;
        result += writer.write_u32_le(VERSION)?;
        result += writer.write_pas_string(&self.caption)?;
        result += writer.write_u32_le(self.width)?;
        result += writer.write_u32_le(self.height)?;
        result += writer.write_u32_le(self.speed)?;
        result += writer.write_u32_le(self.persistent as u32)?;
        result += writer.write_u32_le(self.bg_colour.into())?;
        result += writer.write_u32_le(self.clear_screen as u32)?;
        result += writer.write_pas_string(&self.creation_code)?;

        result += writer.write_u32_le(self.backgrounds.len() as u32)?;
        for background in &self.backgrounds {
            result += writer.write_u32_le(background.visible_on_start as u32)?;
            result += writer.write_u32_le(background.is_foreground as u32)?;
            result += writer.write_i32_le(background.source_bg)?;
            result += writer.write_i32_le(background.xoffset)?;
            result += writer.write_i32_le(background.yoffset)?;
            result += writer.write_u32_le(background.tile_horz as u32)?;
            result += writer.write_u32_le(background.tile_vert as u32)?;
            result += writer.write_i32_le(background.hspeed)?;
            result += writer.write_i32_le(background.vspeed)?;
            result += writer.write_u32_le(background.stretch as u32)?;
        }

        result += writer.write_u32_le(self.views_enabled as u32)?;
        result += writer.write_u32_le(self.views.len() as u32)?;
        for view in &self.views {
            result += writer.write_u32_le(view.visible as u32)?;
            result += writer.write_i32_le(view.source_x)?;
            result += writer.write_i32_le(view.source_y)?;
            result += writer.write_u32_le(view.source_w)?;
            result += writer.write_u32_le(view.source_h)?;
            result += writer.write_i32_le(view.port_x)?;
            result += writer.write_i32_le(view.port_y)?;
            result += writer.write_u32_le(view.port_w)?;
            result += writer.write_u32_le(view.port_h)?;
            result += writer.write_i32_le(view.following.hborder)?;
            result += writer.write_i32_le(view.following.vborder)?;
            result += writer.write_i32_le(view.following.hspeed)?;
            result += writer.write_i32_le(view.following.vspeed)?;
            result += writer.write_i32_le(view.following.target)?;
        }

        result += writer.write_u32_le(self.instances.len() as u32)?;
        for instance in &self.instances {
            result += writer.write_i32_le(instance.x)?;
            result += writer.write_i32_le(instance.y)?;
            result += writer.write_i32_le(instance.object)?;
            result += writer.write_i32_le(instance.id)?;
            result += writer.write_pas_string(&instance.creation_code)?;
        }

        result += writer.write_u32_le(self.tiles.len() as u32)?;
        for tile in &self.tiles {
            result += writer.write_i32_le(tile.x)?;
            result += writer.write_i32_le(tile.y)?;
            result += writer.write_i32_le(tile.source_bg)?;
            result += writer.write_u32_le(tile.tile_x)?;
            result += writer.write_u32_le(tile.tile_y)?;
            result += writer.write_u32_le(tile.width)?;
            result += writer.write_u32_le(tile.height)?;
            result += writer.write_i32_le(tile.depth)?;
            result += writer.write_i32_le(tile.id)?;
        }

        Ok(result)
    }
}
