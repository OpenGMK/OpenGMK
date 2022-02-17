#![cfg_attr(feature = "nightly-docs", feature(doc_cfg))]

pub mod asset;
pub(crate) mod format;
pub mod gmk;
pub use gmk::Gmk;
mod help_dialog;
pub use help_dialog::HelpDialog;
pub mod rsrc;
mod settings;
pub use settings::Settings;

#[cfg_attr(feature = "nightly-docs", doc(cfg(feature = "rayon")))]
#[cfg_attr(not(feature = "nightly-docs"), cfg(feature = "rayon"))]
#[doc(inline)]
pub use rayon_rs as rayon;

#[derive(Copy, Clone, Debug)]
pub enum GameVersion {
    GameMaker8_0,
    GameMaker8_1,
}
