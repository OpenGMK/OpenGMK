use crate::asset::{Action, ByteString, Version};
use byteorder::{LE, ReadBytesExt, WriteBytesExt};
use std::io;

pub struct Event {
    pub version: Version,

    pub index: u32,
    pub actions: Vec<Action>,
}

impl Event {
    pub(crate) fn read_for(
        mut reader: &mut dyn io::Read,
        is_gmk: bool,
        rv_name: &ByteString,
        rv_reason: &'static str,
        index: u32,
    ) -> io::Result<Self> {
        let version = read_version!(reader, rv_name, is_gmk, rv_reason, Gm400)?;
        let action_count = reader.read_u32::<LE>()? as usize;
        let actions = (0..action_count)
            .map(|_| Action::read_for(&mut reader, is_gmk, rv_name, rv_reason))
            .collect::<io::Result<Vec<Action>>>()?;
        Ok(Event { version, index, actions })
    }

    pub(crate) fn write(&self, mut writer: &mut dyn io::Write) -> io::Result<()> {
        assert_eq!(self.version, Version::Gm400);
        writer.write_u32::<LE>(self.version as u32)?;
        writer.write_u32::<LE>(self.index)?;
        let len = i32::try_from(self.actions.len()).map_err(|_| io::Error::from(io::ErrorKind::InvalidData))?;
        writer.write_i32::<LE>(len)?;
        for action in &self.actions {
            action.write(&mut writer)?;
        }
        Ok(())
    }
}
