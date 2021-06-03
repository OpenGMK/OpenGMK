use crate::asset::{Asset, ByteString, Timestamp, Version};

use byteorder::{LE, ReadBytesExt, WriteBytesExt};
use std::io;

pub struct Room {
    pub name: ByteString,
    pub timestamp: Timestamp,
    pub version: Version,


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
    }

    fn write(&self, mut writer: &mut dyn io::Write, is_gmk: bool) -> io::Result<()> {
        assert_eq!(self.version, Version::Gm541);
        self.name.write(&mut writer)?;
        if is_gmk {
            writer.write_f64::<LE>(self.timestamp.0)?;
        }
        writer.write_u32::<LE>(self.version as u32)?;
    }
}
