use crate::{
    asset::{assert_ver, Asset, AssetDataError, PascalString, ReadPascalString, WritePascalString},
    GameVersion,
};

use minio::{ReadPrimitives, WritePrimitives};
use std::io::{self, Seek, SeekFrom};

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
    fn deserialize<B>(bytes: B, strict: bool, _version: GameVersion) -> Result<Self, AssetDataError>
    where
        B: AsRef<[u8]>,
        Self: Sized,
    {
        let mut reader = io::Cursor::new(bytes.as_ref());
        let name = reader.read_pas_string()?;

        if strict {
            let version1 = reader.read_u32_le()?;
            let version2 = reader.read_u32_le()?;
            assert_ver(version1, VERSION1)?;
            assert_ver(version2, VERSION2)?;
        } else {
            reader.seek(SeekFrom::Current(8))?;
        }

        let width = reader.read_u32_le()?;
        let height = reader.read_u32_le()?;
        if width > 0 && height > 0 {
            let data_len = reader.read_u32_le()?;

            // sanity check
            if data_len != (width * height * 4) {
                return Err(AssetDataError::MalformedData);
            }

            let pos = reader.position() as usize;
            let len = data_len as usize;
            let buf = match reader.into_inner().get(pos..pos + len) {
                Some(b) => b.to_vec(),
                None => return Err(AssetDataError::MalformedData),
            };

            Ok(Background { name, width, height, data: Some(buf.into_boxed_slice()) })
        } else {
            Ok(Background { name, width: 0, height: 0, data: None })
        }
    }

    fn serialize<W>(&self, writer: &mut W) -> io::Result<usize>
    where
        W: io::Write,
    {
        let mut result = writer.write_pas_string(&self.name)?;
        result += writer.write_u32_le(VERSION1 as u32)?;
        result += writer.write_u32_le(VERSION2 as u32)?;
        result += writer.write_u32_le(self.width as u32)?;
        result += writer.write_u32_le(self.height as u32)?;
        if let Some(pixeldata) = &self.data {
            result += writer.write_u32_le(pixeldata.len() as u32)?;
            result += writer.write_all(&pixeldata).map(|()| pixeldata.len())?;
        }
        Ok(result)
    }
}
