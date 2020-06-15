use crate::{game::string::RCStr, render::AtlasRef};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Background {
    pub name: RCStr,
    pub width: u32,
    pub height: u32,
    pub atlas_ref: Option<AtlasRef>,
}
