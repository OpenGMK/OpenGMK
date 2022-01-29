pub mod asset;
pub(crate) mod format;
pub mod gmk;
pub use gmk::Gmk;
pub mod rsrc;
pub mod settings;
pub use settings::Settings;

#[derive(Copy, Clone, Debug)]
pub enum GameVersion {
    GameMaker8_0,
    GameMaker8_1,
}
