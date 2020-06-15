use crate::{game::string::RCStr, render::AtlasRef};
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
    pub offset: u32,
    pub distance: u32,
    pub atlas_ref: AtlasRef,
}

impl Font {
    pub fn get_char(&self, index: u32) -> Option<Character> {
        if let Some(index) = index.checked_sub(self.first) { self.chars.get(index as usize).copied() } else { None }
    }
}
