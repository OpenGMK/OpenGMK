use crate::gml::{self, runtime::Instruction};
use serde::{Deserialize, Serialize};
use std::rc::Rc;

#[derive(Clone, Serialize, Deserialize)]
pub struct Script {
    pub name: gml::String,
    pub source: gml::String,
    pub compiled: Rc<[Instruction]>,
}
