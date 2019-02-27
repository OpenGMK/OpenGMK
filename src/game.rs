mod parser;
use crate::assets::Sound;

pub struct Game {
    pub sounds: Vec<Option<Box<Sound>>>,
}
