use crate::asset::{Asset, ByteString, Event, Timestamp, Version};

use byteorder::{LE, ReadBytesExt, WriteBytesExt};
use log::error;
use std::io;

const EVENT_LIST_SIZE: usize = 12;

pub struct Object {
    pub name: ByteString,
    pub timestamp: Timestamp,
    pub version: Version,

    /// Default `sprite_index` for this Object.
    ///
    /// NOTE: Any negative `sprite_index` (usually -1) indicates no sprite.
    pub sprite_index: i32,

    // Default state of the "solid" instance flag.
    pub solid: bool,

    // Default state of the "visible" instance flag.
    pub visible: bool,

    // Default depth value for this object.
    pub depth: i32,

    // Default state of the "persistent" instance flag.
    pub persistent: bool,

    /// Object index for the parent of this object.
    ///
    /// NOTE: Any negative index (usually -1) indicates no parent.
    pub parent_index: i32,

    /// Default `mask_index` (a sprite index) for this object.
    ///
    /// NOTE: Any negative index (usually -1) indicates `sprite_index` should be used as the `mask_index`.
    pub mask_index: i32,

    /// Object event lists.
    ///
    /// An object usually has 12 event lists.
    /// Each list can have 0 or many sub-events.
    pub events: [SubEventList; EVENT_LIST_SIZE],
}

pub struct SubEventList(pub Vec<Event>);

impl Asset for Object {
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

    fn from_gmk<R: io::Read>(mut reader: R) -> io::Result<Self> {
        Self::read(&mut reader, true)
    }

    fn to_gmk<W: io::Write>(&self, mut writer: W) -> io::Result<()> {
        self.write(&mut writer, true)
    }

    fn from_exe<R: io::Read>(mut reader: R) -> io::Result<Self> {
        Self::read(&mut reader, false)
    }

    fn to_exe<W: io::Write>(&self, mut writer: W) -> io::Result<()> {
        self.write(&mut writer, false)
    }
}

impl Object {
    fn read(mut reader: &mut dyn io::Read, is_gmk: bool) -> io::Result<Self> {
        let name = ByteString::read(&mut reader)?;
        let timestamp = if is_gmk {
            Timestamp(reader.read_f64::<LE>()?)
        } else {
            Timestamp::default()
        };
        let version = read_version!(reader, name, is_gmk, "script", Gm430)?;

        let sprite_index = reader.read_i32::<LE>()?;
        let solid = reader.read_u32::<LE>()? != 0;
        let visible = reader.read_u32::<LE>()? != 0;
        let depth = reader.read_i32::<LE>()?;
        let persistent = reader.read_u32::<LE>()? != 0;
        let parent_index = reader.read_i32::<LE>()?;
        let mask_index = reader.read_i32::<LE>()?;

        // This is always 11. I don't know what to do if this isn't 11.
        // We'll probably never know, because it's always 11.
        // We might as well load it instead of hard-coding it everywhere just for clarity,
        // and for easier damage control if this ever becomes a problem.
        // Oh, also, it's 0..=n so the number is actually 11 instead of 12 because there are 12 lists. Yeah.
        let event_list_size = reader.read_u32::<LE>()? as usize;
        if event_list_size != (EVENT_LIST_SIZE - 1) {
            error!(
                "Expected exactly {} entries for event list in object \"{}\", found {}!",
                EVENT_LIST_SIZE, name, event_list_size,
            );
            return Err(io::ErrorKind::InvalidData.into())
        }

        const INIT_DUMMY_HACK: SubEventList = SubEventList(Vec::new());
        let mut events = [INIT_DUMMY_HACK; EVENT_LIST_SIZE];
        for event in &mut events {
            *event = SubEventList::read_for(&mut reader, is_gmk, &name, "sub-event list in object")?;
        }

        Ok(Self {
            name, timestamp, version,
            sprite_index, solid, visible, depth, persistent, parent_index, mask_index, events,
        })
    }

    fn write(&self, mut writer: &mut dyn io::Write, is_gmk: bool) -> io::Result<()> {
        assert_eq!(self.version, Version::Gm430);
        self.name.write(&mut writer)?;
        if is_gmk {
            writer.write_f64::<LE>(self.timestamp.0)?;
        }
        writer.write_u32::<LE>(self.version as u32)?;

        writer.write_i32::<LE>(self.sprite_index)?;
        writer.write_u32::<LE>(self.solid.into())?;
        writer.write_u32::<LE>(self.visible.into())?;
        writer.write_i32::<LE>(self.depth)?;
        writer.write_u32::<LE>(self.persistent.into())?;
        writer.write_i32::<LE>(self.parent_index)?;
        writer.write_i32::<LE>(self.mask_index)?;

        assert!(EVENT_LIST_SIZE <= u32::max_value() as usize);
        writer.write_u32::<LE>((EVENT_LIST_SIZE - 1) as u32)?;
        for sub_event_list in &self.events {
            sub_event_list.write(&mut writer)?;
        }
        Ok(())
    }
}

impl SubEventList {
    pub(crate) fn read_for(
        mut reader: &mut dyn io::Read,
        is_gmk: bool,
        rv_name: &ByteString,
        rv_reason: &'static str,
    ) -> io::Result<Self> {
        let mut sub_events = Vec::new();
        loop {
            let index = reader.read_i32::<LE>()?;
            if let Ok(index) = u32::try_from(index) {
                sub_events.push(Event::read_for(&mut reader, is_gmk, rv_name, rv_reason, index)?);
            } else {
                break
            }
        }
        Ok(Self(sub_events))
    }

    pub(crate) fn write(&self, mut writer: &mut dyn io::Write) -> io::Result<()> {
        for event in &self.0 {
            event.write(&mut writer)?;
        }
        writer.write_i32::<LE>(-1)
    }
}
