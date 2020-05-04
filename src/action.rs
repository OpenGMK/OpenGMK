use crate::{
    game::Game,
    gml::{
        self,
        compiler::{mappings, Compiler},
        runtime::{Instruction, Node},
        Context, Value,
    },
};
use gm8exe::asset::etc::CodeAction;
use std::{cell::RefCell, rc::Rc};

/// Consts which match those used in GM8
pub mod kind {
    pub const NORMAL: u32 = 0;
    pub const BEGIN_GROUP: u32 = 1;
    pub const END_GROUP: u32 = 2;
    pub const ELSE: u32 = 3;
    pub const EXIT: u32 = 4;
    pub const REPEAT: u32 = 5;
    pub const VARIABLE: u32 = 6;
    pub const CODE: u32 = 7;
}
pub mod execution_type {
    pub const NONE: u32 = 0;
    pub const FUNCTION: u32 = 1;
    pub const CODE: u32 = 2;
}

/// A drag-n-drop action.
#[derive(Debug)]
pub struct Action {
    /// The original index of this action in its list, starting at 0
    pub index: usize,

    /// The target ID. May be self (-1) or other (-2) or an object or instance id.
    /// A value of None means applies_to_something was false.
    pub target: Option<i32>,

    /// Whether the "relative" checkbox was used. This is always passed to Context, but usually ignored.
    pub relative: bool,

    /// If this is a question action, this flag means the bool result will be inverted.
    pub invert_condition: bool,

    /// Body of this action. Body type depends on the action_kind.
    pub body: Body,
}

/// Abstraction for a tree of Actions
/// Note that Vec is necessary here due to functions such as object_event_add and object_event_clear
#[derive(Debug)]
pub struct Tree(Vec<Action>);

/// Body of an action, depending on the action kind.
#[derive(Debug)]
pub enum Body {
    Normal {
        /// The arguments to be passed to the function or code body
        args: Box<[Node]>,

        /// The body of this action to be executed
        body: GmlBody,

        /// The 'if' and 'else' actions under this one, if this action is a question.
        if_else: Option<(Box<[Action]>, Box<[Action]>)>,
    },
    Repeat {
        /// The expression giving the number of times to repeat.
        count: Node,

        /// The tree of actions to repeat.
        body: Box<[Action]>,
    },
    Exit,
}

pub enum GmlBody {
    Function(fn(&mut Game, &mut Context, &[Value]) -> gml::Result<Value>),
    Code(Rc<[Instruction]>),
}

pub enum ReturnType {
    Continue,
    Exit,
}

impl std::fmt::Debug for GmlBody {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            GmlBody::Function(_) => write!(f, "Body::Function(..)"),
            GmlBody::Code(c) => write!(f, "Body::Code({:?})", c),
        }
    }
}

impl Tree {
    /// Turn a list of gm8exe CodeActions into an Action tree.
    pub fn from_list(list: &[CodeAction], compiler: &mut Compiler) -> Result<Self, String> {
        let mut iter = list.iter().enumerate().peekable();
        let mut output = Vec::new();
        Self::from_iter(&mut iter, compiler, false, &mut output)?;
        Ok(Self(output))
    }

    fn from_iter<'a, T>(
        iter: &mut std::iter::Peekable<T>,
        compiler: &mut Compiler,
        single_group: bool,
        output: &mut Vec<Action>,
    ) -> Result<(), String>
    where
        T: Iterator<Item = (usize, &'a CodeAction)>,
    {
        // If we're only iterating a single group of actions, and the first is not a BEGIN_GROUP action,
        // then we only want to collect one action.
        let stop_immediately = if let Some((_, CodeAction { action_kind: kind::BEGIN_GROUP, .. })) = iter.peek() {
            false
        } else {
            single_group
        };

        while let Some((i, action)) = iter.next() {
            match action.action_kind {
                kind::NORMAL => {
                    // If the action we got is a condition then immediately parse its if/else bodies from the iterator
                    let if_else = if action.is_condition {
                        let mut if_body = Vec::new();
                        Self::from_iter(iter, compiler, true, &mut if_body)?;
                        let mut else_body = Vec::new();
                        if let Some((_, CodeAction { action_kind: kind::ELSE, .. })) = iter.peek() {
                            iter.next(); // skip "else"
                            Self::from_iter(iter, compiler, true, &mut else_body)?;
                        }
                        Some((if_body.into_boxed_slice(), else_body.into_boxed_slice()))
                    } else {
                        None
                    };

                    match action.execution_type {
                        // Execution type NONE does nothing, so don't compile anything
                        execution_type::NONE => (),

                        // For the FUNCTION execution type, a kernel function name is provided in the action's fn_name.
                        // This is compiled to a function pointer.
                        execution_type::FUNCTION => {
                            if let Some((_, f_ptr, _)) =
                                mappings::FUNCTIONS.iter().find(|(n, _, _)| n == &action.fn_name)
                            {
                                output.push(Action {
                                    index: i,
                                    target: if action.applies_to_something { Some(action.applies_to) } else { None },
                                    relative: action.is_relative,
                                    invert_condition: action.invert_condition,
                                    body: Body::Normal {
                                        args: Self::compile_params(
                                            compiler,
                                            &action.param_strings,
                                            &action.param_types,
                                            action.param_count,
                                        )?,
                                        body: GmlBody::Function(*f_ptr),
                                        if_else,
                                    },
                                });
                            } else {
                                return Err(format!("Unknown function: {} in action {}", action.fn_name, i))
                            }
                        },

                        // Execution type CODE is a bit special depending on the action kind..
                        execution_type::CODE | _ => {
                            // The action's code is provided by its fn_code, so compile that.
                            output.push(Action {
                                index: i,
                                target: if action.applies_to_something { Some(action.applies_to) } else { None },
                                relative: action.is_relative,
                                invert_condition: action.invert_condition,
                                body: Body::Normal {
                                    args: Self::compile_params(
                                        compiler,
                                        &action.param_strings,
                                        &action.param_types,
                                        action.param_count,
                                    )?,
                                    body: GmlBody::Code(compiler.compile(&action.fn_code).map_err(|e| e.message)?),
                                    if_else,
                                },
                            });
                        },
                    }
                },

                kind::BEGIN_GROUP => {
                    Self::from_iter(iter, compiler, true, output)?;
                },

                kind::EXIT => {
                    output.push(Action {
                        index: i,
                        target: if action.applies_to_something { Some(action.applies_to) } else { None },
                        relative: action.is_relative,
                        invert_condition: action.invert_condition,
                        body: Body::Exit,
                    });
                },

                kind::REPEAT => {
                    let mut body = Vec::new();
                    Self::from_iter(iter, compiler, true, &mut body)?;
                    output.push(Action {
                        index: i,
                        target: if action.applies_to_something { Some(action.applies_to) } else { None },
                        relative: action.is_relative,
                        invert_condition: action.invert_condition,
                        body: Body::Repeat {
                            count: compiler.compile_expression(&action.param_strings[0]).map_err(|e| e.message)?,
                            body: body.into_boxed_slice(),
                        },
                    });
                },

                kind::VARIABLE => {
                    let code = action.param_strings[0].clone()
                        + if action.is_relative { "+=" } else { "=" }
                        + &action.param_strings[1];
                    output.push(Action {
                        index: i,
                        target: if action.applies_to_something { Some(action.applies_to) } else { None },
                        relative: action.is_relative,
                        invert_condition: action.invert_condition,
                        body: Body::Normal {
                            args: Box::new([]),
                            body: GmlBody::Code(compiler.compile(&code).map_err(|e| e.message)?),
                            if_else: None,
                        },
                    });
                },

                kind::CODE => {
                    output.push(Action {
                        index: i,
                        target: if action.applies_to_something { Some(action.applies_to) } else { None },
                        relative: action.is_relative,
                        invert_condition: action.invert_condition,
                        body: Body::Normal {
                            args: Box::new([]),
                            body: GmlBody::Code(compiler.compile(&action.param_strings[0]).map_err(|e| e.message)?),
                            if_else: None,
                        },
                    });
                },

                _ => (),
            }

            // Is it time to stop reading actions?
            if (single_group && action.action_kind == kind::END_GROUP) || stop_immediately {
                break
            }
        }

        Ok(())
    }

    fn compile_params(
        compiler: &mut Compiler,
        params: &[String],
        types: &[u32],
        count: usize,
    ) -> Result<Box<[Node]>, String> {
        Ok(params
            .iter()
            .zip(types.iter())
            .take(count)
            .map(|(param, t)| match *t {
                1 | 2 => Ok(Node::Literal { value: Value::Str(param.as_str().into()) }),
                _ => compiler.compile_expression(param),
            })
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.message)?
            .into_boxed_slice())
    }
}

impl Game {
    /// Executes all the actions in a tree.
    pub fn execute_tree(
        &mut self,
        tree: Rc<RefCell<Tree>>,
        this: usize,
        other: usize,
        event_type: usize,
        event_number: usize,
        as_object: i32,
    ) -> gml::Result<()> {
        self.exec_slice(&tree.borrow().0, this, other, event_type, event_number, as_object)?;
        Ok(())
    }

    fn exec_slice(
        &mut self,
        slice: &[Action],
        this: usize,
        other: usize,
        event_type: usize,
        event_number: usize,
        as_object: i32,
    ) -> gml::Result<ReturnType> {
        for action in slice.iter() {
            if self.room_target.is_some() {
                return Ok(ReturnType::Exit)
            }

            match &action.body {
                Body::Normal { args, body: gml_body, if_else } => {
                    let mut context = Context {
                        this,
                        other,
                        event_action: action.index,
                        relative: action.relative,
                        event_type,
                        event_number,
                        event_object: as_object,
                        arguments: Default::default(),
                        argument_count: 0,
                        locals: Default::default(),
                        return_value: Default::default(),
                    };

                    let mut arg_values: [Value; 16] = Default::default();
                    for (dest, src) in arg_values.iter_mut().zip(args.iter()) {
                        *dest = self.eval(src, &mut context)?;
                    }

                    let mut returned_value = Default::default();
                    match action.target {
                        None | Some(gml::SELF) | Some(gml::OTHER) => {
                            if action.target == Some(gml::OTHER) {
                                context.this = other;
                                context.other = this;
                            }
                            returned_value = match gml_body {
                                GmlBody::Function(f) => f(self, &mut context, &arg_values[..args.len()])?,
                                GmlBody::Code(code) => {
                                    context.arguments = arg_values;
                                    context.argument_count = args.len();
                                    self.execute(code, &mut context)?;
                                    context.return_value
                                },
                            };
                        },
                        Some(i) if i < 0 => (),
                        Some(i) => {
                            if let Some(Some(object)) = self.assets.objects.get(i as usize) {
                                context.other = this;
                                let ids = object.children.clone();
                                let mut iter = self.instance_list.iter_by_identity(ids);
                                while let Some(instance) = iter.next(&self.instance_list) {
                                    context.this = instance;
                                    returned_value = match gml_body {
                                        GmlBody::Function(f) => f(self, &mut context, &arg_values[..args.len()])?,
                                        GmlBody::Code(code) => {
                                            context.arguments = arg_values.clone();
                                            context.argument_count = args.len();
                                            self.execute(code, &mut context)?;
                                            context.return_value.clone()
                                        },
                                    };
                                }
                            }
                        },
                    }

                    if let Some((if_body, else_body)) = if_else {
                        let target =
                            if returned_value.is_true() != action.invert_condition { if_body } else { else_body };
                        match self.exec_slice(target, this, other, event_type, event_number, as_object)? {
                            ReturnType::Continue => (),
                            ReturnType::Exit => return Ok(ReturnType::Exit),
                        }
                    }
                },
                Body::Repeat { count, body } => {
                    let mut context = Context {
                        this,
                        other,
                        event_action: action.index,
                        relative: action.relative,
                        event_type,
                        event_number,
                        event_object: as_object,
                        arguments: Default::default(),
                        argument_count: 0,
                        locals: Default::default(),
                        return_value: Default::default(),
                    };
                    let mut count = i32::from(self.eval(count, &mut context)?);
                    while count > 0 {
                        match self.exec_slice(body, this, other, event_type, event_number, as_object)? {
                            ReturnType::Continue => (),
                            ReturnType::Exit => return Ok(ReturnType::Exit),
                        }
                        count -= 1;
                    }
                },
                Body::Exit => return Ok(ReturnType::Exit),
            }
        }

        Ok(ReturnType::Continue)
    }
}
