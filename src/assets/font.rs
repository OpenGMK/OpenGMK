use crate::bytes::{ReadBytes, ReadString, WriteBytes, WriteString};
use crate::game::{parser::ParserOptions, GameVersion};
use crate::types::Dimensions;
use crate::types::Version;
use std::io::{self, Seek, SeekFrom};

pub const VERSION: Version = 800;

pub struct Font {
    /// The asset name present in GML and the editor.
    pub name: String,

    /// The name of the source font found on the system.
    pub sys_name: String,

    /// The size of the font in pixels.
    pub size: u32,

    /// Whether the font is bold.
    pub bold: bool,

    /// Whether the font is italic.
    pub italic: bool,

    /// The charcode range start of the font.
    pub range_start: u32,

    /// The charcode range end of the font.
    pub range_end: u32,

    // The charset that was used to generate this font (usually ANSI)
    pub charset: u32,

    // The anti-aliasing level that was used to generate this font
    pub aa_level: u32,

    /// Lookup table for sections of the font's pixel data, relative to a given character's ord value.
    /// A font supports exactly 256 characters, and each character has six values here which are used to draw that character from the pixel data:
    /// x, y, width, height, cursor offset (ie. how far right of the cursor to draw), and cursor distance (ie. how far right to move the cursor after drawing.)
    pub dmap: Box<[u32; 0x600]>,

    /// The size of the cooked RGBA pixeldata.
    pub image_size: Dimensions,

    /// The raw, cooked RGBA pixeldata.
    /// The font is #FFFFFF (white).
    pub image_data: Box<[u8]>,
}

impl Font {
    pub fn serialize<W>(&self, writer: &mut W) -> io::Result<usize>
    where
        W: io::Write,
    {
        let mut result = writer.write_pas_string(&self.name)?;
        result += writer.write_u32_le(VERSION)?;
        result += writer.write_pas_string(&self.sys_name)?;
        result += writer.write_u32_le(self.size)?;
        result += writer.write_u32_le(if self.bold { 1 } else { 0 })?;
        result += writer.write_u32_le(if self.italic { 1 } else { 0 })?;
        result += writer.write_u32_le(self.range_start)?;
        result += writer.write_u32_le(self.range_end)?;
        result += writer.write_u32_le(self.image_size.width)?;
        result += writer.write_u32_le(self.image_size.height)?;
        result += writer.write_u32_le(self.image_size.width * self.image_size.height)?;
        for i in (3..self.image_data.len()).step_by(4) {
            result += writer.write(&[self.image_data[i] as u8])?;
        }

        Ok(result)
    }

    pub fn deserialize<B>(
        bin: B,
        game_ver: &GameVersion,
        options: &ParserOptions,
    ) -> io::Result<Font>
    where
        B: AsRef<[u8]>,
    {
        let mut reader = io::Cursor::new(bin.as_ref());
        let name = reader.read_pas_string()?;

        if options.strict {
            let version = reader.read_u32_le()? as Version;
            assert_eq!(version, VERSION);
        } else {
            reader.seek(SeekFrom::Current(4))?;
        }

        let sys_name = reader.read_pas_string()?;
        let size = reader.read_u32_le()?;
        let bold = reader.read_u32_le()? != 0;
        let italic = reader.read_u32_le()? != 0;
        let mut range_start = reader.read_u32_le()?;
        let range_end = reader.read_u32_le()?;

        let (charset, aa_level) = match game_ver {
            GameVersion::GameMaker80 => (0, 0),
            GameVersion::GameMaker81 => {
                let charset = (range_start & 0xFF000000) >> 24;
                let aa_level = (range_start & 0x00FF0000) >> 16;
                range_start &= 0x0000FFFF;
                (charset, aa_level)
            }
        };

        let dmap = [0u32; 0x600];
        let width = reader.read_u32_le()?;
        let height = reader.read_u32_le()?;
        let len = reader.read_u32_le()? as usize;
        assert_eq!(width as usize * height as usize, len); // Since these values are redundant, make sure they match up.

        // convert f64 map to RGBA data
        // step 1) Fill entire thing with 0xFF (WHITE)
        // step 2) Read every byte into every 4th byte (Alpha)
        let mut pixels = vec![0xFFu8; len * 4];
        let pos = reader.position() as usize;
        reader.seek(SeekFrom::Current(len as i64))?;
        let src = reader.get_ref();
        let mut pixel_pos = 3;
        for i in pos..pos + len {
            pixels[pixel_pos] = src[i];
            pixel_pos += 4;
        }

        Ok(Font {
            name,
            sys_name,
            size,
            bold,
            italic,
            range_start,
            range_end,
            charset,
            aa_level,
            dmap: Box::new(dmap),
            image_size: Dimensions { width, height },
            image_data: pixels.into_boxed_slice(),
        })
    }
}
