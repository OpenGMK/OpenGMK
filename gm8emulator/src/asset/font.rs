use crate::{asset::Sprite, game::string::RCStr};
use gmio::render::{AtlasRef, Renderer};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Font {
    pub name: RCStr,
    pub sys_name: String,
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
