use crate::{
    game::{Background, View},
    gml::{self, runtime::Instruction},
    tile::Tile,
    types::{Colour, ID},
};
use serde::{Deserialize, Serialize};
use std::rc::Rc;

#[derive(Clone, Serialize, Deserialize)]
pub struct Room {
    pub name: gml::String,
    pub caption: gml::String,
    pub width: u32,
    pub height: u32,
    pub speed: u32,
    pub persistent: bool,
    pub bg_colour: Colour,
    pub clear_screen: bool,
    pub creation_code: Result<Rc<[Instruction]>, String>,

    pub backgrounds: Vec<Background>,
    pub views_enabled: bool,
    pub views: Vec<View>,
    pub instances: Vec<Instance>,
    pub tiles: Vec<Tile>,
}

/// An instance stored in a Room
#[derive(Clone, Serialize, Deserialize)]
pub struct Instance {
    pub x: i32,
    pub y: i32,
    pub object: i32,
    pub id: ID,
    pub creation: Result<Rc<[Instruction]>, String>,
    pub xscale: f64,
    pub yscale: f64,
    pub blend: u32,
    pub angle: f64,
}
