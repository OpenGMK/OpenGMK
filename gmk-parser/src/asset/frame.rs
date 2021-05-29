use crate::asset::{ByteString, Version};

use byteorder::{LE, ReadBytesExt, WriteBytesExt};
use std::io;

pub struct Frame {
    pub version: Version,
    pub size: (u32, u32),
    pub data: Vec<u8>,
}

impl Frame {
    pub(crate) fn read_for(
        reader: &mut dyn io::Read,
        is_gmk: bool,
        rv_name: &ByteString,
        rv_reason: &'static str,
    ) -> io::Result<Self> {
        let version = read_version!(reader, rv_name, is_gmk, rv_reason, Gm800)?;
        let size = (
            reader.read_u32::<LE>()?,
            reader.read_u32::<LE>()?,
        );
        let data = if size.0 > 0 && size.1 > 0 {
            let data_len = reader.read_u32::<LE>()? as usize;
            let mut data = Vec::with_capacity(data_len);
            unsafe { data.set_len(data_len) };
            reader.read_exact(data.as_mut_slice())?;
            data
        } else {
            Vec::new()
        };
        Ok(Frame { version, size, data })
    }

    pub(crate) fn write(&self, writer: &mut dyn io::Write) -> io::Result<()> {
        assert_eq!(self.version, Version::Gm800);
        writer.write_u32::<LE>(self.version as u32)?;
        writer.write_u32::<LE>(self.size.0)?;
        writer.write_u32::<LE>(self.size.1)?;
        if self.size.0 > 0 && self.size.1 > 0 {
            assert!(self.data.len() <= u32::max_value() as usize);
            writer.write_u32::<LE>(self.data.len() as u32)?;
            writer.write_all(self.data.as_slice())?;
        }
        Ok(())
    }
}
