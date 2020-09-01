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
    fn deserialize_exe(mut reader: impl io::Read + io::Seek, version: GameVersion, strict: bool) -> Result<Self, Error> {
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

    fn serialize_exe(&self, mut writer: impl io::Write, version: GameVersion) -> io::Result<()> {
        writer.write_pas_string(&self.name)?;
        writer.write_u32::<LE>(VERSION)?;
        writer.write_pas_string(&self.source)?;
        Ok(())
    }
}
