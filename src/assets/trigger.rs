#![allow(dead_code)] // Shut up???

use crate::bytes::{ReadBytes, ReadString, WriteBytes, WriteString};
use crate::game::parser::ParserOptions;
use crate::types::Version;
use std::fmt::{self, Display};
use std::io::{self, Seek, SeekFrom};

pub const VERSION: Version = 800;

pub struct Trigger {
    /// The asset name present in the editor.
    /// I don't think this actually has any purpose. The trigger is referred to in GML by its constant_name.
    pub name: String,

    /// The trigger condition, a GML expression
    pub condition: String,

    /// Check moment: step, begin step, or end step
    pub moment: TriggerKind,

    /// Constant name
    pub constant_name: String,
}

impl Trigger {
    pub fn serialize<W>(&self, writer: &mut W) -> io::Result<usize>
    where
        W: io::Write,
    {
        let mut result = writer.write_u32_le(VERSION as u32)?;
        result += writer.write_pas_string(&self.name)?;
        result += writer.write_pas_string(&self.condition)?;
        result += writer.write_u32_le(self.moment as u32)?;
        result += writer.write_pas_string(&self.constant_name)?;

        Ok(result)
    }

    pub fn deserialize<B>(bin: B, options: &ParserOptions) -> io::Result<Trigger>
    where
        B: AsRef<[u8]>,
    {
        let mut reader = io::Cursor::new(bin.as_ref());
        if options.strict {
            let version = reader.read_u32_le()?;
            assert_eq!(version, VERSION);
        } else {
            reader.seek(SeekFrom::Current(4))?;
        }

        let name = reader.read_pas_string()?;
        let condition = reader.read_pas_string()?;
        let moment = TriggerKind::from(reader.read_u32_le()?);
        let constant_name = reader.read_pas_string()?;

        Ok(Trigger {
            name,
            condition,
            moment,
            constant_name,
        })
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum TriggerKind {
    Step = 0,
    BeginStep = 1,
    EndStep = 2,
}

impl From<u32> for TriggerKind {
    fn from(n: u32) -> TriggerKind {
        match n {
            0 => TriggerKind::Step,
            1 => TriggerKind::BeginStep,
            2 => TriggerKind::EndStep,

            _ => TriggerKind::Step,
        }
    }
}

impl Display for TriggerKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TriggerKind::Step => "step",
                TriggerKind::BeginStep => "begin step",
                TriggerKind::EndStep => "end step",
            }
        )
    }
}
