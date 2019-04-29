pub mod parser;

use crate::assets::*;
use crate::types::{Color, Dimensions};

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

    pub help_dialog: GameHelpDialog,
    pub last_instance_id: i32, // TODO: type
    pub last_tile_id: i32, // TODO: type
    pub room_order: Vec<u32>, // TODO: type?
}

#[derive(Debug)]
pub struct GameHelpDialog {
    pub bg_color: Color,
    pub new_window: bool,
    pub caption: String,
    pub left: i32,
    pub top: i32,
    pub size: Dimensions,
    pub border: bool,
    pub resizable: bool,
    pub window_on_top: bool,
    pub freeze_game: bool,
    pub info: String,
}

pub enum GameVersion {
    GameMaker80,
    GameMaker81,
}
