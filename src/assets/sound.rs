#![allow(dead_code)] // Shut up.

use crate::bytes::{ReadBytes, ReadString, WriteBytes, WriteString};
use crate::types::Version;
use std::io::{self, Seek, SeekFrom};

pub const VERSION: Version = 800;

pub struct Sound {
    /// The asset name present in GML and the editor.
    pub name: String,

    /// The source file name, including the extension.
    pub source: String,

    /// The file extension.
    pub extension: String,

    /// The raw filedata.
    /// This is optional because the associated data can be blank.
    pub data: Option<Box<[u8]>>,

    /// Currently, this seems to be always 0. If it worked, it should be one of:
    pub kind: SoundKind,

    /// The output volume.
    /// Value is between 0.0 and 1.0, although the editor only allows as low as 0.3.
    pub volume: f64,

    /// Stereo Panning.
    /// Value is between -1.0 and +1.0 (-1 Left <- 0 -> Right +1)
    pub pan: f64,

    /// TODO: I have no idea what this does.
    /// Maybe it preemptively caches the audio samples.
    pub preload: bool,

    /// TODO: I also have no idea what this does.
    /// It might be garbage data.
    unused1: u32,
}

impl Sound {
    pub fn serialize<W>(&self, writer: &mut W) -> io::Result<usize>
    where
        W: io::Write,
    {
        let mut result = writer.write_pas_string(&self.name)?;
        result += writer.write_u32_le(VERSION as u32)?;
        result += writer.write_u32_le(self.kind as u32)?;
        result += writer.write_pas_string(&self.extension)?;
        result += writer.write_pas_string(&self.source)?;

        if let Some(data) = &self.data {
            result += writer.write_u32_le(1)?;
            result += writer.write_u32_le(data.len() as u32)?;
            result += writer.write(&data)?;
        } else {
            result += writer.write_u32_le(0)?;
        }

        result += writer.write_u32_le(self.unused1)?;
        result += writer.write_f64_le(self.volume)?;
        result += writer.write_f64_le(self.pan)?;
        result += writer.write_u32_le(if self.preload { 1 } else { 0 })?;

        Ok(result)
    }

    pub fn deserialize<B>(bin: B, strict: bool) -> io::Result<Sound>
    where
        B: AsRef<[u8]>,
    {
        let mut reader = io::Cursor::new(bin.as_ref());
        let name = reader.read_pas_string()?;

        if strict {
            let version = reader.read_u32_le()?;
            assert_eq!(version, VERSION);
        } else {
            reader.seek(SeekFrom::Current(4))?;
        }

        let kind = SoundKind::from(reader.read_u32_le()?);
        let extension = reader.read_pas_string()?;
        let source = reader.read_pas_string()?;

        let data = if reader.read_u32_le()? != 0 {
            let len = reader.read_u32_le()? as usize;
            let pos = reader.position() as usize;
            reader.seek(SeekFrom::Current(len as i64))?;
            Some(reader.get_ref()[pos..pos + len].to_vec().into_boxed_slice())
        } else {
            None
        };

        let unused1 = reader.read_u32_le()?;
        let volume = reader.read_f64_le()?;
        let pan = reader.read_f64_le()?;
        let preload = reader.read_u32_le()? != 0;

        Ok(Sound {
            name,
            source,
            extension,
            kind,
            data,
            volume,
            pan,
            preload,
            unused1,
        })
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum SoundKind {
    Normal = 0,
    BackgroundMusic = 1,
    ThreeDimensional = 2,
    Multimedia = 3,
}

impl From<u32> for SoundKind {
    fn from(n: u32) -> SoundKind {
        match n {
            0 => SoundKind::Normal,
            1 => SoundKind::BackgroundMusic,
            2 => SoundKind::ThreeDimensional,
            3 => SoundKind::Multimedia,

            _ => SoundKind::Normal,
        }
    }
}
