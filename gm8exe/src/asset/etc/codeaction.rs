use crate::{
    asset::{assert_ver, AssetDataError, PascalString, ReadPascalString, WritePascalString},
    def::ID,
};
use byteorder::{LE, ReadBytesExt, WriteBytesExt};
use std::io::{self, Cursor, Seek, SeekFrom};

pub const VERSION: u32 = 440;
pub const PARAM_COUNT: usize = 8;

pub struct CodeAction {
    /// Unique ID that identifies what type of DnD action this is.
    pub id: u32,

    /// Instance this applies to.
    pub applies_to: ID,

    /// Indicates whether this DnD is a condition,
    /// ie. whether the following DnD should be dependent on the evaluation of this one.
    pub is_condition: bool,

    /// Whether the "NOT" checkbox is checked.
    /// All DnDs have this property, even ones which don't have a NOT option.
    pub invert_condition: bool,

    /// Whether the "relative" checkbox is checked.
    /// All DnDs have this property, even ones which don't have a "relative" option.
    pub is_relative: bool,

    /// What action library the action is loaded from (GameMaker 8 Runner).
    pub lib_id: u32,

    /// What type of drag-n-drop action this is.
    pub action_kind: u32,

    /// How this action will be executed: None, Function or Code.
    pub execution_type: u32,

    /// Whether the relative checkbox appears in the GameMaker 8 IDE.
    pub can_be_relative: u32,

    /// Whether you can change execution target in the GameMaker 8 IDE.
    pub applies_to_something: bool,

    /// Name of the function if applicable. Usually only provided by extensions.
    pub fn_name: PascalString,

    /// The GML code of the Drag-and-Drop if applicable. Usually only provided by extensions.
    pub fn_code: PascalString,

    pub param_count: usize,
    pub param_types: [u32; PARAM_COUNT],
    pub param_strings: [PascalString; PARAM_COUNT],
}

impl CodeAction {
    pub fn from_cur<B>(reader: &mut Cursor<B>, strict: bool) -> Result<Self, AssetDataError>
    where
        B: AsRef<[u8]>,
    {
        if strict {
            let version = reader.read_u32::<LE>()?;
            assert_ver(version, VERSION)?;
        } else {
            reader.seek(SeekFrom::Current(4))?;
        }

        let lib_id = reader.read_u32::<LE>()?;
        let id = reader.read_u32::<LE>()?;
        let action_kind = reader.read_u32::<LE>()?;
        let can_be_relative = reader.read_u32::<LE>()?;
        let is_condition = reader.read_u32::<LE>()? != 0;
        let applies_to_something = reader.read_u32::<LE>()? != 0;
        let execution_type = reader.read_u32::<LE>()?;

        let fn_name = reader.read_pas_string()?;
        let fn_code = reader.read_pas_string()?;

        let param_count = reader.read_u32::<LE>()? as usize;
        if param_count > PARAM_COUNT {
            return Err(AssetDataError::MalformedData);
        }

        // type count - should always be 8
        if reader.read_u32::<LE>()? as usize != PARAM_COUNT {
            return Err(AssetDataError::MalformedData);
        }

        let mut param_types = [0u32; PARAM_COUNT];
        for val in param_types.iter_mut() {
            *val = reader.read_u32::<LE>()?;
        }

        let applies_to = reader.read_i32::<LE>()? as ID;
        let is_relative = reader.read_u32::<LE>()? != 0;

        // arg count - should always be 8
        if reader.read_u32::<LE>()? as usize != PARAM_COUNT {
            return Err(AssetDataError::MalformedData);
        }

        let mut param_strings: [PascalString; 8] = Default::default();
        for val in param_strings.iter_mut() {
            *val = reader.read_pas_string()?;
        }

        let invert_condition = reader.read_u32::<LE>()? != 0;

        Ok(CodeAction {
            id,
            applies_to,
            is_condition,
            invert_condition,
            is_relative,
            lib_id,
            action_kind,
            can_be_relative,
            applies_to_something,
            execution_type,
            fn_name,
            fn_code,
            param_count,
            param_types,
            param_strings,
        })
    }

    pub fn write_to<W>(&self, writer: &mut W) -> io::Result<()>
    where
        W: io::Write,
    {
        writer.write_u32::<LE>(VERSION)?;
        writer.write_u32::<LE>(self.lib_id)?;
        writer.write_u32::<LE>(self.id)?;
        writer.write_u32::<LE>(self.action_kind)?;
        writer.write_u32::<LE>(self.can_be_relative)?;
        writer.write_u32::<LE>(self.is_condition as u32)?;
        writer.write_u32::<LE>(self.applies_to_something as u32)?;
        writer.write_u32::<LE>(self.execution_type)?;
        writer.write_pas_string(&self.fn_name)?;
        writer.write_pas_string(&self.fn_code)?;
        writer.write_u32::<LE>(self.param_count as u32)?;
        writer.write_u32::<LE>(PARAM_COUNT as u32)?;
        for i in &self.param_types {
            // TODO: Write this directly with something like bytemuck
            // TODO: Maybe safe-transmute can help? Unsure. Research.
            writer.write_u32::<LE>(*i)?;
        }
        writer.write_i32::<LE>(self.applies_to)?;
        writer.write_u32::<LE>(self.is_relative as u32)?;
        writer.write_u32::<LE>(PARAM_COUNT as u32)?;
        for i in &self.param_strings {
            writer.write_pas_string(i)?;
        }
        writer.write_u32::<LE>(self.invert_condition as u32)?;
        Ok(())
    }
}
