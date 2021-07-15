use crate::{
    lexer::Lexer,
    token::{Keyword, Operator, Separator, Token},
};

use std::{
    error, fmt,
    iter::{IntoIterator, Peekable},
    ops::{Deref, DerefMut},
};

#[derive(Debug, PartialEq)]
pub struct AST<'a>(Vec<Expr<'a>>);

#[derive(Debug, PartialEq)]
pub enum Expr<'a> {
    LiteralIdentifier(&'a [u8]),
    LiteralReal(f64),
    LiteralString(&'a [u8]),

    Unary(Box<UnaryExpr<'a>>),
    Binary(Box<BinaryExpr<'a>>),

    DoUntil(Box<DoUntilExpr<'a>>),
    For(Box<ForExpr<'a>>),
    Function(Box<FunctionExpr<'a>>),
    Group(Vec<Expr<'a>>),
    If(Box<IfExpr<'a>>),
    Repeat(Box<RepeatExpr<'a>>),
    Switch(Box<SwitchExpr<'a>>),
    Var(Box<VarExpr<'a>>),
    GlobalVar(Box<GlobalVarExpr<'a>>),
    With(Box<WithExpr<'a>>),
    While(Box<WhileExpr<'a>>),

    Case(Box<Expr<'a>>),
    Default,

    Continue,
    Break,
    Exit,
    Return(Box<Expr<'a>>),
}

#[derive(Debug, PartialEq)]
pub struct UnaryExpr<'a> {
    pub op: Operator,
    pub child: Expr<'a>,
}

#[derive(Debug, PartialEq)]
pub struct BinaryExpr<'a> {
    pub op: Operator,
    pub left: Expr<'a>,
    pub right: Expr<'a>,
}

#[derive(Debug, PartialEq)]
pub struct FunctionExpr<'a> {
    pub name: &'a [u8],
    pub params: Vec<Expr<'a>>,
}

#[derive(Debug, PartialEq)]
pub struct DoUntilExpr<'a> {
    pub cond: Expr<'a>,
    pub body: Expr<'a>,
}

#[derive(Debug, PartialEq)]
pub struct ForExpr<'a> {
    pub start: Expr<'a>,
    pub cond: Expr<'a>,
    pub step: Expr<'a>,

    pub body: Expr<'a>,
}

#[derive(Debug, PartialEq)]
pub struct IfExpr<'a> {
    pub cond: Expr<'a>,
    pub body: Expr<'a>,
    pub else_body: Option<Expr<'a>>,
}

#[derive(Debug, PartialEq)]
pub struct RepeatExpr<'a> {
    pub count: Expr<'a>,
    pub body: Expr<'a>,
}

#[derive(Debug, PartialEq)]
pub struct SwitchExpr<'a> {
    pub input: Expr<'a>,
    pub body: Expr<'a>,
}

#[derive(Debug, PartialEq)]
pub struct VarExpr<'a> {
    pub vars: Vec<&'a [u8]>,
}

#[derive(Debug, PartialEq)]
pub struct GlobalVarExpr<'a> {
    pub vars: Vec<&'a [u8]>,
}

#[derive(Debug, PartialEq)]
pub struct WithExpr<'a> {
    pub target: Expr<'a>,
    pub body: Expr<'a>,
}

#[derive(Debug, PartialEq)]
pub struct WhileExpr<'a> {
    pub cond: Expr<'a>,
    pub body: Expr<'a>,
}

#[derive(Debug)]
pub struct Error {
    pub message: String,
}

impl Error {
    pub fn new(message: String) -> Self {
        Error { message }
    }
}

impl<'a> fmt::Display for Expr<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Expr::LiteralIdentifier(id) => write!(f, "{}", String::from_utf8_lossy(id)),
            Expr::LiteralReal(r) => write!(f, "{}", r),
            Expr::LiteralString(s) => write!(f, "\"{}\"", String::from_utf8_lossy(s)),

            Expr::Unary(unary) => write!(f, "({} {})", unary.op, unary.child),
            Expr::Binary(binary) => write!(f, "({} {} {})", binary.op, binary.left, binary.right),

            Expr::DoUntil(dountil) => write!(f, "(do {} until {})", dountil.body, dountil.cond),
            Expr::For(for_ex) => {
                write!(f, "(for ({}, {}, {}) {})", for_ex.start, for_ex.cond, for_ex.step, for_ex.body)
            },
            Expr::Function(call) => write!(
                f,
                "(@{} {})",
                String::from_utf8_lossy(call.name),
                call.params.iter().fold(String::new(), |acc, fnname| acc + &format!("{} ", fnname)).trim_end()
            ),
            Expr::Group(group) => write!(
                f,
                "<{}>",
                group
                    .iter()
                    .fold(String::new(), |acc, expr| acc + &format!("{}, ", expr))
                    .trim_end_matches(|ch| ch == ' ' || ch == ',')
            ),
            Expr::If(if_ex) => match if_ex.else_body {
                Some(ref els) => write!(f, "(if {} {} {})", if_ex.cond, if_ex.body, els),
                None => write!(f, "(if {} {})", if_ex.cond, if_ex.body),
            },
            Expr::Repeat(repeat) => write!(f, "(repeat {} {})", repeat.count, repeat.body),
            Expr::Switch(switch) => write!(f, "(switch {} {})", switch.input, switch.body),
            Expr::Var(var) => write!(
                f,
                "(var {})",
                var.vars
                    .iter()
                    .fold(String::new(), |acc, varname| acc + &format!("{} ", String::from_utf8_lossy(varname)))
                    .trim_end()
            ),
            Expr::GlobalVar(var) => write!(
                f,
                "(globalvar {})",
                var.vars
                    .iter()
                    .fold(String::new(), |acc, varname| acc + &format!("{} ", String::from_utf8_lossy(varname)))
                    .trim_end()
            ),
            Expr::With(with) => write!(f, "(with {} {})", with.target, with.body),
            Expr::While(while_ex) => write!(f, "(while {} {})", while_ex.cond, while_ex.body),

            Expr::Case(e) => write!(f, "(case {})", e),
            Expr::Default => write!(f, "(default)"),

            Expr::Continue => write!(f, "(continue)"),
            Expr::Break => write!(f, "(break)"),
            Expr::Exit => write!(f, "(exit)"),
            Expr::Return(e) => write!(f, "(return {})", e),
        }
    }
}

impl error::Error for Error {}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

// TODO? This is not the prettiest.
macro_rules! expect_token {
    ( $token: expr, $($content: tt)* ) => ({
        match $token {
            Some(Token::$($content)*) => {},
            Some(t) => {
                return Err(Error::new(format!(
                    "Unexpected token {:?}; `{}` expected",
                    t, Token::$($content)*,
                )));
            }
            None => {
                return Err(Error::new(format!(
                    "Unexpected EOF; `{}` expected",
                    Token::$($content)*,
                )));
            }
        }
    });
}

impl<'a> Default for AST<'a> {
    fn default() -> Self {
        AST(Vec::new())
    }
}

impl<'a> Deref for AST<'a> {
    type Target = Vec<Expr<'a>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> DerefMut for AST<'a> {
    fn deref_mut(&mut self) -> &mut <Self as Deref>::Target {
        &mut self.0
    }
}

impl<'a> IntoIterator for AST<'a> {
    type IntoIter = <Vec<Self::Item> as IntoIterator>::IntoIter;
    type Item = Expr<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> AST<'a> {
    pub fn new(source: &'a [u8]) -> Result<Self, Error> {
        let mut lex = Lexer::new(source).peekable();
        let mut expressions = Vec::new();

        loop {
            // Get the first token from the iterator, or exit the loop if there are no more
            match AST::read_line(&mut lex) {
                Ok(Some(expr)) => expressions.push(expr),
                Ok(None) => break,
                Err(e) => return Err(e),
            }
        }

        Ok(Self(expressions))
    }

    pub fn expression(source: &'a [u8]) -> Result<Expr<'a>, Error> {
        let mut lex = Lexer::new(source).peekable();
        if lex.peek().is_some() { AST::read_binary_tree(&mut lex, None, false) } else { Ok(Expr::LiteralReal(0.0)) }
    }

    fn read_line(lex: &mut Peekable<Lexer<'a>>) -> Result<Option<Expr<'a>>, Error> {
        let token = loop {
            match lex.next() {
                Some(Token::Separator(Separator::Semicolon)) => continue,
                Some(t) => break t,
                None => return Ok(None), // EOF
            }
        };

        // Use token type to determine what logic we should apply here
        let ret = match token {
            Token::Keyword(key) => {
                match key {
                    Keyword::Var | Keyword::GlobalVar => {
                        // Read var identifiers
                        if let Some(&Token::Identifier(id)) = lex.peek() {
                            lex.next();
                            let mut vars = vec![id];

                            loop {
                                let mut peek_lex = lex.clone();
                                // Check next token
                                match peek_lex.next() {
                                    // If next token is a comma, skip it and expect another identifier after it
                                    Some(Token::Separator(Separator::Comma)) => {
                                        lex.next();
                                    },

                                    // If next token is an identifier, it might be another var name...
                                    Some(Token::Identifier(_)) => {
                                        // ...but if the token after that is '(' or `.`, then it's actually the start
                                        // of the next line, so stop reading var names here
                                        let next = peek_lex.next();
                                        if matches!(
                                            next,
                                            Some(Token::Separator(Separator::ParenLeft))
                                                | Some(Token::Separator(Separator::Period))
                                        ) {
                                            break
                                        }
                                    },

                                    // Anything else (most likely a semicolon) means there are no more var names.
                                    _ => break,
                                }

                                // Read one identifier and store it as a var name
                                // Alternatively, break if the next token is not a Token::Identifier
                                if let Some(Token::Identifier(id)) = lex.peek() {
                                    vars.push(id);
                                    lex.next();
                                } else {
                                    break
                                }
                            }

                            match key {
                                Keyword::Var => Ok(Some(Expr::Var(Box::new(VarExpr { vars })))),
                                Keyword::GlobalVar => Ok(Some(Expr::GlobalVar(Box::new(GlobalVarExpr { vars })))),
                                _ => unreachable!(),
                            }
                        } else {
                            // This doesn't do anything in GML. We could probably make it a NOP.
                            match key {
                                Keyword::Var => Ok(Some(Expr::Var(Box::new(VarExpr { vars: vec![] })))),
                                Keyword::GlobalVar => {
                                    Ok(Some(Expr::GlobalVar(Box::new(GlobalVarExpr { vars: vec![] }))))
                                },
                                _ => unreachable!(),
                            }
                        }
                    },

                    Keyword::Do => {
                        let body = AST::read_group(lex)?
                            .ok_or_else(|| Error::new("Unexpected EOF after 'do' keyword".to_string()))?;
                        expect_token!(lex.next(), Keyword(Keyword::Until));
                        let cond = AST::read_binary_tree(lex, None, false)?;
                        Ok(Some(Expr::DoUntil(Box::new(DoUntilExpr { cond, body }))))
                    },

                    Keyword::If => {
                        let cond = AST::read_binary_tree(lex, None, false)?;
                        if lex.peek() == Some(&Token::Separator(Separator::Then)) {
                            lex.next();
                        }
                        let body = AST::read_group(lex)?
                            .ok_or_else(|| Error::new("Unexpected EOF after 'if' condition".to_string()))?;
                        let else_body = if lex.peek() == Some(&Token::Keyword(Keyword::Else)) {
                            lex.next(); // consume 'else'
                            Some(
                                AST::read_group(lex)?
                                    .ok_or_else(|| Error::new("Unexpected EOF after 'else' keyword".to_string()))?,
                            )
                        } else {
                            None
                        };
                        Ok(Some(Expr::If(Box::new(IfExpr { cond, body, else_body }))))
                    },

                    Keyword::For => {
                        expect_token!(lex.next(), Separator(Separator::ParenLeft));
                        let start = AST::read_line(lex)?
                            .ok_or_else(|| Error::new("Unexpected EOF during 'for' params".to_string()))?;
                        if lex.peek() == Some(&Token::Separator(Separator::Semicolon)) {
                            lex.next();
                        }
                        let cond = AST::read_binary_tree(lex, None, false)?;
                        if lex.peek() == Some(&Token::Separator(Separator::Semicolon)) {
                            lex.next();
                        }
                        let step = AST::read_line(lex)?
                            .ok_or_else(|| Error::new("Unexpected EOF during 'for' params".to_string()))?;
                        while lex.peek() == Some(&Token::Separator(Separator::Semicolon)) {
                            lex.next();
                        }
                        expect_token!(lex.next(), Separator(Separator::ParenRight));
                        let body = AST::read_group(lex)?
                            .ok_or_else(|| Error::new("Unexpected EOF after 'for' params".to_string()))?;
                        Ok(Some(Expr::For(Box::new(ForExpr { start, cond, step, body }))))
                    },

                    Keyword::Repeat => {
                        let count = AST::read_binary_tree(lex, None, false)?;
                        let body = AST::read_group(lex)?
                            .ok_or_else(|| Error::new("Unexpected EOF after 'repeat' condition".to_string()))?;
                        Ok(Some(Expr::Repeat(Box::new(RepeatExpr { count, body }))))
                    },

                    Keyword::Switch => {
                        let input = AST::read_binary_tree(lex, None, false)?;
                        let body = AST::read_line(lex)?
                            .ok_or_else(|| Error::new("Unexpected EOF after 'switch' condition".to_string()))?;
                        Ok(Some(Expr::Switch(Box::new(SwitchExpr { input, body }))))
                    },

                    Keyword::With => {
                        let target = AST::read_binary_tree(lex, None, false)?;
                        if lex.peek() == Some(&Token::Keyword(Keyword::Do)) {
                            lex.next();
                        }
                        let body = AST::read_group(lex)?
                            .ok_or_else(|| Error::new("Unexpected EOF after 'with' condition".to_string()))?;
                        Ok(Some(Expr::With(Box::new(WithExpr { target, body }))))
                    },

                    Keyword::While => {
                        let cond = AST::read_binary_tree(lex, None, false)?;
                        if lex.peek() == Some(&Token::Keyword(Keyword::Do)) {
                            lex.next();
                        }
                        let body = AST::read_group(lex)?
                            .ok_or_else(|| Error::new("Unexpected EOF after 'while' condition".to_string()))?;
                        Ok(Some(Expr::While(Box::new(WhileExpr { cond, body }))))
                    },

                    Keyword::Case => {
                        let expr = AST::read_binary_tree(lex, None, false)?;
                        expect_token!(lex.next(), Separator(Separator::Colon));
                        Ok(Some(Expr::Case(Box::new(expr))))
                    },

                    Keyword::Default => {
                        expect_token!(lex.next(), Separator(Separator::Colon));
                        Ok(Some(Expr::Default))
                    },

                    Keyword::Break => Ok(Some(Expr::Break)),

                    Keyword::Continue => Ok(Some(Expr::Continue)),

                    Keyword::Exit => Ok(Some(Expr::Exit)),

                    Keyword::Return => {
                        let val = AST::read_binary_tree(lex, None, false)?;
                        Ok(Some(Expr::Return(Box::new(val))))
                    },

                    _ => return Err(Error::new(format!("Invalid Keyword at beginning of expression: {:?}", key))),
                }
            },

            Token::Identifier(id) => {
                // An expression starting with an identifier may be either an assignment or script/function.
                // This is determined by what type of token immediately follows it.
                let next_token = match lex.peek() {
                    Some(t) => t,
                    None => {
                        return Err(Error::new(format!("Stray identifier at EOF: {:?}", String::from_utf8_lossy(id))))
                    },
                };
                match next_token {
                    Token::Separator(ref sep) if *sep == Separator::ParenLeft => {
                        Ok(Some(AST::read_function_call(lex, id)?))
                    },
                    _ => Ok(Some(AST::read_binary_tree(lex, Some(token), true)?)),
                }
            },

            Token::Separator(sep) => {
                match sep {
                    // Code contained in {} is treated here as one single expression, called a Group.
                    Separator::BraceLeft => {
                        let mut inner_expressions = Vec::new();
                        loop {
                            match lex.peek() {
                                Some(Token::Separator(Separator::BraceRight)) => {
                                    lex.next();
                                    break Ok(Some(Expr::Group(inner_expressions)))
                                },
                                _ => match AST::read_line(lex) {
                                    Ok(Some(e)) => inner_expressions.push(e),
                                    Ok(None) => break Err(Error::new("Unclosed brace at EOF".to_string())),
                                    Err(e) => break Err(e),
                                },
                            }
                        }
                    },

                    // An assignment may start with an open-parenthesis, eg: (1).x = 400;
                    Separator::ParenLeft => {
                        let binary_tree =
                            AST::read_binary_tree(lex, Some(Token::Separator(Separator::ParenLeft)), true)?;
                        Ok(Some(binary_tree))
                    },

                    // Default
                    _ => return Err(Error::new(format!("Invalid Separator at beginning of expression: {:?}", sep))),
                }
            },

            _ => return Err(Error::new(format!("Invalid token at beginning of expression: {:?}", token))),
        };

        // skip over trailing semicolons
        while lex.peek() == Some(&Token::Separator(Separator::Semicolon)) {
            lex.next();
        }

        ret
    }

    fn read_group(lex: &mut Peekable<Lexer<'a>>) -> Result<Option<Expr<'a>>, Error> {
        match lex.peek() {
            Some(Token::Separator(Separator::Semicolon)) => {
                while lex.peek() == Some(&Token::Separator(Separator::Semicolon)) {
                    lex.next();
                }
                Ok(Some(Expr::Group(vec![])))
            },
            Some(_) => Self::read_line(lex),
            None => Ok(None),
        }
    }

    fn read_binary_tree(
        lex: &mut Peekable<Lexer<'a>>,
        first_token: Option<Token<'a>>, // Sometimes we've already parsed the first token, so it should be put here.
        expect_assignment: bool,        // Do we expect the first op to be an assignment?
    ) -> Result<Expr<'a>, Error> {
        let (val, op) = AST::read_binary_tree_recursive(lex, first_token, expect_assignment, 0)?;
        if let Some(stray_op) = op {
            Err(Error::new(format!("read_binary_tree has stray operator: {:?}", stray_op)))
        } else {
            Ok(val)
        }
    }

    fn read_binary_tree_recursive(
        lex: &mut Peekable<Lexer<'a>>,
        first_token: Option<Token<'a>>, // Sometimes we've already parsed the first token, so it should be put here.
        expect_assignment: bool,        // Do we expect the first op to be an assignment?
        lowest_prec: u8,                // We are not allowed to go below this operator precedence in this tree.
                                        // If we do, we'll return the next op.
    ) -> Result<(Expr<'a>, Option<Operator>), Error> {
        // Get the first expression before any operators
        let mut lhs = AST::read_btree_expression(lex, first_token)?;

        // Check if the next token is an operator
        let next_token = lex.peek();
        match next_token {
            Some(Token::Operator(op)) => {
                // '=' can be either an assignment or equality check (==) in GML.
                // So if we're not expecting an assignment operator, it should be seen as a comparator instead.
                let mut op = if (op == &Operator::Assign) && (!expect_assignment) { Operator::Equal } else { *op };

                // Consume operator
                lex.next();

                // Now, loop until there are no more buffered operators.
                loop {
                    // Here, we get the precedence of the operator we found.
                    // If this returns None, it's probably an assignment,
                    // so we use that in conjunction with an if-let to check its validity.
                    if let Some(precedence) = AST::get_op_precedence(&op) {
                        // this op is invalid if an assignment is expected
                        if expect_assignment {
                            break Err(Error::new(format!("Invalid operator {:?} found, expected assignment", op)))
                        }
                        // If this op has lower prec than we're allowed to read, we have to return it here.
                        if precedence < lowest_prec {
                            break Ok((lhs, Some(op)))
                        }
                        // We're allowed to use the next operator. Let's read an RHS to put on after it.
                        // We limit this tree to current precedence + 1 to prevent it using operators of our
                        // current precedence.  This way, 1/2/3 is correctly built as (1/2)/3 rather than 1/(2/3).
                        let (rhs, next_op) = AST::read_binary_tree_recursive(lex, None, false, precedence + 1)?;
                        if let Some(next_op) = next_op {
                            // There's another operator even after the RHS.
                            if let Some(next_prec) = AST::get_op_precedence(&next_op) {
                                if next_prec < lowest_prec {
                                    // This next op is lower than we're allowed to go, so we must return it
                                    break Ok((
                                        Expr::Binary(Box::new(BinaryExpr { op, left: lhs, right: rhs })),
                                        Some(next_op),
                                    ))
                                } else {
                                    // Update LHS by sticking RHS onto it,
                                    // set op to the new operator, and go round again.
                                    lhs = Expr::Binary(Box::new(BinaryExpr { op, left: lhs, right: rhs }));
                                    op = next_op;
                                }
                            } else {
                                // Precedence would already have been checked by the returning function.
                                break Err(Error::new(format!(
                                    "read_binary_tree_recursive returned invalid operator: {}",
                                    next_op
                                )))
                            }
                        } else {
                            // No more operators so let's put our lhs and rhs together.
                            break Ok((Expr::Binary(Box::new(BinaryExpr { op, left: lhs, right: rhs })), None))
                        }
                    } else {
                        // this op is invalid if assignment not expected, OR if it's a unary operator
                        // (those have no precedence so they pass the previous test.)
                        if !expect_assignment || op == Operator::Not || op == Operator::Complement {
                            break Err(Error::new(format!("Invalid operator {:?} found, expected evaluable", op)))
                        } else {
                            // No need to do precedence on an assignment, so just grab RHS and return
                            let (rhs, stray_op) = AST::read_binary_tree_recursive(lex, None, false, lowest_prec)?;
                            break if let Some(op) = stray_op {
                                Err(Error::new(format!("Stray operator {:?} in expression", op)))
                            } else {
                                Ok((Expr::Binary(Box::new(BinaryExpr { op, left: lhs, right: rhs })), None))
                            }
                        }
                    }
                }
            },
            _ => {
                if expect_assignment {
                    Err(Error::new(format!("Invalid token {:?} when expecting assignment operator", next_token)))
                } else {
                    Ok((lhs, None))
                }
            },
        }
    }

    fn read_btree_expression(lex: &mut Peekable<Lexer<'a>>, first_token: Option<Token<'a>>) -> Result<Expr<'a>, Error> {
        // Get first token and match it
        let mut lhs = match if first_token.is_some() { first_token } else { lex.next() } {
            Some(Token::Separator(ref sep)) if *sep == Separator::ParenLeft => {
                let binary_tree = AST::read_binary_tree(lex, None, false)?;
                if lex.next() != Some(Token::Separator(Separator::ParenRight)) {
                    return Err(Error::new("Unclosed parenthesis in binary tree".to_string()))
                } else {
                    binary_tree
                }
            },
            Some(Token::Operator(op)) => {
                if op == Operator::Add || op == Operator::Subtract || op == Operator::Not || op == Operator::Complement
                {
                    Expr::Unary(Box::new(UnaryExpr { op, child: AST::read_btree_expression(lex, None)? }))
                } else {
                    return Err(Error::new(format!("Invalid unary operator {:?} in expression", op)))
                }
            },
            Some(Token::Identifier(t)) => {
                if lex.peek() == Some(&Token::Separator(Separator::ParenLeft)) {
                    AST::read_function_call(lex, t)?
                } else {
                    Expr::LiteralIdentifier(t)
                }
            },

            Some(Token::Real(t)) => Expr::LiteralReal(t),
            Some(Token::String(t)) => Expr::LiteralString(t),
            Some(t) => return Err(Error::new(format!("Invalid token while scanning binary tree: {:?}", t))),
            None => return Err(Error::new("Found EOF unexpectedly while reading binary tree".to_string())),
        };

        // Do we need to amend this LHS at all?
        loop {
            match lex.peek() {
                Some(Token::Separator(ref sep)) if *sep == Separator::BracketLeft => {
                    lex.next();
                    let mut dimensions = Vec::new();
                    if lex.peek() == Some(&Token::Separator(Separator::BracketRight)) {
                        lex.next();
                    } else {
                        loop {
                            let dim = AST::read_binary_tree(lex, None, false)?;
                            dimensions.push(dim);
                            match lex.next() {
                                Some(Token::Separator(Separator::BracketRight)) => break,
                                Some(Token::Separator(Separator::Comma)) => {
                                    if lex.peek() == Some(&Token::Separator(Separator::BracketRight)) {
                                        lex.next();
                                        break
                                    }
                                },
                                Some(t) => {
                                    return Err(Error::new(format!("Invalid token {:?}, expected expression", t)))
                                },
                                None => {
                                    return Err(Error::new(
                                        "Found EOF unexpectedly while reading array accessor".to_string(),
                                    ))
                                },
                            }
                        }
                    }
                    lhs = Expr::Binary(Box::new(BinaryExpr {
                        op: Operator::Index,
                        left: lhs,
                        right: Expr::Group(dimensions),
                    }));
                },

                Some(Token::Separator(ref sep)) if *sep == Separator::Period => {
                    lex.next();
                    lhs = match lex.next() {
                        Some(Token::Identifier(id)) => Expr::Binary(Box::new(BinaryExpr {
                            op: Operator::Deref,
                            left: lhs,
                            right: Expr::LiteralIdentifier(id),
                        })),
                        Some(t) => return Err(Error::new(format!("Unexpected token {:?} following deref", t))),
                        None => return Err(Error::new("Found EOF unexpectedly while reading binary tree".to_string())),
                    }
                },
                _ => break,
            }
        }

        Ok(lhs)
    }

    fn read_function_call(lex: &mut Peekable<Lexer<'a>>, function_name: &'a [u8]) -> Result<Expr<'a>, Error> {
        expect_token!(lex.next(), Separator(Separator::ParenLeft));

        let mut params = Vec::new();
        if lex.peek() == Some(&Token::Separator(Separator::ParenRight)) {
            lex.next();
        } else {
            loop {
                let param = AST::read_binary_tree(lex, None, false)?;
                params.push(param);
                match lex.next() {
                    Some(Token::Separator(Separator::ParenRight)) => break,
                    Some(Token::Separator(Separator::Comma)) => {
                        if lex.peek() == Some(&Token::Separator(Separator::ParenRight)) {
                            lex.next();
                            break
                        }
                    },
                    Some(t) => return Err(Error::new(format!("Invalid token {:?}, expected expression", t))),
                    None => return Err(Error::new("Found EOF unexpectedly while reading function call".to_string())),
                }
            }
        }
        Ok(Expr::Function(Box::new(FunctionExpr { name: function_name, params })))
    }

    fn get_op_precedence(op: &Operator) -> Option<u8> {
        match op {
            Operator::Add => Some(4),
            Operator::Subtract => Some(4),
            Operator::Multiply => Some(5),
            Operator::Divide => Some(5),
            Operator::IntDivide => Some(5),
            Operator::BitwiseAnd => Some(2),
            Operator::BitwiseOr => Some(2),
            Operator::BitwiseXor => Some(2),
            Operator::Assign => None,
            Operator::Not => None,
            Operator::LessThan => Some(1),
            Operator::GreaterThan => Some(1),
            Operator::AssignAdd => None,
            Operator::AssignSubtract => None,
            Operator::AssignMultiply => None,
            Operator::AssignDivide => None,
            Operator::AssignBitwiseAnd => None,
            Operator::AssignBitwiseOr => None,
            Operator::AssignBitwiseXor => None,
            Operator::Equal => Some(1),
            Operator::NotEqual => Some(1),
            Operator::LessThanOrEqual => Some(1),
            Operator::GreaterThanOrEqual => Some(1),
            Operator::Modulo => Some(5),
            Operator::And => Some(0),
            Operator::Or => Some(0),
            Operator::Xor => Some(0),
            Operator::BinaryShiftLeft => Some(3),
            Operator::BinaryShiftRight => Some(3),
            Operator::Complement => None,
            Operator::Deref => None,
            Operator::Index => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper function for all the AST testcases.
    fn assert_ast(input: &str, expected_output: Option<Vec<Expr>>) {
        match AST::new(input.as_bytes()) {
            Ok(ast) => {
                if let Some(e) = expected_output {
                    assert_eq!(*ast, e);
                }
            },
            Err(e) => panic!("AST test encountered error: '{}' for input: {}", e, input),
        }
    }

    #[test]
    fn nothing() {
        // Empty string
        assert_ast("", Some(vec![]))
    }

    #[test]
    fn assignment_op_assign() {
        assert_ast(
            // Simple assignment - Assign
            "a = 1",
            Some(vec![Expr::Binary(Box::new(BinaryExpr {
                op: Operator::Assign,
                left: Expr::LiteralIdentifier(b"a"),
                right: Expr::LiteralReal(1.0),
            }))]),
        )
    }

    #[test]
    fn assignment_op_add() {
        assert_ast(
            // Simple assignment - AssignAdd
            "b += 2",
            Some(vec![Expr::Binary(Box::new(BinaryExpr {
                op: Operator::AssignAdd,
                left: Expr::LiteralIdentifier(b"b"),
                right: Expr::LiteralReal(2.0),
            }))]),
        )
    }

    #[test]
    fn assignment_op_subtract() {
        assert_ast(
            // Simple assignment - AssignSubtract
            "c -= 3",
            Some(vec![Expr::Binary(Box::new(BinaryExpr {
                op: Operator::AssignSubtract,
                left: Expr::LiteralIdentifier(b"c"),
                right: Expr::LiteralReal(3.0),
            }))]),
        )
    }

    #[test]
    fn assignment_op_multiply() {
        assert_ast(
            // Simple assignment - AssignMultiply
            "d *= 4",
            Some(vec![Expr::Binary(Box::new(BinaryExpr {
                op: Operator::AssignMultiply,
                left: Expr::LiteralIdentifier(b"d"),
                right: Expr::LiteralReal(4.0),
            }))]),
        )
    }

    #[test]
    fn assignment_op_divide() {
        assert_ast(
            // Simple assignment - AssignDivide
            "e /= 5",
            Some(vec![Expr::Binary(Box::new(BinaryExpr {
                op: Operator::AssignDivide,
                left: Expr::LiteralIdentifier(b"e"),
                right: Expr::LiteralReal(5.0),
            }))]),
        )
    }

    #[test]
    fn assignment_op_and() {
        assert_ast(
            // Simple assignment - AssignBinaryAnd
            "f &= 6",
            Some(vec![Expr::Binary(Box::new(BinaryExpr {
                op: Operator::AssignBitwiseAnd,
                left: Expr::LiteralIdentifier(b"f"),
                right: Expr::LiteralReal(6.0),
            }))]),
        )
    }

    #[test]
    fn assignment_op_or() {
        assert_ast(
            // Simple assignment - AssignBinaryOr
            "g |= 7",
            Some(vec![Expr::Binary(Box::new(BinaryExpr {
                op: Operator::AssignBitwiseOr,
                left: Expr::LiteralIdentifier(b"g"),
                right: Expr::LiteralReal(7.0),
            }))]),
        )
    }

    #[test]
    fn assignment_op_xor() {
        assert_ast(
            // Simple assignment - AssignBinaryXor
            "h ^= 8",
            Some(vec![Expr::Binary(Box::new(BinaryExpr {
                op: Operator::AssignBitwiseXor,
                left: Expr::LiteralIdentifier(b"h"),
                right: Expr::LiteralReal(8.0),
            }))]),
        )
    }

    #[test]
    #[should_panic]
    fn assignment_op_invalid() {
        // Assignment syntax - Multiply - should fail
        // Note: chose "Multiply" specifically as it cannot be unary, unlike Add or Subtract
        assert_ast("i * 9", None);
    }

    #[test]
    #[should_panic]
    fn assignment_op_not() {
        // Assignment syntax - Not - should fail
        assert_ast("j ! 10", None);
    }

    #[test]
    #[should_panic]
    fn assignment_op_complement() {
        // Assignment syntax - Complement - should fail
        assert_ast("k ~ 11", None);
    }

    #[test]
    fn assignment_lhs() {
        assert_ast(
            // Assignment with deref and index on lhs
            "a.b[c] += d;",
            Some(vec![Expr::Binary(Box::new(BinaryExpr {
                op: Operator::AssignAdd,
                left: Expr::Binary(Box::new(BinaryExpr {
                    op: Operator::Index,
                    left: Expr::Binary(Box::new(BinaryExpr {
                        op: Operator::Deref,
                        left: Expr::LiteralIdentifier(b"a"),
                        right: Expr::LiteralIdentifier(b"b"),
                    })),
                    right: Expr::Group(vec![Expr::LiteralIdentifier(b"c")]),
                })),
                right: Expr::LiteralIdentifier(b"d"),
            }))]),
        );
    }

    #[test]
    fn assignment_2d_index() {
        assert_ast(
            // Arbitrary chains of deref, 1- and 2-dimension index ops on both lhs and rhs
            "a.b[c].d.e[f,g]=h[i,j].k",
            Some(vec![Expr::Binary(Box::new(BinaryExpr {
                op: Operator::Assign,
                left: Expr::Binary(Box::new(BinaryExpr {
                    op: Operator::Index,
                    left: Expr::Binary(Box::new(BinaryExpr {
                        op: Operator::Deref,
                        left: Expr::Binary(Box::new(BinaryExpr {
                            op: Operator::Deref,
                            left: Expr::Binary(Box::new(BinaryExpr {
                                op: Operator::Index,
                                left: Expr::Binary(Box::new(BinaryExpr {
                                    op: Operator::Deref,
                                    left: Expr::LiteralIdentifier(b"a"),
                                    right: Expr::LiteralIdentifier(b"b"),
                                })),
                                right: Expr::Group(vec![Expr::LiteralIdentifier(b"c")]),
                            })),
                            right: Expr::LiteralIdentifier(b"d"),
                        })),
                        right: Expr::LiteralIdentifier(b"e"),
                    })),
                    right: Expr::Group(vec![Expr::LiteralIdentifier(b"f"), Expr::LiteralIdentifier(b"g")]),
                })),
                right: Expr::Binary(Box::new(BinaryExpr {
                    op: Operator::Deref,
                    left: Expr::Binary(Box::new(BinaryExpr {
                        op: Operator::Index,
                        left: Expr::LiteralIdentifier(b"h"),
                        right: Expr::Group(vec![Expr::LiteralIdentifier(b"i"), Expr::LiteralIdentifier(b"j")]),
                    })),
                    right: Expr::LiteralIdentifier(b"k"),
                })),
            }))]),
        );
    }

    #[test]
    fn assignment_lhs_expression() {
        assert_ast(
            // Assignment whose LHS is an expression-deref
            "(a + 1).x = 400;",
            Some(vec![Expr::Binary(Box::new(BinaryExpr {
                op: Operator::Assign,
                left: Expr::Binary(Box::new(BinaryExpr {
                    op: Operator::Deref,
                    left: Expr::Binary(Box::new(BinaryExpr {
                        op: Operator::Add,
                        left: Expr::LiteralIdentifier(b"a"),
                        right: Expr::LiteralReal(1.0),
                    })),
                    right: Expr::LiteralIdentifier(b"x"),
                })),
                right: Expr::LiteralReal(400.0),
            }))]),
        );
    }

    #[test]
    fn assignment_assign_equal() {
        assert_ast(
            // Differentiation between usages of '=' - simple
            "a=b=c",
            Some(vec![Expr::Binary(Box::new(BinaryExpr {
                op: Operator::Assign,
                left: Expr::LiteralIdentifier(b"a"),
                right: Expr::Binary(Box::new(BinaryExpr {
                    op: Operator::Equal,
                    left: Expr::LiteralIdentifier(b"b"),
                    right: Expr::LiteralIdentifier(b"c"),
                })),
            }))]),
        );
    }

    #[test]
    fn assignment_assign_equal_complex() {
        assert_ast(
            // Differentiation between usages of '=' - complex
            "(a=b).c[d=e]=f[g=h]=i",
            Some(vec![Expr::Binary(Box::new(BinaryExpr {
                op: Operator::Assign,
                left: Expr::Binary(Box::new(BinaryExpr {
                    op: Operator::Index,
                    left: Expr::Binary(Box::new(BinaryExpr {
                        op: Operator::Deref,
                        left: Expr::Binary(Box::new(BinaryExpr {
                            op: Operator::Equal,
                            left: Expr::LiteralIdentifier(b"a"),
                            right: Expr::LiteralIdentifier(b"b"),
                        })),
                        right: Expr::LiteralIdentifier(b"c"),
                    })),
                    right: Expr::Group(vec![Expr::Binary(Box::new(BinaryExpr {
                        op: Operator::Equal,
                        left: Expr::LiteralIdentifier(b"d"),
                        right: Expr::LiteralIdentifier(b"e"),
                    }))]),
                })),
                right: Expr::Binary(Box::new(BinaryExpr {
                    op: Operator::Equal,
                    left: Expr::Binary(Box::new(BinaryExpr {
                        op: Operator::Index,
                        left: Expr::LiteralIdentifier(b"f"),
                        right: Expr::Group(vec![Expr::Binary(Box::new(BinaryExpr {
                            op: Operator::Equal,
                            left: Expr::LiteralIdentifier(b"g"),
                            right: Expr::LiteralIdentifier(b"h"),
                        }))]),
                    })),
                    right: Expr::LiteralIdentifier(b"i"),
                })),
            }))]),
        );
    }

    #[test]
    #[should_panic]
    fn deref_not_id() {
        // Invalid use of deref operator
        assert_ast("a..=1", None)
    }

    #[test]
    fn btree_unary_positive() {
        assert_ast(
            // Binary tree format - unary operator - positive
            "a=+1",
            Some(vec![Expr::Binary(Box::new(BinaryExpr {
                op: Operator::Assign,
                left: Expr::LiteralIdentifier(b"a"),
                right: Expr::Unary(Box::new(UnaryExpr { op: Operator::Add, child: Expr::LiteralReal(1.0) })),
            }))]),
        )
    }

    #[test]
    fn btree_unary_subtract() {
        assert_ast(
            // Binary tree format - unary operator - negative
            "a=-1",
            Some(vec![Expr::Binary(Box::new(BinaryExpr {
                op: Operator::Assign,
                left: Expr::LiteralIdentifier(b"a"),
                right: Expr::Unary(Box::new(UnaryExpr { op: Operator::Subtract, child: Expr::LiteralReal(1.0) })),
            }))]),
        )
    }

    #[test]
    fn btree_unary_complement() {
        assert_ast(
            // Binary tree format - unary operator - complement
            "a=~1",
            Some(vec![Expr::Binary(Box::new(BinaryExpr {
                op: Operator::Assign,
                left: Expr::LiteralIdentifier(b"a"),
                right: Expr::Unary(Box::new(UnaryExpr { op: Operator::Complement, child: Expr::LiteralReal(1.0) })),
            }))]),
        )
    }

    #[test]
    fn btree_unary_not() {
        assert_ast(
            // Binary tree format - unary operator - negative
            "a=!1",
            Some(vec![Expr::Binary(Box::new(BinaryExpr {
                op: Operator::Assign,
                left: Expr::LiteralIdentifier(b"a"),
                right: Expr::Unary(Box::new(UnaryExpr { op: Operator::Not, child: Expr::LiteralReal(1.0) })),
            }))]),
        )
    }

    #[test]
    fn btree_unary_syntax() {
        assert_ast(
            // Binary tree format - unary operators - syntax parse test
            "a = 1+!~-b.c[+d]-2--3", // (- (- (+ 1 2) 3) 4)
            Some(vec![Expr::Binary(Box::new(BinaryExpr {
                op: Operator::Assign,
                left: Expr::LiteralIdentifier(b"a"),
                right: Expr::Binary(Box::new(BinaryExpr {
                    op: Operator::Subtract,
                    left: Expr::Binary(Box::new(BinaryExpr {
                        op: Operator::Subtract,
                        left: Expr::Binary(Box::new(BinaryExpr {
                            op: Operator::Add,
                            left: Expr::LiteralReal(1.0),
                            right: Expr::Unary(Box::new(UnaryExpr {
                                op: Operator::Not,
                                child: Expr::Unary(Box::new(UnaryExpr {
                                    op: Operator::Complement,
                                    child: Expr::Unary(Box::new(UnaryExpr {
                                        op: Operator::Subtract,
                                        child: Expr::Binary(Box::new(BinaryExpr {
                                            op: Operator::Index,
                                            left: Expr::Binary(Box::new(BinaryExpr {
                                                op: Operator::Deref,
                                                left: Expr::LiteralIdentifier(b"b"),
                                                right: Expr::LiteralIdentifier(b"c"),
                                            })),
                                            right: Expr::Group(vec![Expr::Unary(Box::new(UnaryExpr {
                                                op: Operator::Add,
                                                child: Expr::LiteralIdentifier(b"d"),
                                            }))]),
                                        })),
                                    })),
                                })),
                            })),
                        })),
                        right: Expr::LiteralReal(2.0),
                    })),
                    right: Expr::Unary(Box::new(UnaryExpr { op: Operator::Subtract, child: Expr::LiteralReal(3.0) })),
                })),
            }))]),
        )
    }

    #[test]
    fn btree_unary_grouping() {
        assert_ast(
            // Unary operator applied to sub-tree
            "a = ~(b + 1)",
            Some(vec![Expr::Binary(Box::new(BinaryExpr {
                op: Operator::Assign,
                left: Expr::LiteralIdentifier(b"a"),
                right: Expr::Unary(Box::new(UnaryExpr {
                    op: Operator::Complement,
                    child: Expr::Binary(Box::new(BinaryExpr {
                        op: Operator::Add,
                        left: Expr::LiteralIdentifier(b"b"),
                        right: Expr::LiteralReal(1.0),
                    })),
                })),
            }))]),
        )
    }

    #[test]
    fn function_syntax() {
        assert_ast(
            // Function call syntax
            "instance_create(random(800), random(608,), apple);",
            Some(vec![Expr::Function(Box::new(FunctionExpr {
                name: b"instance_create",
                params: vec![
                    Expr::Function(Box::new(FunctionExpr { name: b"random", params: vec![Expr::LiteralReal(800.0)] })),
                    Expr::Function(Box::new(FunctionExpr { name: b"random", params: vec![Expr::LiteralReal(608.0)] })),
                    Expr::LiteralIdentifier(b"apple"),
                ],
            }))]),
        )
    }

    #[test]
    fn for_syntax_standard() {
        assert_ast(
            // For-loop syntax - standard
            "for(i = 0; i < 10; i += 1) { a = 1; b = c;}",
            Some(vec![Expr::For(Box::new(ForExpr {
                start: Expr::Binary(Box::new(BinaryExpr {
                    op: Operator::Assign,
                    left: Expr::LiteralIdentifier(b"i"),
                    right: Expr::LiteralReal(0.0),
                })),
                cond: Expr::Binary(Box::new(BinaryExpr {
                    op: Operator::LessThan,
                    left: Expr::LiteralIdentifier(b"i"),
                    right: Expr::LiteralReal(10.0),
                })),
                step: Expr::Binary(Box::new(BinaryExpr {
                    op: Operator::AssignAdd,
                    left: Expr::LiteralIdentifier(b"i"),
                    right: Expr::LiteralReal(1.0),
                })),
                body: Expr::Group(vec![
                    Expr::Binary(Box::new(BinaryExpr {
                        op: Operator::Assign,
                        left: Expr::LiteralIdentifier(b"a"),
                        right: Expr::LiteralReal(1.0),
                    })),
                    Expr::Binary(Box::new(BinaryExpr {
                        op: Operator::Assign,
                        left: Expr::LiteralIdentifier(b"b"),
                        right: Expr::LiteralIdentifier(b"c"),
                    })),
                ]),
            }))]),
        )
    }

    #[test]
    fn for_syntax_no_sep() {
        assert_ast(
            // For-loop syntax - no separators
            "for(i=0 i<10 i+=1) c=3",
            Some(vec![Expr::For(Box::new(ForExpr {
                start: Expr::Binary(Box::new(BinaryExpr {
                    op: Operator::Assign,
                    left: Expr::LiteralIdentifier(b"i"),
                    right: Expr::LiteralReal(0.0),
                })),
                cond: Expr::Binary(Box::new(BinaryExpr {
                    op: Operator::LessThan,
                    left: Expr::LiteralIdentifier(b"i"),
                    right: Expr::LiteralReal(10.0),
                })),
                step: Expr::Binary(Box::new(BinaryExpr {
                    op: Operator::AssignAdd,
                    left: Expr::LiteralIdentifier(b"i"),
                    right: Expr::LiteralReal(1.0),
                })),
                body: Expr::Binary(Box::new(BinaryExpr {
                    op: Operator::Assign,
                    left: Expr::LiteralIdentifier(b"c"),
                    right: Expr::LiteralReal(3.0),
                })),
            }))]),
        )
    }

    #[test]
    fn for_syntax_random_sep() {
        assert_ast(
            // For-loop syntax - arbitrary semicolons
            "for(i=0; i<10 i+=1; ;) {d=4}",
            Some(vec![Expr::For(Box::new(ForExpr {
                start: Expr::Binary(Box::new(BinaryExpr {
                    op: Operator::Assign,
                    left: Expr::LiteralIdentifier(b"i"),
                    right: Expr::LiteralReal(0.0),
                })),
                cond: Expr::Binary(Box::new(BinaryExpr {
                    op: Operator::LessThan,
                    left: Expr::LiteralIdentifier(b"i"),
                    right: Expr::LiteralReal(10.0),
                })),
                step: Expr::Binary(Box::new(BinaryExpr {
                    op: Operator::AssignAdd,
                    left: Expr::LiteralIdentifier(b"i"),
                    right: Expr::LiteralReal(1.0),
                })),
                body: Expr::Group(vec![Expr::Binary(Box::new(BinaryExpr {
                    op: Operator::Assign,
                    left: Expr::LiteralIdentifier(b"d"),
                    right: Expr::LiteralReal(4.0),
                }))]),
            }))]),
        )
    }

    #[test]
    fn pascal_init_assign() {
        assert_ast(
            "a := 1",
            Some(vec![Expr::Binary(Box::new(BinaryExpr {
                op: Operator::Assign,
                left: Expr::LiteralIdentifier(b"a"),
                right: Expr::LiteralReal(1.0),
            }))]),
        );
    }

    #[test]
    fn pascal_if() {
        assert_ast(
            "
            if a == 1 then
            begin
                a = 2;
            end
            else if a == 2 then
            begin
                a = 4;
            end
            ",
            Some(vec![Expr::If(Box::new(IfExpr {
                cond: Expr::Binary(Box::new(BinaryExpr {
                    op: Operator::Equal,
                    left: Expr::LiteralIdentifier(b"a"),
                    right: Expr::LiteralReal(1.0),
                })),
                body: Expr::Group(vec![Expr::Binary(Box::new(BinaryExpr {
                    op: Operator::Assign,
                    left: Expr::LiteralIdentifier(b"a"),
                    right: Expr::LiteralReal(2.0),
                }))]),
                else_body: Some(Expr::If(Box::new(IfExpr {
                    cond: Expr::Binary(Box::new(BinaryExpr {
                        op: Operator::Equal,
                        left: Expr::LiteralIdentifier(b"a"),
                        right: Expr::LiteralReal(2.0),
                    })),
                    body: Expr::Group(vec![Expr::Binary(Box::new(BinaryExpr {
                        op: Operator::Assign,
                        left: Expr::LiteralIdentifier(b"a"),
                        right: Expr::LiteralReal(4.0),
                    }))]),
                    else_body: None,
                }))),
            }))]),
        );
    }

    #[test]
    fn var_syntax() {
        assert_ast(
            // var syntax - basic constructions
            "var a; var b, c",
            Some(vec![
                Expr::Var(Box::new(VarExpr { vars: vec![b"a"] })),
                Expr::Var(Box::new(VarExpr { vars: vec![b"b", b"c"] })),
            ]),
        )
    }

    #[test]
    fn var_syntax_complex() {
        assert_ast(
            // var syntax - unusual valid constructions
            "var; var a,b,; var c,var",
            Some(vec![
                Expr::Var(Box::new(VarExpr { vars: vec![] })),
                Expr::Var(Box::new(VarExpr { vars: vec![b"a", b"b"] })),
                Expr::Var(Box::new(VarExpr { vars: vec![b"c"] })),
                Expr::Var(Box::new(VarExpr { vars: vec![] })),
            ]),
        )
    }

    #[test]
    #[should_panic]
    fn var_invalid_comma() {
        // var syntax - invalid comma
        assert_ast("var, a;", None)
    }

    #[test]
    fn var_surprise_function() {
        // var syntax - surprise function call after non-comma-separated var name list
        assert_ast(
            "var a instance_create instance_destroy ()",
            Some(vec![
                Expr::Var(Box::new(VarExpr { vars: vec![b"a", b"instance_create"] })),
                Expr::Function(Box::new(FunctionExpr { name: b"instance_destroy", params: vec![] })),
            ]),
        )
    }

    #[test]
    fn var_surprise_constant() {
        // var syntax - surprise constant after non-comma-separated var name list
        assert_ast(
            "var a b global.g = 0",
            Some(vec![
                Expr::Var(Box::new(VarExpr { vars: vec![b"a", b"b"] })),
                Expr::Binary(Box::new(BinaryExpr {
                    op: Operator::Assign,
                    left: Expr::Binary(Box::new(BinaryExpr {
                        op: Operator::Deref,
                        left: Expr::LiteralIdentifier(b"global"),
                        right: Expr::LiteralIdentifier(b"g"),
                    })),
                    right: Expr::LiteralReal(0.0),
                })),
            ]),
        )
    }

    #[test]
    fn expression_literal_real() {
        // expression - single literal real
        assert_eq!(AST::expression(b"1").unwrap(), Expr::LiteralReal(1.0));
    }

    #[test]
    fn expression_literal_identifier() {
        // expression - literal identifier
        assert_eq!(AST::expression(b"a").unwrap(), Expr::LiteralIdentifier(b"a"));
    }

    #[test]
    fn expression_with_operators() {
        // expression - unary and binary operators
        assert_eq!(
            AST::expression(b"1 * -2").unwrap(),
            Expr::Binary(Box::new(BinaryExpr {
                op: Operator::Multiply,
                left: Expr::LiteralReal(1.0),
                right: Expr::Unary(Box::new(UnaryExpr { op: Operator::Subtract, child: Expr::LiteralReal(2.0) })),
            }))
        );
    }

    #[test]
    fn expression_with_overrun() {
        // expression with extra code after it - extra code should be dropped
        assert_eq!(AST::expression(b"0; a=1; game_end()").unwrap(), Expr::LiteralReal(0.0));
    }
}
