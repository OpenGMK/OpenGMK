use crate::mappings;
use gm8exe::{asset::PascalString, GameAssets};
use gml_parser::{
    ast::{self, AST},
    token::Operator,
};
use std::{
    collections::{HashMap, HashSet},
    io::Write,
};

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Mode {
    On,
    Off,
    Auto,
}

struct DeobfState {
    fields: Vec<Box<[u8]>>,
    constants: HashMap<&'static [u8], f64>,
    vars: HashSet<&'static [u8]>,
}

struct ExprWriter<'a, 'b, 'c> {
    assets: &'a GameAssets,
    deobf: &'b mut DeobfState,
    indent: usize,
    indent_str: String,
    output: &'c mut Vec<u8>,

    is_gml_expr: bool,        // whether we're in a gml expr
    group_skip_newline: bool, // this too
}

pub fn process<'a>(assets: &'a mut GameAssets) {
    let constants = mappings::make_constants_map();
    let vars = mappings::make_kernel_vars_lut();
    let mut deobfuscator = DeobfState { fields: Vec::new(), constants, vars };

    for i in 0..assets.scripts.len() {
        let script = match &assets.scripts[i] {
            Some(x) => x,
            None => continue,
        };
        match deobfuscator.process_gml(&script.source.0, &assets) {
            Ok(res) => {
                assets.scripts[i].as_mut().unwrap().source = PascalString(res.into());
            },
            Err(err) => {
                eprintln!(
                    "[Warning] Failed to deobfuscate script {} ({}): {}",
                    i,
                    std::str::from_utf8(&script.name.0).unwrap_or("<INVALID UTF-8>"),
                    err,
                )
            },
        }
    }

    // Mass rename assets
    for (i, script) in assets.scripts.iter_mut().enumerate().filter_map(|(i, o)| o.as_mut().map(|x| (i, x))) {
        script.name = PascalString(format!("script{}", i).into_bytes().into());
    }
}

impl DeobfState {
    pub fn process_gml(&mut self, input: &[u8], assets: &GameAssets) -> Result<Vec<u8>, ast::Error> {
        let mut output = Vec::new();
        let ast = AST::new(input)?;

        let mut writer = ExprWriter {
            assets,
            deobf: self,
            indent: 0,
            indent_str: "    ".into(),
            output: &mut output,

            is_gml_expr: false,
            group_skip_newline: false,
        };

        for expr in ast {
            writer.process_expr(&expr);
        }

        Ok(output)
    }

    pub fn process_expression(&mut self, input: &[u8], assets: &GameAssets) -> Result<Vec<u8>, ast::Error> {
        let mut output = Vec::new();
        let expr = AST::expression(input)?;
        let mut writer = ExprWriter {
            assets,
            deobf: self,
            indent: 0,
            indent_str: "    ".into(),
            output: &mut output,

            is_gml_expr: true,
            group_skip_newline: false,
        };
        writer.process_expr(&expr);
        Ok(output)
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

    pub fn simplify(&mut self, expr: &ast::Expr, assets: &GameAssets) -> Option<f64> {
        match expr {
            ast::Expr::LiteralIdentifier(ident) => {
                if let Some((_, index)) = self.get_asset_by_name(ident, assets) {
                    Some(index as f64)
                } else if ident == b"pi" {
                    // We don't want to simplify pi.
                    None
                } else {
                    self.constants.get(ident).copied()
                }
            },
            ast::Expr::LiteralReal(real) => Some(*real),
            ast::Expr::Unary(unary) => {
                let child = self.simplify(&unary.child, assets)?;
                match unary.op {
                    Operator::Add => Some(child),
                    Operator::Subtract => Some(-child),
                    _ => None, // technically there's others. none used by obf
                }
            },
            ast::Expr::Binary(binary) => {
                let left = self.simplify(&binary.left, assets)?;
                let right = self.simplify(&binary.right, assets)?;
                match binary.op {
                    Operator::Add => Some(left + right),
                    Operator::Subtract => Some(left - right),
                    _ => None, // rest are unsupported
                }
            },
            _ => None,
        }
    }

    pub fn get_asset_by_name(&self, name: &[u8], assets: &GameAssets) -> Option<(&'static [u8], usize)> {
        fn find_asset<'a, T>(
            ty: &'static str,
            assets: &'a [Option<Box<T>>],
            mut f: impl FnMut(&'a T) -> bool,
        ) -> Option<(&'static [u8], usize)> {
            assets.iter().position(|x| x.as_ref().map(|b| f(b.as_ref())).unwrap_or(false)).map(|i| (ty.as_bytes(), i))
        }

        None.or_else(|| find_asset("object", &assets.objects, |x| &*x.name.0 == name))
            .or_else(|| find_asset("sprite", &assets.sprites, |x| &*x.name.0 == name))
            .or_else(|| find_asset("sound", &assets.sounds, |x| &*x.name.0 == name))
            .or_else(|| find_asset("background", &assets.backgrounds, |x| &*x.name.0 == name))
            .or_else(|| find_asset("path", &assets.paths, |x| &*x.name.0 == name))
            .or_else(|| find_asset("font", &assets.fonts, |x| &*x.name.0 == name))
            .or_else(|| find_asset("timeline", &assets.timelines, |x| &*x.name.0 == name))
            .or_else(|| find_asset("script", &assets.scripts, |x| &*x.name.0 == name))
            .or_else(|| find_asset("room", &assets.rooms, |x| &*x.name.0 == name))
            .or_else(|| find_asset("trigger", &assets.triggers, |x| &*x.constant_name.0 == name))
            .or_else(|| assets.constants.iter().position(|x| &*x.name.0 == name).map(|i| ("constant".as_bytes(), i)))
    }
}

impl<'a, 'b, 'c> ExprWriter<'a, 'b, 'c> {
    pub fn process_expr(&mut self, ex: &'_ ast::Expr) {
        macro_rules! push_str {
            ($lit: literal) => {
                self.output.extend_from_slice(($lit).as_bytes());
            };
        }

        match ex {
            ast::Expr::LiteralIdentifier(expr) => {
                if let Some((ty, index)) = self.deobf.get_asset_by_name(expr, self.assets) {
                    self.output.extend_from_slice(ty);
                    let _ = write!(self.output, "{}", index);
                } else if self.deobf.vars.get(expr).is_some() || self.deobf.constants.get(expr).is_some() {
                    self.output.extend_from_slice(expr);
                } else {
                    self.write_field(expr);
                }
            },
            ast::Expr::LiteralReal(real) => {
                let _ = write!(self.output, "{}", real);
            },
            ast::Expr::LiteralString(string) => {
                let quote = if string.iter().any(|&x| x == b'"') { b'\'' } else { b'"' };
                self.output.push(quote);
                self.output.extend_from_slice(string);
                self.output.push(quote);
            },
            ast::Expr::Unary(expr) => {
                let op = op_to_str(expr.op);
                self.output.extend_from_slice(op);
                let prev_state = self.is_gml_expr;
                self.is_gml_expr = true;
                let is_child_binary = matches!(expr.child, ast::Expr::Binary(_));
                if is_child_binary {
                    push_str!("(");
                }
                self.process_expr(&expr.child);
                if is_child_binary {
                    push_str!(")");
                }
                self.is_gml_expr = prev_state;
            },
            ast::Expr::Binary(expr) => {
                let prev_state = self.is_gml_expr;
                self.is_gml_expr = true;
                if let Some(simple) = self.deobf.simplify(ex, self.assets) {
                    self.process_expr(&ast::Expr::LiteralReal(simple));
                } else {
                    if expr.op == Operator::Index {
                        // array indexing
                        self.process_expr(&expr.left);
                        self.output.push(b'[');
                        if let ast::Expr::Group(group) = &expr.right {
                            for (i, expr) in group.iter().enumerate() {
                                if i != 0 {
                                    push_str!(", ");
                                }
                                self.process_expr(expr);
                            }
                        } else {
                            panic!("index rhs wasn't a group");
                        }
                        self.output.push(b']');
                    } else if expr.op == Operator::Deref {
                        // Deref operator - lots of special cases here
                        // If LHS can be simplified,
                        if let Some(simple) = self.deobf.simplify(&expr.left, self.assets) {
                            // If the simplified number is the ID of an object,
                            let simple_int = simple as i32;
                            if simple_int >= 0
                                && self.assets.objects.get(simple_int as usize).is_some()
                                && simple.fract() == 0.0
                            {
                                // Write eg "object123"
                                let _ = write!(self.output, "object{}", simple_int);
                            } else if simple.fract() == 0.0 {
                                // Special cases for certain keywords, otherwise just write eg "(123)"
                                match simple_int {
                                    -1 => self.output.extend_from_slice(b"self"),
                                    -2 => self.output.extend_from_slice(b"other"),
                                    -5 => self.output.extend_from_slice(b"global"),
                                    -7 => self.output.extend_from_slice(b"local"),
                                    i => {
                                        let _ = write!(self.output, "({})", i);
                                    },
                                }
                            } else {
                                // Write the whole LHS expression normally
                                self.output.push(b'(');
                                self.process_expr(&expr.left);
                                self.output.push(b')');
                            }
                        } else {
                            // Write the whole LHS expression normally
                            self.output.push(b'(');
                            self.process_expr(&expr.left);
                            self.output.push(b')');
                        }

                        self.output.push(b'.');
                        self.process_expr(&expr.right);
                    } else {
                        // TODO: there are some binary expressions we don't want to bubble-wrap here
                        // For example, if !is_assign and expr.left is a deref binary or index binary
                        // this will wrap it - but we don't want that
                        let is_assign = matches!(
                            expr.op,
                            Operator::Assign
                                | Operator::AssignAdd
                                | Operator::AssignSubtract
                                | Operator::AssignMultiply
                                | Operator::AssignDivide
                                | Operator::AssignBitwiseAnd
                                | Operator::AssignBitwiseOr
                                | Operator::AssignBitwiseXor
                        );
                        if !is_assign && matches!(expr.left, ast::Expr::Binary(_)) {
                            self.output.push(b'(');
                            self.process_expr(&expr.left);
                            self.output.push(b')');
                        } else {
                            self.process_expr(&expr.left);
                        }
                        self.output.push(b' ');
                        self.output.extend_from_slice(op_to_str(expr.op));
                        self.output.push(b' ');
                        if !is_assign && matches!(expr.right, ast::Expr::Binary(_)) {
                            self.output.push(b'(');
                            self.process_expr(&expr.right);
                            self.output.push(b')');
                        } else {
                            self.process_expr(&expr.right);
                        }
                    }
                }
                self.is_gml_expr = prev_state;
                if !self.is_gml_expr {
                    push_str!(";\r\n");
                }
            },
            ast::Expr::DoUntil(expr) => {
                push_str!("do ");
                self.write_expr_grouped(&expr.body, false);
                push_str!("until (");
                self.is_gml_expr = true;
                self.process_expr(&expr.cond);
                self.is_gml_expr = false;
                push_str!(");\r\n");
            },
            ast::Expr::For(expr) => {
                fn remove_truncate(x: &mut Vec<u8>, pat: &[u8]) {
                    if x.ends_with(pat) {
                        x.truncate(x.len() - pat.len());
                    }
                }

                push_str!("for (");
                self.is_gml_expr = true;
                self.process_expr(&expr.start);
                remove_truncate(&mut self.output, b"\r\n");
                push_str!("; ");
                self.process_expr(&expr.cond);
                push_str!("; ");
                self.process_expr(&expr.step);
                remove_truncate(&mut self.output, b"\r\n");
                remove_truncate(&mut self.output, b";");
                push_str!(") ");
                self.is_gml_expr = false;
                self.write_expr_grouped(&expr.body, true);
            },
            ast::Expr::Function(expr) => {
                if let Some(idx) = self
                    .assets
                    .scripts
                    .iter()
                    .enumerate()
                    .filter_map(|(i, o)| o.as_ref().map(|x| (i, x)))
                    .find(|(_, scr)| &*scr.name.0 == expr.name)
                    .map(|(i, _)| i)
                {
                    let _ = write!(self.output, "script{}", idx);
                } else {
                    self.output.extend_from_slice(expr.name);
                }
                self.output.push(b'(');
                let prev_expr_state = self.is_gml_expr;
                self.is_gml_expr = true;
                for (i, param) in expr.params.iter().enumerate() {
                    if i != 0 {
                        push_str!(", ");
                    }
                    self.process_expr(param);
                }
                self.is_gml_expr = prev_expr_state;
                if self.is_gml_expr {
                    push_str!(")");
                } else {
                    push_str!(");\r\n");
                }
            },
            ast::Expr::Group(exprs) => {
                let skip_newline = self.group_skip_newline;
                self.group_skip_newline = false;
                push_str!("{\r\n");
                self.indent += 1;
                let mut is_case = false;
                for expr in exprs {
                    if matches!(expr, ast::Expr::Case(_) | ast::Expr::Default) {
                        if is_case {
                            self.indent -= 1;
                        } else {
                            self.indent += 1;
                            is_case = true;
                        }
                        self.write_indent();
                        self.process_expr(expr);
                        self.indent += 1;
                    } else {
                        self.write_indent();
                        self.process_expr(expr);
                    }
                }
                if is_case {
                    self.indent -= 2;
                }
                self.indent -= 1;
                self.write_indent();
                if skip_newline {
                    self.output.push(b'}');
                } else {
                    push_str!("}\r\n");
                }
            },
            ast::Expr::If(expr) => {
                push_str!("if (");
                self.is_gml_expr = true;
                self.process_expr(&expr.cond);
                self.is_gml_expr = false;
                push_str!(") ");

                self.write_expr_grouped(&expr.body, false);

                if let Some(expr_else) = &expr.else_body {
                    push_str!(" else ");
                    self.process_expr(expr_else);
                } else {
                    push_str!("\r\n");
                }
            },
            ast::Expr::Repeat(expr) => {
                push_str!("repeat (");
                self.is_gml_expr = true;
                self.process_expr(&expr.count);
                self.is_gml_expr = false;
                push_str!(") ");
                self.write_expr_grouped(&expr.body, true);
            },
            ast::Expr::Switch(expr) => {
                push_str!("switch (");
                self.is_gml_expr = true;
                self.process_expr(&expr.input);
                self.is_gml_expr = false;
                push_str!(") ");
                self.write_expr_grouped(&expr.body, true);
            },
            ast::Expr::Var(expr) => {
                if !expr.vars.is_empty() {
                    push_str!("var ");
                    for (i, name) in expr.vars.iter().enumerate() {
                        if i != 0 {
                            push_str!(", ");
                        }
                        self.write_field(name);
                    }
                    push_str!(";\r\n");
                }
            },
            ast::Expr::GlobalVar(expr) => {
                if !expr.vars.is_empty() {
                    push_str!("globalvar ");
                    for (i, name) in expr.vars.iter().enumerate() {
                        if i != 0 {
                            push_str!(", ");
                        }
                        self.write_field(name);
                    }
                    push_str!(";\r\n");
                }
            },
            ast::Expr::With(expr) => {
                push_str!("with (");
                self.is_gml_expr = true;
                if let Some(simple) = self.deobf.simplify(&expr.target, self.assets) {
                    let simple_int = simple as i32;
                    if simple_int >= 0
                        && self.assets.objects.get(simple_int as usize).is_some()
                        && simple.fract() == 0.0
                    {
                        let _ = write!(self.output, "object{}", simple_int);
                    } else if simple.fract() == 0.0 {
                        let _ = write!(self.output, "{}", simple_int);
                    } else {
                        self.process_expr(&expr.target);
                    }
                } else {
                    self.process_expr(&expr.target);
                }
                self.is_gml_expr = false;
                push_str!(") ");
                self.write_expr_grouped(&expr.body, true);
            },
            ast::Expr::While(expr) => {
                push_str!("while (");
                self.is_gml_expr = true;
                self.process_expr(&expr.cond);
                self.is_gml_expr = false;
                push_str!(") ");
                self.write_expr_grouped(&expr.body, true);
            },
            ast::Expr::Case(expr) => {
                push_str!("case ");
                self.is_gml_expr = true;
                self.process_expr(expr);
                self.is_gml_expr = false;
                push_str!(":\r\n");
            },
            ast::Expr::Default => push_str!("default:\r\n"),
            ast::Expr::Continue => push_str!("continue;\r\n"),
            ast::Expr::Break => push_str!("break;\r\n"),
            ast::Expr::Exit => push_str!("exit;\r\n"),
            ast::Expr::Return(expr) => {
                push_str!("return ");
                self.is_gml_expr = true;
                self.process_expr(expr);
                self.is_gml_expr = false;
                push_str!(";\r\n");
            },
        }
    }

    pub fn write_expr_grouped(&mut self, expr: &ast::Expr, newline: bool) {
        if matches!(expr, ast::Expr::Group(_)) {
            if !newline {
                self.group_skip_newline = true;
            }
            self.process_expr(expr);
        } else {
            self.output.extend_from_slice(b"{\r\n");
            self.indent += 1;
            self.write_indent();
            self.process_expr(expr);
            self.indent -= 1;
            self.write_indent();
            self.output.extend_from_slice(if newline { b"}\r\n" } else { b"}" });
        }
    }

    pub fn write_field(&mut self, ident: &[u8]) {
        let field_number = self.deobf.register_field(ident);
        let _ = write!(self.output, "field{}", field_number);
    }

    pub fn write_indent(&mut self) {
        for _ in 0..self.indent {
            self.output.extend_from_slice(self.indent_str.as_bytes());
        }
    }
}

fn op_to_str(op: Operator) -> &'static [u8] {
    match op {
        Operator::Add => b"+",
        Operator::Subtract => b"-",
        Operator::Multiply => b"*",
        Operator::Divide => b"/",
        Operator::IntDivide => b"div",
        Operator::BitwiseAnd => b"&",
        Operator::BitwiseOr => b"|",
        Operator::BitwiseXor => b"^",
        Operator::Assign => b"=",
        Operator::Not => b"!",
        Operator::LessThan => b"<",
        Operator::GreaterThan => b">",
        Operator::AssignAdd => b"+=",
        Operator::AssignSubtract => b"-=",
        Operator::AssignMultiply => b"*=",
        Operator::AssignDivide => b"/=",
        Operator::AssignBitwiseAnd => b"&=",
        Operator::AssignBitwiseOr => b"|=",
        Operator::AssignBitwiseXor => b"^=",
        Operator::Equal => b"==",
        Operator::NotEqual => b"!=",
        Operator::LessThanOrEqual => b"<=",
        Operator::GreaterThanOrEqual => b">=",
        Operator::Modulo => b"mod",
        Operator::And => b"&&",
        Operator::Or => b"||",
        Operator::Xor => b"^^",
        Operator::BinaryShiftLeft => b"<<",
        Operator::BinaryShiftRight => b">>",
        Operator::Complement => b"~",
        Operator::Deref => b".",
        Operator::Index => panic!("index op passed to op_to_str"),
    }
}
