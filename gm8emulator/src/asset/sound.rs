use crate::game::{audio::WavHandle, string::RCStr};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Sound {
    pub name: RCStr,
    pub handle: Kind,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum Kind {
    Wav(WavHandle),
    None,
}
