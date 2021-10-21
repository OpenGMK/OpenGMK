use crate::{
    asset::{assert_ver, Asset, CodeAction, Error, PascalString, ReadPascalString, WritePascalString},
    GameVersion,
};
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use std::io::{self, Read};

pub const VERSION: u32 = 430;
pub const VERSION_EVENT: u32 = 400;

pub struct Object {
    /// The asset name present in GML and the editor.
    pub name: PascalString,

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
    ///
    /// Note: any negative index (usually -1) indicates no parent.
    pub parent_index: i32,

    /// Default mask_index (a sprite index) for this Object
    ///
    /// Note: any negative index (usually -1) indicates sprite_index should be used as mask index.
    pub mask_index: i32,

    /// Object event lists.
    ///
    /// An object usually has 12 event lists.
    /// Each list can have 0 or many sub-events, which are indexed by the tuple LHS.
    pub events: Vec<Vec<(u32, Vec<CodeAction>)>>,
}

impl Asset for Object {
    fn deserialize_exe(mut reader: impl Read, version: GameVersion, strict: bool) -> Result<Self, Error> {
        let name = reader.read_pas_string()?;

        let ver = reader.read_u32::<LE>()?;
        if strict {
            assert_ver(ver, VERSION)?;
        }

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
        let event_list_count = reader.read_u32::<LE>()?;
        if event_list_count != 11 {
            return Err(Error::MalformedData)
        }
        let mut events = Vec::with_capacity((event_list_count + 1) as usize);

        for _ in 0..=event_list_count {
            // Read until we get a negative value in place of the sub_index (indicated here by u32::try_from failing)
            let mut sub_event_list: Vec<(u32, Vec<CodeAction>)> = Vec::new();
            while let Ok(index) = u32::try_from(reader.read_i32::<LE>()?) {
                let ver = reader.read_u32::<LE>()?;
                if strict {
                    assert_ver(ver, VERSION_EVENT)?;
                }

                let action_count = reader.read_u32::<LE>()?;
                let actions = (0..action_count)
                    .map(|_| CodeAction::deserialize_exe(&mut reader, version, strict))
                    .collect::<Result<_, _>>()?;
                sub_event_list.push((index, actions));
            }
            events.push(sub_event_list);
        }

        Ok(Object { name, sprite_index, solid, visible, depth, persistent, parent_index, mask_index, events })
    }

    fn serialize_exe(&self, mut writer: impl io::Write, version: GameVersion) -> io::Result<()> {
        writer.write_pas_string(&self.name)?;
        writer.write_u32::<LE>(VERSION)?;
        writer.write_i32::<LE>(self.sprite_index)?;
        writer.write_u32::<LE>(self.solid.into())?;
        writer.write_u32::<LE>(self.visible.into())?;
        writer.write_i32::<LE>(self.depth)?;
        writer.write_u32::<LE>(self.persistent.into())?;
        writer.write_i32::<LE>(self.parent_index)?;
        writer.write_i32::<LE>(self.mask_index)?;
        writer.write_u32::<LE>((self.events.len() - 1) as u32)?; // TODO: checks! cast checks too!
        for sub_list in self.events.iter() {
            for (sub, actions) in sub_list.iter() {
                writer.write_u32::<LE>(*sub)?;
                writer.write_u32::<LE>(VERSION_EVENT)?;
                writer.write_u32::<LE>(actions.len() as u32)?;
                for action in actions.iter() {
                    action.serialize_exe(&mut writer, version)?;
                }
            }
            writer.write_i32::<LE>(-1)?; // -1 in place of a sub-id indicates the end of this list
        }
        Ok(())
    }
}
