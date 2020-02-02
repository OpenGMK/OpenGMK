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
        array: ArrayAccessor,
        owner: VarOwner,
    },
    Variable {
        var: InstanceVariable,
        array: ArrayAccessor,
        owner: VarOwner,
    },
    GameVariable {
        var: GameVariable,
        array: ArrayAccessor,
        owner: VarOwner,
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
    None,
    Single(Box<Node>),
    Double(Box<Node>, Box<Node>),
}

/// Represents the owner of a field/variable.
/// If we know at compile time that a variable is owned by a magic value (self, other, global, local)
/// then we can represent it that way in the tree and skip evaluating it during runtime.
pub enum VarOwner {
    Own, // Can't call it Self, that's a Rust keyword. Yeah, I know, sorry.
    Other,
    Global,
    Local,
    Expression(Box<Node>),
}

pub struct Error {
    pub reason: String,
    // Probably could add more useful things later.
}
