use crate::{asset::Sprite, game::string::RCStr};
use encoding_rs::Encoding;
use gmio::render::{AtlasRef, Renderer};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Font {
    pub name: RCStr,
    pub sys_name: RCStr,
    pub charset: u32,
    pub size: u32,
    pub bold: bool,
    pub italic: bool,
    pub first: u32,
    pub last: u32,
    pub tallest_char_height: u32,
    pub chars: Box<[Character]>,
    pub own_graphics: bool, // Does this Font own the graphics associated with it?
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct Character {
    pub offset: i32,
    pub distance: i32,
    pub atlas_ref: AtlasRef,
}

impl Font {
    pub fn get_char(&self, index: u32) -> Option<Character> {
        if let Some(index) = index.checked_sub(self.first) { self.chars.get(index as usize).copied() } else { None }
    }

    pub fn get_encoding(&self, default: &'static Encoding) -> &'static Encoding {
        match self.charset {
            0x00 => encoding_rs::WINDOWS_1252, // ANSI_CHARSET
            0x80 => encoding_rs::SHIFT_JIS,    // SHIFTJIS_CHARSET
            0x81 => encoding_rs::EUC_KR,       // HANGUL_CHARSET
            0x82 => default,                   // JOHAB_CHARSET
            0x86 => encoding_rs::GBK,          // GB2312_CHARSET
            0x88 => encoding_rs::BIG5,         // CHINESEBIG5_CHARSET
            0xA1 => encoding_rs::WINDOWS_1253, // GREEK_CHARSET
            0xA2 => encoding_rs::WINDOWS_1254, // TURKISH_CHARSET
            0xA3 => encoding_rs::WINDOWS_1258, // VIETNAMESE_CHARSET
            0xB1 => encoding_rs::WINDOWS_1255, // HEBREW_CHARSET
            0xB2 => encoding_rs::WINDOWS_1256, // ARABIC_CHARSET
            0xBA => encoding_rs::WINDOWS_1257, // BALTIC_CHARSET
            0xCC => encoding_rs::WINDOWS_1251, // RUSSIAN_CHARSET
            0xDE => encoding_rs::WINDOWS_874,  // THAI_CHARSET
            0xEE => encoding_rs::WINDOWS_1250, // EASTEUROPE_CHARSET
            _ => default,
        }
    }
}

pub fn create_chars_from_sprite(sprite: &Sprite, prop: bool, sep: i32, renderer: &Renderer) -> Box<[Character]> {
    let mut chars = Vec::with_capacity(sprite.frames.len());
    if prop {
        // proportional font, get the left and right bounds of each character
        for frame in &sprite.frames {
            let data = renderer.dump_sprite(&frame.atlas_ref);
            let column_empty =
                |&x: &u32| (0..sprite.height).any(|y| data[(y * sprite.width + x) as usize * 4 + 3] != 0);
            let left_edge = (0..sprite.width).find(column_empty).map(|x| x as i32).unwrap_or(sprite.width as i32 - 1);
            let right_edge = (0..sprite.width).rfind(column_empty).unwrap_or(0) as i32;
            chars.push(Character {
                offset: right_edge + sep - left_edge,
                distance: -left_edge,
                atlas_ref: frame.atlas_ref.clone(),
            });
        }
    } else {
        // non-proportional font, just add them whole
        chars.extend(sprite.frames.iter().map(|f| Character {
            offset: f.width as i32 + sep,
            distance: 0,
            atlas_ref: f.atlas_ref.clone(),
        }));
    }
    chars.into_boxed_slice()
}
