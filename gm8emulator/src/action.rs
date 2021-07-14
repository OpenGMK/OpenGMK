use crate::{
    game::Game,
    gml::{
        self,
        compiler::Compiler,
        mappings,
        runtime::{Instruction, Node},
        Context, Value,
    },
};
use gm8exe::asset::CodeAction;
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, rc::Rc, str};

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
#[derive(Debug, Serialize, Deserialize)]
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
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Tree(Vec<Action>);

/// Body of an action, depending on the action kind.
#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize)]
pub enum GmlBody {
    Function(usize),
    Code(Rc<[Instruction]>),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ReturnType {
    Continue,
    Exit,
}

impl std::fmt::Debug for GmlBody {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            GmlBody::Function(fn_id) => write!(f, "Body::Function({:?})", mappings::FUNCTIONS.index(*fn_id).unwrap().0),
            GmlBody::Code(code) => write!(f, "Body::Code({:?})", code),
        }
    }
}

impl Tree {
    /// Turn a list of gm8exe CodeActions into an Action tree.
    pub fn from_list(list: &[CodeAction], compiler: &mut Compiler) -> Result<Self, String> {
        let mut iter = list.iter().enumerate().peekable();
        let mut output = Vec::new();
        Self::from_iter(&mut iter, compiler, false, false, &mut output)?;
        Ok(Self(output))
    }

    fn from_iter<'a, T>(
        iter: &mut std::iter::Peekable<T>,
        compiler: &mut Compiler,
        single_group: bool, // Read just one action/group of actions?
        stop_at_end_group: bool, // Stop reading after encountering an END_GROUP action?
        output: &mut Vec<Action>,
    ) -> Result<(), String>
    where
        T: Iterator<Item = (usize, &'a CodeAction)>,
    {
        while let Some((i, action)) = iter.next() {
            match action.action_kind {
                kind::NORMAL => {
                    // If the action we got is a condition then immediately parse its if/else bodies from the iterator
                    let if_else = if action.is_condition {
                        let mut if_body = Vec::new();
                        Self::from_iter(iter, compiler, true, true, &mut if_body)?;
                        let mut else_body = Vec::new();
                        if let Some((_, CodeAction { action_kind: kind::ELSE, .. })) = iter.peek() {
                            iter.next(); // skip "else"
                            Self::from_iter(iter, compiler, true, true, &mut else_body)?;
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
                            if let Some(fn_id) =
                                str::from_utf8(&action.fn_name.0).ok().and_then(|n| mappings::FUNCTIONS.get_index(n))
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
                                        body: GmlBody::Function(fn_id),
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
                                    body: GmlBody::Code(compiler.compile(&action.fn_code.0).map_err(|e| e.message)?),
                                    if_else,
                                },
                            });
                        },
                    }
                },

                kind::BEGIN_GROUP => {
                    Self::from_iter(iter, compiler, false, true, output)?;
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
                    Self::from_iter(iter, compiler, true, true, &mut body)?;
                    output.push(Action {
                        index: i,
                        target: if action.applies_to_something { Some(action.applies_to) } else { None },
                        relative: action.is_relative,
                        invert_condition: action.invert_condition,
                        body: Body::Repeat {
                            count: compiler.compile_expression(&action.param_strings[0].0).map_err(|e| e.message)?,
                            body: body.into_boxed_slice(),
                        },
                    });
                },

                kind::VARIABLE => {
                    let mut code = action.param_strings[0].0.to_vec();
                    code.extend_from_slice(if action.is_relative { b"+=" } else { b"=" });
                    code.extend_from_slice(&action.param_strings[1].0);
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
                            body: GmlBody::Code(compiler.compile(&action.param_strings[0].0).map_err(|e| e.message)?),
                            if_else: None,
                        },
                    });
                },

                _ => (),
            }

            // Is it time to stop reading actions?
            if (stop_at_end_group && action.action_kind == kind::END_GROUP) || single_group {
                break
            }
        }

        Ok(())
    }

    fn compile_params(
        compiler: &mut Compiler,
        params: &[gm8exe::asset::PascalString],
        types: &[u32],
        count: usize,
    ) -> Result<Box<[Node]>, String> {
        Ok(params
            .iter()
            .zip(types.iter())
            .take(count)
            .map(|(param, t)| match *t {
                1 | 2 => Ok(Node::Literal { value: Value::Str(param.0.as_ref().into()) }),
                _ => compiler.compile_expression(&param.0),
            })
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.message)?
            .into_boxed_slice())
    }

    pub fn new_from_code(code: Rc<[Instruction]>) -> Rc<RefCell<Self>> {
        let mut tree = Self(Vec::new());
        tree.push_code(code);
        Rc::new(RefCell::new(tree))
    }

    pub fn push_code(&mut self, code: Rc<[Instruction]>) {
        self.0.push(Action {
            index: self.0.len(),
            target: None,
            relative: false,
            invert_condition: false,
            body: Body::Normal { args: Box::new([]), body: GmlBody::Code(code), if_else: None },
        });
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
            if self.scene_change.is_some() {
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
                        ..Default::default()
                    };

                    /*
                    let mut arg_values: [Value; 16] = Default::default();
                    for (dest, src) in arg_values.iter_mut().zip(args.iter()) {
                        *dest = self.eval(src, &mut context)?;
                    }
                    */

                    let mut returned_value = Default::default();
                    match action.target {
                        None | Some(gml::SELF) | Some(gml::OTHER) => {
                            if action.target == Some(gml::OTHER) {
                                context.this = other;
                                context.other = this;
                            }

                            let mut arg_values: [Value; 16] = Default::default();
                            for (dest, src) in arg_values.iter_mut().zip(args.iter()) {
                                *dest = self.eval(src, &mut context)?;
                            }

                            returned_value = match gml_body {
                                GmlBody::Function(f) => self.invoke(*f, &mut context, &arg_values[..args.len()])?,
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
                            context.other = this;
                            let mut iter = self.room.instance_list.iter_by_identity(i);
                            while let Some(instance) = iter.next(&self.room.instance_list) {
                                context.this = instance;

                                let mut arg_values: [Value; 16] = Default::default();
                                for (dest, src) in arg_values.iter_mut().zip(args.iter()) {
                                    *dest = self.eval(src, &mut context)?;
                                }

                                returned_value = match gml_body {
                                    GmlBody::Function(f) => self.invoke(*f, &mut context, &arg_values[..args.len()])?,
                                    GmlBody::Code(code) => {
                                        context.arguments = arg_values;
                                        context.argument_count = args.len();
                                        self.execute(code, &mut context)?;
                                        context.return_value.clone()
                                    },
                                };
                            }
                        },
                    }

                    if let Some((if_body, else_body)) = if_else {
                        let target =
                            if returned_value.is_truthy() != action.invert_condition { if_body } else { else_body };
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
                        ..Default::default()
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
