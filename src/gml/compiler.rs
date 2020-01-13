use super::{ast, Instruction, Value};
use std::collections::HashMap;

pub struct Compiler {
    /// List of identifiers which represent const values
    pub constants: HashMap<String, Value>,

    /// Lookup table of unique field names
    pub fields: Vec<String>,
}

pub enum Error {
    ASTError(String),
    GMLError(String),
    DnDError(String),
}

impl Compiler {
    /// Create a compiler. The size hint indicates how many constants are likely to be entered.
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

    /// Compile the basic components of a drag-and-drop action into a list of Instructions.
    pub fn compile_action(&mut self) -> Result<Vec<Instruction>, Error> {
        // TODO: this
        Ok(vec![])
    }

    pub fn compile(&mut self, source: &str) -> Result<Vec<Instruction>, Error> {
        let ast = ast::AST::new(source).map_err(|e| Error::ASTError(e.message))?;

        let instructions = Vec::new();
        for _node in ast.into_iter() {
            // TODO: this
        }
        Ok(instructions)
    }
}
