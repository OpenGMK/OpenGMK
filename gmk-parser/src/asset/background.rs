use crate::asset::{frame::Frame, Asset, ByteString, Timestamp, Version};

use byteorder::{LE, ReadBytesExt, WriteBytesExt};
use std::io;

pub struct Background {
    pub name: ByteString,
    pub timestamp: Timestamp,
    pub version: Version,

    pub tile_set_info: TileSetInfo,
    pub frame: Frame,
}

pub struct TileSetInfo {
    pub is_tile_set: bool,
    pub size: (u32, u32),
    pub offset: (u32, u32),
    pub separation: (u32, u32),
}

impl Asset for Background {
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

    fn from_gmk<R: io::Read>(mut reader: R) -> io::Result<Self> {
        Self::read(&mut reader, true)
    }

    fn to_gmk<W: io::Write>(&self, mut writer: W) -> io::Result<()> {
        self.write(&mut writer, true)
    }

    fn from_exe<R: io::Read>(mut reader: R) -> io::Result<Self> {
        Self::read(&mut reader, false)
    }

    fn to_exe<W: io::Write>(&self, mut writer: W) -> io::Result<()> {
        self.write(&mut writer, false)
    }
}

impl Background {
    fn read(mut reader: &mut dyn io::Read, is_gmk: bool) -> io::Result<Self> {
        let name = ByteString::read(&mut reader)?;
        let timestamp = if is_gmk {
            Timestamp(reader.read_f64::<LE>()?)
        } else {
            Timestamp::default()
        };
        let version = read_version!(reader, name, is_gmk, "background", Gm710)?;

        let tile_set_info = if is_gmk {
            TileSetInfo::read(&mut reader)?
        } else {
            TileSetInfo::default()
        };
        let frame = Frame::read_for(&mut reader, is_gmk, &name, "frame in background")?;
        Ok(Self { name, timestamp, version, tile_set_info, frame })
    }

    fn write(&self, mut writer: &mut dyn io::Write, is_gmk: bool) -> io::Result<()> {
        assert_eq!(self.version, Version::Gm710);
        self.name.write(&mut writer)?;
        if is_gmk {
            writer.write_f64::<LE>(self.timestamp.0)?;
        }
        writer.write_u32::<LE>(self.version as u32)?;

        if is_gmk {
            self.tile_set_info.write(&mut writer)?;
        }
        self.frame.write(&mut writer)
    }
}

impl TileSetInfo {
    fn read(reader: &mut dyn io::Read) -> io::Result<Self> {
        let is_tile_set = reader.read_u32::<LE>()? != 0;
        let size = (reader.read_u32::<LE>()?, reader.read_u32::<LE>()?);
        let offset = (reader.read_u32::<LE>()?, reader.read_u32::<LE>()?);
        let separation = (reader.read_u32::<LE>()?, reader.read_u32::<LE>()?);
        Ok(Self { is_tile_set, size, offset, separation })
    }

    fn write(&self, writer: &mut dyn io::Write) -> io::Result<()> {
        writer.write_u32::<LE>(self.is_tile_set as u32)?;
        writer.write_u32::<LE>(self.size.0)?;
        writer.write_u32::<LE>(self.size.1)?;
        writer.write_u32::<LE>(self.offset.0)?;
        writer.write_u32::<LE>(self.offset.1)?;
        writer.write_u32::<LE>(self.separation.0)?;
        writer.write_u32::<LE>(self.separation.1)?;
        Ok(())
    }
}

impl Default for TileSetInfo {
    fn default() -> Self {
        Self {
            is_tile_set: false,
            size: (16, 16),
            offset: (0, 0),
            separation: (0, 0),
        }
    }
}
