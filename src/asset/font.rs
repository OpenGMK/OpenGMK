use crate::render::AtlasRef;
use std::rc::Rc;

pub struct Font {
    pub name: Rc<str>,
    pub sys_name: String,
    pub size: u32,
    pub bold: bool,
    pub italic: bool,
    pub first: u32,
    pub last: u32,
    pub atlas_ref: AtlasRef,
    pub chars: Box<[Character]>,
}

pub struct Character {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub offset: u32,
    pub distance: u32,
}
