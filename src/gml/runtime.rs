use super::{
    compiler::{mappings, token::Operator},
    Context, InstanceVariable, Value,
};
use crate::{
    asset,
    game::Game,
    gml::{self, compiler::mappings::constants as gml_constants},
    instance::{DummyFieldHolder, Field, Instance},
};
use std::fmt;

const DEFAULT_ALARM: i32 = -1;

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
    EmptyRoomOrder,
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
    UninitializedArgument(usize),
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
                        arguments: &mut arg_values[..args.len()],
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
        instance: &Instance,
        var: &InstanceVariable,
        array_index: u32,
        context: &Context,
    ) -> gml::Result<Value> {
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
                Some(i) => Ok((*i).into()),
                None => Ok(DEFAULT_ALARM.into()),
            },
            InstanceVariable::Solid => Ok(instance.solid.get().into()),
            InstanceVariable::Visible => Ok(instance.visible.get().into()),
            InstanceVariable::Persistent => Ok(instance.persistent.get().into()),
            InstanceVariable::Depth => Ok(instance.depth.get().into()),
            InstanceVariable::BboxLeft => {
                instance.update_bbox(self.get_instance_sprite(instance));
                Ok(instance.bbox_left.get().into())
            },
            InstanceVariable::BboxRight => {
                instance.update_bbox(self.get_instance_sprite(instance));
                Ok(instance.bbox_right.get().into())
            },
            InstanceVariable::BboxTop => {
                instance.update_bbox(self.get_instance_sprite(instance));
                Ok(instance.bbox_top.get().into())
            },
            InstanceVariable::BboxBottom => {
                instance.update_bbox(self.get_instance_sprite(instance));
                Ok(instance.bbox_bottom.get().into())
            },
            InstanceVariable::SpriteIndex => Ok(instance.sprite_index.get().into()),
            InstanceVariable::ImageIndex => Ok(instance.image_index.get().into()),
            InstanceVariable::ImageSingle => {
                if instance.image_speed.get() == 0.0 {
                    Ok(instance.image_index.get().into())
                } else {
                    Ok(Value::from(-1i32))
                }
            },
            InstanceVariable::ImageNumber => match self.get_instance_sprite(instance) {
                Some(sprite) => Ok(sprite.frames.len().into()),
                None => Ok(Value::from(0i32)),
            },
            InstanceVariable::SpriteWidth => {
                if let Some(sprite) = self.get_instance_sprite(instance) {
                    Ok(Value::from(f64::from(sprite.width) * instance.image_xscale.get()))
                } else {
                    Ok(Value::from(0.0))
                }
            },
            InstanceVariable::SpriteHeight => {
                if let Some(sprite) = self.get_instance_sprite(instance) {
                    Ok(Value::from(f64::from(sprite.height) * instance.image_yscale.get()))
                } else {
                    Ok(Value::from(0.0))
                }
            },
            InstanceVariable::SpriteXoffset => {
                if let Some(sprite) = self.get_instance_sprite(instance) {
                    Ok(sprite.origin_x.into())
                } else {
                    Ok(Value::from(0.0))
                }
            },
            InstanceVariable::SpriteYoffset => {
                if let Some(sprite) = self.get_instance_sprite(instance) {
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
            InstanceVariable::ArgumentCount => Ok(context.arguments.len().into()),
            InstanceVariable::Room => Ok(self.room_id.into()),
            InstanceVariable::RoomFirst => match self.room_order.get(0) {
                Some(room) => Ok((*room).into()),
                None => Err(Error::EmptyRoomOrder),
            },
            InstanceVariable::RoomLast => match self.room_order.get(self.room_order.len() - 1) {
                Some(room) => Ok((*room).into()),
                None => Err(Error::EmptyRoomOrder),
            },
            InstanceVariable::TransitionKind => Ok(self.transition_kind.into()),
            InstanceVariable::TransitionSteps => Ok(self.transition_steps.into()),
            InstanceVariable::Score => Ok(self.score.into()),
            InstanceVariable::Lives => Ok(self.lives.into()),
            InstanceVariable::Health => Ok(self.health.into()),
            InstanceVariable::GameId => Ok(self.game_id.into()),
            InstanceVariable::WorkingDirectory => todo!(),
            InstanceVariable::TempDirectory => todo!(),
            InstanceVariable::ProgramDirectory => todo!(),
            InstanceVariable::InstanceCount => Ok(self.instance_list.count_all().into()),
            InstanceVariable::InstanceId => Ok(self.instance_list.instance_at(array_index as _).into()),
            InstanceVariable::RoomWidth => Ok(self.room_width.into()),
            InstanceVariable::RoomHeight => Ok(self.room_height.into()),
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
            InstanceVariable::CaptionScore => Ok(self.score_capt.clone().into()),
            InstanceVariable::CaptionLives => Ok(self.lives_capt.clone().into()),
            InstanceVariable::CaptionHealth => Ok(self.health_capt.clone().into()),
            InstanceVariable::Fps => todo!(),
            InstanceVariable::CurrentTime => todo!(),
            InstanceVariable::CurrentYear => todo!(),
            InstanceVariable::CurrentMonth => todo!(),
            InstanceVariable::CurrentDay => todo!(),
            InstanceVariable::CurrentWeekday => todo!(),
            InstanceVariable::CurrentHour => todo!(),
            InstanceVariable::CurrentMinute => todo!(),
            InstanceVariable::CurrentSecond => todo!(),
            InstanceVariable::EventType => Ok(context.event_type.into()),
            InstanceVariable::EventNumber => Ok(context.event_number.into()),
            InstanceVariable::EventObject => Ok(context.event_object.into()),
            InstanceVariable::EventAction => Ok(context.event_action.into()),
            InstanceVariable::SecureMode => todo!(), // TODO: this.. isn't documented? what??
            InstanceVariable::DebugMode => todo!(),
            InstanceVariable::ErrorOccurred => todo!(),
            InstanceVariable::ErrorLast => todo!(),
            InstanceVariable::GamemakerRegistered => Ok(gml::TRUE.into()), // yeah!
            InstanceVariable::GamemakerPro => Ok(gml::TRUE.into()),        // identical to registered
            InstanceVariable::GamemakerVersion => Ok(match self.gm_version {
                // the docs claim these range from 800-809, 810-819. they don't.
                gm8exe::GameVersion::GameMaker8_0 => 800f64.into(),
                gm8exe::GameVersion::GameMaker8_1 => 810f64.into(),
            }),
            InstanceVariable::OsType => Ok(gml_constants::OS_WIN32.into()), // not on other OSes...
            InstanceVariable::OsDevice => Ok(gml_constants::DEVICE_IOS_IPHONE.into()), // default

            // all undocumented, unimplemented and return -1. not even the editor recognizes them
            InstanceVariable::OsBrowser => Ok((-1f64).into()),
            InstanceVariable::OsVersion => Ok((-1f64).into()),
            InstanceVariable::BrowserWidth => Ok((-1f64).into()),
            InstanceVariable::BrowserHeight => Ok((-1f64).into()),

            InstanceVariable::DisplayAa => Ok(14f64.into()), // bitfield - 2x/4x/8x AA is 14
            InstanceVariable::AsyncLoad => todo!(),
        }
    }

    // Set an instance variable on an instance
    fn set_instance_var(
        &mut self,
        instance: &Instance,
        var: &InstanceVariable,
        _array_index: u32,
        value: Value,
        _context: &mut Context,
    ) -> gml::Result<()> {
        match var {
            InstanceVariable::X => {
                instance.bbox_is_stale.set(true);
                instance.x.set(value.into());
            },
            InstanceVariable::Y => {
                instance.bbox_is_stale.set(true);
                instance.y.set(value.into());
            },
            InstanceVariable::Xprevious => instance.xprevious.set(value.into()),
            InstanceVariable::Yprevious => instance.yprevious.set(value.into()),
            InstanceVariable::Xstart => instance.xstart.set(value.into()),
            InstanceVariable::Ystart => instance.ystart.set(value.into()),
            InstanceVariable::Hspeed => instance.set_hspeed(value.into()),
            InstanceVariable::Vspeed => instance.set_vspeed(value.into()),
            InstanceVariable::Direction => instance.set_direction(value.into()),
            InstanceVariable::Speed => instance.set_speed(value.into()),
            InstanceVariable::Friction => todo!(),
            InstanceVariable::Gravity => todo!(),
            InstanceVariable::GravityDirection => todo!(),
            InstanceVariable::Alarm => todo!(),
            InstanceVariable::Solid => instance.solid.set(value.is_true()),
            InstanceVariable::Visible => instance.visible.set(value.is_true()),
            InstanceVariable::Persistent => instance.persistent.set(value.is_true()),
            InstanceVariable::Depth => instance.depth.set(value.into()),
            InstanceVariable::SpriteIndex => {
                instance.bbox_is_stale.set(true);
                instance.sprite_index.set(value.into());
            },
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
            InstanceVariable::PathEndAction => todo!(),
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
            InstanceVariable::Score => self.score = value.into(),
            InstanceVariable::Lives => self.lives = value.into(),
            InstanceVariable::Health => self.health = value.into(),
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
            InstanceVariable::CaptionScore => self.score_capt = value.into(),
            InstanceVariable::CaptionLives => self.lives_capt = value.into(),
            InstanceVariable::CaptionHealth => self.health_capt = value.into(),
            InstanceVariable::ErrorOccurred => todo!(),
            InstanceVariable::ErrorLast => todo!(),
            _ => return Err(Error::ReadOnlyVariable(*var)),
        }
        Ok(())
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

    // Gets the sprite associated with an instance's sprite_index
    fn get_instance_sprite(&self, instance: &Instance) -> Option<&asset::Sprite> {
        let index = instance.sprite_index.get();
        if index >= 0 {
            if let Some(Some(sprite)) = self.assets.sprites.get(index as usize) { Some(sprite) } else { None }
        } else {
            None
        }
    }

    // Gets the sprite associated with an instance's mask_index
    fn get_instance_mask_sprite(&mut self, instance: &Instance) -> Option<&asset::Sprite> {
        let index = {
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
            if self.uninit_args_are_zero { Ok(Value::Real(0.0)) } else { Err(Error::UninitializedArgument(arg)) }
        }
    }
}
