mod parser;
use crate::assets::*;

pub struct Game {
    pub sprites: Vec<Option<Box<GMSprite>>>,
    pub sounds: Vec<Option<Box<Sound>>>,
    pub backgrounds: Vec<Option<Box<GMBackground>>>,
    pub paths: Vec<Option<Box<Path>>>,
    pub scripts: Vec<Option<Box<GMScript>>>,
    pub fonts: Vec<Option<Box<GMFont>>>,
    // Timelines
    // Objects
    // Rooms

    // Triggers
    // Constants
    // Extensions
}
