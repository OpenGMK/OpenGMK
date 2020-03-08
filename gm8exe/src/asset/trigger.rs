use crate::{
    asset::{assert_ver, Asset, AssetDataError, ReadPascalString, WritePascalString},
    GameVersion,
};

use minio::{ReadPrimitives, WritePrimitives};
use std::{
    fmt::{self, Display},
    io::{self, Seek, SeekFrom},
};

pub const VERSION: u32 = 800;

pub struct Trigger {
    /// The asset name present in the editor.
    ///
    /// I don't think this actually has any purpose.
    /// The trigger is referred to in GML by its constant_name field.
    pub name: String,

    /// The trigger condition - a single GML expression to evaluate.
    pub condition: String,

    /// What event to run the check on.
    pub moment: TriggerKind,

    /// Constant name used to refer to the trigger in GML.
    pub constant_name: String,
}

#[derive(Copy, Clone, PartialEq)]
pub enum TriggerKind {
    Step = 0,
    BeginStep = 1,
    EndStep = 2,
}

impl Display for TriggerKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TriggerKind::Step => write!(f, "step"),
            TriggerKind::BeginStep => write!(f, "begin step"),
            TriggerKind::EndStep => write!(f, "end step"),
        }
    }
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

impl Asset for Trigger {
    fn deserialize<B>(bytes: B, strict: bool, _version: GameVersion) -> Result<Self, AssetDataError>
    where
        B: AsRef<[u8]>,
        Self: Sized,
    {
        let mut reader = io::Cursor::new(bytes.as_ref());
        if strict {
            let version = reader.read_u32_le()?;
            assert_ver(version, VERSION)?;
        } else {
            reader.seek(SeekFrom::Current(4))?;
        }

        let name = reader.read_pas_string()?;
        let condition = reader.read_pas_string()?;
        let moment = TriggerKind::from(reader.read_u32_le()?);
        let constant_name = reader.read_pas_string()?;

        Ok(Trigger { name, condition, moment, constant_name })
    }

    fn serialize<W>(&self, writer: &mut W) -> io::Result<usize>
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
}
