use crate::gml::{Context, runtime::{Instruction, Node, Runtime}, Value};

/// A drag-n-drop action.
pub struct Action {
    /// The original index of this action in its list, starting at 0
    pub index: usize,

    /// The target ID. May be self (-1) or other (-2) or an object or instance id.
    /// A value of None means applies_to_something was false.
    pub target: Option<i32>,

    /// The arguments to be passed to the function or code body
    pub args: Box<[Argument]>,

    /// Whether the "relative" checkbox was used. This is always passed to Context, but usually ignored.
    pub relative: bool,

    /// If this is a question action, this flag means the bool result will be inverted.
    pub invert_condition: bool,

    /// The body of this action to be executed
    pub body: Body,

    /// The 'if' and 'else' actions under this one, if this action is a question.
    pub if_else: Option<(Box<[Action]>, Box<[Action]>)>,
}

pub enum Body {
    Function(fn(&mut Runtime, &mut Context, &[Value]) -> Value),
    Code(Vec<Instruction>),
}

pub enum Argument {
    Constant(Value),
    Expression(Node),
}
