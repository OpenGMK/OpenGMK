use crate::asset::{Action, Asset, ByteString, Timestamp, Version, object::Event};

use byteorder::{LE, ReadBytesExt, WriteBytesExt};
use std::io;

pub struct Timeline {
    pub name: ByteString,
    pub timestamp: Timestamp,
    pub version: Version,

    pub moments: Vec<Event>,
}

impl Asset for Timeline {
    #[inline]
    fn name(&self) -> &[u8] {
        self.name.0.as_slice()
    }

    #[inline]
    fn timestamp(&self) -> Timestamp {
        self.timestamp
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

impl Timeline {
    fn read(mut reader: &mut dyn io::Read, is_gmk: bool) -> io::Result<Self> {
        let name = ByteString::read(&mut reader)?;
        let timestamp = if is_gmk {
            Timestamp(reader.read_f64::<LE>()?)
        } else {
            Timestamp::default()
        };
        let version = read_version!(reader, name, is_gmk, "timeline", Gm500)?;

        let moment_count = reader.read_u32::<LE>()? as usize;
        let moments = (0..moment_count).map(|_| {
            let index = reader.read_u32::<LE>()?;
            let version = read_version!(reader, name, is_gmk, "moment", Gm400)?;

            let action_count = reader.read_u32::<LE>()? as usize;
            let actions = (0..action_count)
                .map(|_| Action::read_for(&mut reader, is_gmk, &name, "action in timeline"))
                .collect::<Result<_, io::Error>>()?;

            Ok(Event { version, index, actions })
        }).collect::<io::Result<Vec<Event>>>()?;

        Ok(Self { name, timestamp, version, moments })
    }

    fn write(&self, mut writer: &mut dyn io::Write, is_gmk: bool) -> io::Result<()> {
        assert_eq!(self.version, Version::Gm500);
        self.name.write(&mut writer)?;
        if is_gmk {
            writer.write_f64::<LE>(self.timestamp.0)?;
        }
        writer.write_u32::<LE>(self.version as u32)?;

        writer.write_u32::<LE>(self.moments.len() as u32)?;
        for moment in &self.moments {
            assert_eq!(moment.version, Version::Gm400);
            writer.write_u32::<LE>(moment.index)?;
            writer.write_u32::<LE>(moment.version as u32)?;

            assert!(moment.actions.len() <= u32::max_value() as usize);
            writer.write_u32::<LE>(moment.actions.len() as u32)?;
            for action in &moment.actions {
                action.write(&mut writer)?;
            }
        }
        Ok(())
    }
}
