#![allow(dead_code)]

use crate::bytes::{ReadBytes, ReadString};
use crate::gml;

use std::io::{self, Seek, SeekFrom};

pub struct CodeAction {
    pub id: u32,
    pub parameters: Vec<CodeActionParam>,

    // TOOD: should enum prob
    pub applies_to: i32,

    pub is_condition: bool,

    pub invert_condition: bool,

    pub is_relative: bool,
}

#[derive(Debug)]
pub enum CodeActionParam {
    Expression(String),
    GML(String),
    Literal(gml::Value),
}

impl CodeAction {
    pub fn deserialize(reader: &mut io::Cursor<&[u8]>) -> io::Result<CodeAction> {
        //let mut reader = io::Cursor::new(bin.as_ref());
        reader.seek(SeekFrom::Current(8))?; // "Skips version id and useless lib id" TODO strict
        let id = reader.read_u32_le()?;
        reader.seek(SeekFrom::Current(8))?; // TODO"Skips the useless "kind" variable and a flag for whether it can be relative"
        let is_condition = reader.read_u32_le()? != 0;
        let applies_to_something = reader.read_u32_le()? != 0;
        reader.seek(SeekFrom::Current(4))?; // Skips the useless "type" var

        // According to Adam, this is redundant. Might be worth checking out. TODO
        let fn_name_len = reader.read_u32_le()? as i64;
        reader.seek(SeekFrom::Current(fn_name_len))?;
        let fn_code_len = reader.read_u32_le()? as i64;
        reader.seek(SeekFrom::Current(fn_code_len))?;

        let param_count = reader.read_u32_le()? as usize;
        assert!(param_count <= 8);

        reader.seek(SeekFrom::Current(4))?; // more version id TODO

        let mut types = [0u32; 8];
        for i in 0..8 {
            types[i] = reader.read_u32_le()?;
        }

        let applies_to = if applies_to_something {
            reader.read_i32_le()?
        } else {
            reader.seek(SeekFrom::Current(4))?;
            -1 // TODO: gml::constnats::SELF or something, this is self
        };
        let is_relative = reader.read_u32_le()? != 0;

        reader.seek(SeekFrom::Current(4))?; // and even more version id! TODO

        let mut args = Vec::with_capacity(param_count);
        for _ in 0..param_count {
            args.push(reader.read_pas_string()?);
        }

        // TODO: comment on this better for the love of god what the fuck
        // It's a bunch of unused strings with 1 character in them, SUPPOSEDLY
        reader.seek(SeekFrom::Current(((8 - param_count) * 5) as i64))?;

        let invert_condition = reader.read_u32_le()? != 0;

        let parameters: Vec<CodeActionParam> = args
            .drain(0..param_count)
            .enumerate()
            .map(|(i, arg)| match types[i] {
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
        })
    }
}
