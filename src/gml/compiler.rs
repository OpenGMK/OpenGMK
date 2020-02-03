pub mod ast;
pub mod lexer;
pub mod mappings;
pub mod token;

use super::{
    runtime::{ArrayAccessor, Instruction, Node, VarOwner},
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
        Self {
            constants: HashMap::new(),
            script_names: HashMap::new(),
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
            panic!(
                "Two scripts with the same name registered: at index {} and {}",
                v, index
            );
        }
    }

    /// Compile a GML string into instructions.
    pub fn compile(&mut self, source: &str) -> Result<Vec<Instruction>, ast::Error> {
        let ast = ast::AST::new(source)?;

        let instructions = Vec::new();
        for _node in ast.into_iter() {
            // TODO: this
        }
        Ok(instructions)
    }

    /// Compile an expression into a format which can be evaluated.
    pub fn compile_expression(&mut self, source: &str) -> Result<Node, ast::Error> {
        let expr = ast::AST::expression(source)?;
        Ok(self.compile_ast_expr(&expr, &vec![]))
    }

    fn compile_ast_expr(&mut self, expr: &ast::Expr, locals: &[&str]) -> Node {
        match expr {
            ast::Expr::LiteralReal(real) => Node::Literal {
                value: Value::Real(*real),
            },

            ast::Expr::LiteralString(string) => Node::Literal {
                value: Value::Str((*string).into()),
            },

            ast::Expr::LiteralIdentifier(string) => {
                if let Some(entry) = self.constants.get(*string) {
                    Node::Literal { value: entry.clone() }
                } else if let Some(f) = mappings::CONSTANTS.iter().find(|(s, _)| s == string).map(|(_, v)| v) {
                    Node::Literal { value: Value::Real(*f) }
                } else {
                    self.identifier_to_variable(string, None, ArrayAccessor::None, locals)
                }
            }

            ast::Expr::Binary(binary_expr) => match &binary_expr.op {
                Operator::Deref => match &binary_expr.right {
                    ast::Expr::LiteralIdentifier(var_name) => {
                        let owner = self.make_varowner(&binary_expr.left, locals);
                        self.identifier_to_variable(var_name, Some(owner), ArrayAccessor::None, locals)
                    }
                    _ => Node::RuntimeError {
                        error: format!("Invalid deref RHS: {:?}", binary_expr.right),
                    },
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
                            }
                            ast::Expr::Binary(binary_expr) => {
                                if let ast::BinaryExpr {
                                    left,
                                    right: ast::Expr::LiteralIdentifier(i),
                                    op: Operator::Deref,
                                } = binary_expr.as_ref()
                                {
                                    let owner = self.make_varowner(left, locals);
                                    self.identifier_to_variable(i, Some(owner), accessor, locals)
                                } else {
                                    Node::RuntimeError {
                                        error: format!("Invalid LHS for indexing: {:?}", binary_expr),
                                    }
                                }
                            }
                            _ => Node::RuntimeError {
                                error: format!("Invalid object for indexing: {:?}", binary_expr.left),
                            },
                        }
                    }
                    _ => Node::RuntimeError {
                        error: format!("Invalid array accessor: {:?}", binary_expr.right),
                    },
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
                            return Node::RuntimeError {
                                error: format!("Invalid binary operator: {}", op),
                            };
                        }
                    };

                    let left = self.compile_ast_expr(&binary_expr.left, locals);
                    let right = self.compile_ast_expr(&binary_expr.right, locals);

                    match (left, right) {
                        (
                            Node::Literal {
                                value: lhs @ Value::Real(_),
                            },
                            Node::Literal {
                                value: rhs @ Value::Real(_),
                            },
                        ) => Node::Literal {
                            value: op_function(lhs, rhs),
                        },
                        (
                            Node::Literal {
                                value: lhs @ Value::Str(_),
                            },
                            Node::Literal {
                                value: rhs @ Value::Str(_),
                            },
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
                            Node::Literal {
                                value: op_function(lhs, rhs),
                            }
                        }
                        (left, right) => Node::Binary {
                            left: Box::new(left),
                            right: Box::new(right),
                            operator: op_function,
                        },
                    }
                }
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
            }

            ast::Expr::Unary(unary_expr) => {
                let new_node = self.compile_ast_expr(&unary_expr.child, locals);
                let operator = match unary_expr.op {
                    Operator::Add => return new_node,
                    Operator::Subtract => Value::neg,
                    Operator::Not => Value::not,
                    Operator::Complement => Value::complement,
                    _ => {
                        return Node::RuntimeError {
                            error: format!("Unknown unary operator: {:?}", unary_expr.op),
                        };
                    }
                };
                if let Node::Literal {
                    value: v @ Value::Real(_),
                } = new_node
                {
                    Node::Literal { value: operator(v) }
                } else {
                    Node::Unary {
                        child: Box::new(new_node),
                        operator,
                    }
                }
            }

            _ => Node::RuntimeError {
                error: format!("Unexpected type of AST Expr in expression: {:?}", expr),
            },
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

    /// Converts an identifier to a Field, Variable or GameVariable accessor.
    /// If no VarOwner is provided (ie. the variable wasn't specified with one), this function will infer one.
    fn identifier_to_variable(
        &mut self,
        identifier: &str,
        owner: Option<VarOwner>,
        array: ArrayAccessor,
        locals: &[&str],
    ) -> Node {
        let owner = match owner {
            Some(o) => o,
            None => {
                if locals.iter().position(|x| *x == identifier).is_some() {
                    VarOwner::Local
                } else {
                    VarOwner::Own
                }
            }
        };

        if let Some(var) = mappings::GAME_VARIABLES
            .iter()
            .find(|(s, _)| *s == identifier)
            .map(|(_, v)| v)
        {
            Node::GameVariable {
                var: *var,
                array,
                owner,
            }
        } else if let Some(var) = mappings::INSTANCE_VARIABLES
            .iter()
            .find(|(s, _)| *s == identifier)
            .map(|(_, v)| v)
        {
            Node::Variable {
                var: *var,
                array,
                owner,
            }
        } else {
            let index = self.get_field_id(identifier);
            Node::Field { index, array, owner }
        }
    }

    /// Converts an AST node to a VarOwner.
    fn make_varowner(&mut self, expression: &ast::Expr, locals: &[&str]) -> VarOwner {
        let node = self.compile_ast_expr(expression, locals);
        if let Node::Literal {
            value: v @ Value::Real(_),
        } = &node
        {
            match v.round() {
                -1 => VarOwner::Own,
                -2 => VarOwner::Other,
                -5 => VarOwner::Global,
                -7 => VarOwner::Local,
                _ => VarOwner::Expression(Box::new(node)),
            }
        } else {
            VarOwner::Expression(Box::new(node))
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
