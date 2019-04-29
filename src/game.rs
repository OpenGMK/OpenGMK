pub mod parser;

use crate::assets::*;

pub struct Game {
    pub sprites: Vec<Option<Box<Sprite>>>,
    pub sounds: Vec<Option<Box<Sound>>>,
    pub backgrounds: Vec<Option<Box<Background>>>,
    pub paths: Vec<Option<Box<Path>>>,
    pub scripts: Vec<Option<Box<Script>>>,
    pub fonts: Vec<Option<Box<Font>>>,
    pub timelines: Vec<Option<Box<Timeline>>>,
    pub objects: Vec<Option<Box<Object>>>,
    pub rooms: Vec<Option<Box<Room>>>,
    pub triggers: Vec<Option<Box<Trigger>>>,
    pub constants: Vec<Constant>,
    // Extensions
    pub version: GameVersion,
}

pub enum GameVersion {
    GameMaker80,
    GameMaker81,
}
