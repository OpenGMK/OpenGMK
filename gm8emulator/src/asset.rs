pub mod background;
pub mod font;
pub mod object;
pub mod path;
pub mod room;
pub mod script;
pub mod sound;
pub mod sprite;
pub mod timeline;
pub mod trigger;

pub use background::Background;
pub use font::Font;
pub use object::Object;
pub use path::Path;
pub use room::Room;
pub use script::Script;
pub use sound::Sound;
pub use sprite::Sprite;
pub use timeline::Timeline;
pub use trigger::Trigger;

use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Type {
    Background,
    Font,
    Object,
    Path,
    Room,
    Script,
    Sound,
    Sprite,
    Timeline,
    Trigger,
    Constant,
}

impl Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Background => write!(f, "background"),
            Self::Font => write!(f, "font"),
            Self::Object => write!(f, "object"),
            Self::Path => write!(f, "path"),
            Self::Room => write!(f, "room"),
            Self::Script => write!(f, "script"),
            Self::Sound => write!(f, "sound"),
            Self::Sprite => write!(f, "sprite"),
            Self::Timeline => write!(f, "timeline"),
            Self::Trigger => write!(f, "trigger"),
            Self::Constant => write!(f, "constant"),
        }
    }
}
