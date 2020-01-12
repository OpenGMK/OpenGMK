use super::{ast, Instruction, Value};
use std::collections::HashMap;

pub struct Compiler {
    /// List of identifiers which represent const values
    pub constants: HashMap<String, Value>,

    /// Lookup table of unique field names
    pub fields: Vec<String>,
}

impl Compiler {
    pub fn new(constants_size_hint: usize) -> Self {
        let mut constants = HashMap::with_capacity(constants_size_hint + super::CONSTANTS.len());
        super::CONSTANTS.iter().for_each(|(name, value)| {
            constants.insert(String::from(*name), Value::Real(*value));
        });
        Self {
            constants,
            fields: vec![],
        }
    }

    pub fn compile(source: &str) -> Result<Vec<Instruction>, String> {
        let ast = ast::AST::new(source).map_err(|e| e.message)?;

        let instructions = Vec::new();
        for _node in ast.into_iter() {
            // TODO: this
        }
        Ok(instructions)
    }
}
