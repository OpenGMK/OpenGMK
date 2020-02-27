use crate::gml::{compiler::mappings, Context, runtime::{Instruction, Node, Runtime}, Value};
use gm8exe::asset::etc::CodeAction;

/// Consts which match those used in GM8
pub mod execution_type {
    pub const NONE: u32 = 0;
    pub const FUNCTION: u32 = 1;
    pub const CODE: u32 = 2;
}

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

/// Abstraction for a tree of Actions
/// Note that Vec is necessary here due to functions such as object_event_add and object_event_clear
pub struct Tree(Vec<Action>);

pub enum Body {
    Function(fn(&mut Runtime, &mut Context, &[Value]) -> Value),
    Code(Vec<Instruction>),
}

pub enum Argument {
    Constant(Value),
    Expression(Node),
}

impl Tree {
    /// Turn a list of gm8exe CodeActions into an Action tree.
    pub fn from_list(list: &[CodeAction]) -> Result<Self, String> {
        let mut iter = list.iter().peekable().enumerate();
        let mut output = Vec::new();

        while let Some((i, action)) = iter.next() {
            match action.execution_type {
                execution_type::NONE => (),
                execution_type::FUNCTION => {
                    if let Some((_, f_ptr, _)) = mappings::FUNCTIONS.iter().find(|(n, _, _)| n == &action.fn_name) {
                        output.push(Action {
                            index: i,
                            target: if action.applies_to_something {Some(action.applies_to)} else {None},
                            args: Box::new([]),
                            relative: action.is_relative,
                            invert_condition: action.invert_condition,
                            body: Body::Function(*f_ptr),
                            if_else: None, // TODO: handle action.is_condition
                        });
                    } else {
                        return Err(format!("Unknown function: {} in action {}", action.fn_name, i));
                    }
                },
                execution_type::CODE | _ => {
                    // TODO: handle code actions
                },
            }
        }

        Ok(Self(output))
    }
}
