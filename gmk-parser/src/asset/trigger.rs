use crate::asset::{Asset, ByteString, Timestamp, Version};

use byteorder::{LE, ReadBytesExt, WriteBytesExt};
use std::io;

pub struct Trigger {
    /// I don't think this actually has any purpose.
    /// The trigger is referred to in GML by its `constant_name` field.
    pub name: ByteString,
    pub version: Version,

    /// The trigger condition - a single GML expression to evaluate.
    pub condition: ByteString,

    /// What event to run the check on.
    pub moment: TriggerKind,

    /// Constant name used to refer to the trigger in GML.
    pub constant_name: ByteString,
}

#[derive(Copy, Clone, PartialEq)]
#[repr(u32)]
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

impl Asset for Trigger {
    #[inline]
    fn name(&self) -> &[u8] {
        self.name.0.as_slice()
    }

    #[inline]
    fn timestamp(&self) -> Timestamp {
        Timestamp::default()
    }

    #[inline]
    fn version(&self) -> Version {
        self.version
    }

    fn from_gmk<R: io::Read>(&self, mut reader: R) -> io::Result<Self> {
        Self::read(&mut reader, true)
    }

    fn to_gmk<W: io::Write>(&self, mut writer: W) -> io::Result<()> {
        self.write(&mut writer, true)
    }

    fn from_exe<R: io::Read>(&self, mut reader: R) -> io::Result<Self> {
        Self::read(&mut reader, false)
    }

    fn to_exe<W: io::Write>(&self, mut writer: W) -> io::Result<()> {
        self.write(&mut writer, false)
    }
}

impl Trigger {
    fn read(mut reader: &mut dyn io::Read, is_gmk: bool) -> io::Result<Self> {
        let version = read_version!(reader, &ByteString::default(), is_gmk, "trigger", Gm800)?;
        let name = ByteString::read(&mut reader)?;
        let condition = ByteString::read(&mut reader)?;
        let moment = TriggerKind::from(reader.read_u32::<LE>()?);
        let constant_name = ByteString::read(&mut reader)?;
        Ok(Self { name, version, condition, moment, constant_name })
    }

    fn write(&self, mut writer: &mut dyn io::Write, _is_gmk: bool) -> io::Result<()> {
        assert_eq!(self.version, Version::Gm800);
        writer.write_u32::<LE>(self.version as u32)?;
        self.name.write(&mut writer)?;
        self.condition.write(&mut writer)?;
        writer.write_u32::<LE>(self.moment as u32)?;
        self.constant_name.write(&mut writer)
    }
}
