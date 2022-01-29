use crate::{gml, render::atlas::AtlasRef};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Background {
    pub name: gml::String,
    pub width: u32,
    pub height: u32,
    pub atlas_ref: Option<AtlasRef>,
}
