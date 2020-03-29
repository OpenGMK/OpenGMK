use crate::gml::runtime::Instruction;
use std::rc::Rc;

pub struct Script {
    pub name: String,
    pub source: String,
    pub compiled: Rc<[Instruction]>,
}
