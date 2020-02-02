pub mod ast;
pub mod lexer;
pub mod mappings;
pub mod token;

use super::{
    runtime::{Instruction, Node},
    Value,
};
use std::{
    collections::HashMap,
    ops::{Neg, Not},
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
        Ok(self.compile_ast_expr(expr))
    }

    fn compile_ast_expr(&mut self, expr: ast::Expr) -> Node {
        match expr {
            ast::Expr::LiteralReal(real) => Node::Literal {
                value: Value::Real(real),
            },

            ast::Expr::LiteralString(string) => Node::Literal {
                value: Value::Str(string.into()),
            },

            ast::Expr::LiteralIdentifier(string) => {
                if let Some(entry) = self.constants.get(string) {
                    Node::Literal { value: entry.clone() }
                } else if let Some(f) = mappings::GML_CONSTANTS
                    .iter()
                    .find(|(s, _)| *s == string)
                    .map(|(_, v)| v)
                {
                    Node::Literal { value: Value::Real(*f) }
                } else {
                    todo!("Distinguish game vars, instance vars, fields")
                }
            }

            ast::Expr::Function(function) => {
                if let Some(script_id) = self.script_names.get(function.name) {
                    let script_id = *script_id;
                    Node::Script {
                        args: function
                            .params
                            .into_iter()
                            .map(|x| self.compile_ast_expr(x))
                            .collect::<Vec<_>>()
                            .into_boxed_slice(),
                        script_id,
                    }
                } else {
                    todo!("Functions")
                }
            }

            ast::Expr::Unary(unary_expr) => {
                let new_node = self.compile_ast_expr(unary_expr.child);
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

    fn get_field_id(&mut self, name: &str) -> usize {
        if let Some(i) = self.fields.iter().position(|x| x == name) {
            i
        } else {
            let i = self.fields.len();
            self.fields.push(String::from(name));
            i
        }
    }
}
