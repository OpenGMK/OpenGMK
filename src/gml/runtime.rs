use super::{compiler::token::Operator, Context, InstanceVariable, Value};
use crate::{game::Game, gml};
use std::fmt;

/// A compiled runtime instruction. Generally represents a line of code.
#[derive(Debug)]
pub enum Instruction {
    SetField { accessor: FieldAccessor, value: Node, assignment_type: AssignmentType },
    SetVariable { accessor: VariableAccessor, value: Node, assignment_type: AssignmentType },
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

/// Type of assignment.
#[derive(Debug)]
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
    InvalidDeref(String),         // string repr. because Expr<'a>
    InvalidIndexLhs(String),      // string repr. because Expr<'a>
    InvalidIndex(String),         // string repr. because Expr<'a>
    InvalidSwitchBody(String),    // string repr. because Expr<'a>
    UnknownFunction(String),
    UnexpectedASTExpr(String), // string repr. because Expr<'a>
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
            Instruction::SetField { accessor: _, value: _, assignment_type: _ } => todo!(),
            Instruction::SetVariable { accessor: _, value: _, assignment_type: _ } => todo!(),
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
            Node::Script { args: _, script_id: _ } => todo!(),
            Node::Field { accessor: _ } => todo!(),
            Node::Variable { accessor: _ } => todo!(),
            Node::Binary { left, right, operator } => operator(self.eval(left, context)?, self.eval(right, context)?),
            Node::Unary { child, operator } => operator(self.eval(child, context)?),
            Node::RuntimeError { error } => Err(error.clone()),
        }
    }
}
