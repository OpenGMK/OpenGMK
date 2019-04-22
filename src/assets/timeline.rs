#![allow(dead_code)]

use crate::bytes::{
    ReadBytes,
    ReadString,
    //WriteBytes, WriteString
};
use crate::game::parser::ParserOptions;
use crate::gml::dnd::CodeAction;
use crate::types::Version;

use std::io::{self, Seek, SeekFrom};

pub const VERSION: Version = 500;
pub const VERSION_MOMENT: Version = 400;

pub struct Timeline {
    /// The asset name present in GML and the editor.
    pub name: String,

    /// The list of moments in the timeline with their associated [THING].
    pub moments: Vec<(u32, Vec<CodeAction>)>,
}

impl Timeline {
    pub fn serialize<W>(&self, _writer: &mut W) -> io::Result<usize>
    where
        W: io::Write,
    {
        let result = 0;

        Ok(result)
    }

    pub fn deserialize<B>(bin: B, options: &ParserOptions) -> io::Result<Timeline>
    where
        B: AsRef<[u8]>,
    {
        let mut reader = io::Cursor::new(bin.as_ref());
        let name = reader.read_pas_string()?;

        if options.strict {
            let version = reader.read_u32_le()? as Version;
            assert_eq!(version, VERSION);
        } else {
            reader.seek(SeekFrom::Current(4))?;
        }

        let moment_count = reader.read_u32_le()? as usize;
        let mut moments = Vec::with_capacity(moment_count);
        for _ in 0..moment_count {
            let moment_index = reader.read_u32_le()?;

            if options.strict {
                let version = reader.read_u32_le()? as Version;
                assert_eq!(version, VERSION_MOMENT);
            } else {
                reader.seek(SeekFrom::Current(4))?;
            }

            let action_count = reader.read_u32_le()? as usize;

            let mut actions = Vec::with_capacity(action_count);
            for _ in 0..action_count {
                actions.push(CodeAction::deserialize(&mut reader)?);
            }

            moments.push((moment_index, actions));
        }

        Ok(Timeline { name, moments })
    }
}
