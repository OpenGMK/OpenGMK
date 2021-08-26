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

        /// Whether the action is a question
        is_condition: bool,
    },
    Else,
    Repeat {
        /// The expression giving the number of times to repeat.
        count: Node,
    },
    BlockBegin,
    BlockEnd,
    Comment,
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
        let mut output = Vec::new();
        Self::from_slice(list, 0, compiler, &mut output)?;
        Ok(Self(output))
    }

    fn from_slice(
        slice: &[CodeAction],
        start_index: usize,
        compiler: &mut Compiler,
        output: &mut Vec<Action>,
    ) -> Result<(), String> {
        for (i, action) in slice.iter().enumerate().map(|(i, a)| (i + start_index, a)) {
            match action.action_kind {
                kind::NORMAL => {
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
                                        is_condition: action.is_condition,
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
                                    is_condition: action.is_condition,
                                },
                            });
                        },
                    }
                },

                kind::ELSE => {
                    output.push(Action {
                        index: i,
                        target: if action.applies_to_something { Some(action.applies_to) } else { None },
                        relative: action.is_relative,
                        invert_condition: action.invert_condition,
                        body: Body::Else,
                    });
                },

                kind::BEGIN_GROUP => output.push(Action {
                    index: i,
                    target: if action.applies_to_something { Some(action.applies_to) } else { None },
                    relative: action.is_relative,
                    invert_condition: action.invert_condition,
                    body: Body::BlockBegin,
                }),

                kind::END_GROUP => output.push(Action {
                    index: i,
                    target: if action.applies_to_something { Some(action.applies_to) } else { None },
                    relative: action.is_relative,
                    invert_condition: action.invert_condition,
                    body: Body::BlockEnd,
                }),

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
                    output.push(Action {
                        index: i,
                        target: if action.applies_to_something { Some(action.applies_to) } else { None },
                        relative: action.is_relative,
                        invert_condition: action.invert_condition,
                        body: Body::Repeat {
                            count: compiler.compile_expression(&action.param_strings[0].0).map_err(|e| e.message)?,
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
                            is_condition: false,
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
                            is_condition: false,
                        },
                    });
                },

                _ => (),
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
            body: Body::Normal { args: Box::new([]), body: GmlBody::Code(code), is_condition: false },
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
        self.exec_slice(&tree.borrow().0, this, other, event_type, event_number, as_object, false)?;
        Ok(())
    }

    fn skip_actions(slice: &[Action]) -> usize {
        let mut block_depth: u32 = 0;
        for (i, action) in slice.iter().enumerate() {
            match action.body {
                Body::BlockBegin => block_depth += 1,
                Body::BlockEnd => {
                    block_depth = block_depth.saturating_sub(1);
                    if block_depth == 0 {
                        return i + 1
                    }
                },
                Body::Repeat { .. } => (),
                Body::Normal { is_condition, .. } if is_condition => (),
                Body::Normal { .. } | Body::Comment | Body::Exit | Body::Else if block_depth > 0 => (),
                Body::Normal { .. } | Body::Comment | Body::Exit | Body::Else => return i + 1,
            }
        }
        slice.len()
    }

    fn exec_slice(
        &mut self,
        slice: &[Action],
        this: usize,
        other: usize,
        event_type: usize,
        event_number: usize,
        as_object: i32,
        one_block: bool,
    ) -> gml::Result<(ReturnType, usize)> {
        let mut block_depth = 0usize;
        let mut skip_count = 0;
        for (i, action) in slice.iter().enumerate() {
            if self.scene_change.is_some() {
                return Ok((ReturnType::Exit, i))
            }

            if skip_count > 0 {
                skip_count -= 1;
                continue
            }

            match &action.body {
                Body::Normal { args, body: gml_body, is_condition } => {
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

                    if *is_condition {
                        let do_if = returned_value.is_truthy() != action.invert_condition;
                        if do_if {
                            if let Some(target) = slice.get(i + 1..) {
                                match self.exec_slice(target, this, other, event_type, event_number, as_object, true)? {
                                    (ReturnType::Continue, len) => {
                                        skip_count = len;
                                        if let Some(Body::Else) = slice.get(i + len + 1).map(|a| &a.body) {
                                            skip_count +=
                                                1 + slice.get(i + 1 + len + 1..).map(Self::skip_actions).unwrap_or(0);
                                        };
                                    },
                                    (ReturnType::Exit, len) => return Ok((ReturnType::Exit, i + len)),
                                }
                            }
                        } else {
                            skip_count = slice.get(i + 1..).map(Self::skip_actions).unwrap_or(0);
                            if slice
                                .get(i + 1 + skip_count)
                                .map(|action| matches!(action.body, Body::Else))
                                .unwrap_or(false)
                            {
                                if let Some(target) = slice.get(i + 1 + skip_count + 1..) {
                                    match self.exec_slice(
                                        target,
                                        this,
                                        other,
                                        event_type,
                                        event_number,
                                        as_object,
                                        true,
                                    )? {
                                        (ReturnType::Continue, len) => skip_count += 1 + len,
                                        (ReturnType::Exit, len) => {
                                            return Ok((ReturnType::Exit, i + 1 + skip_count + 1 + len))
                                        },
                                    }
                                }
                            }
                        }
                    }
                    if one_block && block_depth == 0 {
                        return Ok((ReturnType::Continue, i + 1 + skip_count))
                    }
                },
                Body::Repeat { count } => {
                    if let Some(body) = slice.get(i + 1..) {
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
                            match self.exec_slice(body, this, other, event_type, event_number, as_object, true)? {
                                (ReturnType::Continue, _) => (),
                                (ReturnType::Exit, len) => return Ok((ReturnType::Exit, len)),
                            }
                            count -= 1;
                        }
                        skip_count = slice.get(i + 1..).map(Self::skip_actions).unwrap_or(0);
                    }
                },
                Body::BlockBegin => block_depth += 1,
                Body::BlockEnd => {
                    block_depth = block_depth.saturating_sub(1);
                    if one_block && block_depth == 0 {
                        return Ok((ReturnType::Continue, i + 1 + skip_count))
                    }
                },
                Body::Exit => return Ok((ReturnType::Exit, i)),
                Body::Else | Body::Comment => (),
            }
        }

        Ok((ReturnType::Continue, slice.len()))
    }
}
