#![allow(clippy::cognitive_complexity)]
#![allow(clippy::unreadable_literal)]

#[macro_use]
macro_rules! log {
    ($logger: expr, $x: expr) => {
        if let Some(logger) = &$logger {
            logger($x.into());
        }
    };
    ($logger: expr, $format: expr, $($x: expr),*) => {
        if let Some(logger) = &$logger {
            logger(&format!(
                $format,
                $($x),*
            ));
        }
    };
    ($($x:expr,)*) => (log![$($x),*]); // leveraged from vec![]
}

pub mod asset;
pub mod def;
pub mod gamedata;
pub mod reader;
pub mod rsrc;
pub mod settings;
pub mod upx;

mod colour;

use crate::{asset::*, rsrc::WindowsIcon};
use settings::{GameHelpDialog, Settings};

pub struct GameAssets {
    pub triggers: Vec<Option<Box<Trigger>>>,
    pub constants: Vec<Constant>,
    pub extensions: Vec<Extension>,
    pub sprites: Vec<Option<Box<Sprite>>>,
    pub sounds: Vec<Option<Box<Sound>>>,
    pub backgrounds: Vec<Option<Box<Background>>>,
    pub paths: Vec<Option<Box<Path>>>,
    pub scripts: Vec<Option<Box<Script>>>,
    pub fonts: Vec<Option<Box<Font>>>,
    pub timelines: Vec<Option<Box<Timeline>>>,
    pub objects: Vec<Option<Box<Object>>>,
    pub rooms: Vec<Option<Box<Room>>>,
    pub included_files: Vec<IncludedFile>,
    pub version: GameVersion,

    pub dx_dll: Vec<u8>,
    pub icon_data: Vec<WindowsIcon>,
    pub ico_file_raw: Vec<u8>,
    pub help_dialog: GameHelpDialog,
    pub last_instance_id: i32,
    pub last_tile_id: i32,
    pub library_init_strings: Vec<PascalString>,
    pub room_order: Vec<i32>,

    pub settings: Settings,
    pub game_id: u32,
    pub guid: [u32; 4],
}

#[derive(Copy, Clone, Debug)]
pub enum GameVersion {
    GameMaker8_0,
    GameMaker8_1,
}

pub use colour::Colour;

pub mod deps {
    pub use minio;
}
