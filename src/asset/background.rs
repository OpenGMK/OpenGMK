use crate::render::AtlasRef;
use std::rc::Rc;

pub struct Background {
    pub name: Rc<str>,
    pub width: u32,
    pub height: u32,
    pub atlas_ref: Option<AtlasRef>,
}
