use crate::gml::runtime::Instruction;

pub struct Script {
    pub name: String,
    pub source: String,
    pub compiled: Vec<Instruction>,
}
