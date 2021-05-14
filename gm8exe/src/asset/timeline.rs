use crate::{
    asset::{assert_ver, Asset, CodeAction, Error, PascalString, ReadPascalString, WritePascalString},
    GameVersion,
};
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use std::io::{self, Read};

pub const VERSION: u32 = 500;
pub const VERSION_MOMENT: u32 = 400;

pub struct Timeline {
    /// The asset name present in GML and the editor.
    pub name: PascalString,

    /// The list of moments in the timeline with their associated [THING].
    pub moments: Vec<(u32, Vec<CodeAction>)>,
}

impl Asset for Timeline {
    fn deserialize_exe(mut reader: impl Read, version: GameVersion, strict: bool) -> Result<Self, Error> {
        let name = reader.read_pas_string()?;

        let ver = reader.read_u32::<LE>()?;
        if strict {
            assert_ver(ver, VERSION)?;
        }

        let moment_count = reader.read_u32::<LE>()? as usize;
        let moments = (0..moment_count)
            .map(|_| {
                let moment_index = reader.read_u32::<LE>()?;

                let ver = reader.read_u32::<LE>()?;
                if strict {
                    assert_ver(ver, VERSION_MOMENT)?;
                }

                let action_count = reader.read_u32::<LE>()? as usize;

                let actions = (0..action_count)
                    .map(|_| CodeAction::deserialize_exe(&mut reader, version, strict))
                    .collect::<Result<_, _>>()?;

                Ok((moment_index, actions))
            })
            .collect::<Result<_, Error>>()?;

        Ok(Timeline { name, moments })
    }

    fn serialize_exe(&self, mut writer: impl io::Write, version: GameVersion) -> io::Result<()> {
        writer.write_pas_string(&self.name)?;
        writer.write_u32::<LE>(VERSION)?;
        writer.write_u32::<LE>(self.moments.len() as u32)?;
        for (moment_index, actions) in &self.moments {
            writer.write_u32::<LE>(*moment_index)?;
            writer.write_u32::<LE>(VERSION_MOMENT)?;
            writer.write_u32::<LE>(actions.len() as u32)?;
            for action in actions {
                action.serialize_exe(&mut writer, version)?;
            }
        }
        Ok(())
    }
}
