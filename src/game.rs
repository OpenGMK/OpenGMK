pub mod parser;

use crate::assets::*;

pub struct Game {
    pub sprites: Vec<Option<Box<Sprite>>>,
    pub sounds: Vec<Option<Box<Sound>>>,
    pub backgrounds: Vec<Option<Box<Background>>>,
    pub paths: Vec<Option<Box<Path>>>,
    pub scripts: Vec<Option<Box<Script>>>,
    pub fonts: Vec<Option<Box<Font>>>,
    // Timelines
    // Objects
    // Rooms
    pub triggers: Vec<Option<Box<Trigger>>>,
    pub constants: Vec<(String, String)>,
    // Extensions
    pub version: GameVersion,
}

pub enum GameVersion {
    GameMaker80,
    GameMaker81,
}
