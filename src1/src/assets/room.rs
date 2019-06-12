#![allow(dead_code)] // Shut up.

use crate::bytes::{ReadBytes, ReadString};
use crate::game::parser::ParserOptions;
use crate::types::{Color, Dimensions, Point, Rectangle, Version};
use std::io::{self, Seek, SeekFrom};

pub const VERSION: Version = 541;

pub struct Background {
    pub visible_on_start: bool,

    /// If this is true then it's actually a foreground and not a background.
    /// Incredible design! Thank you Mark Overmars!
    pub is_foreground: bool,

    pub source_bg: i32, // we need an ID type. TODO btw
    pub offset: Point,
    pub tile_horz: bool,
    pub tile_vert: bool,
    pub hspeed: i32,
    pub vspeed: i32,
    pub stretch: bool,
}

pub struct Instance {
    pub position: Point,
    pub object: i32, // TODO tyhing
    pub id: i32,     // TODO ^   ^ ^ ^
    pub creation_code: String,
}

pub struct Room {
    /// The asset name present in GML and the editor.
    pub name: String,

    /// The default window title when this room is loaded.
    pub caption: String,

    /// The size of the room in pixels.
    pub size: Dimensions,

    /// The frames per second the room runs at.
    pub speed: u32,

    /// Whether the room contents will persist after loading a different room.
    pub persistent: bool,

    /// The background colour the room gets cleared to every frame before drawing.
    /// Unused if clear_screen is true.
    pub bg_color: Color,

    /// Whether to clear the screen inbetween frames.
    pub clear_screen: bool,

    /// The GML source executed when the room is created,
    /// for more see [event order doc name here]. <- TODO
    pub creation_code: String,

    pub backgrounds: Vec<Background>,

    pub views_enabled: bool,

    pub views: Vec<View>,

    pub tiles: Vec<Tile>,
}

pub struct Tile {
    position: Point,
    source_bg: i32, // TODO!!! :miyanoCheer:
    tile_x: bool,
    tile_y: bool,
    size: Dimensions,
    depth: i32,
    id: i32, // TODO!!!
}

pub struct View {
    pub visible: bool,
    pub source: Rectangle,
    pub port: Rectangle,
    pub following: ViewFollowData,
}

pub struct ViewFollowData {
    pub hborder: i32,
    pub vborder: i32,
    pub hspeed: i32,
    pub vspeed: i32,
    pub target: i32, // TODO: id type thign
}

impl Room {
    pub fn serialize<W>(&self, _writer: &mut W) -> io::Result<usize>
    where
        W: io::Write,
    {
        panic!("unimplemented");
    }

    pub fn deserialize<B>(bin: B, options: &ParserOptions) -> io::Result<Room>
    where
        B: AsRef<[u8]>,
    {
        let mut reader = io::Cursor::new(bin.as_ref());
        let name = reader.read_pas_string()?;

        if options.strict {
            let version = reader.read_u32_le()?;
            assert_eq!(version, VERSION);
        } else {
            reader.seek(SeekFrom::Current(4))?;
        }

        let caption = reader.read_pas_string()?;
        let width = reader.read_u32_le()?;
        let height = reader.read_u32_le()?;
        let speed = reader.read_u32_le()?;
        let persistent = reader.read_u32_le()? != 0;
        let bg_color: Color = reader.read_u32_le()?.into();
        let clear_screen = reader.read_u32_le()? != 0;
        let creation_code = reader.read_pas_string()?;

        let background_count = reader.read_u32_le()? as usize;
        let mut backgrounds = Vec::with_capacity(background_count);
        for _ in 0..background_count {
            backgrounds.push(Background {
                visible_on_start: reader.read_u32_le()? != 0,
                is_foreground: reader.read_u32_le()? != 0,
                source_bg: reader.read_i32_le()?,
                offset: Point {
                    x: reader.read_i32_le()?,
                    y: reader.read_i32_le()?,
                },
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
                source: Rectangle {
                    x: reader.read_i32_le()?,
                    y: reader.read_i32_le()?,
                    width: reader.read_u32_le()?,
                    height: reader.read_u32_le()?,
                },
                port: Rectangle {
                    x: reader.read_i32_le()?,
                    y: reader.read_i32_le()?,
                    width: reader.read_u32_le()?,
                    height: reader.read_u32_le()?,
                },
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
                position: Point {
                    x: reader.read_i32_le()?,
                    y: reader.read_i32_le()?,
                },
                object: reader.read_i32_le()?,
                id: reader.read_i32_le()?,
                creation_code: reader.read_pas_string()?,
            });
        }

        let tile_count = reader.read_u32_le()? as usize;
        let mut tiles = Vec::with_capacity(instance_count);
        for _ in 0..tile_count {
            tiles.push(self::Tile {
                position: Point {
                    x: reader.read_i32_le()?,
                    y: reader.read_i32_le()?,
                },
                source_bg: reader.read_i32_le()?,
                tile_x: reader.read_u32_le()? != 0,
                tile_y: reader.read_u32_le()? != 0,
                size: Dimensions {
                    width: reader.read_u32_le()?,
                    height: reader.read_u32_le()?,
                },
                depth: reader.read_i32_le()?,
                id: reader.read_i32_le()?,
            });
        }

        Ok(Room {
            name,
            caption,
            size: Dimensions { width, height },
            speed,
            persistent,
            bg_color,
            clear_screen,
            creation_code,
            backgrounds,
            views_enabled,
            views,
            tiles,
        })
    }
}
