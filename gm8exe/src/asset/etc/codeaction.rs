use crate::{
    asset::{assert_ver, AssetDataError, PascalString, ReadPascalString, WritePascalString},
    def::ID,
};

use minio::{ReadPrimitives, WritePrimitives};
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
            let version = reader.read_u32_le()?;
            assert_ver(version, VERSION)?;
        } else {
            reader.seek(SeekFrom::Current(4))?;
        }

        let lib_id = reader.read_u32_le()?;
        let id = reader.read_u32_le()?;
        let action_kind = reader.read_u32_le()?;
        let can_be_relative = reader.read_u32_le()?;
        let is_condition = reader.read_u32_le()? != 0;
        let applies_to_something = reader.read_u32_le()? != 0;
        let execution_type = reader.read_u32_le()?;

        let fn_name = reader.read_pas_string()?;
        let fn_code = reader.read_pas_string()?;

        let param_count = reader.read_u32_le()? as usize;
        if param_count > PARAM_COUNT {
            return Err(AssetDataError::MalformedData);
        }

        // type count - should always be 8
        if reader.read_u32_le()? as usize != PARAM_COUNT {
            return Err(AssetDataError::MalformedData);
        }

        let mut param_types = [0u32; PARAM_COUNT];
        for val in param_types.iter_mut() {
            *val = reader.read_u32_le()?;
        }

        let applies_to = reader.read_i32_le()? as ID;
        let is_relative = reader.read_u32_le()? != 0;

        // arg count - should always be 8
        if reader.read_u32_le()? as usize != PARAM_COUNT {
            return Err(AssetDataError::MalformedData);
        }

        let mut param_strings: [PascalString; 8] = Default::default();
        for val in param_strings.iter_mut() {
            *val = reader.read_pas_string()?;
        }

        let invert_condition = reader.read_u32_le()? != 0;

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

    pub fn write_to<W>(&self, writer: &mut W) -> io::Result<usize>
    where
        W: io::Write,
    {
        let mut result = writer.write_u32_le(VERSION)?;
        result += writer.write_u32_le(self.lib_id)?;
        result += writer.write_u32_le(self.id)?;
        result += writer.write_u32_le(self.action_kind)?;
        result += writer.write_u32_le(self.can_be_relative)?;
        result += writer.write_u32_le(self.is_condition as u32)?;
        result += writer.write_u32_le(self.applies_to_something as u32)?;
        result += writer.write_u32_le(self.execution_type)?;
        result += writer.write_pas_string(&self.fn_name)?;
        result += writer.write_pas_string(&self.fn_code)?;
        result += writer.write_u32_le(self.param_count as u32)?;
        result += writer.write_u32_le(PARAM_COUNT as u32)?;
        for i in &self.param_types {
            result += writer.write_u32_le(*i)?;
        }
        result += writer.write_i32_le(self.applies_to)?;
        result += writer.write_u32_le(self.is_relative as u32)?;
        result += writer.write_u32_le(PARAM_COUNT as u32)?;
        for i in &self.param_strings {
            result += writer.write_pas_string(i)?;
        }
        result += writer.write_u32_le(self.invert_condition as u32)?;

        Ok(result)
    }
}
