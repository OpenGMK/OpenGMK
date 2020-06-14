use crate::{game::string::RCStr, gml::runtime::Instruction};
use serde::{Deserialize, Serialize};
use std::rc::Rc;

#[derive(Serialize, Deserialize)]
pub struct Script {
    pub name: RCStr,
    pub source: RCStr,
    pub compiled: Rc<[Instruction]>,
}
