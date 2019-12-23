use crate::render::Texture;

pub struct Sprite {
    pub name: String,
    pub frames: Vec<Texture>,
    pub width: u32,
    pub height: u32,
    pub origin_x: i32,
    pub origin_y: i32,
}
