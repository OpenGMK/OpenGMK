pub mod asset;
pub mod def;

mod byteio;

pub enum GameVersion {
    GameMaker8_0,
    GameMaker8_1,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
