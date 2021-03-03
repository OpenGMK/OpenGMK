use crate::mappings;
use gm8exe::GameAssets;
use gml_parser::ast::{self, AST};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Mode {
    On,
    Off,
    Auto,
}

struct Deobfuscator<'a> {
    assets: &'a mut GameAssets,
    fields: Vec<Box<[u8]>>,
    constants: HashMap<&'static str, f64>,
    vars: HashSet<&'static str>,
}

struct ExprWriter<'dest> {
    indent: usize,
    output: &'dest mut Vec<u8>,
}

pub fn process<'a>(assets: &'a mut GameAssets) {
    let constants = mappings::make_constants_map();
    let vars = mappings::make_kernel_vars_lut();
    let deobfuscator = Deobfuscator {
        assets,
        fields: Vec::new(),
        constants,
        vars,
    };
}

impl<'a> Deobfuscator<'a> {
    pub fn process_gml(&mut self, input: &'a [u8]) -> Result<Vec<u8>, ast::Error> {
        let mut output = Vec::new();
        let ast = AST::new(input)?;

        let mut writer = ExprWriter {
            indent: 0,
            output: &mut output,
        };

        for expr in ast {
            self.process_expr(&expr, &mut writer);
        }

        Ok(output)
    }

    pub fn process_expression(&mut self, input: &'a [u8]) -> Result<Vec<u8>, ast::Error> {
        let mut output = Vec::new();
        let expr = AST::expression(input)?;

        let mut writer = ExprWriter {
            indent: 0,
            output: &mut output,
        };

        self.process_expr(&expr, &mut writer);

        Ok(output)
    }

    pub fn process_expr(&mut self, expr: &ast::Expr, writer: &mut ExprWriter) {
        for _ in 0..writer.indent {
            writer.output.extend_from_slice(b"    ");
        }
        match expr {
            ast::Expr::LiteralIdentifier(expr) => (),
            ast::Expr::LiteralReal(real) => {
                let real = real.to_string();
                writer.output.extend_from_slice(real.as_bytes());
            },
            ast::Expr::LiteralString(string) => {
                let quote = if string.iter().any(|&x| x == b'"') {b'\''} else {b'"'};
                writer.output.push(quote);
                writer.output.extend_from_slice(string);
                writer.output.push(quote);
            },
            ast::Expr::Unary(expr) => (),
            ast::Expr::Binary(expr) => (),
            ast::Expr::DoUntil(expr) => (),
            ast::Expr::For(expr) => (),
            ast::Expr::Function(expr) => (),
            ast::Expr::Group(expr) => (),
            ast::Expr::If(expr) => (),
            ast::Expr::Repeat(expr) => (),
            ast::Expr::Switch(expr) => (),
            ast::Expr::Var(expr) => {
                if expr.vars.len() > 0 {
                    writer.output.extend_from_slice(b"var ");
                    for (i, name) in expr.vars.iter().enumerate() {
                        if i != 0 {
                            writer.output.extend_from_slice(b", ");
                        }
                        let field_number = self.register_field(name);
                        write!(writer.output, "field{}", field_number);
                    }
                    writer.output.extend_from_slice(b";\r\n");
                }
            },
            ast::Expr::GlobalVar(expr) => (),
            ast::Expr::With(expr) => (),
            ast::Expr::While(expr) => (),
            ast::Expr::Case(expr) => {
                writer.output.extend_from_slice(b"case ");
                self.process_expr(expr, writer);
                writer.output.extend_from_slice(b":\r\n");
            },
            ast::Expr::Default => writer.output.extend_from_slice(b"default:\r\n"),
            ast::Expr::Continue => writer.output.extend_from_slice(b"continue;\r\n"),
            ast::Expr::Break => writer.output.extend_from_slice(b"break;\r\n"),
            ast::Expr::Exit => writer.output.extend_from_slice(b"exit;\r\n"),
            ast::Expr::Return(expr) => {
                writer.output.extend_from_slice(b"return ");
                self.process_expr(expr, writer);
                writer.output.extend_from_slice(b";\r\n");
            }
        }
    }

    pub fn register_field(&mut self, field: &[u8]) -> usize {
        match self.fields.iter().position(|x| &**x == field) {
            Some(x) => x,
            None => {
                let pos = self.fields.len();
                self.fields.push(field.into());
                pos
            },
        }
    }
}
