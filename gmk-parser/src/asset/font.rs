use crate::asset::{Asset, ByteString, Timestamp, Version};

use byteorder::{LE, ReadBytesExt, WriteBytesExt};
use log::warn;
use std::{io, mem};

pub struct Font {
    pub name: ByteString,
    pub timestamp: Timestamp,
    pub version: Version,

    pub system_name: ByteString,
    pub size: u32,
    pub bold: bool,
    pub italic: bool,
    pub antialias_level: u8,
    pub charset: u8,
    pub range: (u32, u32),

    pub baked: Option<Box<BakedFont>>,
}

pub struct BakedFont {
    pub dmap: [u32; 0x600],
    pub map_size: (u32, u32),
    pub map: Vec<u8>,
}

impl Asset for Font {
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

impl Font {
    fn read(mut reader: &mut dyn io::Read, is_gmk: bool) -> io::Result<Self> {
        let name = ByteString::read(&mut reader)?;
        let timestamp = if is_gmk {
            Timestamp(reader.read_f64::<LE>()?)
        } else {
            Timestamp::default()
        };
        let version = read_version!(reader, name, is_gmk, "font", Gm800)?;

        let system_name = ByteString::read(&mut reader)?;
        let size = reader.read_u32::<LE>()?;
        let bold = reader.read_u32::<LE>()? != 0;
        let italic = reader.read_u32::<LE>()? != 0;
        let antialias_level = reader.read_u8()?;
        let charset = reader.read_u8()?;
        let range = (u32::from(reader.read_u16::<LE>()?), reader.read_u32::<LE>()?);

        let baked = if is_gmk {
            None
        } else {
            let mut dmap = mem::MaybeUninit::<[u32; 0x600]>::uninit();
            for val in unsafe { &mut *dmap.as_mut_ptr() } {
                *val = reader.read_u32::<LE>()?;
            }
            let dmap = unsafe { dmap.assume_init() };

            let map_size = (reader.read_u32::<LE>()?, reader.read_u32::<LE>()?);
            let len = reader.read_u32::<LE>()? as usize;
            let mut map = Vec::with_capacity(len);
            unsafe { map.set_len(len) };
            reader.read_exact(map.as_mut_slice())?;
            Some(Box::new(BakedFont { dmap, map_size, map }))
        };

        Ok(Self { name, timestamp, version, system_name, size, bold, italic, antialias_level, charset, range, baked })
    }

    fn write(&self, mut writer: &mut dyn io::Write, is_gmk: bool) -> io::Result<()> {
        assert_eq!(self.version, Version::Gm800);
        self.name.write(&mut writer)?;
        if is_gmk {
            writer.write_f64::<LE>(self.timestamp.0)?;
        }
        writer.write_u32::<LE>(self.version as u32)?;

        self.system_name.write(&mut writer)?;
        writer.write_u32::<LE>(self.size)?;
        writer.write_u32::<LE>(self.bold.into())?;
        writer.write_u32::<LE>(self.italic.into())?;
        writer.write_u8(self.antialias_level)?;
        writer.write_u8(self.charset)?;
        assert!(self.range.0 <= u32::from(u16::max_value()));
        writer.write_u16::<LE>(self.range.0 as u16)?;
        writer.write_u32::<LE>(self.range.1)?;

        if let Some(baked) = &self.baked {
            if is_gmk {
                warn!("Writing font \"{}\", baked font will be lost in EXE->GMK...", self.name);
            } else {
                for value in &baked.dmap {
                    writer.write_u32::<LE>(*value)?;
                }
                writer.write_u32::<LE>(baked.map_size.0)?;
                writer.write_u32::<LE>(baked.map_size.1)?;
                assert!(baked.map.len() <= u32::max_value() as usize);
                writer.write_u32::<LE>(baked.map.len() as u32)?;
                writer.write_all(baked.map.as_slice())?;
            }
        }
        Ok(())
    }
}
