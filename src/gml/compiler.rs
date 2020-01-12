use super::{ast, Instruction, Value};
use std::collections::HashMap;

pub struct Compiler {
    /// List of identifiers which represent const values
    pub constants: HashMap<String, Value>,

    /// Lookup table of unique field names
    pub fields: Vec<String>,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            constants: HashMap::new(),
            fields: vec![],
        }
    }

    pub fn compile(source: &str) -> Result<Vec<Instruction>, String> {
        let my_ast = ast::AST::new(source).map_err(|e| e.message)?; // I can't call it "ast", so...

        let instructions = Vec::new();
        for _node in my_ast.into_iter() {
            // TODO: this
        }
        Ok(instructions)
    }
}
