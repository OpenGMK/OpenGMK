use crate::{
    asset::{assert_ver, Asset, Error, PascalString, ReadChunk, ReadPascalString, WritePascalString},
    GameVersion,
};
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use std::io::{self, Read};

pub const VERSION1: u32 = 710;
pub const VERSION2: u32 = 800;

pub struct Background {
    /// The asset name present in GML and the editor.
    pub name: PascalString,

    /// The width of the background image in pixels.
    pub width: u32,

    /// The height of the background image in pixels.
    pub height: u32,

    /// The raw BGRA pixeldata.
    /// This is optional because the associated data can be blank
    /// since in the IDE when you create a new background it has no associated data.
    pub data: Option<Box<[u8]>>,
}

impl Asset for Background {
    fn deserialize_exe(mut reader: impl Read, _version: GameVersion, strict: bool) -> Result<Self, Error> {
        let name = reader.read_pas_string()?;

        let version1 = reader.read_u32::<LE>()?;
        let version2 = reader.read_u32::<LE>()?;
        if strict {
            assert_ver(version1, VERSION1)?;
            assert_ver(version2, VERSION2)?;
        }

        let width = reader.read_u32::<LE>()?;
        let height = reader.read_u32::<LE>()?;
        if width > 0 && height > 0 {
            let len = reader.read_u32::<LE>()? as usize;

            // sanity check
            if len != (width as usize * height as usize * 4) {
                return Err(Error::MalformedData)
            }

            let data = Some(reader.read_chunk(len)?.into_boxed_slice());
            Ok(Background { name, width, height, data })
        } else {
            Ok(Background { name, width: 0, height: 0, data: None })
        }
    }

    fn serialize_exe(&self, mut writer: impl io::Write, _version: GameVersion) -> io::Result<()> {
        writer.write_pas_string(&self.name)?;
        writer.write_u32::<LE>(VERSION1)?;
        writer.write_u32::<LE>(VERSION2)?;
        writer.write_u32::<LE>(self.width)?;
        writer.write_u32::<LE>(self.height)?;
        if let Some(pixeldata) = &self.data {
            writer.write_u32::<LE>(pixeldata.len() as u32)?; // TODO: safety. also grep for casts
            writer.write_all(&pixeldata)?;
        }
        Ok(())
    }
}
