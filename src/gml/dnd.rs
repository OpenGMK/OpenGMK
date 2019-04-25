#![allow(dead_code)]

use crate::bytes::{ReadBytes, ReadString, WriteBytes, WriteString};
use crate::game::parser::ParserOptions;
use crate::gml;
use crate::types::Version;

use std::io::{self, Seek, SeekFrom};

pub const VERSION: Version = 440;
pub const PARAM_COUNT: usize = 8;

pub struct CodeAction {
    /// Unique ID that identifies what type of DnD action this is
    pub id: u32,

    /// Parameters to the DnD option. There can be up to 8. These can have various types and will be evaluated at runtime.
    pub parameters: Vec<CodeActionParam>,

    /// Instance this applies to - can be -1 (self), -2 (other) or a positive number (Object ID). TODO: should be an enum?
    pub applies_to: i32,

    /// Indicates whether this DnD is a condition, ie. whether the following DnD should be dependent on the evaluation of this one.
    pub is_condition: bool,

    /// Whether the "NOT" checkbox is checked. All DnDs have this property, even ones which don't have a NOT option.
    pub invert_condition: bool,

    /// Whether the "relative" checkbox is checked. All DnDs have this property, even ones which don't have a "relative" option.
    pub is_relative: bool,

    /// Various redundant information
    pub lib_id: u32,
    pub action_kind: u32,
    pub can_be_relative: u32,
    pub applies_to_something: bool,
    pub action_type: u32,
    pub fn_name: String,
    pub fn_code: String,

    pub param_types: [u32; PARAM_COUNT],
    pub param_strings: Vec<String>,
}

#[derive(Debug)]
pub enum CodeActionParam {
    Expression(String),
    GML(String),
    Literal(gml::Value),
}

impl CodeAction {
    pub fn serialize<W>(&self, _writer: &mut W) -> io::Result<usize>
    where
        W: io::Write,
    {
        let result = 0;
        // TODO: this
        Ok(result)
    }

    pub fn deserialize(
        reader: &mut io::Cursor<&[u8]>,
        options: &ParserOptions,
    ) -> io::Result<CodeAction> {
        if options.strict {
            let version = reader.read_u32_le()?;
            assert_eq!(version, VERSION);
        } else {
            reader.seek(SeekFrom::Current(4))?;
        }

        let lib_id = reader.read_u32_le()?;
        let id = reader.read_u32_le()?;
        let action_kind = reader.read_u32_le()?;
        let can_be_relative = reader.read_u32_le()?;
        let is_condition = reader.read_u32_le()? != 0;
        let applies_to_something = reader.read_u32_le()? != 0;
        let action_type = reader.read_u32_le()?;

        let fn_name = reader.read_pas_string()?;
        let fn_code = reader.read_pas_string()?;

        let param_count = reader.read_u32_le()?;
        assert!(param_count <= PARAM_COUNT as u32);

        // For some reason it always compiles 8 params in but doesn't use all of them, so it has two different counters for
        // the number present and the number actually used. The number present should always be 8.
        let type_count = reader.read_u32_le()?;
        assert_eq!(type_count, PARAM_COUNT as u32);

        let mut param_types = [0u32; PARAM_COUNT];
        for i in 0..type_count {
            param_types[i] = reader.read_u32_le()?;
        }

        let applies_to = if applies_to_something {
            reader.read_i32_le()?
        } else {
            reader.seek(SeekFrom::Current(4))?;
            -1 // TODO: gml::constants::SELF or something, this is self
        };
        let is_relative = reader.read_u32_le()? != 0;

        // Like above, it tells us the number of arg strings here but this should always be 8.
        // If you ever change this, at the very least, make sure arg_count is equal to type_count. Right now I'm asserting that both are 8.
        let arg_count = reader.read_u32_le()?;
        assert_eq!(arg_count, PARAM_COUNT as u32);

        let mut param_strings = Vec::with_capacity(arg_count as usize);
        for _ in 0..arg_count {
            param_strings.push(reader.read_pas_string()?);
        }

        let invert_condition = reader.read_u32_le()? != 0;

        let parameters: Vec<CodeActionParam> = param_strings
            .drain(0..(param_count as usize))
            .enumerate()
            .map(|(i, arg)| match param_types[i] {
                0 => CodeActionParam::Expression(arg),
                1 => CodeActionParam::GML(arg),
                2 => CodeActionParam::Literal(gml::Value::String(arg)),
                3..=14 => CodeActionParam::Literal(gml::Value::Real(arg.parse().unwrap_or(0.0))),

                _ => panic!("i am an invalid id, no this should not be a panic"),
            })
            .collect();

        Ok(CodeAction {
            id,
            parameters,
            applies_to,
            is_condition,
            invert_condition,
            is_relative,
            lib_id,
            action_kind,
            can_be_relative,
            applies_to_something,
            action_type,
            fn_name,
            fn_code,
            param_types,
            param_strings,
        })
    }
}
