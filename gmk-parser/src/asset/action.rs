use crate::asset::{ByteString, Version};

use byteorder::{LE, ReadBytesExt, WriteBytesExt};
use std::io;

pub const PARAM_COUNT: usize = 8;

pub struct Action {
    pub version: Version,

    /// Identifier resolving to logic that interprets the rest of the action.
    pub id: u32,

    /// Instance this applies to.
    pub target: i32,

    /// Whether the following action is dependent on this action's boolean result.
    pub is_condition: bool,

    /// Whether the condition this action evaluates to is inverted ("NOT" checkbox).
    /// All actions have this property, even ones which don't have a "NOT" option.
    pub invert_condition: bool,

    /// What action library the action is loaded from (extensions).
    pub library_id: u32,

    /// What kind of action this is.
    pub kind: u32,

    /// How this action will be executed: None, Function or Code.
    /// 
    /// Only used if `kind` is Normal (0).
    pub execution_type: u32,

    /// Whether the "relative" checkbox appears in the GameMaker IDE.
    pub can_be_relative: bool,

    /// Whether the "relative" checkbox is checked.
    /// 
    /// All DnDs have this property, even ones which don't actually have a "relative" checkbox.
    pub relative: bool,

    /// Whether you can change execution target in the GameMaker IDE.
    pub is_applicative: bool,

    /// Name of the kernel function for this action to call, if applicable.
    ///
    /// Only used if `kind` is Normal (0) and `execution_type` is Function (1).
    pub function_name: ByteString,

    /// The GML source code of the action if applicable.
    ///
    /// Only used if `kind` is Code (7), or if `kind` is Normal (0) and `execution_type` is Code (2).
    pub function_code: ByteString,

    pub param_count: usize,
    pub param_types: [u32; PARAM_COUNT],
    pub param_strings: [ByteString; PARAM_COUNT],
}

impl Action {
    pub(crate) fn read_for(
        mut reader: &mut dyn io::Read,
        is_gmk: bool,
        rv_name: &ByteString,
        rv_reason: &'static str,
    ) -> io::Result<Self> {
        let version = read_version!(reader, rv_name, is_gmk, rv_reason, Gm440)?;
        let library_id = reader.read_u32::<LE>()?;
        let id = reader.read_u32::<LE>()?;
        let kind = reader.read_u32::<LE>()?;
        let can_be_relative = reader.read_u32::<LE>()? != 0;
        let is_condition = reader.read_u32::<LE>()? != 0;
        let is_applicative = reader.read_u32::<LE>()? != 0;
        let execution_type = reader.read_u32::<LE>()?;

        let function_name = ByteString::read(&mut reader)?;
        let function_code = ByteString::read(&mut reader)?;

        let param_count = reader.read_u32::<LE>()? as usize;
        // verify the number of used parameters isn't greater than PARAM_COUNT, otherwise this gmk is
        // probably corrupted, or would need a heavy runner modification to work correctly
        if param_count > PARAM_COUNT {
            return Err(io::ErrorKind::InvalidData.into())
        }

        // type count - should always be PARAM_COUNT because that's the size of the internal array
        if reader.read_u32::<LE>()? as usize != PARAM_COUNT {
            return Err(io::ErrorKind::InvalidData.into())
        }

        let mut param_types = [0u32; PARAM_COUNT];
        for val in param_types.iter_mut() {
            *val = reader.read_u32::<LE>()?;
        }

        let target = reader.read_i32::<LE>()?;
        let relative = reader.read_u32::<LE>()? != 0;

        // arg count - again, should always be 8
        if reader.read_u32::<LE>()? as usize != PARAM_COUNT {
            return Err(io::ErrorKind::InvalidData.into())
        }

        let mut param_strings: [ByteString; 8] = Default::default();
        for val in param_strings.iter_mut() {
            *val = ByteString::read(&mut reader)?;
        }

        let invert_condition = reader.read_u32::<LE>()? != 0;

        Ok(Action {
            version,
            id,
            target,
            is_condition,
            invert_condition,
            relative,
            library_id,
            kind,
            execution_type,
            can_be_relative,
            is_applicative,
            function_name,
            function_code,
            param_count,
            param_types,
            param_strings,
        })
    }

    pub(crate) fn write(&self, mut writer: &mut dyn io::Write) -> io::Result<()> {
        assert_eq!(self.version, Version::Gm440);
        writer.write_u32::<LE>(self.version as u32)?;
        writer.write_u32::<LE>(self.library_id)?;
        writer.write_u32::<LE>(self.id)?;
        writer.write_u32::<LE>(self.kind as u32)?;
        writer.write_u32::<LE>(self.can_be_relative.into())?;
        writer.write_u32::<LE>(self.is_condition.into())?;
        writer.write_u32::<LE>(self.is_applicative.into())?;
        writer.write_u32::<LE>(self.execution_type as u32)?;
        self.function_name.write(&mut writer)?;
        self.function_code.write(&mut writer)?;
        writer.write_u32::<LE>(self.param_count as u32)?;

        writer.write_u32::<LE>(self.param_types.len() as u32)?;
        for value in self.param_types.iter().copied() {
            writer.write_u32::<LE>(value)?;
        }

        writer.write_i32::<LE>(self.target)?;
        writer.write_u32::<LE>(self.relative.into())?;

        writer.write_u32::<LE>(self.param_strings.len() as u32)?;
        for value in self.param_strings.iter() {
            value.write(&mut writer)?;
        }

        writer.write_u32::<LE>(self.invert_condition.into())?;

        Ok(())
    }
}
