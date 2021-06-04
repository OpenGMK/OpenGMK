pub mod asset;
pub(crate) mod format;
pub mod gmk;
pub use gmk::Gmk;

#[derive(Copy, Clone, Debug)]
pub enum GameVersion {
    GameMaker8_0,
    GameMaker8_1,
}
