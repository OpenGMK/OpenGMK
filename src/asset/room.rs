use crate::{background::Background, gml::runtime::Instruction, tile::Tile, types::Color, view::View};

pub struct Room {
    pub name: String,
    pub caption: String,
    pub width: u32,
    pub height: u32,
    pub speed: u32,
    pub persistent: bool,
    pub bg_colour: Color,
    pub clear_screen: bool,
    pub creation_code: Vec<Instruction>,

    pub backgrounds: Vec<Background>,
    pub views_enabled: bool,
    pub views: Vec<View>,
    pub instances: Vec<Instance>,
    pub tiles: Vec<Tile>,
}

/// An instance stored in a Room
pub struct Instance {
    pub x: i32,
    pub y: i32,
    pub object: i32,
    pub id: usize,
    pub creation: Vec<Instruction>,
}
