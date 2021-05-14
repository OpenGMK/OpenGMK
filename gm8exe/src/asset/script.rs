use crate::{
    asset::{assert_ver, Asset, Error, PascalString, ReadPascalString, WritePascalString},
    GameVersion,
};
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use std::io::{self, Read};

pub const VERSION: u32 = 800;

pub struct Script {
    /// The asset name present in GML and the editor.
    pub name: PascalString,

    /// The GML code in the script.
    pub source: PascalString,
}

impl Asset for Script {
    fn deserialize_exe(mut reader: impl Read, _version: GameVersion, strict: bool) -> Result<Self, Error> {
        let name = reader.read_pas_string()?;

        let version = reader.read_u32::<LE>()?;
        if strict {
            assert_ver(version, VERSION)?;
        }

        let source = reader.read_pas_string()?;
        Ok(Script { name, source })
    }

    fn serialize_exe(&self, mut writer: impl io::Write, _version: GameVersion) -> io::Result<()> {
        writer.write_pas_string(&self.name)?;
        writer.write_u32::<LE>(VERSION)?;
        writer.write_pas_string(&self.source)?;
        Ok(())
    }
}
