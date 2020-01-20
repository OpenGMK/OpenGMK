use super::{GameVariable, InstanceVariable, Value};

#[derive(Debug)]
pub enum Instruction {
    // TODO: GML runtime instructions
    RuntimeError { error: String },
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
        script_id: usize,
    },
    Field {
        index: usize,
        array: Option<ArrayAccessor>,
        owner: Box<Node>,
        value: Box<Node>,
    },
    Variable {
        var: InstanceVariable,
        array: Option<ArrayAccessor>,
        owner: Box<Node>,
        value: Box<Node>,
    },
    GameVariable {
        var: GameVariable,
        array: Option<ArrayAccessor>,
        owner: Box<Node>,
        value: Box<Node>,
    },
    Binary {
        left: Box<Node>,
        right: Box<Node>,
        operator: fn(Value, Value) -> Value,
    },
    Unary {
        child: Box<Node>,
        operator: fn(Value) -> Value,
    },
    RuntimeError {
        error: String,
    },
}

/// Represents an array accessor, which can be either 1D or 2D.
/// Variables with 0D arrays, and ones with no array accessor, implicitly refer to [0].
/// Anything beyond a 2D array results in a runtime error.
pub enum ArrayAccessor {
    Single(Box<Node>),
    Double(Box<Node>, Box<Node>),
}

pub struct Error {
    pub reason: String,
    // Probably could add more useful things later.
}
