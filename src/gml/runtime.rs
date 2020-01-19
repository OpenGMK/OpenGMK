use std::rc::Rc;
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
    Function {
        args: Box<[Node]>,
        function: fn(&[Value]) -> Value,
    },
    Script {
        args: Box<[Node]>,
        script: Rc<[Instruction]>,
    },
    Field, // TODO
    Variable, // TODO
    GlobalVariable, // TODO
    Binary {
        left: Box<Node>,
        right: Box<Node>,
        operator: fn(&Value, &Value) -> Value,
    },
    Unary {
        child: Box<Node>,
        operator: fn(&Value) -> Value,
    }

}

pub struct Error {
    pub reason: String,
    // Probably could add more useful things later.
}
