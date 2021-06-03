use crate::asset::{Asset, ByteString, Timestamp, Version};

use byteorder::{LE, ReadBytesExt, WriteBytesExt};
use std::io;

pub struct Sound {
    pub name: ByteString,
    pub timestamp: Timestamp,
    pub version: Version,

    pub kind: Kind,
    pub extension: ByteString,
    pub source: ByteString,
    pub data: Option<Vec<u8>>,
    pub fx: SoundEffects,
    pub volume: f64,
    pub pan: f64,
    pub preload: bool,
}

pub struct SoundEffects {
    pub chorus: bool,
    pub echo: bool,
    pub flanger: bool,
    pub gargle: bool,
    pub reverb: bool,
}

#[derive(Copy, Clone)]
#[repr(u32)]
pub enum Kind {
    Normal = 0,
    BackgroundMusic = 1,
    ThreeDimensional = 2,
    Multimedia = 3,
}

impl From<u32> for Kind {
    fn from(n: u32) -> Self {
        match n {
            0 => Self::Normal,
            1 => Self::BackgroundMusic,
            2 => Self::ThreeDimensional,
            3 => Self::Multimedia,
            _ => Self::Normal,
        }
    }
}

impl Asset for Sound {
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

impl Sound {
    fn read(mut reader: &mut dyn io::Read, is_gmk: bool) -> io::Result<Self> {
        let name = ByteString::read(&mut reader)?;
        let timestamp = if is_gmk {
            Timestamp(reader.read_f64::<LE>()?)
        } else {
            Timestamp::default()
        };
        let version = read_version!(reader, name, is_gmk, "sound", Gm800)?;

        let kind = Kind::from(reader.read_u32::<LE>()?);
        let extension = ByteString::read(&mut reader)?;
        let source = ByteString::read(&mut reader)?;
        let data = if reader.read_u32::<LE>()? != 0 {
            let data_len = reader.read_u32::<LE>()? as usize;
            let mut data = Vec::with_capacity(data_len);
            unsafe { data.set_len(data_len) };
            reader.read_exact(data.as_mut_slice())?;
            Some(data)
        } else {
            None
        };
        let fx = SoundEffects::read(&mut reader)?;
        let volume = reader.read_f64::<LE>()?;
        let pan = reader.read_f64::<LE>()?;
        let preload = reader.read_u32::<LE>()? != 0;
        Ok(Self { name, timestamp, version, kind, extension, source, data, fx, volume, pan, preload })
    }

    fn write(&self, mut writer: &mut dyn io::Write, is_gmk: bool) -> io::Result<()> {
        assert_eq!(self.version, Version::Gm800);
        self.name.write(&mut writer)?;
        if is_gmk {
            writer.write_f64::<LE>(self.timestamp.0)?;
        }
        writer.write_u32::<LE>(self.version as u32)?;

        writer.write_u32::<LE>(self.kind as u32)?;
        self.extension.write(&mut writer)?;
        self.source.write(&mut writer)?;
        writer.write_u32::<LE>(self.data.is_some() as u32)?;
        if let Some(data) = &self.data {
            assert!(data.len() <= u32::max_value() as usize);
            writer.write_u32::<LE>(data.len() as u32)?;
            writer.write_all(data.as_slice())?;
        }
        self.fx.write(&mut writer)?;
        writer.write_f64::<LE>(self.volume)?;
        writer.write_f64::<LE>(self.pan)?;
        writer.write_u32::<LE>(self.preload.into())
    }
}

impl SoundEffects {
    fn read(reader: &mut dyn io::Read) -> io::Result<Self> {
        let effects = reader.read_u32::<LE>()?;
        Ok(Self {
            chorus: (effects & 1) != 0,
            echo: (effects & (1 << 1)) != 0,
            flanger: (effects & (1 << 2)) != 0,
            gargle: (effects & (1 << 3)) != 0,
            reverb: (effects & (1 << 4)) != 0,
        })
    }

    fn write(&self, writer: &mut dyn io::Write) -> io::Result<()> {
        writer.write_u32::<LE>(
            0 |
                u32::from(self.chorus) |
                u32::from(self.echo) << 1 |
                u32::from(self.flanger) << 2 |
                u32::from(self.gargle) << 3 |
                u32::from(self.reverb) << 4
        )
    }
}
