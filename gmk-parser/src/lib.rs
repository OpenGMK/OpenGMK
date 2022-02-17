pub mod asset;
pub(crate) mod format;
pub mod gmk;
pub use gmk::Gmk;
mod help_dialog;
pub use help_dialog::HelpDialog;
pub mod rsrc;
mod settings;
pub use settings::Settings;

#[cfg(feature = "rayon")]
pub use rayon_rs as rayon;

#[derive(Copy, Clone, Debug)]
pub enum GameVersion {
    GameMaker8_0,
    GameMaker8_1,
}
