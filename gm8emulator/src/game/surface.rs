use crate::render::atlas::AtlasRef;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Surface {
    pub width: u32,
    pub height: u32,
    pub atlas_ref: AtlasRef,
}
