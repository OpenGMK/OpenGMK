use crate::game::{audio::{Mp3Handle, WavHandle}, string::RCStr};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Sound {
    pub name: RCStr,
    pub handle: FileType,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum FileType {
    Mp3(Mp3Handle),
    Wav(WavHandle),
    None,
}
