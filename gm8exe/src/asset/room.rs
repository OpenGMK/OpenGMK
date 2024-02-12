use crate::{
    asset::{assert_ver_multiple, Asset, Error, PascalString, ReadPascalString, WritePascalString},
    colour::Colour,
    def::ID,
    GameVersion,
};
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use std::io::{self, Read};

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

    /// Whether to fill the drawing region with window colour.
    pub clear_region: bool,

    /// The GML source executed when the room is created.
    pub creation_code: PascalString,

    pub backgrounds: Vec<Background>,

    pub views_enabled: bool,

    pub views: Vec<View>,

    pub instances: Vec<Instance>,

    pub tiles: Vec<Tile>,

    pub uses_810_features: bool,
    pub uses_811_features: bool,
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
    pub xscale: f64,
    pub yscale: f64,
    pub blend: u32,
    pub angle: f64,
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
    pub xscale: f64,
    pub yscale: f64,
    pub blend: u32,
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
    fn deserialize_exe(mut reader: impl Read, version: GameVersion, strict: bool) -> Result<Self, Error> {
        let name = reader.read_pas_string()?;

        let entry_version = reader.read_u32::<LE>()?;
        if strict {
            assert_ver_multiple(entry_version, &[VERSION, 810, 811])?;
        }

        let uses_810_features = entry_version >= 810;
        let uses_811_features = entry_version >= 811;

        let caption = reader.read_pas_string()?;
        let width = reader.read_u32::<LE>()?;
        let height = reader.read_u32::<LE>()?;
        let speed = reader.read_u32::<LE>()?;
        let persistent = reader.read_u32::<LE>()? != 0;
        let bg_colour = reader.read_u32::<LE>()?.into();
        let (clear_screen, clear_region) = match (version, reader.read_u32::<LE>()?) {
            (GameVersion::GameMaker8_0, x) => (x != 0, true),
            (GameVersion::GameMaker8_1, x) => ((x & 0b01) != 0, (x & 0b10) == 0),
        };
        let creation_code = reader.read_pas_string()?;

        let background_count = reader.read_u32::<LE>()? as usize;
        let backgrounds = (0..background_count)
            .map(|_| {
                Ok(Background {
                    visible_on_start: reader.read_u32::<LE>()? != 0,
                    is_foreground: reader.read_u32::<LE>()? != 0,
                    source_bg: reader.read_i32::<LE>()?,
                    xoffset: reader.read_i32::<LE>()?,
                    yoffset: reader.read_i32::<LE>()?,
                    tile_horz: reader.read_u32::<LE>()? != 0,
                    tile_vert: reader.read_u32::<LE>()? != 0,
                    hspeed: reader.read_i32::<LE>()?,
                    vspeed: reader.read_i32::<LE>()?,
                    stretch: reader.read_u32::<LE>()? != 0,
                })
            })
            .collect::<io::Result<_>>()?;

        let views_enabled = reader.read_u32::<LE>()? != 0;
        let view_count = reader.read_u32::<LE>()? as usize;
        let views = (0..view_count)
            .map(|_| {
                Ok(View {
                    visible: reader.read_u32::<LE>()? != 0,
                    source_x: reader.read_i32::<LE>()?,
                    source_y: reader.read_i32::<LE>()?,
                    source_w: reader.read_u32::<LE>()?,
                    source_h: reader.read_u32::<LE>()?,
                    port_x: reader.read_i32::<LE>()?,
                    port_y: reader.read_i32::<LE>()?,
                    port_w: reader.read_u32::<LE>()?,
                    port_h: reader.read_u32::<LE>()?,
                    following: ViewFollowData {
                        hborder: reader.read_i32::<LE>()?,
                        vborder: reader.read_i32::<LE>()?,
                        hspeed: reader.read_i32::<LE>()?,
                        vspeed: reader.read_i32::<LE>()?,
                        target: reader.read_i32::<LE>()?,
                    },
                })
            })
            .collect::<io::Result<_>>()?;

        let instance_count = reader.read_u32::<LE>()? as usize;
        let instances = (0..instance_count)
            .map(|_| {
                Ok(self::Instance {
                    x: reader.read_i32::<LE>()?,
                    y: reader.read_i32::<LE>()?,
                    object: reader.read_i32::<LE>()?,
                    id: reader.read_i32::<LE>()?,
                    creation_code: reader.read_pas_string()?,
                    xscale: if uses_810_features { reader.read_f64::<LE>()? } else { 1.0 },
                    yscale: if uses_810_features { reader.read_f64::<LE>()? } else { 1.0 },
                    blend: if uses_810_features { reader.read_u32::<LE>()? } else { u32::MAX },
                    angle: if uses_811_features { reader.read_f64::<LE>()? } else { 0.0 },
                })
            })
            .collect::<io::Result<_>>()?;

        let tile_count = reader.read_u32::<LE>()? as usize;
        let tiles = (0..tile_count)
            .map(|_| {
                Ok(self::Tile {
                    x: reader.read_i32::<LE>()?,
                    y: reader.read_i32::<LE>()?,
                    source_bg: reader.read_i32::<LE>()?,
                    tile_x: reader.read_u32::<LE>()?,
                    tile_y: reader.read_u32::<LE>()?,
                    width: reader.read_u32::<LE>()?,
                    height: reader.read_u32::<LE>()?,
                    depth: reader.read_i32::<LE>()?,
                    id: reader.read_i32::<LE>()?,
                    xscale: if uses_810_features { reader.read_f64::<LE>()? } else { 1.0 },
                    yscale: if uses_810_features { reader.read_f64::<LE>()? } else { 1.0 },
                    blend: if uses_810_features { reader.read_u32::<LE>()? } else { u32::MAX },
                })
            })
            .collect::<io::Result<_>>()?;

        Ok(Room {
            name,
            caption,
            width,
            height,
            speed,
            persistent,
            bg_colour,
            clear_screen,
            clear_region,
            creation_code,
            backgrounds,
            views_enabled,
            views,
            instances,
            tiles,
            uses_810_features,
            uses_811_features,
        })
    }

    fn serialize_exe(&self, mut writer: impl io::Write, version: GameVersion) -> io::Result<()> {
        writer.write_pas_string(&self.name)?;
        writer.write_u32::<LE>(VERSION)?;
        writer.write_pas_string(&self.caption)?;
        writer.write_u32::<LE>(self.width)?;
        writer.write_u32::<LE>(self.height)?;
        writer.write_u32::<LE>(self.speed)?;
        writer.write_u32::<LE>(self.persistent.into())?;
        writer.write_u32::<LE>(self.bg_colour.into())?;
        match version {
            GameVersion::GameMaker8_0 => writer.write_u32::<LE>(self.clear_screen.into())?,
            GameVersion::GameMaker8_1 => {
                writer.write_u32::<LE>((u32::from(!self.clear_region) << 1) | u32::from(self.clear_screen))?
            },
        };
        writer.write_pas_string(&self.creation_code)?;
        writer.write_u32::<LE>(self.backgrounds.len() as u32)?;
        for background in &self.backgrounds {
            writer.write_u32::<LE>(background.visible_on_start.into())?;
            writer.write_u32::<LE>(background.is_foreground.into())?;
            writer.write_i32::<LE>(background.source_bg)?;
            writer.write_i32::<LE>(background.xoffset)?;
            writer.write_i32::<LE>(background.yoffset)?;
            writer.write_u32::<LE>(background.tile_horz.into())?;
            writer.write_u32::<LE>(background.tile_vert.into())?;
            writer.write_i32::<LE>(background.hspeed)?;
            writer.write_i32::<LE>(background.vspeed)?;
            writer.write_u32::<LE>(background.stretch.into())?;
        }
        writer.write_u32::<LE>(self.views_enabled.into())?;
        writer.write_u32::<LE>(self.views.len() as u32)?;
        for view in &self.views {
            writer.write_u32::<LE>(view.visible.into())?;
            writer.write_i32::<LE>(view.source_x)?;
            writer.write_i32::<LE>(view.source_y)?;
            writer.write_u32::<LE>(view.source_w)?;
            writer.write_u32::<LE>(view.source_h)?;
            writer.write_i32::<LE>(view.port_x)?;
            writer.write_i32::<LE>(view.port_y)?;
            writer.write_u32::<LE>(view.port_w)?;
            writer.write_u32::<LE>(view.port_h)?;
            writer.write_i32::<LE>(view.following.hborder)?;
            writer.write_i32::<LE>(view.following.vborder)?;
            writer.write_i32::<LE>(view.following.hspeed)?;
            writer.write_i32::<LE>(view.following.vspeed)?;
            writer.write_i32::<LE>(view.following.target)?;
        }
        writer.write_u32::<LE>(self.instances.len() as u32)?; // TODO: srsly grep for 'len as u32'
        for instance in &self.instances {
            writer.write_i32::<LE>(instance.x)?;
            writer.write_i32::<LE>(instance.y)?;
            writer.write_i32::<LE>(instance.object)?;
            writer.write_i32::<LE>(instance.id)?;
            writer.write_pas_string(&instance.creation_code)?;
        }
        writer.write_u32::<LE>(self.tiles.len() as u32)?;
        for tile in &self.tiles {
            writer.write_i32::<LE>(tile.x)?;
            writer.write_i32::<LE>(tile.y)?;
            writer.write_i32::<LE>(tile.source_bg)?;
            writer.write_u32::<LE>(tile.tile_x)?;
            writer.write_u32::<LE>(tile.tile_y)?;
            writer.write_u32::<LE>(tile.width)?;
            writer.write_u32::<LE>(tile.height)?;
            writer.write_i32::<LE>(tile.depth)?;
            writer.write_i32::<LE>(tile.id)?;
        }
        Ok(())
    }
}
