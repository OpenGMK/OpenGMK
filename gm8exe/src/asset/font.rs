use crate::{
    asset::{assert_ver, Asset, AssetDataError, ReadPascalString, WritePascalString},
    GameVersion,
};

use minio::{ReadPrimitives, WritePrimitives};
use std::io::{self, Seek, SeekFrom};

pub const VERSION: u32 = 800;

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

    /// Lookup table for sections of the font's pixel data, relative to a given character's ASCII numeric value.
    /// A font supports exactly 256 characters, and each character has six values here,
    /// which are used to draw that character from the pixel data:
    /// - x
    /// - y
    /// - width
    /// - height
    /// - cursor offset (ie. how far right of the cursor to draw)
    /// - cursor distance (ie. how far right to move the cursor after drawing.)
    pub dmap: Box<[u32; 0x600]>,

    /// The width of the pixel map.
    pub map_width: u32,

    /// The height of the pixel map.
    pub map_height: u32,

    /// The raw pixel data for this font. It's a map of alpha values for each pixel, 0 to 255.
    pub pixel_map: Box<[u8]>,
}

impl Asset for Font {
    fn deserialize<B>(bytes: B, strict: bool, version: GameVersion) -> Result<Self, AssetDataError>
    where
        B: AsRef<[u8]>,
        Self: Sized,
    {
        let mut reader = io::Cursor::new(bytes.as_ref());
        let name = reader.read_pas_string()?;

        if strict {
            let ver = reader.read_u32_le()?;
            assert_ver(ver, VERSION)?;
        } else {
            reader.seek(SeekFrom::Current(4))?;
        }

        let sys_name = reader.read_pas_string()?;
        let size = reader.read_u32_le()?;
        let bold = reader.read_u32_le()? != 0;
        let italic = reader.read_u32_le()? != 0;
        let mut range_start = reader.read_u32_le()?;
        let range_end = reader.read_u32_le()?;

        let (charset, aa_level) = match version {
            GameVersion::GameMaker8_0 => (0, 0),
            GameVersion::GameMaker8_1 => {
                let charset = (range_start & 0xFF000000) >> 24;
                let aa_level = (range_start & 0x00FF0000) >> 16;
                range_start &= 0x0000FFFF;
                (charset, aa_level)
            },
        };

        let mut dmap = [0u32; 0x600];
        for val in dmap.iter_mut() {
            *val = reader.read_u32_le()?;
        }
        let map_width = reader.read_u32_le()?;
        let map_height = reader.read_u32_le()?;
        let len = reader.read_u32_le()? as usize;

        let pos = reader.position() as usize;
        let pixel_map = match reader.into_inner().get(pos..pos + len) {
            Some(b) => b.to_vec().into_boxed_slice(),
            None => return Err(AssetDataError::MalformedData),
        };

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
            map_width,
            map_height,
            pixel_map,
        })
    }

    fn serialize<W>(&self, writer: &mut W) -> io::Result<usize>
    where
        W: io::Write,
    {
        let mut result = writer.write_pas_string(&self.name)?;
        result += writer.write_u32_le(VERSION)?;
        result += writer.write_pas_string(&self.sys_name)?;
        result += writer.write_u32_le(self.size)?;
        result += writer.write_u32_le(self.bold as u32)?;
        result += writer.write_u32_le(self.italic as u32)?;
        result += writer.write_u32_le(self.range_start)?;
        result += writer.write_u32_le(self.range_end)?;
        result += writer.write_u32_le(self.map_width)?;
        result += writer.write_u32_le(self.map_height)?;
        result += writer.write_u32_le(self.pixel_map.len() as u32)?;
        result += writer.write(&self.pixel_map)?;

        Ok(result)
    }
}
