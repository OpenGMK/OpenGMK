use crate::{
    game::audio::{Mp3Handle, WavHandle},
    gml,
    math::Real,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Sound {
    pub name: gml::String,
    pub handle: FileType,
    pub gml_kind: Real,    // no purpose besides gml function sound_get_kind()
    pub gml_preload: Real, // no purpose besides gml function sound_get_preload()
}

#[derive(Clone, Serialize, Deserialize)]
pub enum FileType {
    Mp3(Mp3Handle),
    Wav(WavHandle),
    None,
}
