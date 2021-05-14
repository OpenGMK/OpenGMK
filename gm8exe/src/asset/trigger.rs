use crate::{
    asset::{assert_ver, Asset, Error, PascalString, ReadPascalString, WritePascalString},
    GameVersion,
};
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use std::{
    fmt,
    io::{self, Read},
};

pub const VERSION: u32 = 800;

pub struct Trigger {
    /// The asset name present in the editor.
    ///
    /// I don't think this actually has any purpose.
    /// The trigger is referred to in GML by its constant_name field.
    pub name: PascalString,

    /// The trigger condition - a single GML expression to evaluate.
    pub condition: PascalString,

    /// What event to run the check on.
    pub moment: TriggerKind,

    /// Constant name used to refer to the trigger in GML.
    pub constant_name: PascalString,
}

#[derive(Copy, Clone, PartialEq)]
pub enum TriggerKind {
    Step = 0,
    BeginStep = 1,
    EndStep = 2,
}

impl fmt::Display for TriggerKind {
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
    fn deserialize_exe(mut reader: impl Read, _version: GameVersion, strict: bool) -> Result<Self, Error> {
        let version = reader.read_u32::<LE>()?;
        if strict {
            assert_ver(version, VERSION)?;
        }

        let name = reader.read_pas_string()?;
        let condition = reader.read_pas_string()?;
        let moment = TriggerKind::from(reader.read_u32::<LE>()?);
        let constant_name = reader.read_pas_string()?;

        Ok(Trigger { name, condition, moment, constant_name })
    }

    fn serialize_exe(&self, mut writer: impl io::Write, _version: GameVersion) -> io::Result<()> {
        writer.write_u32::<LE>(VERSION)?;
        writer.write_pas_string(&self.name)?;
        writer.write_pas_string(&self.condition)?;
        writer.write_u32::<LE>(self.moment as u32)?;
        writer.write_pas_string(&self.constant_name)?;
        Ok(())
    }
}
