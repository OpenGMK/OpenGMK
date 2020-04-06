use crate::gml::runtime::Instruction;
use std::rc::Rc;

pub struct Script {
    pub name: Rc<str>,
    pub source: Rc<str>,
    pub compiled: Rc<[Instruction]>,
}
