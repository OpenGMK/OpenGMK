use crate::bytes::{ReadBytes, ReadString, WriteBytes, WriteString};
use crate::game::parser::ParserOptions;
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

    /// Indicates whether the "preload" option was checked for this sound
    /// We will most likely ignore this and preload everything.
    pub preload: bool,

    /// Various "effects" which can be checked in the sound editor.
    pub chorus: bool,
    pub echo: bool,
    pub flanger: bool,
    pub gargle: bool,
    pub reverb: bool,
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

        let effects = (if self.chorus { 1 } else { 0 })
            + (if self.echo { 2 } else { 0 })
            + (if self.flanger { 4 } else { 0 })
            + (if self.gargle { 8 } else { 0 })
            + (if self.reverb { 16 } else { 0 });
        result += writer.write_u32_le(effects)?;
        result += writer.write_f64_le(self.volume)?;
        result += writer.write_f64_le(self.pan)?;
        result += writer.write_u32_le(if self.preload { 1 } else { 0 })?;

        Ok(result)
    }

    pub fn deserialize<B>(bin: B, options: &ParserOptions) -> io::Result<Sound>
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

        let effects = reader.read_u32_le()?;
        let chorus: bool = (effects & 1) != 0;
        let echo: bool = (effects & 2) != 0;
        let flanger: bool = (effects & 4) != 0;
        let gargle: bool = (effects & 8) != 0;
        let reverb: bool = (effects & 16) != 0;

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
            chorus,
            echo,
            flanger,
            gargle,
            reverb,
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
