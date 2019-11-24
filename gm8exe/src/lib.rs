#![allow(clippy::cognitive_complexity)]
#![allow(clippy::unreadable_literal)]

pub mod asset;
pub mod def;
pub mod reader;

mod colour;

#[derive(Copy, Clone, Debug)]
pub enum GameVersion {
    GameMaker8_0,
    GameMaker8_1,
}

pub use colour::Colour;

pub mod deps {
    pub use minio;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
