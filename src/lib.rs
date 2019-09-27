pub mod asset;
pub mod def;
pub mod reader;

mod color;

#[derive(Copy, Clone)]
pub enum GameVersion {
    GameMaker8_0,
    GameMaker8_1,
}

pub use color::Color;

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
