use crate::render::AtlasRef;
use std::rc::Rc;

#[derive(Clone)]
pub struct Font {
    pub name: Rc<str>,
    pub sys_name: String,
    pub size: u32,
    pub bold: bool,
    pub italic: bool,
    pub first: u32,
    pub last: u32,
    pub atlas_ref: AtlasRef,
    pub tallest_char_height: u32,
    pub chars: Rc<[Character]>,
}

#[derive(Clone, Copy)]
pub struct Character {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub offset: u32,
    pub distance: u32,
}

impl Font {
    pub fn get_char(&self, index: u32) -> Option<Character> {
        if let Some(index) = index.checked_sub(self.first) { self.chars.get(index as usize).copied() } else { None }
    }
}
