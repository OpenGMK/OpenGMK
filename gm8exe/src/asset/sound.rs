use crate::{
    asset::{assert_ver, Asset, Error, PascalString, ReadPascalString, WritePascalString},
    GameVersion,
};
use byteorder::{LE, ReadBytesExt, WriteBytesExt};
use std::io::{self, SeekFrom};
use crate::asset::ReadChunk;

pub const VERSION: u32 = 800;

pub struct Sound {
    /// The asset name present in GML and the editor.
    pub name: PascalString,

    /// The source file name, including the extension.
    pub source: PascalString,

    /// The file type (extension).
    pub extension: PascalString,

    /// The raw filedata.
    /// This is optional because the associated data can be blank
    /// since in the IDE when you create a new sound it has no associated data.
    pub data: Option<Box<[u8]>>,

    /// Stupid legacy garbage indicating what kind of sound it is.
    /// If it's SoundKind::Multimedia, it should be opened with your system's media player.
    pub kind: SoundKind,

    /// The output volume.
    /// Value is between 0.0 and 1.0, although the editor only allows as low as 0.3.
    /// This is meant to be applied at first load time and cannot be changed.
    pub volume: f64,

    /// Stereo Panning.
    /// Value is between -1.0 and +1.0 (-1 Left <- 0 -> Right +1)
    /// This is meant to be applied at first load time and cannot be changed.
    pub pan: f64,

    /// Indicates whether the "preload" option was checked for this sound.
    /// This would mean the samples would be decoded at load time,
    /// and you wouldn't be able to manipulate volume, pan, etc from GML.
    pub preload: bool,

    /// Various "effects" which can be checked in the sound editor.
    /// These are meant to be computed at load time and aren't dynamic.
    pub fx: SoundFX,
}

/// Various filters which can be set on any sound.
pub struct SoundFX {
    pub chorus: bool,
    pub echo: bool,
    pub flanger: bool,
    pub gargle: bool,
    pub reverb: bool,
}

#[derive(Copy, Clone, PartialEq)]
pub enum SoundKind {
    /// Normal Sound
    Normal = 0,
    /// Background music
    BackgroundMusic = 1,
    /// 3D Sound
    ThreeDimensional = 2,
    /// Use multimedia player
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

impl Asset for Sound {
    fn deserialize_exe(mut reader: impl io::Read + io::Seek, _version: GameVersion, strict: bool) -> Result<Self, Error> {
        let name = reader.read_pas_string()?;

        if strict {
            let version = reader.read_u32::<LE>()?;
            assert_ver(version, VERSION)?;
        } else {
            reader.seek(SeekFrom::Current(4))?;
        }

        let kind = SoundKind::from(reader.read_u32::<LE>()?);
        let extension = reader.read_pas_string()?;
        let source = reader.read_pas_string()?;

        let data = if reader.read_u32::<LE>()? != 0 {
            let len = reader.read_u32::<LE>()? as usize;
            Some(reader.read_chunk(len)?.into_boxed_slice())
        } else {
            None
        };

        let effects = reader.read_u32::<LE>()?;
        let chorus: bool = (effects & 0b1) != 0;
        let echo: bool = (effects & 0b10) != 0;
        let flanger: bool = (effects & 0b100) != 0;
        let gargle: bool = (effects & 0b1000) != 0;
        let reverb: bool = (effects & 0b10000) != 0;
        let fx = SoundFX { chorus, echo, flanger, gargle, reverb };

        let volume = reader.read_f64::<LE>()?;
        let pan = reader.read_f64::<LE>()?;
        let preload = reader.read_u32::<LE>()? != 0;

        Ok(Sound {
            name,
            source,
            extension,
            kind,
            data,
            volume,
            pan,
            preload,
            fx,
        })
    }

    fn serialize_exe(&self, mut writer: impl io::Write, _version: GameVersion) -> io::Result<()> {
        writer.write_pas_string(&self.name)?;
        writer.write_u32::<LE>(VERSION)?;
        writer.write_u32::<LE>(self.kind as u32)?;
        writer.write_pas_string(&self.extension)?;
        writer.write_pas_string(&self.source)?;
        if let Some(data) = &self.data {
            writer.write_u32::<LE>(true as u32)?;
            writer.write_u32::<LE>(data.len() as u32)?;
            writer.write_all(&data)?;
        } else {
            writer.write_u32::<LE>(0)?;
        }
        writer.write_u32::<LE>({
            let mut effects = self.fx.chorus as u32;
            effects |= (self.fx.echo as u32) << 1;
            effects |= (self.fx.flanger as u32) << 2;
            effects |= (self.fx.gargle as u32) << 3;
            effects |= (self.fx.reverb as u32) << 4;
            effects
        })?;
        writer.write_f64::<LE>(self.volume)?;
        writer.write_f64::<LE>(self.pan)?;
        writer.write_u32::<LE>(self.preload as u32)?;
        Ok(())
    }
}
