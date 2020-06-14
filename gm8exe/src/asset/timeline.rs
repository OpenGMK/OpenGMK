use crate::{
    asset::{assert_ver, etc::CodeAction, Asset, AssetDataError, PascalString, ReadPascalString, WritePascalString},
    GameVersion,
};

use minio::{ReadPrimitives, WritePrimitives};
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
    fn deserialize<B>(bytes: B, strict: bool, _version: GameVersion) -> Result<Self, AssetDataError>
    where
        B: AsRef<[u8]>,
        Self: Sized,
    {
        let mut reader = io::Cursor::new(bytes.as_ref());
        let name = reader.read_pas_string()?;

        if strict {
            let version = reader.read_u32_le()?;
            assert_ver(version, VERSION)?;
        } else {
            reader.seek(SeekFrom::Current(4))?;
        }

        let moment_count = reader.read_u32_le()? as usize;
        let mut moments = Vec::with_capacity(moment_count);
        for _ in 0..moment_count {
            let moment_index = reader.read_u32_le()?;

            if strict {
                let version = reader.read_u32_le()?;
                assert_ver(version, VERSION_MOMENT)?;
            } else {
                reader.seek(SeekFrom::Current(4))?;
            }

            let action_count = reader.read_u32_le()? as usize;

            let mut actions = Vec::with_capacity(action_count);
            for _ in 0..action_count {
                actions.push(CodeAction::from_cur(&mut reader, strict)?);
            }

            moments.push((moment_index, actions));
        }

        Ok(Timeline { name, moments })
    }

    fn serialize<W>(&self, writer: &mut W) -> io::Result<usize>
    where
        W: io::Write,
    {
        let mut result = writer.write_pas_string(&self.name)?;
        result += writer.write_u32_le(VERSION)?;
        result += writer.write_u32_le(self.moments.len() as u32)?;
        for (moment_index, actions) in &self.moments {
            result += writer.write_u32_le(*moment_index)?;
            result += writer.write_u32_le(VERSION_MOMENT as u32)?;
            result += writer.write_u32_le(actions.len() as u32)?;
            for action in actions {
                result += action.write_to(writer)?;
            }
        }
        Ok(result)
    }
}
