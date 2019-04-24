#![allow(dead_code)] // Shut up.

use crate::bytes::{ReadBytes, ReadString, WriteBytes, WriteString};
use crate::game::parser::ParserOptions;
use crate::gml::dnd::CodeAction;
use crate::types::Version;
use std::io::{self, Seek, SeekFrom};

pub const VERSION: Version = 430;
pub const VERSION_EVENT: Version = 400;

pub struct Object {
    /// The asset name present in GML and the editor.
    pub name: String,

    /// Default sprite_index for this Object
    /// Note: any negative sprite_index (usually -1) indicates no sprite.
    pub sprite_index: i32,

    // Default state of "solid" instance flag
    pub solid: bool,

    // Default state of "visible" instance flag
    pub visible: bool,

    // Default depth value for this Object
    pub depth: i32,

    // Default state of "persistent" instance flag
    pub persistent: bool,

    /// Object index for the parent of this Object
    /// Note: any negative index (usually -1) indicates no parent.
    pub parent_index: i32,

    /// Default mask_index (a sprite index) for this Object
    /// Note: any negative index (usually -1) indicates sprite_index should be used as mask index
    pub mask_index: i32,

    /// Object event lists
    /// An object usually has 12 event lists. Each list can have 0 or many sub-events, which are indexed by the tuple LHS.
    pub events: Vec<Vec<(u32, Vec<CodeAction>)>>,
}

impl Object {
    pub fn serialize<W>(&self, writer: &mut W) -> io::Result<usize>
    where
        W: io::Write,
    {
        let mut result = writer.write_pas_string(&self.name)?;
        result += writer.write_u32_le(VERSION as u32)?;
        result += writer.write_u32_le(self.sprite_index as u32)?;
        result += writer.write_u32_le(if self.solid {1} else {0})?;
        result += writer.write_u32_le(if self.visible {1} else {0})?;
        result += writer.write_u32_le(self.depth as u32)?;
        result += writer.write_u32_le(if self.persistent {1} else {0})?;
        result += writer.write_u32_le(self.parent_index as u32)?;
        result += writer.write_u32_le(self.mask_index as u32)?;

        result += writer.write_u32_le((self.events.len() - 1) as u32)?;
        for sub_list in self.events.iter() {
            for (sub, actions) in sub_list.iter() {
                result += writer.write_u32_le(*sub)?;
                result += writer.write_u32_le(VERSION_EVENT as u32)?;
                result += writer.write_u32_le(actions.len() as u32)?;
                for action in actions.iter() {
                    result += CodeAction::serialize(action, writer)?;
                }
            }
            result += writer.write_i32_le(-1_i32)?; // Better way to do this?
        }
        Ok(result)
    }

    pub fn deserialize<B>(bin: B, options: &ParserOptions) -> io::Result<Object>
    where
        B: AsRef<[u8]>,
    {
        let mut reader = io::Cursor::new(bin.as_ref());
        let name = reader.read_pas_string()?;

        if options.strict {
            let version = reader.read_u32_le()?;
            assert_eq!(version, VERSION);
        } else {
            reader.seek(SeekFrom::Current(4))?;
        }

        let sprite_index = reader.read_u32_le()? as i32;
        let solid = reader.read_u32_le()? != 0;
        let visible = reader.read_u32_le()? != 0;
        let depth = reader.read_u32_le()? as i32;
        let persistent = reader.read_u32_le()? != 0;
        let parent_index = reader.read_u32_le()? as i32;
        let mask_index = reader.read_u32_le()? as i32;

        // This is always 11. I don't know what to do if this isn't 11. We'll probably never know, because it's always 11.
        // We might as well load it instead of hard-coding it everywhere just for clarity, and for easier damage control if this ever becomes a problem.
        // Oh, also, it's 0..=n so the number is actually 11 instead of 12 because there are 12 lists. Yeah.
        let event_list_count = reader.read_u32_le()?;
        assert_eq!(event_list_count, 11);
        let mut events: Vec<Vec<(u32, Vec<CodeAction>)>> = Vec::with_capacity((event_list_count + 1) as usize);

        for _ in 0..=event_list_count {
            let mut sub_event_list: Vec<(u32, Vec<CodeAction>)> = Vec::new();
            loop {
                let index = reader.read_i32_le()?;
                if index == -1 {
                    break;
                }

                if options.strict {
                    let version = reader.read_u32_le()?;
                    assert_eq!(version, VERSION_EVENT);
                } else {
                    reader.seek(SeekFrom::Current(4))?;
                }

                let action_count = reader.read_u32_le()?;
                let mut actions: Vec<CodeAction> = Vec::with_capacity(action_count as usize);
                for _ in 0..action_count {
                    actions.push(CodeAction::deserialize(&mut reader)?);
                }
                sub_event_list.push((index as u32, actions));
            }
            events.push(sub_event_list);
        }

        Ok(Object {
            name,
            sprite_index,
            solid,
            visible,
            depth,
            persistent,
            parent_index,
            mask_index,
            events,
        })
    }
}