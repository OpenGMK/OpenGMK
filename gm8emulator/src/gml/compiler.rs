use super::{
    mappings,
    runtime::{
        ArrayAccessor, BinaryOperator, FieldAccessor, InstanceIdentifier, Instruction, Node, ReturnType, UnaryOperator,
        VariableAccessor,
    },
    Value,
};
use crate::{gml, math::Real};
use gml_parser::{ast, token::Operator};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, rc::Rc, str};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Compiler {
    /// List of identifiers which represent const values
    constants: HashMap<Box<[u8]>, Value>,

    /// Table of user-defined constants to IDs
    user_constant_names: HashMap<Box<[u8]>, usize>,

    /// Table of script names to IDs
    script_names: HashMap<Box<[u8]>, usize>,

    /// Table of extension function names
    extension_fn_names: HashMap<Box<[u8]>, usize>,

    /// Lookup table of unique field names
    fields: Vec<Box<[u8]>>,
}

impl Compiler {
    /// Create a compiler.
    pub fn new() -> Self {
        Self {
            constants: HashMap::new(),
            user_constant_names: HashMap::new(),
            script_names: HashMap::new(),
            extension_fn_names: HashMap::new(),
            fields: Vec::new(),
        }
    }

    /// Reserve space to register at least the given number of constants.
    pub fn reserve_constants(&mut self, size: usize) {
        self.constants.reserve(size)
    }

    /// Reserve space to register at least the given number of script names.
    pub fn reserve_scripts(&mut self, size: usize) {
        self.script_names.reserve(size)
    }

    /// Reserve space to register at least the given number of ExtensionFunction names.
    pub fn reserve_extension_functions(&mut self, size: usize) {
        self.extension_fn_names.reserve(size)
    }

    /// Reserve space to register at least the given number of user-defined constants.
    pub fn reserve_user_constants(&mut self, size: usize) {
        self.user_constant_names.reserve(size)
    }

    /// Add a constant and its associated f64 value, such as an asset name.
    /// These constants will override built-in ones, such as c_red. However, if the same constant name is
    /// registered twice, the old one will NOT be overwritten and the value will be dropped, as per GM8.
    pub fn register_constant(&mut self, name: Box<[u8]>, value: f64) {
        self.constants.entry(name).or_insert(Value::Real(value.into()));
    }

    /// Register a script name and its index. Duplicate script names are ignored.
    pub fn register_script(&mut self, name: Box<[u8]>, index: usize) {
        self.script_names.entry(name).or_insert(index);
    }

    /// Register an ExtensionFunction and its index.
    pub fn register_extension_function(&mut self, name: Box<[u8]>, index: usize) {
        self.extension_fn_names.entry(name).or_insert(index);
    }

    /// Register a user constant and its index.
    pub fn register_user_constant(&mut self, name: Box<[u8]>, index: usize) {
        self.user_constant_names.insert(name, index);
    }

    /// Compile a GML string into instructions.
    pub fn compile(&mut self, source: &[u8]) -> Result<Rc<[Instruction]>, ast::Error> {
        let ast = ast::AST::new(source)?;

        let mut instructions = Vec::new();
        let mut locals: Vec<&[u8]> = Vec::new();
        for node in ast.iter() {
            self.compile_ast_line(node, &mut instructions, &mut locals);
        }
        Ok(instructions.into())
    }

    /// Compile an expression into a format which can be evaluated.
    pub fn compile_expression(&mut self, source: &[u8]) -> Result<Node, ast::Error> {
        let expr = ast::AST::expression(source)?;
        Ok(self.compile_ast_expr(&expr, &[]))
    }

    /// Compile a single line of code from an AST expression.
    fn compile_ast_line<'a>(&mut self, line: &'a ast::Expr, output: &mut Vec<Instruction>, locals: &mut Vec<&'a [u8]>) {
        match line {
            // Line of code identified by an assignment operator
            ast::Expr::Binary(binary_expr) => {
                output.push(self.binary_to_instruction(binary_expr.as_ref(), &locals));
            },

            // Break
            ast::Expr::Break => {
                output.push(Instruction::Return { return_type: ReturnType::Break });
            },

            // Continue
            ast::Expr::Continue => {
                output.push(Instruction::Return { return_type: ReturnType::Continue });
            },

            // Exit
            ast::Expr::Exit => {
                output.push(Instruction::Return { return_type: ReturnType::Exit });
            },

            // For loop
            ast::Expr::For(for_expr) => {
                self.compile_ast_line(&for_expr.start, output, locals);
                let cond = self.compile_ast_expr(&for_expr.cond, locals);
                let mut body = Vec::new();
                self.compile_ast_line(&for_expr.body, &mut body, locals);
                let mut step = Vec::new();
                self.compile_ast_line(&for_expr.step, &mut step, locals);
                output.push(Instruction::LoopFor {
                    cond,
                    body: body.into_boxed_slice(),
                    step: step.into_boxed_slice(),
                });
            },

            // Function or Script
            f @ ast::Expr::Function(_) => {
                output.push(Instruction::EvalExpression { node: self.compile_ast_expr(f, locals) });
            },

            // Group of expressions
            ast::Expr::Group(group) => {
                for expr in group {
                    self.compile_ast_line(expr, output, locals);
                }
            },

            // If/else body
            ast::Expr::If(if_expr) => {
                let cond = self.compile_ast_expr(&if_expr.cond, locals);
                if let Node::Literal { value: v } = cond {
                    // The "if" condition is constant, so we can optimize this away
                    if v.is_truthy() {
                        self.compile_ast_line(&if_expr.body, output, locals);
                    } else if let Some(expr_else_body) = &if_expr.else_body {
                        self.compile_ast_line(expr_else_body, output, locals);
                    }
                } else {
                    let mut if_body = Vec::new();
                    self.compile_ast_line(&if_expr.body, &mut if_body, locals);
                    let mut else_body = Vec::new();
                    if let Some(expr_else_body) = &if_expr.else_body {
                        self.compile_ast_line(expr_else_body, &mut else_body, locals);
                    }
                    output.push(Instruction::IfElse {
                        cond,
                        if_body: if_body.into_boxed_slice(),
                        else_body: else_body.into_boxed_slice(),
                    });
                }
            },

            // "repeat" block
            ast::Expr::Repeat(repeat_expr) => {
                let count = self.compile_ast_expr(&repeat_expr.count, locals);
                let mut body = Vec::new();
                self.compile_ast_line(&repeat_expr.body, &mut body, locals);
                output.push(Instruction::Repeat { count, body: body.into_boxed_slice() });
            },

            // Return
            ast::Expr::Return(expr) => {
                let value = self.compile_ast_expr(&expr, locals);
                output.push(Instruction::SetReturnValue { value });
                output.push(Instruction::Return { return_type: ReturnType::Exit });
            },

            // "switch" block
            ast::Expr::Switch(switch_expr) => {
                let input = self.compile_ast_expr(&switch_expr.input, locals);
                if let ast::Expr::Group(group) = &switch_expr.body {
                    let mut cases = Vec::new();
                    let mut body = Vec::new();
                    let mut default: Option<usize> = None;
                    for expr in group {
                        if let ast::Expr::Case(case_expr) = expr {
                            if default.is_none() {
                                cases.push((self.compile_ast_expr(case_expr, locals), body.len()));
                            }
                        } else if let ast::Expr::Default = expr {
                            if default.is_none() {
                                default = Some(body.len());
                            }
                        } else {
                            self.compile_ast_line(expr, &mut body, locals);
                        }
                    }
                    output.push(Instruction::Switch {
                        input,
                        cases: cases.into_boxed_slice(),
                        default,
                        body: body.into_boxed_slice(),
                    });
                } else {
                    output.push(Instruction::RuntimeError {
                        error: gml::Error::InvalidSwitchBody(switch_expr.body.to_string()),
                    });
                }
            },

            // "do-until" block
            ast::Expr::DoUntil(while_expr) => {
                let cond = self.compile_ast_expr(&while_expr.cond, locals);
                let mut body = Vec::new();
                self.compile_ast_line(&while_expr.body, &mut body, locals);
                output.push(Instruction::LoopUntil { cond, body: body.into_boxed_slice() });
            },

            // "var" declaration
            ast::Expr::Var(var_expr) => {
                locals.extend_from_slice(&var_expr.vars);
            },

            ast::Expr::GlobalVar(globalvar_expr) => {
                // globalvar doesn't work on builtins
                let fields = globalvar_expr.vars.iter().map(|x| self.get_field_id(x)).collect();
                output.push(Instruction::GlobalVar { fields });
            },

            // "while" block
            ast::Expr::While(while_expr) => {
                let cond = self.compile_ast_expr(&while_expr.cond, locals);
                let mut body = Vec::new();
                self.compile_ast_line(&while_expr.body, &mut body, locals);
                output.push(Instruction::LoopWhile { cond, body: body.into_boxed_slice() });
            },

            // "with" block
            ast::Expr::With(with_expr) => {
                let target = self.compile_ast_expr(&with_expr.target, locals);
                let mut body = Vec::new();
                self.compile_ast_line(&with_expr.body, &mut body, locals);
                output.push(Instruction::With { target, body: body.into_boxed_slice() });
            },

            // Unknown/invalid AST
            _ => {
                output.push(Instruction::RuntimeError { error: gml::Error::UnexpectedASTExpr(line.to_string()) });
            },
        }
    }

    /// Compile an AST expression into a Node.
    fn compile_ast_expr(&mut self, expr: &ast::Expr, locals: &[&[u8]]) -> Node {
        match expr {
            ast::Expr::LiteralReal(real) => Node::Literal { value: Value::Real(Real::from(*real)) },

            ast::Expr::LiteralString(string) => Node::Literal { value: Value::Str((*string).into()) },

            ast::Expr::LiteralIdentifier(string) => {
                if let Some(entry) = self.constants.get(*string) {
                    Node::Literal { value: entry.clone() }
                } else if let Some(constant_id) = self.user_constant_names.get(*string) {
                    Node::Constant { constant_id: *constant_id }
                } else if let Some(&v) = str::from_utf8(string).ok().and_then(|n| mappings::CONSTANTS.get(n)) {
                    Node::Literal { value: Value::Real(Real::from(v)) }
                } else {
                    self.identifier_to_variable(string, None, ArrayAccessor::None, locals)
                }
            },

            ast::Expr::Binary(binary_expr) => match &binary_expr.op {
                Operator::Deref => match &binary_expr.right {
                    ast::Expr::LiteralIdentifier(var_name) => {
                        let owner = self.make_instance_identifier(&binary_expr.left, locals);
                        self.identifier_to_variable(var_name, Some(owner), ArrayAccessor::None, locals)
                    },
                    _ => Node::RuntimeError { error: gml::Error::InvalidDeref(binary_expr.right.to_string()) },
                },

                Operator::Index => match &binary_expr.right {
                    ast::Expr::Group(dimensions) => {
                        let accessor = match self.make_array_accessor(dimensions, locals) {
                            Ok(a) => a,
                            Err(e) => return Node::RuntimeError { error: gml::Error::TooManyArrayDimensions(e) },
                        };
                        match &binary_expr.left {
                            ast::Expr::LiteralIdentifier(string) => {
                                self.identifier_to_variable(string, None, accessor, locals)
                            },
                            ast::Expr::Binary(binary_expr) => {
                                if let ast::BinaryExpr {
                                    left,
                                    right: ast::Expr::LiteralIdentifier(i),
                                    op: Operator::Deref,
                                } = binary_expr.as_ref()
                                {
                                    let owner = self.make_instance_identifier(left, locals);
                                    self.identifier_to_variable(i, Some(owner), accessor, locals)
                                } else {
                                    Node::RuntimeError {
                                        error: gml::Error::InvalidIndexLhs(format!("{:?}", binary_expr)),
                                    }
                                }
                            },
                            _ => {
                                Node::RuntimeError { error: gml::Error::InvalidIndexLhs(binary_expr.left.to_string()) }
                            },
                        }
                    },
                    _ => Node::RuntimeError { error: gml::Error::InvalidArrayAccessor(binary_expr.right.to_string()) },
                },

                op => {
                    let op_function = match op {
                        Operator::Add => BinaryOperator::Add,
                        Operator::And => BinaryOperator::And,
                        Operator::BitwiseAnd => BinaryOperator::BitwiseAnd,
                        Operator::BitwiseOr => BinaryOperator::BitwiseOr,
                        Operator::BinaryShiftLeft => BinaryOperator::BinaryShiftLeft,
                        Operator::BinaryShiftRight => BinaryOperator::BinaryShiftRight,
                        Operator::BitwiseXor => BinaryOperator::BitwiseXor,
                        Operator::Divide => BinaryOperator::Divide,
                        Operator::Equal => BinaryOperator::Equal,
                        Operator::GreaterThan => BinaryOperator::GreaterThan,
                        Operator::GreaterThanOrEqual => BinaryOperator::GreaterThanOrEqual,
                        Operator::IntDivide => BinaryOperator::IntDivide,
                        Operator::LessThan => BinaryOperator::LessThan,
                        Operator::LessThanOrEqual => BinaryOperator::LessThanOrEqual,
                        Operator::Multiply => BinaryOperator::Multiply,
                        Operator::Modulo => BinaryOperator::Modulo,
                        Operator::NotEqual => BinaryOperator::NotEqual,
                        Operator::Or => BinaryOperator::Or,
                        Operator::Subtract => BinaryOperator::Subtract,
                        Operator::Xor => BinaryOperator::Xor,
                        op => return Node::RuntimeError { error: gml::Error::InvalidBinaryOperator(*op) },
                    };

                    let left = self.compile_ast_expr(&binary_expr.left, locals);
                    let right = self.compile_ast_expr(&binary_expr.right, locals);

                    match (left, right) {
                        (Node::Literal { value: lhs @ _ }, Node::Literal { value: rhs @ _ }) => {
                            match op_function.call(lhs, rhs) {
                                Ok(value) => Node::Literal { value },
                                Err(error) => Node::RuntimeError { error },
                            }
                        },
                        (left, right) => Node::Binary {
                            left: Box::new(left),
                            right: Box::new(right),
                            operator: op_function,
                            type_unsafe: false,
                        },
                    }
                },
            },

            ast::Expr::Function(function) => {
                let args = function
                    .params
                    .iter()
                    .map(|x| self.compile_ast_expr(&x, locals))
                    .collect::<Vec<_>>()
                    .into_boxed_slice();

                if let Some(script_id) = self.get_script_id(function.name) {
                    Node::Script { args, script_id }
                } else if let Some(id) = self.extension_fn_names.get(function.name).copied() {
                    Node::ExtensionFunction { args, id }
                } else if let Some(function) =
                    str::from_utf8(function.name).ok().and_then(|n| mappings::FUNCTIONS.get(n))
                {
                    match function {
                        gml::Function::Runtime(f) => Node::ContextFunction { args, function: gml::FunctionPtr(*f) },
                        gml::Function::Engine(f) => Node::StateFunction { args, function: gml::FunctionPtr(*f) },
                        gml::Function::Volatile(f) |
                        gml::Function::Constant(f) => Node::RoutineFunction { args, function: gml::FunctionPtr(*f) },
                        gml::Function::Pure(f) => Node::ValueFunction { args, function: gml::FunctionPtr(*f) },
                    }
                } else {
                    Node::RuntimeError {
                        error: gml::Error::UnknownFunction(String::from_utf8_lossy(function.name).into()),
                    }
                }
            },

            ast::Expr::Unary(unary_expr) => {
                let new_node = self.compile_ast_expr(&unary_expr.child, locals);
                let operator = match unary_expr.op {
                    Operator::Add => return new_node,
                    Operator::Subtract => UnaryOperator::Neg,
                    Operator::Not => UnaryOperator::Not,
                    Operator::Complement => UnaryOperator::Complement,
                    _ => return Node::RuntimeError { error: gml::Error::InvalidUnaryOperator(unary_expr.op) },
                };
                match new_node {
                    Node::Literal { value } => match operator.call(value) {
                        Ok(value) => Node::Literal { value },
                        Err(error) => Node::RuntimeError { error },
                    },
                    node => Node::Unary { child: Box::new(node), operator },
                }
            },

            _ => Node::RuntimeError { error: gml::Error::UnexpectedASTExpr(expr.to_string()) },
        }
    }

    /// Searches for the fieldname id.
    pub fn find_field_id(&self, name: &[u8]) -> Option<usize> {
        self.fields.iter().position(|x| x.as_ref() == name)
    }

    /// Gets the unique id of a fieldname, registering one if it doesn't already exist.
    pub fn get_field_id(&mut self, name: &[u8]) -> usize {
        if let Some(i) = self.find_field_id(name) {
            i
        } else {
            // Note: this isn't thread-safe. Add a mutex lock if you want it to be thread-safe.
            let i = self.fields.len();
            self.fields.push(name.to_vec().into_boxed_slice());
            i
        }
    }

    pub fn get_script_id(&mut self, name: &[u8]) -> Option<usize> {
        self.script_names.get(name).copied()
    }

    /// Converts an AST BinaryExpr to an Instruction.
    fn binary_to_instruction(&mut self, binary_expr: &ast::BinaryExpr, locals: &[&[u8]]) -> Instruction {
        let modification_type = match binary_expr.op {
            Operator::Assign => None,
            Operator::AssignAdd => Some(BinaryOperator::Add),
            Operator::AssignSubtract => Some(BinaryOperator::Subtract),
            Operator::AssignMultiply => Some(BinaryOperator::Multiply),
            Operator::AssignDivide => Some(BinaryOperator::Divide),
            Operator::AssignBitwiseAnd => Some(BinaryOperator::BitwiseAnd),
            Operator::AssignBitwiseOr => Some(BinaryOperator::BitwiseOr),
            Operator::AssignBitwiseXor => Some(BinaryOperator::BitwiseXor),
            _ => unreachable!("Invalid assignment operator: {}", binary_expr.op),
        };

        let value = self.compile_ast_expr(&binary_expr.right, locals);
        match &binary_expr.left {
            ast::Expr::LiteralIdentifier(string) => {
                if let Some(mod_type) = modification_type {
                    self.make_modify_instruction(string, None, ArrayAccessor::None, mod_type, value, locals)
                } else {
                    self.make_set_instruction(string, None, ArrayAccessor::None, value, locals)
                }
            },
            ast::Expr::Binary(binary_expr) if binary_expr.op == Operator::Deref => {
                if let ast::Expr::LiteralIdentifier(string) = binary_expr.right {
                    let owner = self.make_instance_identifier(&binary_expr.left, locals);
                    if let Some(mod_type) = modification_type {
                        self.make_modify_instruction(string, Some(owner), ArrayAccessor::None, mod_type, value, locals)
                    } else {
                        self.make_set_instruction(string, Some(owner), ArrayAccessor::None, value, locals)
                    }
                } else {
                    Instruction::RuntimeError { error: gml::Error::InvalidDeref(binary_expr.right.to_string()) }
                }
            },
            ast::Expr::Binary(binary_expr) if binary_expr.op == Operator::Index => {
                if let ast::Expr::Group(dimensions) = &binary_expr.right {
                    let accessor = match self.make_array_accessor(dimensions, locals) {
                        Ok(a) => a,
                        Err(e) => return Instruction::RuntimeError { error: gml::Error::TooManyArrayDimensions(e) },
                    };
                    match &binary_expr.left {
                        ast::Expr::LiteralIdentifier(string) => {
                            if let Some(mod_type) = modification_type {
                                self.make_modify_instruction(string, None, accessor, mod_type, value, locals)
                            } else {
                                self.make_set_instruction(string, None, accessor, value, locals)
                            }
                        },
                        ast::Expr::Binary(binary_expr) if binary_expr.op == Operator::Deref => {
                            if let ast::Expr::LiteralIdentifier(string) = binary_expr.right {
                                let owner = self.make_instance_identifier(&binary_expr.left, locals);
                                if let Some(mod_type) = modification_type {
                                    self.make_modify_instruction(string, Some(owner), accessor, mod_type, value, locals)
                                } else {
                                    self.make_set_instruction(string, Some(owner), accessor, value, locals)
                                }
                            } else {
                                Instruction::RuntimeError {
                                    error: gml::Error::InvalidDeref(binary_expr.right.to_string()),
                                }
                            }
                        },
                        _ => Instruction::RuntimeError {
                            error: gml::Error::InvalidIndexLhs(binary_expr.left.to_string()),
                        },
                    }
                } else {
                    Instruction::RuntimeError { error: gml::Error::InvalidIndex(binary_expr.right.to_string()) }
                }
            },
            _ => Instruction::RuntimeError { error: gml::Error::InvalidAssignment(binary_expr.left.to_string()) },
        }
    }

    /// Converts an identifier to a Field, Variable or GameVariable accessor.
    /// If no VarOwner is provided (ie. the variable wasn't specified with one), this function will infer one.
    fn identifier_to_variable(
        &mut self,
        identifier: &[u8],
        owner: Option<InstanceIdentifier>,
        array: ArrayAccessor,
        locals: &[&[u8]],
    ) -> Node {
        let owner = match owner {
            Some(o) => o,
            None => {
                if locals.iter().any(|x| *x == identifier) {
                    InstanceIdentifier::Local
                } else {
                    InstanceIdentifier::Unknown
                }
            },
        };

        if let Some(var) = mappings::get_instance_variable_by_name(identifier) {
            Node::Variable { accessor: VariableAccessor { var: *var, array, owner } }
        } else {
            let index = self.get_field_id(identifier);
            Node::Field { accessor: FieldAccessor { index, array, owner } }
        }
    }

    /// Converts an identifier, owner, array accessor and value into a set instruction.
    /// If no owner is provided (ie. the variable wasn't specified with one), this function will infer one.
    fn make_set_instruction(
        &mut self,
        identifier: &[u8],
        owner: Option<InstanceIdentifier>,
        array: ArrayAccessor,
        value: Node,
        locals: &[&[u8]],
    ) -> Instruction {
        let owner = match owner {
            Some(o) => o,
            None => {
                if locals.iter().any(|x| *x == identifier) {
                    InstanceIdentifier::Local
                } else {
                    InstanceIdentifier::Unknown
                }
            },
        };

        if let Some(var) = mappings::get_instance_variable_by_name(identifier) {
            Instruction::SetVariable { accessor: VariableAccessor { var: *var, array, owner }, value }
        } else {
            let index = self.get_field_id(identifier);
            Instruction::SetField { accessor: FieldAccessor { index, array, owner }, value }
        }
    }

    /// Converts an identifier, owner, array accessor, modification-type and value into an instruction.
    /// If no owner is provided (ie. the variable wasn't specified with one), this function will infer one.
    fn make_modify_instruction(
        &mut self,
        identifier: &[u8],
        owner: Option<InstanceIdentifier>,
        array: ArrayAccessor,
        operator: BinaryOperator,
        value: Node,
        locals: &[&[u8]],
    ) -> Instruction {
        let owner = match owner {
            Some(o) => o,
            None => {
                if locals.iter().any(|x| *x == identifier) {
                    InstanceIdentifier::Local
                } else {
                    InstanceIdentifier::Unknown
                }
            },
        };

        if let Some(var) = mappings::get_instance_variable_by_name(identifier) {
            Instruction::SetVariable {
                accessor: VariableAccessor { var: *var, array: array.clone(), owner: owner.clone() },
                value: Node::Binary {
                    left: Box::new(Node::Variable { accessor: VariableAccessor { var: *var, array, owner } }),
                    right: Box::new(value),
                    operator,
                    type_unsafe: false,
                },
            }
        } else {
            let index = self.get_field_id(identifier);
            Instruction::SetField {
                accessor: FieldAccessor { index, array: array.clone(), owner: owner.clone() },
                value: Node::Binary {
                    left: Box::new(Node::Field { accessor: FieldAccessor { index, array, owner } }),
                    right: Box::new(value),
                    operator,
                    type_unsafe: true,
                },
            }
        }
    }

    /// Converts an AST node to an InstanceIdentifier.
    fn make_instance_identifier(&mut self, expression: &ast::Expr, locals: &[&[u8]]) -> InstanceIdentifier {
        let node = self.compile_ast_expr(expression, locals);
        if let Node::Literal { value: v @ Value::Real(_) } = &node {
            match v.round() {
                gml::SELF | gml::UNSPECIFIED => InstanceIdentifier::Own,
                gml::OTHER => InstanceIdentifier::Other,
                gml::GLOBAL => InstanceIdentifier::Global,
                gml::LOCAL => InstanceIdentifier::Local,
                _ => InstanceIdentifier::Expression(Box::new(node)),
            }
        } else {
            InstanceIdentifier::Expression(Box::new(node))
        }
    }

    /// Converts a list of expressions into an array accessor (or an error message).
    fn make_array_accessor(&mut self, expression_list: &[ast::Expr], locals: &[&[u8]]) -> Result<ArrayAccessor, usize> {
        match expression_list {
            [] => Ok(ArrayAccessor::None),
            [d] => Ok(ArrayAccessor::Single(Box::new(self.compile_ast_expr(d, locals)))),
            [d1, d2] => Ok(ArrayAccessor::Double(
                Box::new(self.compile_ast_expr(d1, locals)),
                Box::new(self.compile_ast_expr(d2, locals)),
            )),
            _ => Err(expression_list.len()),
        }
    }

    /// Get a field name by its ID. This clones the string; it should only be used in the case of an error.
    pub fn get_field_name(&self, id: usize) -> Option<String> {
        self.fields.get(id).map(|s| String::from_utf8_lossy(s).into())
    }
}
