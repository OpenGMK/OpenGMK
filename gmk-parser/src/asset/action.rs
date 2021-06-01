use crate::asset::{Asset, ByteString, Timestamp, Version};

use byteorder::{LE, ReadBytesExt, WriteBytesExt};
use std::io;

pub const PARAM_COUNT: usize = 8;

pub struct Action {
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
    // TODO: why not an enum
    pub execution_type: u32,

    /// Whether the "relative" checkbox appears in the GameMaker IDE.
    pub is_relative: u32,

    /// Whether you can change execution target in the GameMaker IDE.
    pub is_applicative: bool,

    /// Name of the function if applicable.
    ///
    /// Usually only provided by extensions.
    pub function_name: ByteString,

    /// The GML source code of the action if applicable.
    ///
    /// Usually only provided by extensions.
    pub function_code: ByteString,

    pub param_count: usize,
    pub param_types: [u32; PARAM_COUNT],
    pub param_strings: [ByteString; PARAM_COUNT],
}

impl Action {
    fn read(mut reader: &mut dyn io::Read, is_gmk: bool) -> io::Result<Self> {

    }

    fn write(&self, mut writer: &mut dyn io::Write, is_gmk: bool) -> io::Result<()> {

    }
}
