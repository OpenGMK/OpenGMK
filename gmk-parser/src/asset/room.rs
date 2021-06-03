use crate::asset::{Asset, ByteString, Timestamp, Version};

use byteorder::{LE, ReadBytesExt, WriteBytesExt};
use std::io;

pub struct Room {
    pub name: ByteString,
    pub timestamp: Timestamp,
    pub version: Version,

    pub caption: ByteString,
    pub width: u32,
    pub height: u32,
    pub speed: u32,
    pub persistent: bool,
    pub bg_colour: u32,
    pub clear_screen: u32,
    pub creation_code: ByteString,
    pub backgrounds: Vec<Background>,
    pub views_enabled: bool,
    pub views: Vec<View>,
    pub instances: Vec<Instance>,
    pub tiles: Vec<Tile>,
}

pub struct Background {
    pub visible_on_start: bool,
    pub is_foreground: bool,
    pub source_bg: i32,
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
    pub object: i32,
    pub id: i32,
    pub creation_code: ByteString,
}

pub struct Tile {
    pub x: i32,
    pub y: i32,
    pub source_bg: i32,
    pub tile_x: u32,
    pub tile_y: u32,
    pub width: u32,
    pub height: u32,
    pub depth: i32,
    pub id: i32,
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
    pub target: i32,
}

impl Asset for Room {
    #[inline]
    fn name(&self) -> &[u8] {
        self.name.0.as_slice()
    }

    #[inline]
    fn timestamp(&self) -> Timestamp {
        self.timestamp
    }

    #[inline]
    fn version(&self) -> Version {
        self.version
    }

    fn from_gmk<R: io::Read>(&self, mut reader: R) -> io::Result<Self> {
        Self::read(&mut reader, true)
    }

    fn to_gmk<W: io::Write>(&self, mut writer: W) -> io::Result<()> {
        self.write(&mut writer, true)
    }

    fn from_exe<R: io::Read>(&self, mut reader: R) -> io::Result<Self> {
        Self::read(&mut reader, false)
    }

    fn to_exe<W: io::Write>(&self, mut writer: W) -> io::Result<()> {
        self.write(&mut writer, false)
    }
}

impl Room {
    fn read(mut reader: &mut dyn io::Read, is_gmk: bool) -> io::Result<Self> {
        let name = ByteString::read(&mut reader)?;
        let timestamp = if is_gmk {
            Timestamp(reader.read_f64::<LE>()?)
        } else {
            Timestamp::default()
        };
        let version = read_version!(reader, name, is_gmk, "room", Gm541)?;

        let caption = ByteString::read(&mut reader)?;
        let width = reader.read_u32::<LE>()?;
        let height = reader.read_u32::<LE>()?;
        let speed = reader.read_u32::<LE>()?;
        let persistent = reader.read_u32::<LE>()? != 0;
        let bg_colour = reader.read_u32::<LE>()?.into();
        let clear_screen = reader.read_u32::<LE>()?;
        let creation_code = ByteString::read(&mut reader)?;

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
                    creation_code: ByteString::read(&mut reader)?,
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
                })
            })
            .collect::<io::Result<_>>()?;

        Ok(Self {name, timestamp, version, caption, width, height, speed, persistent, bg_colour,
            clear_screen, creation_code, backgrounds, views_enabled, views, instances, tiles})
    }

    fn write(&self, mut writer: &mut dyn io::Write, is_gmk: bool) -> io::Result<()> {
        use std::convert::TryFrom;

        assert_eq!(self.version, Version::Gm541);
        self.name.write(&mut writer)?;
        if is_gmk {
            writer.write_f64::<LE>(self.timestamp.0)?;
        }
        writer.write_u32::<LE>(self.version as u32)?;
        self.caption.write(&mut writer)?;
        writer.write_u32::<LE>(self.width)?;
        writer.write_u32::<LE>(self.height)?;
        writer.write_u32::<LE>(self.speed)?;
        writer.write_u32::<LE>(self.persistent.into())?;
        writer.write_u32::<LE>(self.bg_colour.into())?;
        writer.write_u32::<LE>(self.clear_screen.into())?;
        self.creation_code.write(&mut writer)?;

        let len = i32::try_from(self.backgrounds.len()).map_err(|_| io::Error::from(io::ErrorKind::InvalidData))?;
        writer.write_i32::<LE>(len)?;
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
        let len = i32::try_from(self.views.len()).map_err(|_| io::Error::from(io::ErrorKind::InvalidData))?;
        writer.write_i32::<LE>(len)?;
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
        let len = i32::try_from(self.instances.len()).map_err(|_| io::Error::from(io::ErrorKind::InvalidData))?;
        writer.write_i32::<LE>(len)?;
        for instance in &self.instances {
            writer.write_i32::<LE>(instance.x)?;
            writer.write_i32::<LE>(instance.y)?;
            writer.write_i32::<LE>(instance.object)?;
            writer.write_i32::<LE>(instance.id)?;
            instance.creation_code.write(&mut writer)?;
        }
        let len = i32::try_from(self.tiles.len()).map_err(|_| io::Error::from(io::ErrorKind::InvalidData))?;
        writer.write_i32::<LE>(len)?;
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
