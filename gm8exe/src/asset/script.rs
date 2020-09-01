use crate::{
    asset::{assert_ver, Asset, Error, PascalString, ReadPascalString, WritePascalString},
    GameVersion,
};
use byteorder::{LE, ReadBytesExt, WriteBytesExt};
use std::io::{self, Seek, SeekFrom};

pub const VERSION: u32 = 800;

pub struct Script {
    /// The asset name present in GML and the editor.
    pub name: PascalString,

    /// The GML code in the script.
    pub source: PascalString,
}

impl Asset for Script {
    fn deserialize<B>(bytes: B, strict: bool, _version: GameVersion) -> Result<Self, Error>
    where
        B: AsRef<[u8]>,
        Self: Sized,
    {
        let mut reader = io::Cursor::new(bytes.as_ref());
        let name = reader.read_pas_string()?;

        if strict {
            let version = reader.read_u32::<LE>()?;
            assert_ver(version, VERSION)?;
        } else {
            reader.seek(SeekFrom::Current(4))?;
        }

        let source = reader.read_pas_string()?;
        Ok(Script { name, source })
    }

    fn serialize<W>(&self, writer: &mut W) -> io::Result<usize>
    where
        W: io::Write,
    {
        let mut result = writer.write_pas_string(&self.name)?;
        result += writer.write_u32::<LE>(VERSION as u32)?;
        result += writer.write_pas_string(&self.source)?;

        Ok(result)
    }
}
