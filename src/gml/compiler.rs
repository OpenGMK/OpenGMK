pub mod ast;
pub mod lexer;
pub mod mappings;
pub mod token;

use super::{
    runtime::{
        ArrayAccessor, AssignmentType, FieldAccessor, GameVariableAccessor, InstanceIdentifier, Instruction, Node,
        ReturnType, VariableAccessor,
    },
    Value,
};
use std::{
    collections::HashMap,
    ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Neg, Not, Rem, Shl, Shr, Sub},
};
use token::Operator;

pub struct Compiler {
    /// List of identifiers which represent const values
    constants: HashMap<String, Value>,

    /// Table of script names to IDs
    script_names: HashMap<String, usize>,

    /// Lookup table of unique field names
    fields: Vec<String>,
}

impl Compiler {
    /// Create a compiler.
    pub fn new() -> Self {
        Self { constants: HashMap::new(), script_names: HashMap::new(), fields: Vec::new() }
    }

    /// Reserve space to register at least the given number of constants.
    pub fn reserve_constants(&mut self, size: usize) {
        self.constants.reserve(size)
    }

    /// Reserve space to register at least the given number of script names.
    pub fn reserve_scripts(&mut self, size: usize) {
        self.script_names.reserve(size)
    }

    /// Add a constant and its associated f64 value, such as an asset name.
    /// These constants will override built-in ones, such as c_red. However, if the same constant name is
    /// registered twice, the old one will NOT be overwritten and the value will be dropped, as per GM8.
    pub fn register_constant(&mut self, name: String, value: f64) {
        self.constants.entry(name).or_insert(Value::Real(value));
    }

    /// Register a script name and its index.
    /// Panics if two identical script names are registered - GM8 does not allow this.
    pub fn register_script(&mut self, name: String, index: usize) {
        if let Some(v) = self.script_names.insert(name, index) {
            panic!("Two scripts with the same name registered: at index {} and {}", v, index);
        }
    }

    /// Compile a GML string into instructions.
    pub fn compile(&mut self, source: &str) -> Result<Vec<Instruction>, ast::Error> {
        let ast = ast::AST::new(source)?;

        let mut instructions = Vec::new();
        let mut locals: Vec<&str> = Vec::new();
        for node in ast.iter() {
            self.compile_ast_line(node, &mut instructions, &mut locals);
        }
        Ok(instructions)
    }

    /// Compile an expression into a format which can be evaluated.
    pub fn compile_expression(&mut self, source: &str) -> Result<Node, ast::Error> {
        let expr = ast::AST::expression(source)?;
        Ok(self.compile_ast_expr(&expr, &[]))
    }

    /// Compile a single line of code from an AST expression.
    fn compile_ast_line<'a>(&mut self, line: &'a ast::Expr, output: &mut Vec<Instruction>, locals: &mut Vec<&'a str>) {
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
                self.compile_ast_line(&for_expr.step, &mut body, locals);
                output.push(Instruction::LoopWhile { cond, body: body.into_boxed_slice() });
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
                    if v.is_true() {
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
                    for expr in group {
                        if let ast::Expr::Case(case_expr) = expr {
                            cases.push((self.compile_ast_expr(case_expr, locals), body.len()));
                        }
                        else {
                            self.compile_ast_line(expr, &mut body, locals);
                        }
                    }
                    output.push(Instruction::Switch { input, cases: cases.into_boxed_slice(), body: body.into_boxed_slice() });
                } else {
                    output.push(Instruction::RuntimeError { error: format!("Invalid switch body: {:?}", switch_expr.body) });
                }
            }

            // "var" declaration
            ast::Expr::Var(var_expr) => {
                locals.extend_from_slice(&var_expr.vars);
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
                output.push(Instruction::RuntimeError { error: format!("Invalid AST expression: {:?}", line) });
            },
        }
    }

    /// Compile an AST expression into a Node.
    fn compile_ast_expr(&mut self, expr: &ast::Expr, locals: &[&str]) -> Node {
        match expr {
            ast::Expr::LiteralReal(real) => Node::Literal { value: Value::Real(*real) },

            ast::Expr::LiteralString(string) => Node::Literal { value: Value::Str((*string).into()) },

            ast::Expr::LiteralIdentifier(string) => {
                if let Some(entry) = self.constants.get(*string) {
                    Node::Literal { value: entry.clone() }
                } else if let Some(f) = mappings::CONSTANTS.iter().find(|(s, _)| s == string).map(|(_, v)| v) {
                    Node::Literal { value: Value::Real(*f) }
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
                    _ => Node::RuntimeError { error: format!("Invalid deref RHS: {:?}", binary_expr.right) },
                },

                Operator::Index => match &binary_expr.right {
                    ast::Expr::Group(dimensions) => {
                        let accessor = match self.make_array_accessor(dimensions, locals) {
                            Ok(a) => a,
                            Err(e) => return Node::RuntimeError { error: e },
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
                                    Node::RuntimeError { error: format!("Invalid LHS for indexing: {:?}", binary_expr) }
                                }
                            },
                            _ => Node::RuntimeError {
                                error: format!("Invalid object for indexing: {:?}", binary_expr.left),
                            },
                        }
                    },
                    _ => Node::RuntimeError { error: format!("Invalid array accessor: {:?}", binary_expr.right) },
                },

                op => {
                    let op_function = match op {
                        Operator::Add => Value::add,
                        Operator::BinaryAnd => Value::bitand,
                        Operator::BinaryOr => Value::bitor,
                        Operator::BinaryShiftLeft => Value::shl,
                        Operator::BinaryShiftRight => Value::shr,
                        Operator::BinaryXor => Value::bitxor,
                        Operator::Divide => Value::div,
                        Operator::Equal => Value::gml_eq,
                        Operator::GreaterThan => Value::gml_gt,
                        Operator::GreaterThanOrEqual => Value::gml_gte,
                        Operator::IntDivide => Value::intdiv,
                        Operator::LessThan => Value::gml_lt,
                        Operator::LessThanOrEqual => Value::gml_lte,
                        Operator::Multiply => Value::mul,
                        Operator::Modulo => Value::rem,
                        Operator::NotEqual => Value::gml_ne,
                        Operator::Subtract => Value::sub,
                        op => {
                            return Node::RuntimeError { error: format!("Invalid binary operator: {}", op) };
                        },
                    };

                    let left = self.compile_ast_expr(&binary_expr.left, locals);
                    let right = self.compile_ast_expr(&binary_expr.right, locals);

                    match (left, right) {
                        (
                            Node::Literal { value: lhs @ Value::Real(_) },
                            Node::Literal { value: rhs @ Value::Real(_) },
                        ) => Node::Literal { value: op_function(lhs, rhs) },
                        (
                            Node::Literal { value: lhs @ Value::Str(_) },
                            Node::Literal { value: rhs @ Value::Str(_) },
                        ) if match *op {
                            Operator::Add
                            | Operator::Equal
                            | Operator::NotEqual
                            | Operator::GreaterThan
                            | Operator::GreaterThanOrEqual
                            | Operator::LessThan
                            | Operator::LessThanOrEqual => true,
                            _ => false,
                        } =>
                        {
                            Node::Literal { value: op_function(lhs, rhs) }
                        },
                        (left, right) => {
                            Node::Binary { left: Box::new(left), right: Box::new(right), operator: op_function }
                        },
                    }
                },
            },

            ast::Expr::Function(function) => {
                if let Some(script_id) = self.script_names.get(function.name) {
                    let script_id = *script_id;
                    Node::Script {
                        args: function
                            .params
                            .iter()
                            .map(|x| self.compile_ast_expr(&x, locals))
                            .collect::<Vec<_>>()
                            .into_boxed_slice(),
                        script_id,
                    }
                } else {
                    todo!("Functions")
                }
            },

            ast::Expr::Unary(unary_expr) => {
                let new_node = self.compile_ast_expr(&unary_expr.child, locals);
                let operator = match unary_expr.op {
                    Operator::Add => return new_node,
                    Operator::Subtract => Value::neg,
                    Operator::Not => Value::not,
                    Operator::Complement => Value::complement,
                    _ => {
                        return Node::RuntimeError { error: format!("Unknown unary operator: {:?}", unary_expr.op) };
                    },
                };
                if let Node::Literal { value: v @ Value::Real(_) } = new_node {
                    Node::Literal { value: operator(v) }
                } else {
                    Node::Unary { child: Box::new(new_node), operator }
                }
            },

            _ => Node::RuntimeError { error: format!("Unexpected type of AST Expr in expression: {:?}", expr) },
        }
    }

    /// Gets the unique id of a fieldname, registering one if it doesn't already exist.
    fn get_field_id(&mut self, name: &str) -> usize {
        if let Some(i) = self.fields.iter().position(|x| x == name) {
            i
        } else {
            // Note: this isn't thread-safe. Add a mutex lock if you want it to be thread-safe.
            let i = self.fields.len();
            self.fields.push(String::from(name));
            i
        }
    }

    /// Converts an AST BinaryExpr to an Instruction.
    fn binary_to_instruction(&mut self, binary_expr: &ast::BinaryExpr, locals: &[&str]) -> Instruction {
        let assignment_type = match binary_expr.op {
            Operator::Assign => AssignmentType::Set,
            Operator::AssignAdd => AssignmentType::Add,
            Operator::AssignSubtract => AssignmentType::Subtract,
            Operator::AssignMultiply => AssignmentType::Multiply,
            Operator::AssignDivide => AssignmentType::Divide,
            Operator::AssignBinaryAnd => AssignmentType::BitAnd,
            Operator::AssignBinaryOr => AssignmentType::BitOr,
            Operator::AssignBinaryXor => AssignmentType::BitXor,
            _ => {
                return Instruction::RuntimeError { error: format!("Invalid assignment operator: {}", binary_expr.op) };
            },
        };

        let value = self.compile_ast_expr(&binary_expr.right, locals);
        match &binary_expr.left {
            ast::Expr::LiteralIdentifier(string) => {
                self.make_set_instruction(string, None, ArrayAccessor::None, assignment_type, value, locals)
            },
            ast::Expr::Binary(binary_expr) if binary_expr.op == Operator::Deref => {
                if let ast::Expr::LiteralIdentifier(string) = binary_expr.right {
                    let owner = self.make_instance_identifier(&binary_expr.left, locals);
                    self.make_set_instruction(string, Some(owner), ArrayAccessor::None, assignment_type, value, locals)
                } else {
                    Instruction::RuntimeError { error: format!("Invalid deref RHS: {}", binary_expr.right) }
                }
            },
            ast::Expr::Binary(binary_expr) if binary_expr.op == Operator::Index => {
                if let ast::Expr::Group(dimensions) = &binary_expr.right {
                    let accessor = match self.make_array_accessor(dimensions, locals) {
                        Ok(a) => a,
                        Err(e) => return Instruction::RuntimeError { error: e },
                    };
                    match &binary_expr.left {
                        ast::Expr::LiteralIdentifier(string) => {
                            self.make_set_instruction(string, None, accessor, assignment_type, value, locals)
                        },
                        ast::Expr::Binary(binary_expr) if binary_expr.op == Operator::Deref => {
                            if let ast::Expr::LiteralIdentifier(string) = binary_expr.right {
                                let owner = self.make_instance_identifier(&binary_expr.left, locals);
                                self.make_set_instruction(string, Some(owner), accessor, assignment_type, value, locals)
                            } else {
                                Instruction::RuntimeError { error: format!("Invalid deref RHS: {}", binary_expr.right) }
                            }
                        },
                        _ => Instruction::RuntimeError { error: format!("Invalid index LHS: {}", binary_expr.left) },
                    }
                } else {
                    Instruction::RuntimeError { error: format!("Invalid index RHS: {}", binary_expr.right) }
                }
            },
            _ => Instruction::RuntimeError { error: format!("Invalid assignment LHS: {}", binary_expr.left) },
        }
    }

    /// Converts an identifier to a Field, Variable or GameVariable accessor.
    /// If no VarOwner is provided (ie. the variable wasn't specified with one), this function will infer one.
    fn identifier_to_variable(
        &mut self,
        identifier: &str,
        owner: Option<InstanceIdentifier>,
        array: ArrayAccessor,
        locals: &[&str],
    ) -> Node {
        let owner = match owner {
            Some(o) => o,
            None => {
                if locals.iter().any(|x| *x == identifier) {
                    InstanceIdentifier::Local
                } else {
                    InstanceIdentifier::Own
                }
            },
        };

        if let Some(var) = mappings::GAME_VARIABLES.iter().find(|(s, _)| *s == identifier).map(|(_, v)| v) {
            Node::GameVariable { accessor: GameVariableAccessor { var: *var, array, owner } }
        } else if let Some(var) = mappings::INSTANCE_VARIABLES.iter().find(|(s, _)| *s == identifier).map(|(_, v)| v) {
            Node::Variable { accessor: VariableAccessor { var: *var, array, owner } }
        } else {
            let index = self.get_field_id(identifier);
            Node::Field { accessor: FieldAccessor { index, array, owner } }
        }
    }

    /// Converts an identifier, owner, array accessor, assignment-type and value into an instruction.
    /// If no owner is provided (ie. the variable wasn't specified with one), this function will infer one.
    fn make_set_instruction(
        &mut self,
        identifier: &str,
        owner: Option<InstanceIdentifier>,
        array: ArrayAccessor,
        assignment_type: AssignmentType,
        value: Node,
        locals: &[&str],
    ) -> Instruction {
        let owner = match owner {
            Some(o) => o,
            None => {
                if locals.iter().any(|x| *x == identifier) {
                    InstanceIdentifier::Local
                } else {
                    InstanceIdentifier::Own
                }
            },
        };

        if let Some(var) = mappings::GAME_VARIABLES.iter().find(|(s, _)| *s == identifier).map(|(_, v)| v) {
            Instruction::SetGameVariable {
                accessor: GameVariableAccessor { var: *var, array, owner },
                assignment_type,
                value,
            }
        } else if let Some(var) = mappings::INSTANCE_VARIABLES.iter().find(|(s, _)| *s == identifier).map(|(_, v)| v) {
            Instruction::SetVariable { accessor: VariableAccessor { var: *var, array, owner }, assignment_type, value }
        } else {
            let index = self.get_field_id(identifier);
            Instruction::SetField { accessor: FieldAccessor { index, array, owner }, assignment_type, value }
        }
    }

    /// Converts an AST node to an InstanceIdentifier.
    fn make_instance_identifier(&mut self, expression: &ast::Expr, locals: &[&str]) -> InstanceIdentifier {
        let node = self.compile_ast_expr(expression, locals);
        if let Node::Literal { value: v @ Value::Real(_) } = &node {
            match v.round() {
                -1 => InstanceIdentifier::Own,
                -2 => InstanceIdentifier::Other,
                -5 => InstanceIdentifier::Global,
                -7 => InstanceIdentifier::Local,
                _ => InstanceIdentifier::Expression(Box::new(node)),
            }
        } else {
            InstanceIdentifier::Expression(Box::new(node))
        }
    }

    /// Converts a list of expressions into an array accessor (or an error message).
    fn make_array_accessor(&mut self, expression_list: &[ast::Expr], locals: &[&str]) -> Result<ArrayAccessor, String> {
        match expression_list {
            [] => Ok(ArrayAccessor::None),
            [d] => Ok(ArrayAccessor::Single(Box::new(self.compile_ast_expr(d, locals)))),
            [d1, d2] => Ok(ArrayAccessor::Double(
                Box::new(self.compile_ast_expr(d1, locals)),
                Box::new(self.compile_ast_expr(d2, locals)),
            )),
            _ => Err(format!("Tried to access {}-dimensional array", expression_list.len())),
        }
    }
}
