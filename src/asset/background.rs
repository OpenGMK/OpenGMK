use crate::render::AtlasRef;

pub struct Background {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub atlas_ref: Option<AtlasRef>,
}
