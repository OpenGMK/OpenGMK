use crate::{
    asset::{assert_ver, Asset, Error, PascalString, ReadChunk, ReadPascalString, WritePascalString},
    GameVersion,
};
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use std::io::{self, Read};

pub const VERSION: u32 = 800;

pub struct Font {
    /// The asset name present in GML and the editor.
    pub name: PascalString,

    /// The name of the source font found on the system.
    pub sys_name: PascalString,

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
    fn deserialize_exe(mut reader: impl Read, version: GameVersion, strict: bool) -> Result<Self, Error> {
        let name = reader.read_pas_string()?;

        let ver = reader.read_u32::<LE>()?;
        if strict {
            assert_ver(ver, VERSION)?;
        }

        let sys_name = reader.read_pas_string()?;
        let size = reader.read_u32::<LE>()?;
        let bold = reader.read_u32::<LE>()? != 0;
        let italic = reader.read_u32::<LE>()? != 0;
        let mut range_start = reader.read_u32::<LE>()?;
        let range_end = reader.read_u32::<LE>()?;

        let (aa_level, charset) = match version {
            GameVersion::GameMaker8_0 => (0, 0),
            GameVersion::GameMaker8_1 => {
                let aa_level = (range_start & 0xFF000000) >> 24;
                let charset = (range_start & 0x00FF0000) >> 16;
                range_start &= 0x0000FFFF;
                (aa_level, charset)
            },
        };

        let mut dmap = [0u32; 0x600];
        for val in dmap.iter_mut() {
            *val = reader.read_u32::<LE>()?;
        }
        let map_width = reader.read_u32::<LE>()?;
        let map_height = reader.read_u32::<LE>()?;
        let len = reader.read_u32::<LE>()? as usize;
        let pixel_map = reader.read_chunk(len)?.into_boxed_slice();

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

    fn serialize_exe(&self, mut writer: impl io::Write, version: GameVersion) -> io::Result<()> {
        writer.write_pas_string(&self.name)?;
        writer.write_u32::<LE>(VERSION)?;
        writer.write_pas_string(&self.sys_name)?;
        writer.write_u32::<LE>(self.size)?;
        writer.write_u32::<LE>(self.bold.into())?;
        writer.write_u32::<LE>(self.italic.into())?;
        match version {
            GameVersion::GameMaker8_0 => writer.write_u32::<LE>(self.range_start)?,
            GameVersion::GameMaker8_1 => writer
                .write_u32::<LE>(self.range_start | ((self.aa_level % 0x100) << 24) | ((self.charset % 0x100) << 16))?,
        }
        writer.write_u32::<LE>(self.range_end)?;
        writer.write_u32::<LE>(self.map_width)?;
        writer.write_u32::<LE>(self.map_height)?;
        writer.write_u32::<LE>(self.pixel_map.len() as u32)?; // TODO: len as u32
        writer.write_all(&self.pixel_map)?;
        Ok(())
    }
}
