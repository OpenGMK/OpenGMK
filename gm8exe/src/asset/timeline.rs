use crate::{
    asset::{assert_ver, CodeAction, Asset, Error, PascalString, ReadPascalString, WritePascalString},
    GameVersion,
};
use byteorder::{LE, ReadBytesExt, WriteBytesExt};
use std::io::{self, Seek, SeekFrom};

pub const VERSION: u32 = 500;
pub const VERSION_MOMENT: u32 = 400;

pub struct Timeline {
    /// The asset name present in GML and the editor.
    pub name: PascalString,

    /// The list of moments in the timeline with their associated [THING].
    pub moments: Vec<(u32, Vec<CodeAction>)>,
}

impl Asset for Timeline {
    fn deserialize_exe(mut reader: impl io::Read + io::Seek, version: GameVersion, strict: bool) -> Result<Self, Error> {
        let name = reader.read_pas_string()?;

        if strict {
            let version = reader.read_u32::<LE>()?;
            assert_ver(version, VERSION)?;
        } else {
            reader.seek(SeekFrom::Current(4))?;
        }

        let moment_count = reader.read_u32::<LE>()? as usize;
        let mut moments = Vec::with_capacity(moment_count);
        for _ in 0..moment_count {
            let moment_index = reader.read_u32::<LE>()?;

            if strict {
                let version = reader.read_u32::<LE>()?;
                assert_ver(version, VERSION_MOMENT)?;
            } else {
                reader.seek(SeekFrom::Current(4))?;
            }

            let action_count = reader.read_u32::<LE>()? as usize;

            let mut actions = Vec::with_capacity(action_count);
            for _ in 0..action_count {
                actions.push(CodeAction::deserialize_exe(&mut reader, version, strict)?);
            }

            moments.push((moment_index, actions));
        }

        Ok(Timeline { name, moments })
    }

    fn serialize_exe(&self, mut writer: impl io::Write, version: GameVersion) -> io::Result<()> {
        writer.write_pas_string(&self.name)?;
        writer.write_u32::<LE>(VERSION)?;
        writer.write_u32::<LE>(self.moments.len() as u32)?;
        for (moment_index, actions) in &self.moments {
            writer.write_u32::<LE>(*moment_index)?;
            writer.write_u32::<LE>(VERSION_MOMENT as u32)?;
            writer.write_u32::<LE>(actions.len() as u32)?;
            for action in actions {
                action.serialize_exe(&mut writer, version)?;
            }
        }
        Ok(())
    }
}
