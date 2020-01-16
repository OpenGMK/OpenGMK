use super::Value;

#[derive(Debug)]
pub enum Instruction {
    // TODO: GML runtime instructions
    InterpretationError { error: String },
}

/// Node representing one value in an expression.
pub enum Node {
    Literal {
        value: Value,
    },
    Function, // TODO
    Variable, // TODO
    Binary {
        left: Box<Node>,
        right: Box<Node>,
        operator: fn(&Value, &Value) -> Value,
    },
}

pub struct Error {
    pub reason: String,
    // Probably could add more useful things later.
}
