use crate::render::Texture;

pub struct Background {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub texture: Option<Texture>,
}
