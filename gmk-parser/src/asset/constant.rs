use crate::asset::ByteString;
use std::io;

pub struct Constant {
    pub name: ByteString,
    pub expression: ByteString,
}

impl Constant {
    pub fn read(mut reader: &mut dyn io::Read) -> io::Result<Self> {
        Ok(Self {
            name: ByteString::read(&mut reader)?,
            expression: ByteString::read(&mut reader)?,
        })
    }

    pub fn write(&self, mut writer: &mut dyn io::Write) -> io::Result<()> {
        self.name.write(&mut writer)?;
        self.expression.write(&mut writer)
    }
}
