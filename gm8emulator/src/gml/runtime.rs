use crate::{
    asset,
    game::{Game, GetAsset, SceneChange, Version},
    gml::{
        self,
        datetime::DateTime,
        mappings::{self, constants as gml_constants},
        Context, InstanceVariable, Value,
    },
    instance::Field,
    math::Real,
};
use gml_parser::token::Operator;
use serde::{Deserialize, Serialize};
use std::{
    convert::TryFrom,
    fmt::{self, Display},
    time,
};

const DEFAULT_ALARM: i32 = -1;

/// A compiled runtime instruction. Generally represents a line of code.
#[derive(Serialize, Deserialize)]
pub enum Instruction {
    SetField { accessor: FieldAccessor, value: Node },
    SetVariable { accessor: VariableAccessor, value: Node },
    EvalExpression { node: Node },
    IfElse { cond: Node, if_body: Box<[Instruction]>, else_body: Box<[Instruction]> },
    LoopUntil { cond: Node, body: Box<[Instruction]> },
    LoopWhile { cond: Node, body: Box<[Instruction]> },
    LoopFor { cond: Node, body: Box<[Instruction]>, step: Box<[Instruction]> },
    Return { return_type: ReturnType },
    Repeat { count: Node, body: Box<[Instruction]> },
    SetReturnValue { value: Node },
    Switch { input: Node, cases: Box<[(Node, usize)]>, default: Option<usize>, body: Box<[Instruction]> },
    With { target: Node, body: Box<[Instruction]> },
    GlobalVar { fields: Vec<usize> },
    RuntimeError { error: Error },
}

/// Node representing one value in an expression.
#[derive(Clone, Serialize, Deserialize)]
pub enum Node {
    Literal { value: Value },
    Constant { constant_id: usize },
    Function { args: Box<[Node]>, function_id: usize },
    Script { args: Box<[Node]>, script_id: usize },
    ExtensionFunction { args: Box<[Node]>, id: usize },
    Field { accessor: FieldAccessor },
    Variable { accessor: VariableAccessor },
    Binary { left: Box<Node>, right: Box<Node>, operator: BinaryOperator },
    Unary { child: Box<Node>, operator: UnaryOperator },
    RuntimeError { error: Error },
}

/// Represents a compiled binary operator
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum BinaryOperator {
    Add,
    And,
    BitwiseAnd,
    BitwiseOr,
    BinaryShiftLeft,
    BinaryShiftRight,
    BitwiseXor,
    Divide,
    Equal,
    GreaterThan,
    GreaterThanOrEqual,
    IntDivide,
    LessThan,
    LessThanOrEqual,
    Multiply,
    Modulo,
    NotEqual,
    Or,
    Subtract,
    Xor,
}

/// Represents a compiled unary operator - does not include + as this isn't compiled
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum UnaryOperator {
    Neg,
    Not,
    Complement,
}

/// The reason for stopping execution of the current function.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ReturnType {
    Normal,
    Continue,
    Break,
    Exit,
}

/// Represents an owned field which can either be read or set.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FieldAccessor {
    pub index: usize,
    pub array: ArrayAccessor,
    pub owner: InstanceIdentifier,
}

/// Represents an owned field which can either be read or set.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VariableAccessor {
    pub var: InstanceVariable,
    pub array: ArrayAccessor,
    pub owner: InstanceIdentifier,
}

/// Represents an array accessor, which can be either 1D or 2D.
/// Variables with 0D arrays, and ones with no array accessor, implicitly refer to `x[0]`.
/// Anything beyond a 2D array results in a runtime error.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ArrayAccessor {
    None,
    Single(Box<Node>),
    Double(Box<Node>, Box<Node>),
}

/// Identifies an instance or multiple instances.
/// If we know at compile time that this represents a magic value (self, other, global, local)
/// then we can represent it that way in the tree and skip evaluating it during runtime.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum InstanceIdentifier {
    Unknown,
    Own, // Can't call it Self, that's a Rust keyword. Yeah, I know, sorry.
    Other,
    Global,
    Local,
    Expression(Box<Node>),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Error {
    EndOfRoomOrder,
    ExtensionFunctionNotLoaded(usize),
    InvalidOperandsUnary(Operator, Value),
    InvalidOperandsBinary(Operator, Value, Value),
    InvalidUnaryOperator(Operator),
    InvalidBinaryOperator(Operator),
    InvalidAssignment(String),    // string repr. because Expr<'a>
    InvalidArrayAccessor(String), // string repr. because Expr<'a>
    InvalidArrayIndex(i32),
    InvalidDeref(String),    // string repr. because Expr<'a>
    InvalidIndexLhs(String), // string repr. because Expr<'a>
    InvalidIndex(String),    // string repr. because Expr<'a>
    InvalidRoomSpeed(i32),
    InvalidSwitchBody(String), // string repr. because Expr<'a>
    NonexistentAsset(asset::Type, i32),
    ReadOnlyVariable(InstanceVariable),
    UnknownFunction(String),
    UnexpectedASTExpr(String), // string repr. because Expr<'a>
    UninitializedVariable(String, u32),
    UninitializedArgument(usize),
    TooManyArrayDimensions(usize),
    WrongArgumentCount(usize, usize),
    FunctionError(String, String),
    ReplayError(String),
    BadDirectoryError(String),
    ExternalFunction(String, String),
}

impl std::error::Error for Error {}
impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::EndOfRoomOrder => write!(f, "end of room order reached"),
            Self::ExtensionFunctionNotLoaded(id) => write!(f, "extension function {} not loaded", id),
            Self::InvalidOperandsUnary(op, x) => {
                write!(f, "invalid operands {} to {} operator ({1}{})", x.ty_str(), op, x)
            },
            Self::InvalidOperandsBinary(op, x, y) => {
                write!(f, "invalid operands {} and {} to {} operator ({} {2} {})", x.ty_str(), y.ty_str(), op, x, y)
            },
            Self::InvalidUnaryOperator(op) => write!(f, "invalid unary operator {}", op),
            Self::InvalidBinaryOperator(op) => write!(f, "invalid binary operator {}", op),
            Self::InvalidAssignment(expr) => write!(f, "invalid assignment {}", expr),
            Self::InvalidArrayAccessor(expr) => write!(f, "invalid array accessor {}", expr),
            Self::InvalidArrayIndex(idx) => write!(f, "invalid array index {}", idx),
            Self::InvalidDeref(expr) => write!(f, "invalid deref {}", expr),
            Self::InvalidIndex(expr) => write!(f, "invalid index {}", expr),
            Self::InvalidIndexLhs(expr) => write!(f, "invalid index lhs {}", expr),
            Self::InvalidRoomSpeed(value) => write!(f, "invalid room_speed {}", value),
            Self::InvalidSwitchBody(expr) => write!(f, "invalid switch body {}", expr),
            Self::NonexistentAsset(ty, id) => write!(f, "nonexistent asset id {} ({})", id, ty),
            Self::ReadOnlyVariable(v) => write!(
                f,
                "read-only variable {}",
                gml::mappings::INSTANCE_VARIABLES.iter().find(|(_, x)| v == x).map(|(x, _)| x).unwrap()
            ),
            Self::UnknownFunction(fname) => write!(f, "unknown function \"{}\"", fname),
            Self::UnexpectedASTExpr(expr) => write!(f, "unexpected AST expr {}", expr),
            Self::UninitializedVariable(v, i) => {
                if *i == 0 {
                    write!(f, "uninitialized variable \"{}\"", v)
                } else {
                    write!(f, "uninitialized variable \"{}[{}]\"", v, *i)
                }
            },
            Self::UninitializedArgument(n) => write!(f, "uninitialized argument #{}", n),
            Self::TooManyArrayDimensions(n) => write!(f, "too many array dimensions ({})", n),
            Self::WrongArgumentCount(exp, got) => write!(f, "wrong argument count (expected: {}, got: {})", exp, got),
            Self::FunctionError(fname, s) => write!(f, "{}: {}", fname, s),
            Self::ReplayError(s) => write!(f, "{}", s),
            Self::BadDirectoryError(s) => write!(f, "cannot encode working directory {} with current encoding", s),
            Self::ExternalFunction(s, e) => write!(f, "failed to call external function \"{}\": {}", s, e),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
enum Target {
    Single(Option<usize>),
    Objects(i32),
    All,
    Global,
    Local,
}

impl fmt::Debug for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Instruction::SetField { accessor, value } => write!(f, "SetField({:?}, {:?})", accessor, value),
            Instruction::SetVariable { accessor, value } => write!(f, "SetVariable({:?}, {:?})", accessor, value),
            Instruction::EvalExpression { node } => write!(f, "EvalExpression({:?})", node),
            Instruction::IfElse { cond, if_body, else_body } => {
                write!(f, "IfElse({:?}, if={:?}, else={:?}", cond, if_body, else_body)
            },
            Instruction::LoopUntil { cond, body } => write!(f, "LoopUntil({:?}, {:?})", cond, body),
            Instruction::LoopWhile { cond, body } => write!(f, "LoopWhile({:?}, {:?})", cond, body),
            Instruction::LoopFor { cond, body, step } => write!(f, "LoopFor({:?}, {:?}, {:?})", cond, body, step),
            Instruction::Return { return_type } => write!(f, "Return({:?})", return_type),
            Instruction::Repeat { count, body } => write!(f, "Repeat({:?}, {:?})", count, body),
            Instruction::SetReturnValue { value } => write!(f, "SetReturnValue({:?})", value),
            Instruction::Switch { input, cases, default, body } => {
                write!(f, "Switch({:?}, cases={:?}, default={:?}, {:?}", input, cases, default, body)
            },
            Instruction::With { target, body } => write!(f, "With({:?}, {:?})", target, body),
            Instruction::GlobalVar { fields } => write!(f, "GlobalVar({:?})", fields),
            Instruction::RuntimeError { error } => write!(f, "RuntimeError({:?})", error),
        }
    }
}

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Node::Literal { value } => match value {
                Value::Real(r) => write!(f, "{:?}", r),
                Value::Str(s) => write!(f, "{:?}", s),
            },
            Node::Constant { constant_id } => write!(f, "<constant {:?}>", constant_id),
            Node::Function { args, function_id } => {
                write!(f, "<function {:?}: {:?}>", mappings::FUNCTIONS.index(*function_id).unwrap().0, args)
            },
            Node::Script { args, script_id } => write!(f, "<script {:?}: {:?}>", script_id, args),
            Node::ExtensionFunction { args, id } => write!(f, "<extfn {:?}: {:?}>", id, args),
            Node::Field { accessor } => write!(f, "<field: {:?}>", accessor),
            Node::Variable { accessor } => write!(f, "<variable: {:?}>", accessor),
            Node::Binary { left, right, operator } => write!(f, "<binary {:?}: {:?}, {:?}>", operator, left, right),
            Node::Unary { child, operator } => write!(f, "<unary {:?}: {:?}>", operator, child),
            Node::RuntimeError { error } => write!(f, "<error: {:?}>", error),
        }
    }
}

impl BinaryOperator {
    pub fn call(&self, lhs: Value, rhs: Value) -> gml::Result<Value> {
        let f = match self {
            Self::Add => Value::add,
            Self::And => Value::bool_and,
            Self::BitwiseAnd => Value::bitand,
            Self::BitwiseOr => Value::bitor,
            Self::BinaryShiftLeft => Value::shl,
            Self::BinaryShiftRight => Value::shr,
            Self::BitwiseXor => Value::bitxor,
            Self::Divide => Value::div,
            Self::Equal => Value::gml_eq,
            Self::GreaterThan => Value::gml_gt,
            Self::GreaterThanOrEqual => Value::gml_gte,
            Self::IntDivide => Value::intdiv,
            Self::LessThan => Value::gml_lt,
            Self::LessThanOrEqual => Value::gml_lte,
            Self::Multiply => Value::mul,
            Self::Modulo => Value::modulo,
            Self::NotEqual => Value::gml_ne,
            Self::Or => Value::bool_or,
            Self::Subtract => Value::sub,
            Self::Xor => Value::bool_xor,
        };
        f(lhs, rhs)
    }
}

impl UnaryOperator {
    pub fn call(&self, value: Value) -> gml::Result<Value> {
        let f = match self {
            Self::Neg => Value::neg,
            Self::Not => Value::not,
            Self::Complement => Value::complement,
        };
        f(value)
    }
}

impl Game {
    pub fn invoke(&mut self, function_id: usize, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        mappings::FUNCTIONS.index(function_id).unwrap().1.invoke(self, context, args)
    }

    pub fn execute(&mut self, instructions: &[Instruction], context: &mut Context) -> gml::Result<ReturnType> {
        for instruction in instructions.iter() {
            match self.exec_instruction(instruction, context)? {
                ReturnType::Normal => (),
                r => return Ok(r),
            }
        }
        Ok(ReturnType::Normal)
    }

    fn exec_instruction(&mut self, instruction: &Instruction, context: &mut Context) -> gml::Result<ReturnType> {
        match instruction {
            Instruction::SetField { accessor, value } => {
                let target = self.get_target(context, &accessor.owner, self.globalvars.contains(&accessor.index))?;
                let array_index = self.get_array_index(&accessor.array, context)?;
                let value = self.eval(value, context)?;
                context.return_value = value.clone();
                match target {
                    Target::Single(None) => (),
                    Target::Single(Some(instance)) => {
                        self.set_instance_field(instance, accessor.index, array_index, value);
                    },
                    Target::Objects(index) => {
                        let mut iter = self.room.instance_list.iter_by_identity(index);
                        while let Some(instance) = iter.next(&self.room.instance_list) {
                            self.set_instance_field(instance, accessor.index, array_index, value.clone());
                        }
                    },
                    Target::All => {
                        let mut iter = self.room.instance_list.iter_by_insertion();
                        while let Some(instance) = iter.next(&self.room.instance_list) {
                            self.set_instance_field(instance, accessor.index, array_index, value.clone());
                        }
                    },
                    Target::Global => {
                        if let Some(field) = self.globals.fields.get_mut(&accessor.index) {
                            field.set(array_index, value)
                        } else {
                            self.globals.fields.insert(accessor.index, Field::new(array_index, value));
                        }
                    },
                    Target::Local => {
                        if let Some(field) = context.locals.fields.get_mut(&accessor.index) {
                            field.set(array_index, value)
                        } else {
                            context.locals.fields.insert(accessor.index, Field::new(array_index, value));
                        }
                    },
                }
            },
            Instruction::SetVariable { accessor, value } => {
                let target = self.get_target(context, &accessor.owner, false)?;
                let array_index = self.get_array_index(&accessor.array, context)?;
                let value = self.eval(value, context)?;
                context.return_value = value.clone();
                match target {
                    Target::Single(None) => (),
                    Target::Single(Some(instance)) => {
                        self.set_instance_var(instance, &accessor.var, array_index, value, context)?;
                    },
                    Target::Objects(index) => {
                        let mut iter = self.room.instance_list.iter_by_identity(index);
                        while let Some(instance) = iter.next(&self.room.instance_list) {
                            self.set_instance_var(instance, &accessor.var, array_index, value.clone(), context)?;
                        }
                    },
                    Target::All => {
                        let mut iter = self.room.instance_list.iter_by_insertion();
                        while let Some(instance) = iter.next(&self.room.instance_list) {
                            self.set_instance_var(instance, &accessor.var, array_index, value.clone(), context)?;
                        }
                    },
                    Target::Global => {
                        if let Some(field) = self.globals.vars.get_mut(&accessor.var) {
                            field.set(array_index, value)
                        } else {
                            self.globals.vars.insert(accessor.var, Field::new(array_index, value));
                        }
                    },
                    Target::Local => {
                        if let Some(field) = context.locals.vars.get_mut(&accessor.var) {
                            field.set(array_index, value)
                        } else {
                            context.locals.vars.insert(accessor.var, Field::new(array_index, value));
                        }
                    },
                }
            },
            Instruction::EvalExpression { node } => {
                context.return_value = self.eval(node, context)?;
            },
            Instruction::IfElse { cond, if_body, else_body } => {
                let return_type = if self.eval(cond, context)?.is_truthy() {
                    self.execute(if_body, context)
                } else {
                    self.execute(else_body, context)
                }?;
                if return_type != ReturnType::Normal {
                    return Ok(return_type)
                }
            },
            Instruction::LoopUntil { cond, body } => loop {
                match self.execute(body, context)? {
                    ReturnType::Normal => (),
                    ReturnType::Continue => continue,
                    ReturnType::Break => break,
                    ReturnType::Exit => return Ok(ReturnType::Exit),
                }
                if self.eval(cond, context)?.is_truthy() {
                    break
                }
            },
            Instruction::LoopWhile { cond, body } => {
                while self.eval(cond, context)?.is_truthy() {
                    match self.execute(body, context)? {
                        ReturnType::Normal => (),
                        ReturnType::Continue => continue,
                        ReturnType::Break => break,
                        ReturnType::Exit => return Ok(ReturnType::Exit),
                    }
                }
            },
            Instruction::LoopFor { cond, body, step } => {
                while self.eval(cond, context)?.is_truthy() {
                    match self.execute(body, context)? {
                        ReturnType::Normal => {
                            self.execute(step, context)?;
                        },
                        ReturnType::Continue => {
                            self.execute(step, context)?;
                            continue
                        },
                        ReturnType::Break => break,
                        ReturnType::Exit => return Ok(ReturnType::Exit),
                    }
                }
            },
            Instruction::Return { return_type } => return Ok(*return_type),
            Instruction::Repeat { count, body } => {
                let mut count = self.eval(count, context)?.round();
                while count > 0 {
                    match self.execute(body, context)? {
                        ReturnType::Normal => (),
                        ReturnType::Continue => continue,
                        ReturnType::Break => break,
                        ReturnType::Exit => return Ok(ReturnType::Exit),
                    }
                    count -= 1;
                }
            },
            Instruction::SetReturnValue { value } => {
                context.return_value = self.eval(value, context)?;
            },
            Instruction::Switch { input, cases, default, body } => {
                let input = self.eval(input, context)?;
                for (cond, start) in cases.iter() {
                    if self.eval(cond, context)?.almost_equals(&input) {
                        return Ok(match self.execute(&body[*start..], context)? {
                            ReturnType::Break => ReturnType::Normal,
                            x => x,
                        })
                    }
                }
                if let Some(start) = default {
                    return Ok(match self.execute(&body[*start..], context)? {
                        ReturnType::Break => ReturnType::Normal,
                        x => x,
                    })
                }
            },
            Instruction::With { target, body } => {
                let old_this = context.this;
                let old_other = context.other;

                let target_id = i32::from(self.eval(target, context)?);
                context.other = context.this;

                match target_id {
                    gml::SELF | gml::UNSPECIFIED => {
                        if self.execute(body, context)? == ReturnType::Exit {
                            context.other = old_other;
                            return Ok(ReturnType::Exit)
                        }
                    },
                    gml::OTHER => {
                        context.this = old_other;
                        if self.execute(body, context)? == ReturnType::Exit {
                            context.this = old_this;
                            context.other = old_other;
                            return Ok(ReturnType::Exit)
                        }
                    },
                    gml::ALL => {
                        let mut iter = self.room.instance_list.iter_by_insertion();
                        while let Some(instance) = iter.next(&self.room.instance_list) {
                            context.this = instance;
                            match self.execute(body, context)? {
                                ReturnType::Normal => (),
                                ReturnType::Continue => continue,
                                ReturnType::Break => break,
                                ReturnType::Exit => {
                                    context.this = old_this;
                                    context.other = old_other;
                                    return Ok(ReturnType::Exit)
                                },
                            }
                        }
                    },
                    i if i < 0 => (),
                    i if i < 100_000 => {
                        let mut iter = self.room.instance_list.iter_by_identity(i);
                        while let Some(instance) = iter.next(&self.room.instance_list) {
                            context.this = instance;
                            match self.execute(body, context)? {
                                ReturnType::Normal => (),
                                ReturnType::Continue => continue,
                                ReturnType::Break => break,
                                ReturnType::Exit => {
                                    context.this = old_this;
                                    context.other = old_other;
                                    return Ok(ReturnType::Exit)
                                },
                            }
                        }
                    },
                    i => {
                        if let Some(instance) = self.room.instance_list.get_by_instid(i) {
                            context.this = instance;
                            match self.execute(body, context)? {
                                ReturnType::Exit => {
                                    context.this = old_this;
                                    context.other = old_other;
                                    return Ok(ReturnType::Exit)
                                },
                                _ => (),
                            }
                        }
                    },
                }

                context.this = old_this;
                context.other = old_other;
            },
            Instruction::GlobalVar { fields } => {
                self.globalvars.extend(fields);
                for &field in fields {
                    self.globals.fields.entry(field).or_insert(Field::new(0, Default::default()));
                }
            },
            Instruction::RuntimeError { error } => return Err(error.clone()),
        }

        Ok(ReturnType::Normal)
    }

    pub fn eval(&mut self, node: &Node, context: &mut Context) -> gml::Result<Value> {
        match node {
            Node::Literal { value } => Ok(value.clone()),
            Node::Constant { constant_id } => {
                if let Some(value) = self.constants.get(*constant_id) {
                    Ok(value.clone())
                } else {
                    Err(gml::Error::NonexistentAsset(asset::Type::Constant, *constant_id as i32))
                }
            },
            Node::Function { args, function_id } => {
                let mut arg_values: [Value; 16] = Default::default();
                for (src, dest) in args.iter().zip(arg_values.iter_mut()) {
                    *dest = self.eval(src, context)?;
                }
                self.invoke(*function_id, context, &arg_values[..args.len()])
            },
            Node::Script { args, script_id } => {
                if let Some(Some(script)) = self.assets.scripts.get(*script_id) {
                    let instructions = script.compiled.clone();

                    let mut arg_values: [Value; 16] = Default::default();
                    for (src, dest) in args.iter().zip(arg_values.iter_mut()) {
                        *dest = self.eval(src, context)?;
                    }

                    let mut new_context = Context::copy_with_args(context, arg_values, args.len());
                    self.execute(&instructions, &mut new_context)?;
                    Ok(new_context.return_value)
                } else {
                    Err(Error::NonexistentAsset(asset::Type::Script, *script_id as i32))
                }
            },
            Node::ExtensionFunction { args, id } => {
                let mut arg_values: [Value; 16] = Default::default();
                for (src, dest) in args.iter().zip(arg_values.iter_mut()) {
                    *dest = self.eval(src, context)?;
                }

                self.run_extension_function(*id, Context::copy_with_args(context, arg_values, args.len()))
            },
            Node::Field { accessor } => {
                let target = self.get_target(context, &accessor.owner, self.globalvars.contains(&accessor.index))?;
                let array_index = self.get_array_index(&accessor.array, context)?;
                match target {
                    Target::Single(None) if self.uninit_fields_are_zero => Ok(Default::default()),
                    Target::Single(None) => Err(Error::UninitializedVariable(
                        self.compiler.get_field_name(accessor.index).unwrap(),
                        array_index,
                    )),
                    Target::Single(Some(instance)) => self.get_instance_field(instance, accessor.index, array_index),
                    Target::Objects(index) => {
                        if let Some(instance) =
                            self.room.instance_list.iter_by_identity(index).next(&self.room.instance_list)
                        {
                            self.get_instance_field(instance, accessor.index, array_index)
                        } else {
                            if self.uninit_fields_are_zero {
                                Ok(Default::default())
                            } else {
                                Err(Error::UninitializedVariable(
                                    self.compiler.get_field_name(accessor.index).unwrap(),
                                    array_index,
                                ))
                            }
                        }
                    },
                    Target::All => {
                        if let Some(instance) =
                            self.room.instance_list.iter_by_insertion().next(&self.room.instance_list)
                        {
                            self.get_instance_field(instance, accessor.index, array_index)
                        } else {
                            if self.uninit_fields_are_zero {
                                Ok(Default::default())
                            } else {
                                Err(Error::UninitializedVariable(
                                    self.compiler.get_field_name(accessor.index).unwrap(),
                                    array_index,
                                ))
                            }
                        }
                    },
                    Target::Global => match self.globals.fields.get(&accessor.index).and_then(|x| x.get(array_index)) {
                        Some(i) => Ok(i),
                        None => {
                            if self.uninit_fields_are_zero {
                                Ok(Default::default())
                            } else {
                                return Err(Error::UninitializedVariable(
                                    self.compiler.get_field_name(accessor.index).unwrap(),
                                    array_index,
                                ))
                            }
                        },
                    },
                    Target::Local => {
                        match context.locals.fields.get(&accessor.index).and_then(|x| x.get(array_index)) {
                            Some(i) => Ok(i),
                            None => {
                                if self.uninit_fields_are_zero {
                                    Ok(Default::default())
                                } else {
                                    return Err(Error::UninitializedVariable(
                                        self.compiler.get_field_name(accessor.index).unwrap(),
                                        array_index,
                                    ))
                                }
                            },
                        }
                    },
                }
            },
            Node::Variable { accessor } => {
                let target = self.get_target(context, &accessor.owner, false)?;
                let array_index = self.get_array_index(&accessor.array, context)?;
                match target {
                    Target::Single(None) if self.uninit_fields_are_zero => Ok(Default::default()),
                    Target::Single(None) => Err(Error::UninitializedVariable(
                        String::from(mappings::INSTANCE_VARIABLES.iter().find(|(_, x)| x == &accessor.var).unwrap().0),
                        array_index,
                    )),
                    Target::Single(Some(instance)) => {
                        self.get_instance_var(instance, &accessor.var, array_index, context)
                    },
                    Target::Objects(index) => {
                        if let Some(instance) =
                            self.room.instance_list.iter_by_identity(index).next(&self.room.instance_list)
                        {
                            self.get_instance_var(instance, &accessor.var, array_index, context)
                        } else {
                            if self.uninit_fields_are_zero {
                                Ok(Default::default())
                            } else {
                                Err(Error::UninitializedVariable(
                                    String::from(
                                        mappings::INSTANCE_VARIABLES
                                            .iter()
                                            .find(|(_, x)| x == &accessor.var)
                                            .unwrap()
                                            .0,
                                    ),
                                    array_index,
                                ))
                            }
                        }
                    },
                    Target::All => {
                        if let Some(instance) =
                            self.room.instance_list.iter_by_insertion().next(&self.room.instance_list)
                        {
                            self.get_instance_var(instance, &accessor.var, array_index, context)
                        } else {
                            if self.uninit_fields_are_zero {
                                Ok(Default::default())
                            } else {
                                Err(Error::UninitializedVariable(
                                    String::from(
                                        mappings::INSTANCE_VARIABLES
                                            .iter()
                                            .find(|(_, x)| x == &accessor.var)
                                            .unwrap()
                                            .0,
                                    ),
                                    array_index,
                                ))
                            }
                        }
                    },
                    Target::Global => match self.globals.vars.get(&accessor.var).and_then(|x| x.get(array_index)) {
                        Some(i) => Ok(i),
                        None => {
                            if self.uninit_fields_are_zero {
                                Ok(Default::default())
                            } else {
                                return Err(Error::UninitializedVariable(
                                    String::from(
                                        mappings::INSTANCE_VARIABLES
                                            .iter()
                                            .find(|(_, x)| x == &accessor.var)
                                            .unwrap()
                                            .0,
                                    ),
                                    array_index,
                                ))
                            }
                        },
                    },
                    Target::Local => match context.locals.vars.get(&accessor.var).and_then(|x| x.get(array_index)) {
                        Some(i) => Ok(i),
                        None => {
                            if self.uninit_fields_are_zero {
                                Ok(Default::default())
                            } else {
                                return Err(Error::UninitializedVariable(
                                    String::from(
                                        mappings::INSTANCE_VARIABLES
                                            .iter()
                                            .find(|(_, x)| x == &accessor.var)
                                            .unwrap()
                                            .0,
                                    ),
                                    array_index,
                                ))
                            }
                        },
                    },
                }
            },
            Node::Binary { left, right, operator } => {
                operator.call(self.eval(left, context)?, self.eval(right, context)?)
            },
            Node::Unary { child, operator } => operator.call(self.eval(child, context)?),
            Node::RuntimeError { error } => Err(error.clone()),
        }
    }

    // Resolves an ArrayAccessor to an index (u32)
    fn get_array_index(&mut self, accessor: &ArrayAccessor, context: &mut Context) -> gml::Result<u32> {
        match accessor {
            ArrayAccessor::None => Ok(0),
            ArrayAccessor::Single(node) => {
                let index = self.eval(node, context)?.round();
                if index < 0 || index >= 32000 { Err(Error::InvalidArrayIndex(index)) } else { Ok(index as u32) }
            },
            ArrayAccessor::Double(node1, node2) => {
                let index1 = self.eval(node1, context)?.round();
                let index2 = self.eval(node2, context)?.round();
                if index1 < 0 || index1 >= 32000 {
                    Err(Error::InvalidArrayIndex(index1))
                } else if index2 < 0 || index2 >= 32000 {
                    Err(Error::InvalidArrayIndex(index2))
                } else {
                    Ok(((index1 * 32000) + index2) as u32)
                }
            },
        }
    }

    // Get a field value from an instance
    fn get_instance_field(&self, instance: usize, field_id: usize, array_index: u32) -> gml::Result<Value> {
        if let Some(Some(value)) =
            self.room.instance_list.get(instance).fields.borrow().get(&field_id).map(|field| field.get(array_index))
        {
            Ok(value)
        } else {
            if self.uninit_fields_are_zero {
                Ok(Value::Real(Real::from(0.0)))
            } else {
                Err(Error::UninitializedVariable(self.compiler.get_field_name(field_id).unwrap(), array_index))
            }
        }
    }

    // Set a field on an instance
    fn set_instance_field(&self, instance: usize, field_id: usize, array_index: u32, value: Value) {
        let mut fields = self.room.instance_list.get(instance).fields.borrow_mut();
        if let Some(field) = fields.get_mut(&field_id) {
            field.set(array_index, value)
        } else {
            fields.insert(field_id, Field::new(array_index, value));
        }
    }

    // Get an instance variable from an instance, converted into a Value
    pub fn get_instance_var(
        &self,
        instance_handle: usize,
        var: &InstanceVariable,
        array_index: u32,
        context: &Context,
    ) -> gml::Result<Value> {
        let instance = self.room.instance_list.get(instance_handle);

        match var {
            InstanceVariable::X => Ok(instance.x.get().into()),
            InstanceVariable::Y => Ok(instance.y.get().into()),
            InstanceVariable::Xprevious => Ok(instance.xprevious.get().into()),
            InstanceVariable::Yprevious => Ok(instance.yprevious.get().into()),
            InstanceVariable::Xstart => Ok(instance.xstart.get().into()),
            InstanceVariable::Ystart => Ok(instance.ystart.get().into()),
            InstanceVariable::Hspeed => Ok(instance.hspeed.get().into()),
            InstanceVariable::Vspeed => Ok(instance.vspeed.get().into()),
            InstanceVariable::Direction => Ok(instance.direction.get().into()),
            InstanceVariable::Speed => Ok(instance.speed.get().into()),
            InstanceVariable::Friction => Ok(instance.friction.get().into()),
            InstanceVariable::Gravity => Ok(instance.gravity.get().into()),
            InstanceVariable::GravityDirection => Ok(instance.gravity_direction.get().into()),
            InstanceVariable::ObjectIndex => Ok(instance.object_index.get().into()),
            InstanceVariable::Id => Ok(instance.id.get().into()),
            InstanceVariable::Alarm => match instance.alarms.borrow().get(&array_index) {
                Some(&i) => Ok(i.into()),
                _ => Ok(DEFAULT_ALARM.into()),
            },
            InstanceVariable::Solid => Ok(instance.solid.get().into()),
            InstanceVariable::Visible => Ok(instance.visible.get().into()),
            InstanceVariable::Persistent => Ok(instance.persistent.get().into()),
            InstanceVariable::Depth => Ok(instance.depth.get().into()),
            InstanceVariable::BboxLeft => {
                let sprite = self.get_instance_mask_sprite(instance_handle);
                instance.update_bbox(sprite);
                Ok(instance.bbox_left.get().into())
            },
            InstanceVariable::BboxRight => {
                let sprite = self.get_instance_mask_sprite(instance_handle);
                instance.update_bbox(sprite);
                Ok(instance.bbox_right.get().into())
            },
            InstanceVariable::BboxTop => {
                let sprite = self.get_instance_mask_sprite(instance_handle);
                instance.update_bbox(sprite);
                Ok(instance.bbox_top.get().into())
            },
            InstanceVariable::BboxBottom => {
                let sprite = self.get_instance_mask_sprite(instance_handle);
                instance.update_bbox(sprite);
                Ok(instance.bbox_bottom.get().into())
            },
            InstanceVariable::SpriteIndex => Ok(instance.sprite_index.get().into()),
            InstanceVariable::ImageIndex => Ok(instance.image_index.get().into()),
            InstanceVariable::ImageSingle => {
                if instance.image_speed.get() == Real::from(0.0) {
                    Ok(instance.image_index.get().into())
                } else {
                    Ok(Value::from(-1i32))
                }
            },
            InstanceVariable::ImageNumber => match self.get_instance_sprite(instance_handle) {
                Some(sprite) => Ok(sprite.frames.len().into()),
                None => Ok(Value::from(0i32)),
            },
            InstanceVariable::SpriteWidth => {
                if let Some(sprite) = self.get_instance_sprite(instance_handle) {
                    let width: Real = sprite.width.into();
                    Ok((instance.image_xscale.get() * width).into())
                } else {
                    Ok(Value::from(0.0))
                }
            },
            InstanceVariable::SpriteHeight => {
                if let Some(sprite) = self.get_instance_sprite(instance_handle) {
                    let height: Real = sprite.height.into();
                    Ok((instance.image_yscale.get() * height).into())
                } else {
                    Ok(Value::from(0.0))
                }
            },
            InstanceVariable::SpriteXoffset => {
                if let Some(sprite) = self.get_instance_sprite(instance_handle) {
                    Ok(sprite.origin_x.into())
                } else {
                    Ok(Value::from(0.0))
                }
            },
            InstanceVariable::SpriteYoffset => {
                if let Some(sprite) = self.get_instance_sprite(instance_handle) {
                    Ok(sprite.origin_y.into())
                } else {
                    Ok(Value::from(0.0))
                }
            },
            InstanceVariable::ImageXscale => Ok(instance.image_xscale.get().into()),
            InstanceVariable::ImageYscale => Ok(instance.image_yscale.get().into()),
            InstanceVariable::ImageAngle => Ok(instance.image_angle.get().into()),
            InstanceVariable::ImageAlpha => Ok(instance.image_alpha.get().into()),
            InstanceVariable::ImageBlend => Ok(instance.image_blend.get().into()),
            InstanceVariable::ImageSpeed => Ok(instance.image_speed.get().into()),
            InstanceVariable::MaskIndex => Ok(instance.mask_index.get().into()),
            InstanceVariable::PathIndex => Ok(instance.path_index.get().into()),
            InstanceVariable::PathPosition => Ok(instance.path_position.get().into()),
            InstanceVariable::PathPositionprevious => Ok(instance.path_positionprevious.get().into()),
            InstanceVariable::PathSpeed => Ok(instance.path_speed.get().into()),
            InstanceVariable::PathScale => Ok(instance.path_scale.get().into()),
            InstanceVariable::PathOrientation => Ok(instance.path_orientation.get().into()),
            InstanceVariable::PathEndAction => Ok(instance.path_endaction.get().into()),
            InstanceVariable::TimelineIndex => Ok(instance.timeline_index.get().into()),
            InstanceVariable::TimelinePosition => Ok(instance.timeline_position.get().into()),
            InstanceVariable::TimelineSpeed => Ok(instance.timeline_speed.get().into()),
            InstanceVariable::TimelineRunning => Ok(instance.timeline_running.get().into()),
            InstanceVariable::TimelineLoop => Ok(instance.timeline_loop.get().into()),
            InstanceVariable::ArgumentRelative => Ok(context.relative.into()),
            InstanceVariable::Argument0 => self.get_argument(context, 0),
            InstanceVariable::Argument1 => self.get_argument(context, 1),
            InstanceVariable::Argument2 => self.get_argument(context, 2),
            InstanceVariable::Argument3 => self.get_argument(context, 3),
            InstanceVariable::Argument4 => self.get_argument(context, 4),
            InstanceVariable::Argument5 => self.get_argument(context, 5),
            InstanceVariable::Argument6 => self.get_argument(context, 6),
            InstanceVariable::Argument7 => self.get_argument(context, 7),
            InstanceVariable::Argument8 => self.get_argument(context, 8),
            InstanceVariable::Argument9 => self.get_argument(context, 9),
            InstanceVariable::Argument10 => self.get_argument(context, 10),
            InstanceVariable::Argument11 => self.get_argument(context, 11),
            InstanceVariable::Argument12 => self.get_argument(context, 12),
            InstanceVariable::Argument13 => self.get_argument(context, 13),
            InstanceVariable::Argument14 => self.get_argument(context, 14),
            InstanceVariable::Argument15 => self.get_argument(context, 15),
            InstanceVariable::Argument => self.get_argument(context, array_index as usize),
            InstanceVariable::ArgumentCount => Ok(context.argument_count.into()),
            InstanceVariable::Room => Ok(self.room.id.into()),
            InstanceVariable::RoomFirst => match self.room_order.get(0) {
                Some(room) => Ok((*room).into()),
                None => Err(Error::EndOfRoomOrder),
            },
            InstanceVariable::RoomLast => match self.room_order.get(self.room_order.len() - 1) {
                Some(room) => Ok((*room).into()),
                None => Err(Error::EndOfRoomOrder),
            },
            InstanceVariable::TransitionKind => Ok(self.transition_kind.into()),
            InstanceVariable::TransitionSteps => Ok(self.transition_steps.into()),
            InstanceVariable::Score => Ok(self.score.into()),
            InstanceVariable::Lives => Ok(self.lives.into()),
            InstanceVariable::Health => Ok(self.health.into()),
            InstanceVariable::GameId => Ok(self.game_id.into()),
            InstanceVariable::WorkingDirectory => {
                // get cwd in WTF-8
                let os_cwd = std::env::current_dir().unwrap();
                // try to read as UTF-8
                let mut cwd =
                    os_cwd.to_str().ok_or(gml::Error::BadDirectoryError(os_cwd.to_string_lossy().into_owned()))?;
                // trim if on windows
                if cfg!(target_os = "windows") {
                    cwd = cwd.trim_start_matches("\\\\?\\");
                }
                // try to encode with current encoding
                // TODO: maybe try and get the short path name on windows?
                self.encode_str_maybe(cwd)
                    .map(|x| x.into_owned().into())
                    .ok_or(gml::Error::BadDirectoryError(cwd.to_string()))
            },
            InstanceVariable::TempDirectory => Ok(self.temp_directory.clone().into()),
            InstanceVariable::ProgramDirectory => Ok(self.program_directory.clone().into()),
            InstanceVariable::InstanceCount => Ok(self.room.instance_list.count_all().into()),
            InstanceVariable::InstanceId => Ok(self.room.instance_list.instance_at(array_index as _).into()),
            InstanceVariable::RoomWidth => Ok(self.room.width.into()),
            InstanceVariable::RoomHeight => Ok(self.room.height.into()),
            InstanceVariable::RoomCaption => Ok(self.room.caption.clone().into()),
            InstanceVariable::RoomSpeed => Ok(self.room.speed.into()),
            InstanceVariable::RoomPersistent => Ok(self.room.persistent.into()),
            InstanceVariable::BackgroundColor => Ok(self.room.colour.as_decimal().into()),
            InstanceVariable::BackgroundShowcolor => Ok(self.room.show_colour.into()),
            InstanceVariable::BackgroundVisible => {
                Ok(self.room.backgrounds.get(array_index as usize).unwrap_or(&self.room.backgrounds[0]).visible.into())
            },
            InstanceVariable::BackgroundForeground => Ok(self
                .room
                .backgrounds
                .get(array_index as usize)
                .unwrap_or(&self.room.backgrounds[0])
                .is_foreground
                .into()),
            InstanceVariable::BackgroundIndex => Ok(self
                .room
                .backgrounds
                .get(array_index as usize)
                .unwrap_or(&self.room.backgrounds[0])
                .background_id
                .into()),
            InstanceVariable::BackgroundX => {
                Ok(self.room.backgrounds.get(array_index as usize).unwrap_or(&self.room.backgrounds[0]).x_offset.into())
            },
            InstanceVariable::BackgroundY => {
                Ok(self.room.backgrounds.get(array_index as usize).unwrap_or(&self.room.backgrounds[0]).y_offset.into())
            },
            InstanceVariable::BackgroundWidth => {
                let index =
                    self.room.backgrounds.get(array_index as usize).unwrap_or(&self.room.backgrounds[0]).background_id;
                if let Some(bg) = self.assets.backgrounds.get_asset(index) { Ok(bg.width.into()) } else { Ok(0.into()) }
            },
            InstanceVariable::BackgroundHeight => {
                let index =
                    self.room.backgrounds.get(array_index as usize).unwrap_or(&self.room.backgrounds[0]).background_id;
                if let Some(bg) = self.assets.backgrounds.get_asset(index) {
                    Ok(bg.height.into())
                } else {
                    Ok(0.into())
                }
            },
            InstanceVariable::BackgroundHtiled => Ok(self
                .room
                .backgrounds
                .get(array_index as usize)
                .unwrap_or(&self.room.backgrounds[0])
                .tile_horizontal
                .into()),
            InstanceVariable::BackgroundVtiled => Ok(self
                .room
                .backgrounds
                .get(array_index as usize)
                .unwrap_or(&self.room.backgrounds[0])
                .tile_vertical
                .into()),
            InstanceVariable::BackgroundXscale => {
                Ok(self.room.backgrounds.get(array_index as usize).unwrap_or(&self.room.backgrounds[0]).xscale.into())
            },
            InstanceVariable::BackgroundYscale => {
                Ok(self.room.backgrounds.get(array_index as usize).unwrap_or(&self.room.backgrounds[0]).yscale.into())
            },
            InstanceVariable::BackgroundHspeed => {
                Ok(self.room.backgrounds.get(array_index as usize).unwrap_or(&self.room.backgrounds[0]).hspeed.into())
            },
            InstanceVariable::BackgroundVspeed => {
                Ok(self.room.backgrounds.get(array_index as usize).unwrap_or(&self.room.backgrounds[0]).vspeed.into())
            },
            InstanceVariable::BackgroundBlend => {
                Ok(self.room.backgrounds.get(array_index as usize).unwrap_or(&self.room.backgrounds[0]).blend.into())
            },
            InstanceVariable::BackgroundAlpha => {
                Ok(self.room.backgrounds.get(array_index as usize).unwrap_or(&self.room.backgrounds[0]).alpha.into())
            },
            InstanceVariable::ViewEnabled => Ok(self.room.views_enabled.into()),
            InstanceVariable::ViewCurrent => Ok(self.view_current.into()),
            InstanceVariable::ViewVisible => {
                Ok(self.room.views.get(array_index as usize).unwrap_or(&self.room.views[0]).visible.into())
            },
            InstanceVariable::ViewXview => {
                Ok(self.room.views.get(array_index as usize).unwrap_or(&self.room.views[0]).source_x.into())
            },
            InstanceVariable::ViewYview => {
                Ok(self.room.views.get(array_index as usize).unwrap_or(&self.room.views[0]).source_y.into())
            },
            InstanceVariable::ViewWview => {
                Ok(self.room.views.get(array_index as usize).unwrap_or(&self.room.views[0]).source_w.into())
            },
            InstanceVariable::ViewHview => {
                Ok(self.room.views.get(array_index as usize).unwrap_or(&self.room.views[0]).source_h.into())
            },
            InstanceVariable::ViewXport => {
                Ok(self.room.views.get(array_index as usize).unwrap_or(&self.room.views[0]).port_x.into())
            },
            InstanceVariable::ViewYport => {
                Ok(self.room.views.get(array_index as usize).unwrap_or(&self.room.views[0]).port_y.into())
            },
            InstanceVariable::ViewWport => {
                Ok(self.room.views.get(array_index as usize).unwrap_or(&self.room.views[0]).port_w.into())
            },
            InstanceVariable::ViewHport => {
                Ok(self.room.views.get(array_index as usize).unwrap_or(&self.room.views[0]).port_h.into())
            },
            InstanceVariable::ViewAngle => {
                Ok(self.room.views.get(array_index as usize).unwrap_or(&self.room.views[0]).angle.into())
            },
            InstanceVariable::ViewHborder => {
                Ok(self.room.views.get(array_index as usize).unwrap_or(&self.room.views[0]).follow_hborder.into())
            },
            InstanceVariable::ViewVborder => {
                Ok(self.room.views.get(array_index as usize).unwrap_or(&self.room.views[0]).follow_vborder.into())
            },
            InstanceVariable::ViewHspeed => {
                Ok(self.room.views.get(array_index as usize).unwrap_or(&self.room.views[0]).follow_hspeed.into())
            },
            InstanceVariable::ViewVspeed => {
                Ok(self.room.views.get(array_index as usize).unwrap_or(&self.room.views[0]).follow_vspeed.into())
            },
            InstanceVariable::ViewObject => {
                Ok(self.room.views.get(array_index as usize).unwrap_or(&self.room.views[0]).follow_target.into())
            },
            InstanceVariable::MouseX => Ok(self.get_mouse_in_room().0.into()),
            InstanceVariable::MouseY => Ok(self.get_mouse_in_room().1.into()),
            InstanceVariable::MouseButton => Ok(f64::from(self.input.mouse_button()).into()),
            InstanceVariable::MouseLastbutton => Ok(f64::from(self.input.mouse_lastbutton()).into()),
            InstanceVariable::KeyboardKey => Ok(f64::from(self.input.keyboard_key()).into()),
            InstanceVariable::KeyboardLastkey => Ok(f64::from(self.input.keyboard_lastkey()).into()),
            InstanceVariable::KeyboardLastchar => todo!("keyboard_lastchar getter"),
            InstanceVariable::KeyboardString => todo!("keyboard_string getter"),
            InstanceVariable::CursorSprite => Ok(self.cursor_sprite.into()),
            InstanceVariable::ShowScore => Ok(self.score_capt_d.into()),
            InstanceVariable::ShowLives => Ok(self.lives_capt_d.into()),
            InstanceVariable::ShowHealth => Ok(self.health_capt_d.into()),
            InstanceVariable::CaptionScore => Ok(self.score_capt.clone().into()),
            InstanceVariable::CaptionLives => Ok(self.lives_capt.clone().into()),
            InstanceVariable::CaptionHealth => Ok(self.health_capt.clone().into()),
            InstanceVariable::Fps => Ok(self.fps.into()),
            InstanceVariable::CurrentTime => {
                // GM uses GetTickCount, which has a resolution of *roughly* 16ms.
                if let Some(nanos) = self.spoofed_time_nanos {
                    // When we spoof, it only goes up once per frame anyway, so we can keep it as is.
                    Ok(((nanos / 1_000_000) as u32).into())
                } else {
                    // In realtime, it's probably more accurate to force it to increase in 16ms increments.
                    Ok(time::SystemTime::now()
                        .duration_since(time::UNIX_EPOCH)
                        .map(|x| x.as_millis() as u32 & 0xFFFFFFF0)
                        .unwrap_or(0)
                        .into())
                }
            },
            InstanceVariable::CurrentYear => Ok(DateTime::now_or_nanos(self.spoofed_time_nanos).year().into()),
            InstanceVariable::CurrentMonth => Ok(DateTime::now_or_nanos(self.spoofed_time_nanos).month().into()),
            InstanceVariable::CurrentDay => Ok(DateTime::now_or_nanos(self.spoofed_time_nanos).day().into()),
            InstanceVariable::CurrentWeekday => Ok(DateTime::now_or_nanos(self.spoofed_time_nanos).weekday().into()),
            InstanceVariable::CurrentHour => Ok(DateTime::now_or_nanos(self.spoofed_time_nanos).hour().into()),
            InstanceVariable::CurrentMinute => Ok(DateTime::now_or_nanos(self.spoofed_time_nanos).minute().into()),
            InstanceVariable::CurrentSecond => Ok(DateTime::now_or_nanos(self.spoofed_time_nanos).second().into()),
            InstanceVariable::EventType => Ok(context.event_type.into()),
            InstanceVariable::EventNumber => Ok(context.event_number.into()),
            InstanceVariable::EventObject => Ok(context.event_object.into()),
            InstanceVariable::EventAction => Ok(context.event_action.into()),
            InstanceVariable::SecureMode => Ok(gml::FALSE.into()),
            InstanceVariable::DebugMode => Ok(gml::FALSE.into()),
            InstanceVariable::ErrorOccurred => Ok(self.error_occurred.into()),
            InstanceVariable::ErrorLast => Ok(self.error_last.clone().into()),
            InstanceVariable::GamemakerStandard => Ok(gml::TRUE.into()), // yeah!
            InstanceVariable::GamemakerVersion => Ok(match self.gm_version {
                // the docs claim these range from 800-809, 810-819. they don't.
                Version::GameMaker8_0 => 800f64.into(),
                Version::GameMaker8_1 => 810f64.into(),
            }),
            InstanceVariable::OsType => Ok(gml::GM81_OS_TYPE.into()),
            InstanceVariable::OsDevice => Ok(gml::GM81_OS_DEVICE.into()),
            InstanceVariable::OsVersion => Ok((-1f64).into()),
            InstanceVariable::OsBrowser => Ok(gml_constants::BROWSER_NOT_A_BROWSER.into()),
            InstanceVariable::BrowserWidth => Ok((-1f64).into()),
            InstanceVariable::BrowserHeight => Ok((-1f64).into()),
            InstanceVariable::DisplayAa => Ok(14f64.into()), // bitfield - 2x/4x/8x AA is 14
            InstanceVariable::AsyncLoad => Ok((-1f64).into()),
        }
    }

    // Set an instance variable on an instance
    pub fn set_instance_var(
        &mut self,
        instance_handle: usize,
        var: &InstanceVariable,
        array_index: u32,
        value: Value,
        context: &mut Context,
    ) -> gml::Result<()> {
        let instance = self.room.instance_list.get(instance_handle);

        match var {
            InstanceVariable::X => {
                let v: Real = value.into();
                if v != instance.x.get() {
                    instance.bbox_is_stale.set(true);
                    instance.x.set(v);
                }
            },
            InstanceVariable::Y => {
                let v: Real = value.into();
                if v != instance.y.get() {
                    instance.bbox_is_stale.set(true);
                    instance.y.set(v);
                }
            },
            InstanceVariable::Xprevious => instance.xprevious.set(value.into()),
            InstanceVariable::Yprevious => instance.yprevious.set(value.into()),
            InstanceVariable::Xstart => instance.xstart.set(value.into()),
            InstanceVariable::Ystart => instance.ystart.set(value.into()),
            InstanceVariable::Hspeed => instance.set_hspeed(value.into()),
            InstanceVariable::Vspeed => instance.set_vspeed(value.into()),
            InstanceVariable::Direction => instance.set_direction(value.into()),
            InstanceVariable::Speed => instance.set_speed(value.into()),
            InstanceVariable::Friction => instance.friction.set(value.into()),
            InstanceVariable::Gravity => instance.gravity.set(value.into()),
            InstanceVariable::GravityDirection => instance.gravity_direction.set(value.into()),
            InstanceVariable::Alarm => {
                instance.alarms.borrow_mut().insert(array_index, value.into());
            },
            InstanceVariable::Solid => instance.solid.set(value.is_truthy()),
            InstanceVariable::Visible => instance.visible.set(value.is_truthy()),
            InstanceVariable::Persistent => instance.persistent.set(value.is_truthy()),
            InstanceVariable::Depth => instance.depth.set(value.into()),
            InstanceVariable::SpriteIndex => {
                let v: i32 = value.into();

                if v != instance.sprite_index.get() {
                    instance.bbox_is_stale.set(true);
                    instance.sprite_index.set(v);
                }

                let frame_count =
                    if let Some(sprite) = self.assets.sprites.get_asset(v) { sprite.frames.len() as f64 } else { 0.0 };
                if frame_count <= instance.image_index.get().floor().into() {
                    instance.image_index.set(Real::from(0.0));
                }
            },
            InstanceVariable::ImageIndex => {
                instance.image_index.set(value.into());
            },
            InstanceVariable::ImageSingle => {
                let img = Real::from(value);
                if img >= Real::from(0.0) {
                    instance.image_index.set(img);
                    instance.image_speed.set(Real::from(0.0));
                } else {
                    instance.image_speed.set(Real::from(1.0));
                }
            },
            InstanceVariable::ImageXscale => {
                let v: Real = value.into();
                if v != instance.image_xscale.get() {
                    instance.bbox_is_stale.set(true);
                    instance.image_xscale.set(v);
                }
            },
            InstanceVariable::ImageYscale => {
                let v: Real = value.into();
                if v != instance.image_yscale.get() {
                    instance.bbox_is_stale.set(true);
                    instance.image_yscale.set(v);
                }
            },
            InstanceVariable::ImageAngle => {
                let v: Real = value.into();
                if v != instance.image_angle.get() {
                    instance.bbox_is_stale.set(true);
                    instance.image_angle.set(v);
                }
            },
            InstanceVariable::ImageAlpha => instance.image_alpha.set(value.into()),
            InstanceVariable::ImageBlend => instance.image_blend.set(value.into()),
            InstanceVariable::ImageSpeed => instance.image_speed.set(value.into()),
            InstanceVariable::MaskIndex => {
                let v: i32 = value.into();
                if v != instance.mask_index.get() {
                    instance.bbox_is_stale.set(true);
                    instance.mask_index.set(v);
                }
            },
            InstanceVariable::PathPosition => {
                let new_value = Real::from(value).max(Real::from(0.0)).min(Real::from(1.0));
                instance.path_position.set(new_value);
            },
            InstanceVariable::PathPositionprevious => instance.path_positionprevious.set(value.into()),
            InstanceVariable::PathSpeed => instance.path_speed.set(value.into()),
            InstanceVariable::PathScale => instance.path_scale.set(value.into()),
            InstanceVariable::PathOrientation => instance.path_orientation.set(value.into()),
            InstanceVariable::PathEndAction => instance.path_endaction.set(value.into()),
            InstanceVariable::TimelineIndex => instance.timeline_index.set(value.into()),
            InstanceVariable::TimelinePosition => instance.timeline_position.set(value.into()),
            InstanceVariable::TimelineSpeed => instance.timeline_speed.set(value.into()),
            InstanceVariable::TimelineRunning => instance.timeline_running.set(value.is_truthy()),
            InstanceVariable::TimelineLoop => instance.timeline_loop.set(value.is_truthy()),
            InstanceVariable::Argument0 => self.set_argument(context, 0, value)?,
            InstanceVariable::Argument1 => self.set_argument(context, 1, value)?,
            InstanceVariable::Argument2 => self.set_argument(context, 2, value)?,
            InstanceVariable::Argument3 => self.set_argument(context, 3, value)?,
            InstanceVariable::Argument4 => self.set_argument(context, 4, value)?,
            InstanceVariable::Argument5 => self.set_argument(context, 5, value)?,
            InstanceVariable::Argument6 => self.set_argument(context, 6, value)?,
            InstanceVariable::Argument7 => self.set_argument(context, 7, value)?,
            InstanceVariable::Argument8 => self.set_argument(context, 8, value)?,
            InstanceVariable::Argument9 => self.set_argument(context, 9, value)?,
            InstanceVariable::Argument10 => self.set_argument(context, 10, value)?,
            InstanceVariable::Argument11 => self.set_argument(context, 11, value)?,
            InstanceVariable::Argument12 => self.set_argument(context, 12, value)?,
            InstanceVariable::Argument13 => self.set_argument(context, 13, value)?,
            InstanceVariable::Argument14 => self.set_argument(context, 14, value)?,
            InstanceVariable::Argument15 => self.set_argument(context, 15, value)?,
            InstanceVariable::Argument => self.set_argument(context, array_index as usize, value)?,
            InstanceVariable::Room => self.scene_change = Some(SceneChange::Room(value.into())),
            InstanceVariable::TransitionKind => self.transition_kind = value.into(),
            InstanceVariable::TransitionSteps => self.transition_steps = value.into(),
            InstanceVariable::Score => self.score = value.into(),
            InstanceVariable::Lives => {
                let old_lives = self.lives;
                self.lives = value.into();
                if old_lives > 0 && self.lives <= 0 {
                    self.run_other_event(6)?;
                }
            },
            InstanceVariable::Health => {
                let old_health = self.health;
                self.health = value.into();
                if old_health > 0.into() && self.health <= 0.into() {
                    self.run_other_event(9)?;
                }
            },
            InstanceVariable::RoomCaption => {
                self.room.caption = value.into();
            },
            InstanceVariable::RoomSpeed => {
                let speed: i32 = value.into();
                if speed <= 0 {
                    return Err(Error::InvalidRoomSpeed(speed))
                }
                self.room.speed = speed as _
            },
            InstanceVariable::RoomPersistent => self.room.persistent = value.is_truthy(),
            InstanceVariable::BackgroundColor => self.room.colour = (value.round() as u32).into(),
            InstanceVariable::BackgroundShowcolor => self.room.show_colour = value.is_truthy(),
            InstanceVariable::BackgroundVisible => match self.room.backgrounds.get_mut(array_index as usize) {
                Some(background) => background.visible = value.is_truthy(),
                None => self.room.backgrounds[0].visible = value.is_truthy(),
            },
            InstanceVariable::BackgroundForeground => match self.room.backgrounds.get_mut(array_index as usize) {
                Some(background) => background.is_foreground = value.is_truthy(),
                None => self.room.backgrounds[0].is_foreground = value.is_truthy(),
            },
            InstanceVariable::BackgroundIndex => match self.room.backgrounds.get_mut(array_index as usize) {
                Some(background) => background.background_id = value.into(),
                None => self.room.backgrounds[0].background_id = value.into(),
            },
            InstanceVariable::BackgroundX => match self.room.backgrounds.get_mut(array_index as usize) {
                Some(background) => background.x_offset = value.into(),
                None => self.room.backgrounds[0].x_offset = value.into(),
            },
            InstanceVariable::BackgroundY => match self.room.backgrounds.get_mut(array_index as usize) {
                Some(background) => background.y_offset = value.into(),
                None => self.room.backgrounds[0].y_offset = value.into(),
            },
            InstanceVariable::BackgroundHtiled => match self.room.backgrounds.get_mut(array_index as usize) {
                Some(background) => background.tile_horizontal = value.is_truthy(),
                None => self.room.backgrounds[0].tile_horizontal = value.is_truthy(),
            },
            InstanceVariable::BackgroundVtiled => match self.room.backgrounds.get_mut(array_index as usize) {
                Some(background) => background.tile_vertical = value.is_truthy(),
                None => self.room.backgrounds[0].tile_vertical = value.is_truthy(),
            },
            InstanceVariable::BackgroundXscale => match self.room.backgrounds.get_mut(array_index as usize) {
                Some(background) => background.xscale = value.into(),
                None => self.room.backgrounds[0].xscale = value.into(),
            },
            InstanceVariable::BackgroundYscale => match self.room.backgrounds.get_mut(array_index as usize) {
                Some(background) => background.yscale = value.into(),
                None => self.room.backgrounds[0].yscale = value.into(),
            },
            InstanceVariable::BackgroundHspeed => match self.room.backgrounds.get_mut(array_index as usize) {
                Some(background) => background.hspeed = value.into(),
                None => self.room.backgrounds[0].hspeed = value.into(),
            },
            InstanceVariable::BackgroundVspeed => match self.room.backgrounds.get_mut(array_index as usize) {
                Some(background) => background.vspeed = value.into(),
                None => self.room.backgrounds[0].vspeed = value.into(),
            },
            InstanceVariable::BackgroundBlend => match self.room.backgrounds.get_mut(array_index as usize) {
                Some(background) => background.blend = value.into(),
                None => self.room.backgrounds[0].blend = value.into(),
            },
            InstanceVariable::BackgroundAlpha => match self.room.backgrounds.get_mut(array_index as usize) {
                Some(background) => background.alpha = value.into(),
                None => self.room.backgrounds[0].alpha = value.into(),
            },
            InstanceVariable::ViewEnabled => self.room.views_enabled = value.is_truthy(),
            InstanceVariable::ViewVisible => match self.room.views.get_mut(array_index as usize) {
                Some(view) => view.visible = value.is_truthy(),
                None => self.room.views[0].visible = value.is_truthy(),
            },
            InstanceVariable::ViewXview => match self.room.views.get_mut(array_index as usize) {
                Some(view) => view.source_x = value.into(),
                None => self.room.views[0].source_x = value.into(),
            },
            InstanceVariable::ViewYview => match self.room.views.get_mut(array_index as usize) {
                Some(view) => view.source_y = value.into(),
                None => self.room.views[0].source_y = value.into(),
            },
            InstanceVariable::ViewWview => match self.room.views.get_mut(array_index as usize) {
                Some(view) => view.source_w = value.into(),
                None => self.room.views[0].source_w = value.into(),
            },
            InstanceVariable::ViewHview => match self.room.views.get_mut(array_index as usize) {
                Some(view) => view.source_h = value.into(),
                None => self.room.views[0].source_h = value.into(),
            },
            InstanceVariable::ViewXport => match self.room.views.get_mut(array_index as usize) {
                Some(view) => view.port_x = value.into(),
                None => self.room.views[0].port_x = value.into(),
            },
            InstanceVariable::ViewYport => match self.room.views.get_mut(array_index as usize) {
                Some(view) => view.port_y = value.into(),
                None => self.room.views[0].port_y = value.into(),
            },
            InstanceVariable::ViewWport => match self.room.views.get_mut(array_index as usize) {
                Some(view) => view.port_w = value.into(),
                None => self.room.views[0].port_w = value.into(),
            },
            InstanceVariable::ViewHport => match self.room.views.get_mut(array_index as usize) {
                Some(view) => view.port_h = value.into(),
                None => self.room.views[0].port_h = value.into(),
            },
            InstanceVariable::ViewAngle => match self.room.views.get_mut(array_index as usize) {
                Some(view) => view.angle = value.into(),
                None => self.room.views[0].angle = value.into(),
            },
            InstanceVariable::ViewHborder => match self.room.views.get_mut(array_index as usize) {
                Some(view) => view.follow_hborder = value.into(),
                None => self.room.views[0].follow_hborder = value.into(),
            },
            InstanceVariable::ViewVborder => match self.room.views.get_mut(array_index as usize) {
                Some(view) => view.follow_vborder = value.into(),
                None => self.room.views[0].follow_vborder = value.into(),
            },
            InstanceVariable::ViewHspeed => match self.room.views.get_mut(array_index as usize) {
                Some(view) => view.follow_hspeed = value.into(),
                None => self.room.views[0].follow_hspeed = value.into(),
            },
            InstanceVariable::ViewVspeed => match self.room.views.get_mut(array_index as usize) {
                Some(view) => view.follow_vspeed = value.into(),
                None => self.room.views[0].follow_vspeed = value.into(),
            },
            InstanceVariable::ViewObject => match self.room.views.get_mut(array_index as usize) {
                Some(view) => view.follow_target = value.into(),
                None => self.room.views[0].follow_target = value.into(),
            },
            InstanceVariable::MouseButton => {
                let button = value.round();
                if let Ok(mb) = i8::try_from(button) {
                    if matches!(mb, 1 | 2 | 3) {
                        self.input.set_mouse_button(mb);
                    }
                }
            },
            InstanceVariable::MouseLastbutton => {
                let button = value.round();
                if let Ok(mb) = i8::try_from(button) {
                    if matches!(mb, 1 | 2 | 3) {
                        self.input.set_mouse_lastbutton(mb);
                    }
                }
            },
            InstanceVariable::KeyboardKey => {
                let code = value.round();
                if let Ok(vk) = u8::try_from(code) {
                    self.input.set_keyboard_key(vk);
                }
            },
            InstanceVariable::KeyboardLastkey => {
                let code = value.round();
                if let Ok(vk) = u8::try_from(code) {
                    self.input.set_keyboard_lastkey(vk);
                }
            },
            InstanceVariable::KeyboardLastchar => todo!("keyboard_lastchar setter"),
            InstanceVariable::KeyboardString => todo!("keyboard_string setter"),
            InstanceVariable::CursorSprite => self.cursor_sprite = value.round(),
            InstanceVariable::ShowScore => {
                self.has_set_show_score = true;
                self.score_capt_d = value.is_truthy()
            },
            InstanceVariable::ShowLives => self.lives_capt_d = value.is_truthy(),
            InstanceVariable::ShowHealth => self.health_capt_d = value.is_truthy(),
            InstanceVariable::CaptionScore => self.score_capt = value.into(),
            InstanceVariable::CaptionLives => self.lives_capt = value.into(),
            InstanceVariable::CaptionHealth => self.health_capt = value.into(),
            InstanceVariable::ErrorOccurred => self.error_occurred = value.is_truthy(),
            InstanceVariable::ErrorLast => self.error_last = value.into(),
            _ => return Err(Error::ReadOnlyVariable(*var)),
        }
        Ok(())
    }

    // Gets the sprite associated with an instance's sprite_index
    pub fn get_instance_sprite(&self, instance: usize) -> Option<&asset::Sprite> {
        let instance = self.room.instance_list.get(instance);
        let index = instance.sprite_index.get();
        if index >= 0 {
            if let Some(Some(sprite)) = self.assets.sprites.get(index as usize) { Some(sprite) } else { None }
        } else {
            None
        }
    }

    // Gets the sprite associated with an instance's mask_index
    pub fn get_instance_mask_sprite(&self, instance: usize) -> Option<&asset::Sprite> {
        let index = {
            let instance = self.room.instance_list.get(instance);
            let index = instance.mask_index.get();
            if index >= 0 { index } else { instance.sprite_index.get() }
        };
        if index >= 0 {
            if let Some(Some(sprite)) = self.assets.sprites.get(index as usize) { Some(sprite) } else { None }
        } else {
            None
        }
    }

    // Gets an argument from the context. If the argument is out-of-bounds, then it will either
    // return an error or return 0.0, depending on the uninit_args_are_zero setting.
    fn get_argument(&self, context: &Context, arg: usize) -> gml::Result<Value> {
        if let Some(value) = context.arguments.get(arg) {
            Ok(value.clone())
        } else {
            if self.uninit_args_are_zero {
                Ok(Value::Real(Real::from(0.0)))
            } else {
                Err(Error::UninitializedArgument(arg))
            }
        }
    }

    // Sets an argument from the context. If the argument is out-of-bounds, then it will either
    // return an error or return 0.0, depending on the uninit_args_are_zero setting.
    fn set_argument(&self, context: &mut Context, arg: usize, value: Value) -> gml::Result<()> {
        let arg_count = context.argument_count;
        match context.arguments.get_mut(arg) {
            Some(a) if arg < arg_count || self.uninit_args_are_zero => Ok(*a = value),
            None if self.uninit_args_are_zero => Ok(()), // This corrupts stack in GM8...
            _ => Err(Error::UninitializedArgument(arg)),
        }
    }

    // Resolves an InstanceIdentifier to a Target
    fn get_target(
        &mut self,
        context: &mut Context,
        identifier: &InstanceIdentifier,
        in_globalvars: bool,
    ) -> gml::Result<Target> {
        match identifier {
            InstanceIdentifier::Own => Ok(Target::Single(Some(context.this))),
            InstanceIdentifier::Other => Ok(Target::Single(Some(context.other))),
            InstanceIdentifier::Global => Ok(Target::Global),
            InstanceIdentifier::Local => Ok(Target::Local),
            InstanceIdentifier::Unknown => {
                if in_globalvars {
                    Ok(Target::Global)
                } else {
                    Ok(Target::Single(Some(context.this)))
                }
            },
            InstanceIdentifier::Expression(node) => {
                let value = self.eval(node, context).map(i32::from)?;
                match value {
                    gml::SELF | gml::UNSPECIFIED => Ok(Target::Single(Some(context.this))),
                    gml::OTHER => Ok(Target::Single(Some(context.other))),
                    gml::ALL => Ok(Target::All),
                    gml::NOONE => Ok(Target::Single(None)),
                    gml::GLOBAL => Ok(Target::Global),
                    gml::LOCAL => Ok(Target::Local),
                    i if i >= 100_000 => Ok(Target::Single(self.room.instance_list.get_by_instid(i))),
                    i => Ok(Target::Objects(i)),
                }
            },
        }
    }
}
