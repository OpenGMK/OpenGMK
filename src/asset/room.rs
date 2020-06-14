use crate::{
    game::{string::RCStr, Background, View},
    gml::runtime::Instruction,
    tile::Tile,
    types::{Colour, ID},
};
use serde::{Deserialize, Serialize};
use std::rc::Rc;

#[derive(Clone, Serialize, Deserialize)]
pub struct Room {
    pub name: RCStr,
    pub caption: RCStr,
    pub width: u32,
    pub height: u32,
    pub speed: u32,
    pub persistent: bool,
    pub bg_colour: Colour,
    pub clear_screen: bool,
    pub creation_code: Rc<[Instruction]>,

    pub backgrounds: Rc<Vec<Background>>,
    pub views_enabled: bool,
    pub views: Rc<Vec<View>>,
    pub instances: Rc<Vec<Instance>>,
    pub tiles: Rc<Vec<Tile>>,
}

/// An instance stored in a Room
#[derive(Serialize, Deserialize)]
pub struct Instance {
    pub x: i32,
    pub y: i32,
    pub object: i32,
    pub id: ID,
    pub creation: Rc<[Instruction]>,
}
