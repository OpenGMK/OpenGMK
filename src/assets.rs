pub mod background;
pub mod path;
pub mod script;
pub mod sound;

pub use self::background::Background;
pub use self::path::Path;
pub use self::script::Script;
pub use self::sound::Sound;

use crate::types::{CollisionMap, Dimensions, Point};

pub type GMCodeAction = u32;

pub struct GMFont {
    pub name: String,
    pub sys_name: String,
    pub size: u32,
    pub bold: bool,
    pub italic: bool,
    pub range_start: u32,
    pub range_end: u32,
    pub dmap: Box<[u32; 0x600]>,
    pub image_size: Dimensions,
    pub image_data: Box<[u8]>,
}

pub struct GMSprite {
    pub name: String,
    pub size: Dimensions,
    pub origin: Point,
    pub frame_count: u32,
    pub frames: Option<Vec<Box<[u8]>>>,
    pub colliders: Option<Vec<CollisionMap>>,
    pub per_frame_colliders: bool,
}
