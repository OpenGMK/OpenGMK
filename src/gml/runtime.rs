use super::{GameVariable, InstanceVariable, Value};

pub enum Instruction {
    SetField { accessor: FieldAccessor, value: Node, assignment_type: AssignmentType },
    SetVariable { accessor: VariableAccessor, value: Node, assignment_type: AssignmentType },
    SetGameVariable { accessor: GameVariableAccessor, value: Node, assignment_type: AssignmentType },
    IfElse { cond: Node, if_body: Box<[Instruction]>, else_body: Box<[Instruction]> },
    With { target: Node, body: Box<[Instruction]> },
    RuntimeError { error: String },
}

/// Node representing one value in an expression.
pub enum Node {
    Literal { value: Value },
    Function { args: Box<[Node]>, function: fn(&[Value]) -> Value },
    Script { args: Box<[Node]>, script_id: usize },
    Field { accessor: FieldAccessor },
    Variable { accessor: VariableAccessor },
    GameVariable { accessor: GameVariableAccessor },
    Binary { left: Box<Node>, right: Box<Node>, operator: fn(Value, Value) -> Value },
    Unary { child: Box<Node>, operator: fn(Value) -> Value },
    RuntimeError { error: String },
}

/// Type of assignment.
pub enum AssignmentType {
    Set,
    Add,
    Subtract,
    Multiply,
    Divide,
    BitAnd,
    BitOr,
    BitXor,
}

/// Represents an owned field which can either be read or set.
pub struct FieldAccessor {
    pub index: usize,
    pub array: ArrayAccessor,
    pub owner: InstanceIdentifier,
}

/// Represents an owned field which can either be read or set.
pub struct VariableAccessor {
    pub var: InstanceVariable,
    pub array: ArrayAccessor,
    pub owner: InstanceIdentifier,
}

/// Represents a game variable which can either be read or set.
pub struct GameVariableAccessor {
    pub var: GameVariable,
    pub array: ArrayAccessor,
    pub owner: InstanceIdentifier,
}

/// Represents an array accessor, which can be either 1D or 2D.
/// Variables with 0D arrays, and ones with no array accessor, implicitly refer to [0].
/// Anything beyond a 2D array results in a runtime error.
pub enum ArrayAccessor {
    None,
    Single(Box<Node>),
    Double(Box<Node>, Box<Node>),
}

/// Identifies an instance or multiple instances.
/// If we know at compile time that this represents a magic value (self, other, global, local)
/// then we can represent it that way in the tree and skip evaluating it during runtime.
pub enum InstanceIdentifier {
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
