#![allow(dead_code)] // Shut up.

use crate::bytes::{ReadBytes, ReadString, WriteBytes, WriteString};
use crate::game::parser::ParserOptions;
use crate::types::{Dimensions, Version};
use crate::util::{bgra2rgba, rgba2bgra};
use std::io::{self, Seek, SeekFrom};

pub const VERSION1: Version = 710;
pub const VERSION2: Version = 800;

pub struct Background {
    /// The asset name present in GML and the editor.
    pub name: String,

    /// The size of the background image.
    pub size: Dimensions,

    /// The raw RGBA pixeldata.
    pub data: Option<Box<[u8]>>,
}

impl Background {
    pub fn serialize<W>(&self, writer: &mut W) -> io::Result<usize>
    where
        W: io::Write,
    {
        let mut result = writer.write_pas_string(&self.name)?;
        result += writer.write_u32_le(VERSION1 as u32)?;
        result += writer.write_u32_le(VERSION2 as u32)?;
        result += writer.write_u32_le(self.size.width as u32)?;
        result += writer.write_u32_le(self.size.height as u32)?;
        if let Some(data) = &self.data {
            let mut pixeldata = data.clone();
            rgba2bgra(&mut pixeldata);
            result += writer.write(&pixeldata)?;

            result += writer.write_u32_le(pixeldata.len() as u32)?;
            result += writer.write(&pixeldata)?;
        }

        Ok(result)
    }

    pub fn deserialize<B>(bin: B, options: &ParserOptions) -> io::Result<Background>
    where
        B: AsRef<[u8]>,
    {
        let mut reader = io::Cursor::new(bin.as_ref());
        let name = reader.read_pas_string()?;

        if options.strict {
            let version1 = reader.read_u32_le()?;
            let version2 = reader.read_u32_le()?;
            assert_eq!(version1, VERSION1);
            assert_eq!(version2, VERSION2);
        } else {
            reader.seek(SeekFrom::Current(8))?;
        }

        let width = reader.read_u32_le()?;
        let height = reader.read_u32_le()?;
        if width > 0 && height > 0 {
            let data_len = reader.read_u32_le()?;

            // sanity check
            if data_len != (width * height * 4) {
                panic!("Inconsistent pixel data length with dimensions");
            }

            // BGRA -> RGBA
            let pos = reader.position() as usize;
            let len = data_len as usize;
            reader.seek(SeekFrom::Current(len as i64))?;
            let mut buf = reader.get_ref()[pos..pos + len].to_vec();
            bgra2rgba(&mut buf);

            Ok(Background {
                name,
                size: Dimensions { width, height },
                data: Some(buf.into_boxed_slice()),
            })
        } else {
            Ok(Background {
                name,
                size: Dimensions {
                    width: 0,
                    height: 0,
                },
                data: None,
            })
        }
    }
}
