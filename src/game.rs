mod parser;
use crate::assets::{GMSound, GMSprite};

pub struct Game {
    pub sounds: Vec<Option<Box<GMSound>>>,
    pub sprites: Vec<Option<Box<GMSprite>>>,
}
