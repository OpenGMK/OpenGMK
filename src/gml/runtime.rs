use super::{
    compiler::{mappings, token::Operator},
    Context, InstanceVariable, Value,
};
use crate::{
    asset,
    game::Game,
    gml,
    instance::{DummyFieldHolder, Field, Instance},
};
use std::fmt;

/// A compiled runtime instruction. Generally represents a line of code.
#[derive(Debug)]
pub enum Instruction {
    SetField { accessor: FieldAccessor, value: Node },
    SetVariable { accessor: VariableAccessor, value: Node },
    ModifyField { accessor: FieldAccessor, value: Node, modification_type: ModificationType },
    ModifyVariable { accessor: VariableAccessor, value: Node, modification_type: ModificationType },
    EvalExpression { node: Node },
    IfElse { cond: Node, if_body: Box<[Instruction]>, else_body: Box<[Instruction]> },
    LoopUntil { cond: Node, body: Box<[Instruction]> },
    LoopWhile { cond: Node, body: Box<[Instruction]> },
    Return { return_type: ReturnType },
    Repeat { count: Node, body: Box<[Instruction]> },
    SetReturnValue { value: Node },
    Switch { input: Node, cases: Box<[(Node, usize)]>, default: Option<usize>, body: Box<[Instruction]> },
    With { target: Node, body: Box<[Instruction]> },
    RuntimeError { error: Error },
}

/// Node representing one value in an expression.
pub enum Node {
    Literal { value: Value },
    Function { args: Box<[Node]>, function: fn(&mut Game, &mut Context, &[Value]) -> gml::Result<Value> },
    Script { args: Box<[Node]>, script_id: usize },
    Field { accessor: FieldAccessor },
    Variable { accessor: VariableAccessor },
    Binary { left: Box<Node>, right: Box<Node>, operator: fn(Value, Value) -> gml::Result<Value> },
    Unary { child: Box<Node>, operator: fn(Value) -> gml::Result<Value> },
    RuntimeError { error: Error },
}

/// Type of variable modification.
#[derive(Debug)]
pub enum ModificationType {
    Add,
    Subtract,
    Multiply,
    Divide,
    BitAnd,
    BitOr,
    BitXor,
}

/// The reason for stopping execution of the current function.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ReturnType {
    Normal,
    Continue,
    Break,
    Exit,
}

/// Represents an owned field which can either be read or set.
#[derive(Debug)]
pub struct FieldAccessor {
    pub index: usize,
    pub array: ArrayAccessor,
    pub owner: InstanceIdentifier,
}

/// Represents an owned field which can either be read or set.
#[derive(Debug)]
pub struct VariableAccessor {
    pub var: InstanceVariable,
    pub array: ArrayAccessor,
    pub owner: InstanceIdentifier,
}

/// Represents an array accessor, which can be either 1D or 2D.
/// Variables with 0D arrays, and ones with no array accessor, implicitly refer to [0].
/// Anything beyond a 2D array results in a runtime error.
#[derive(Debug)]
pub enum ArrayAccessor {
    None,
    Single(Box<Node>),
    Double(Box<Node>, Box<Node>),
}

/// Identifies an instance or multiple instances.
/// If we know at compile time that this represents a magic value (self, other, global, local)
/// then we can represent it that way in the tree and skip evaluating it during runtime.
#[derive(Debug)]
pub enum InstanceIdentifier {
    Own, // Can't call it Self, that's a Rust keyword. Yeah, I know, sorry.
    Other,
    Global,
    Local,
    Expression(Box<Node>),
}

#[derive(Clone, Debug)]
pub enum Error {
    InvalidOperandsUnary(Operator, Value),
    InvalidOperandsBinary(Operator, Value, Value),
    InvalidUnaryOperator(Operator),
    InvalidBinaryOperator(Operator),
    InvalidAssignment(String),    // string repr. because Expr<'a>
    InvalidArrayAccessor(String), // string repr. because Expr<'a>
    InvalidArrayIndex(i32),
    InvalidDeref(String),      // string repr. because Expr<'a>
    InvalidIndexLhs(String),   // string repr. because Expr<'a>
    InvalidIndex(String),      // string repr. because Expr<'a>
    InvalidSwitchBody(String), // string repr. because Expr<'a>
    NonexistentAsset(asset::Type, usize),
    ReadOnlyVariable(InstanceVariable),
    UnknownFunction(String),
    UnexpectedASTExpr(String), // string repr. because Expr<'a>
    UninitializedVariable(String, u32),
    TooManyArrayDimensions(usize),
}

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Node::Literal { value } => match value {
                Value::Real(r) => write!(f, "{:?}", r),
                Value::Str(s) => write!(f, "{:?}", s),
            },
            Node::Function { args, function: _ } => write!(f, "<function: {:?}>", args),
            Node::Script { args, script_id } => write!(f, "<script {:?}: {:?}>", script_id, args),
            Node::Field { accessor } => write!(f, "<field: {:?}>", accessor),
            Node::Variable { accessor } => write!(f, "<variable: {:?}>", accessor),
            Node::Binary { left, right, operator: _ } => write!(f, "<binary: {:?}, {:?}>", left, right),
            Node::Unary { child, operator: _ } => write!(f, "<unary: {:?}>", child),
            Node::RuntimeError { error } => write!(f, "<error: {:?}>", error),
        }
    }
}

impl Game {
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
            Instruction::SetField { accessor: _, value: _ } => todo!(),
            Instruction::SetVariable { accessor: _, value: _ } => todo!(),
            Instruction::ModifyField { accessor: _, value: _, modification_type: _ } => todo!(),
            Instruction::ModifyVariable { accessor: _, value: _, modification_type: _ } => todo!(),
            Instruction::EvalExpression { node } => match self.eval(node, context) {
                Err(e) => return Err(e),
                _ => (),
            },
            Instruction::IfElse { cond, if_body, else_body } => {
                let return_type = if self.eval(cond, context)?.is_true() {
                    self.execute(if_body, context)
                } else {
                    self.execute(else_body, context)
                }?;
                if return_type != ReturnType::Normal {
                    return Ok(return_type)
                }
            },
            Instruction::LoopUntil { cond, body } => loop {
                let return_type = self.execute(body, context)?;
                if return_type != ReturnType::Normal {
                    return Ok(return_type)
                }
                if self.eval(cond, context)?.is_true() {
                    break
                }
            },
            Instruction::LoopWhile { cond, body } => {
                while self.eval(cond, context)?.is_true() {
                    let return_type = self.execute(body, context)?;
                    if return_type != ReturnType::Normal {
                        return Ok(return_type)
                    }
                }
            },
            Instruction::Return { return_type } => return Ok(*return_type),
            Instruction::Repeat { count, body } => {
                let mut count = self.eval(count, context)?.round();
                while count > 0 {
                    let return_type = self.execute(body, context)?;
                    if return_type != ReturnType::Normal {
                        return Ok(return_type)
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
                        return self.execute(&body[*start..], context)
                    }
                }
                if let Some(start) = default {
                    return self.execute(&body[*start..], context)
                }
            },
            Instruction::With { target: _, body: _ } => todo!(),
            Instruction::RuntimeError { error } => return Err(error.clone()),
        }

        Ok(ReturnType::Normal)
    }

    fn eval(&mut self, node: &Node, context: &mut Context) -> gml::Result<Value> {
        match node {
            Node::Literal { value } => Ok(value.clone()),
            Node::Function { args, function } => {
                let mut arg_values: [Value; 16] = Default::default();
                for (src, dest) in args.iter().zip(arg_values.iter_mut()) {
                    *dest = self.eval(src, context)?;
                }
                function(self, context, &arg_values[..args.len()])
            },
            Node::Script { args, script_id } => {
                if let Some(Some(script)) = self.assets.scripts.get(*script_id) {
                    let instructions = script.compiled.clone();

                    let mut arg_values: [Value; 16] = Default::default();
                    for (src, dest) in args.iter().zip(arg_values.iter_mut()) {
                        *dest = self.eval(src, context)?;
                    }

                    let mut new_context = Context {
                        this: context.this,
                        other: context.other,
                        event_action: context.event_action,
                        relative: context.relative,
                        event_type: context.event_type,
                        event_number: context.event_number,
                        event_object: context.event_object,
                        arguments: &arg_values[..args.len()],
                        locals: DummyFieldHolder::new(),
                        return_value: Default::default(),
                    };
                    self.execute(&instructions, &mut new_context)?;
                    Ok(new_context.return_value)
                } else {
                    Err(Error::NonexistentAsset(asset::Type::Script, *script_id))
                }
            },
            Node::Field { accessor: _ } => todo!(),
            Node::Variable { accessor: _ } => todo!(),
            Node::Binary { left, right, operator } => operator(self.eval(left, context)?, self.eval(right, context)?),
            Node::Unary { child, operator } => operator(self.eval(child, context)?),
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
    fn get_instance_field(&self, instance: &Instance, field_id: usize, array_index: u32) -> gml::Result<Value> {
        if let Some(Some(value)) = instance.fields.borrow().get(&field_id).map(|field| field.get(array_index)) {
            Ok(value)
        } else {
            if self.uninit_fields_are_zero {
                Ok(Value::Real(0.0))
            } else {
                Err(Error::UninitializedVariable(self.compiler.get_field_name(field_id).unwrap(), array_index))
            }
        }
    }

    // Set a field on an instance
    fn set_instance_field(&self, instance: &Instance, field_id: usize, array_index: u32, value: Value) {
        let mut fields = instance.fields.borrow_mut();
        if let Some(field) = fields.get_mut(&field_id) {
            field.set(array_index, value)
        } else {
            fields.insert(field_id, Field::new(array_index, value));
        }
    }

    // Get an instance variable from an instance, converted into a Value
    fn get_instance_var(
        &self,
        _instance: &Instance,
        var: &InstanceVariable,
        _array_index: u32,
        _context: &Context,
    ) -> gml::Result<Value> {
        match var {
            InstanceVariable::X => todo!(),
            InstanceVariable::Y => todo!(),
            InstanceVariable::Xprevious => todo!(),
            InstanceVariable::Yprevious => todo!(),
            InstanceVariable::Xstart => todo!(),
            InstanceVariable::Ystart => todo!(),
            InstanceVariable::Hspeed => todo!(),
            InstanceVariable::Vspeed => todo!(),
            InstanceVariable::Direction => todo!(),
            InstanceVariable::Speed => todo!(),
            InstanceVariable::Friction => todo!(),
            InstanceVariable::Gravity => todo!(),
            InstanceVariable::GravityDirection => todo!(),
            InstanceVariable::ObjectIndex => todo!(),
            InstanceVariable::Id => todo!(),
            InstanceVariable::Alarm => todo!(),
            InstanceVariable::Solid => todo!(),
            InstanceVariable::Visible => todo!(),
            InstanceVariable::Persistent => todo!(),
            InstanceVariable::Depth => todo!(),
            InstanceVariable::BboxLeft => todo!(),
            InstanceVariable::BboxRight => todo!(),
            InstanceVariable::BboxTop => todo!(),
            InstanceVariable::BboxBottom => todo!(),
            InstanceVariable::SpriteIndex => todo!(),
            InstanceVariable::ImageIndex => todo!(),
            InstanceVariable::ImageSingle => todo!(),
            InstanceVariable::ImageNumber => todo!(),
            InstanceVariable::SpriteWidth => todo!(),
            InstanceVariable::SpriteHeight => todo!(),
            InstanceVariable::SpriteXoffset => todo!(),
            InstanceVariable::SpriteYoffset => todo!(),
            InstanceVariable::ImageXscale => todo!(),
            InstanceVariable::ImageYscale => todo!(),
            InstanceVariable::ImageAngle => todo!(),
            InstanceVariable::ImageAlpha => todo!(),
            InstanceVariable::ImageBlend => todo!(),
            InstanceVariable::ImageSpeed => todo!(),
            InstanceVariable::MaskIndex => todo!(),
            InstanceVariable::PathIndex => todo!(),
            InstanceVariable::PathPosition => todo!(),
            InstanceVariable::PathPositionprevious => todo!(),
            InstanceVariable::PathSpeed => todo!(),
            InstanceVariable::PathScale => todo!(),
            InstanceVariable::PathOrientation => todo!(),
            InstanceVariable::PathEndaction => todo!(),
            InstanceVariable::TimelineIndex => todo!(),
            InstanceVariable::TimelinePosition => todo!(),
            InstanceVariable::TimelineSpeed => todo!(),
            InstanceVariable::TimelineRunning => todo!(),
            InstanceVariable::TimelineLoop => todo!(),
            InstanceVariable::ArgumentRelative => todo!(),
            InstanceVariable::Argument0 => todo!(),
            InstanceVariable::Argument1 => todo!(),
            InstanceVariable::Argument2 => todo!(),
            InstanceVariable::Argument3 => todo!(),
            InstanceVariable::Argument4 => todo!(),
            InstanceVariable::Argument5 => todo!(),
            InstanceVariable::Argument6 => todo!(),
            InstanceVariable::Argument7 => todo!(),
            InstanceVariable::Argument8 => todo!(),
            InstanceVariable::Argument9 => todo!(),
            InstanceVariable::Argument10 => todo!(),
            InstanceVariable::Argument11 => todo!(),
            InstanceVariable::Argument12 => todo!(),
            InstanceVariable::Argument13 => todo!(),
            InstanceVariable::Argument14 => todo!(),
            InstanceVariable::Argument15 => todo!(),
            InstanceVariable::Argument => todo!(),
            InstanceVariable::ArgumentCount => todo!(),
            InstanceVariable::Room => todo!(),
            InstanceVariable::RoomFirst => todo!(),
            InstanceVariable::RoomLast => todo!(),
            InstanceVariable::TransitionKind => todo!(),
            InstanceVariable::TransitionSteps => todo!(),
            InstanceVariable::Score => todo!(),
            InstanceVariable::Lives => todo!(),
            InstanceVariable::Health => todo!(),
            InstanceVariable::GameId => todo!(),
            InstanceVariable::WorkingDirectory => todo!(),
            InstanceVariable::TempDirectory => todo!(),
            InstanceVariable::ProgramDirectory => todo!(),
            InstanceVariable::InstanceCount => todo!(),
            InstanceVariable::InstanceId => todo!(),
            InstanceVariable::RoomWidth => todo!(),
            InstanceVariable::RoomHeight => todo!(),
            InstanceVariable::RoomCaption => todo!(),
            InstanceVariable::RoomSpeed => todo!(),
            InstanceVariable::RoomPersistent => todo!(),
            InstanceVariable::BackgroundColor => todo!(),
            InstanceVariable::BackgroundShowcolor => todo!(),
            InstanceVariable::BackgroundVisible => todo!(),
            InstanceVariable::BackgroundForeground => todo!(),
            InstanceVariable::BackgroundIndex => todo!(),
            InstanceVariable::BackgroundX => todo!(),
            InstanceVariable::BackgroundY => todo!(),
            InstanceVariable::BackgroundWidth => todo!(),
            InstanceVariable::BackgroundHeight => todo!(),
            InstanceVariable::BackgroundHtiled => todo!(),
            InstanceVariable::BackgroundVtiled => todo!(),
            InstanceVariable::BackgroundXscale => todo!(),
            InstanceVariable::BackgroundYscale => todo!(),
            InstanceVariable::BackgroundHspeed => todo!(),
            InstanceVariable::BackgroundVspeed => todo!(),
            InstanceVariable::BackgroundBlend => todo!(),
            InstanceVariable::BackgroundAlpha => todo!(),
            InstanceVariable::ViewEnabled => todo!(),
            InstanceVariable::ViewCurrent => todo!(),
            InstanceVariable::ViewVisible => todo!(),
            InstanceVariable::ViewXview => todo!(),
            InstanceVariable::ViewYview => todo!(),
            InstanceVariable::ViewWview => todo!(),
            InstanceVariable::ViewHview => todo!(),
            InstanceVariable::ViewXport => todo!(),
            InstanceVariable::ViewYport => todo!(),
            InstanceVariable::ViewWport => todo!(),
            InstanceVariable::ViewHport => todo!(),
            InstanceVariable::ViewAngle => todo!(),
            InstanceVariable::ViewHborder => todo!(),
            InstanceVariable::ViewVborder => todo!(),
            InstanceVariable::ViewHspeed => todo!(),
            InstanceVariable::ViewVspeed => todo!(),
            InstanceVariable::ViewObject => todo!(),
            InstanceVariable::MouseX => todo!(),
            InstanceVariable::MouseY => todo!(),
            InstanceVariable::MouseButton => todo!(),
            InstanceVariable::MouseLastbutton => todo!(),
            InstanceVariable::KeyboardKey => todo!(),
            InstanceVariable::KeyboardLastkey => todo!(),
            InstanceVariable::KeyboardLastchar => todo!(),
            InstanceVariable::KeyboardString => todo!(),
            InstanceVariable::CursorSprite => todo!(),
            InstanceVariable::ShowScore => todo!(),
            InstanceVariable::ShowLives => todo!(),
            InstanceVariable::ShowHealth => todo!(),
            InstanceVariable::CaptionScore => todo!(),
            InstanceVariable::CaptionLives => todo!(),
            InstanceVariable::CaptionHealth => todo!(),
            InstanceVariable::Fps => todo!(),
            InstanceVariable::CurrentTime => todo!(),
            InstanceVariable::CurrentYear => todo!(),
            InstanceVariable::CurrentMonth => todo!(),
            InstanceVariable::CurrentDay => todo!(),
            InstanceVariable::CurrentWeekday => todo!(),
            InstanceVariable::CurrentHour => todo!(),
            InstanceVariable::CurrentMinute => todo!(),
            InstanceVariable::CurrentSecond => todo!(),
            InstanceVariable::EventType => todo!(),
            InstanceVariable::EventNumber => todo!(),
            InstanceVariable::EventObject => todo!(),
            InstanceVariable::EventAction => todo!(),
            InstanceVariable::SecureMode => todo!(),
            InstanceVariable::DebugMode => todo!(),
            InstanceVariable::ErrorOccurred => todo!(),
            InstanceVariable::ErrorLast => todo!(),
            InstanceVariable::GamemakerRegistered => todo!(),
            InstanceVariable::GamemakerPro => todo!(),
            InstanceVariable::GamemakerVersion => todo!(),
            InstanceVariable::OsType => todo!(),
            InstanceVariable::OsDevice => todo!(),
            InstanceVariable::OsBrowser => todo!(),
            InstanceVariable::OsVersion => todo!(),
            InstanceVariable::BrowserWidth => todo!(),
            InstanceVariable::BrowserHeight => todo!(),
            InstanceVariable::DisplayAa => todo!(),
            InstanceVariable::AsyncLoad => todo!(),
        }
    }

    // Set an instance variable on an instance
    fn set_instance_var(
        &self,
        _instance: &Instance,
        var: &InstanceVariable,
        _array_index: u32,
        _value: Value,
        _context: &mut Context,
    ) -> gml::Result<()> {
        match var {
            InstanceVariable::X => todo!(),
            InstanceVariable::Y => todo!(),
            InstanceVariable::Xprevious => todo!(),
            InstanceVariable::Yprevious => todo!(),
            InstanceVariable::Xstart => todo!(),
            InstanceVariable::Ystart => todo!(),
            InstanceVariable::Hspeed => todo!(),
            InstanceVariable::Vspeed => todo!(),
            InstanceVariable::Direction => todo!(),
            InstanceVariable::Speed => todo!(),
            InstanceVariable::Friction => todo!(),
            InstanceVariable::Gravity => todo!(),
            InstanceVariable::GravityDirection => todo!(),
            InstanceVariable::Alarm => todo!(),
            InstanceVariable::Solid => todo!(),
            InstanceVariable::Visible => todo!(),
            InstanceVariable::Persistent => todo!(),
            InstanceVariable::Depth => todo!(),
            InstanceVariable::SpriteIndex => todo!(),
            InstanceVariable::ImageIndex => todo!(),
            InstanceVariable::ImageSingle => todo!(),
            InstanceVariable::ImageXscale => todo!(),
            InstanceVariable::ImageYscale => todo!(),
            InstanceVariable::ImageAngle => todo!(),
            InstanceVariable::ImageAlpha => todo!(),
            InstanceVariable::ImageBlend => todo!(),
            InstanceVariable::ImageSpeed => todo!(),
            InstanceVariable::MaskIndex => todo!(),
            InstanceVariable::PathPosition => todo!(),
            InstanceVariable::PathPositionprevious => todo!(),
            InstanceVariable::PathSpeed => todo!(),
            InstanceVariable::PathScale => todo!(),
            InstanceVariable::PathOrientation => todo!(),
            InstanceVariable::PathEndaction => todo!(),
            InstanceVariable::TimelineIndex => todo!(),
            InstanceVariable::TimelinePosition => todo!(),
            InstanceVariable::TimelineSpeed => todo!(),
            InstanceVariable::TimelineRunning => todo!(),
            InstanceVariable::TimelineLoop => todo!(),
            InstanceVariable::Argument0 => todo!(),
            InstanceVariable::Argument1 => todo!(),
            InstanceVariable::Argument2 => todo!(),
            InstanceVariable::Argument3 => todo!(),
            InstanceVariable::Argument4 => todo!(),
            InstanceVariable::Argument5 => todo!(),
            InstanceVariable::Argument6 => todo!(),
            InstanceVariable::Argument7 => todo!(),
            InstanceVariable::Argument8 => todo!(),
            InstanceVariable::Argument9 => todo!(),
            InstanceVariable::Argument10 => todo!(),
            InstanceVariable::Argument11 => todo!(),
            InstanceVariable::Argument12 => todo!(),
            InstanceVariable::Argument13 => todo!(),
            InstanceVariable::Argument14 => todo!(),
            InstanceVariable::Argument15 => todo!(),
            InstanceVariable::Argument => todo!(),
            InstanceVariable::Room => todo!(),
            InstanceVariable::TransitionKind => todo!(),
            InstanceVariable::TransitionSteps => todo!(),
            InstanceVariable::Score => todo!(),
            InstanceVariable::Lives => todo!(),
            InstanceVariable::Health => todo!(),
            InstanceVariable::RoomCaption => todo!(),
            InstanceVariable::RoomSpeed => todo!(),
            InstanceVariable::RoomPersistent => todo!(),
            InstanceVariable::BackgroundColor => todo!(),
            InstanceVariable::BackgroundShowcolor => todo!(),
            InstanceVariable::BackgroundVisible => todo!(),
            InstanceVariable::BackgroundForeground => todo!(),
            InstanceVariable::BackgroundIndex => todo!(),
            InstanceVariable::BackgroundX => todo!(),
            InstanceVariable::BackgroundY => todo!(),
            InstanceVariable::BackgroundHtiled => todo!(),
            InstanceVariable::BackgroundVtiled => todo!(),
            InstanceVariable::BackgroundXscale => todo!(),
            InstanceVariable::BackgroundYscale => todo!(),
            InstanceVariable::BackgroundHspeed => todo!(),
            InstanceVariable::BackgroundVspeed => todo!(),
            InstanceVariable::BackgroundBlend => todo!(),
            InstanceVariable::BackgroundAlpha => todo!(),
            InstanceVariable::ViewEnabled => todo!(),
            InstanceVariable::ViewVisible => todo!(),
            InstanceVariable::ViewXview => todo!(),
            InstanceVariable::ViewYview => todo!(),
            InstanceVariable::ViewWview => todo!(),
            InstanceVariable::ViewHview => todo!(),
            InstanceVariable::ViewXport => todo!(),
            InstanceVariable::ViewYport => todo!(),
            InstanceVariable::ViewWport => todo!(),
            InstanceVariable::ViewHport => todo!(),
            InstanceVariable::ViewAngle => todo!(),
            InstanceVariable::ViewHborder => todo!(),
            InstanceVariable::ViewVborder => todo!(),
            InstanceVariable::ViewHspeed => todo!(),
            InstanceVariable::ViewVspeed => todo!(),
            InstanceVariable::ViewObject => todo!(),
            InstanceVariable::MouseButton => todo!(),
            InstanceVariable::MouseLastbutton => todo!(),
            InstanceVariable::KeyboardKey => todo!(),
            InstanceVariable::KeyboardLastkey => todo!(),
            InstanceVariable::KeyboardLastchar => todo!(),
            InstanceVariable::KeyboardString => todo!(),
            InstanceVariable::CursorSprite => todo!(),
            InstanceVariable::ShowScore => todo!(),
            InstanceVariable::ShowLives => todo!(),
            InstanceVariable::ShowHealth => todo!(),
            InstanceVariable::CaptionScore => todo!(),
            InstanceVariable::CaptionLives => todo!(),
            InstanceVariable::CaptionHealth => todo!(),
            InstanceVariable::ErrorOccurred => todo!(),
            InstanceVariable::ErrorLast => todo!(),
            _ => return Err(Error::ReadOnlyVariable(*var)),
        }
        //Ok(()) //uncomment this when it's not unreachable
    }

    // Get a field value from a DummyFieldHolder
    fn get_dummy_field(&self, dummy: &DummyFieldHolder, field_id: usize, array_index: u32) -> gml::Result<Value> {
        if let Some(Some(value)) = dummy.fields.get(&field_id).map(|field| field.get(array_index)) {
            Ok(value)
        } else {
            if self.uninit_fields_are_zero {
                Ok(Value::Real(0.0))
            } else {
                Err(Error::UninitializedVariable(self.compiler.get_field_name(field_id).unwrap(), array_index))
            }
        }
    }

    // Set a field on a DummyFieldHolder
    fn set_dummy_field(&self, dummy: &mut DummyFieldHolder, field_id: usize, array_index: u32, value: Value) {
        if let Some(field) = dummy.fields.get_mut(&field_id) {
            field.set(array_index, value)
        } else {
            dummy.fields.insert(field_id, Field::new(array_index, value));
        }
    }

    // Get an instance variable value from a DummyFieldHolder
    fn get_dummy_var(&self, dummy: &DummyFieldHolder, var: &InstanceVariable, array_index: u32) -> gml::Result<Value> {
        if let Some(Some(value)) = dummy.vars.get(var).map(|field| field.get(array_index)) {
            Ok(value)
        } else {
            if self.uninit_fields_are_zero {
                Ok(Value::Real(0.0))
            } else {
                Err(Error::UninitializedVariable(
                    String::from(mappings::INSTANCE_VARIABLES.iter().find(|(_, x)| x == var).unwrap().0),
                    array_index,
                ))
            }
        }
    }

    // Set an instance variable on a DummyFieldHolder
    fn set_dummy_var(&self, dummy: &mut DummyFieldHolder, var: &InstanceVariable, array_index: u32, value: Value) {
        if let Some(field) = dummy.vars.get_mut(var) {
            field.set(array_index, value)
        } else {
            dummy.vars.insert(*var, Field::new(array_index, value));
        }
    }
}
