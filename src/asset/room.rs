use crate::{background::Background, gml::runtime::Instruction, tile::Tile, types::{Color, ID}, view::View};
use std::rc::Rc;

#[derive(Clone)]
pub struct Room {
    pub name: Rc<str>,
    pub caption: Rc<str>,
    pub width: u32,
    pub height: u32,
    pub speed: u32,
    pub persistent: bool,
    pub bg_colour: Color,
    pub clear_screen: bool,
    pub creation_code: Rc<[Instruction]>,

    pub backgrounds: Rc<Vec<Background>>,
    pub views_enabled: bool,
    pub views: Rc<Vec<View>>,
    pub instances: Rc<Vec<Instance>>,
    pub tiles: Rc<Vec<Tile>>,
}

/// An instance stored in a Room
pub struct Instance {
    pub x: i32,
    pub y: i32,
    pub object: i32,
    pub id: ID,
    pub creation: Rc<[Instruction]>,
}
