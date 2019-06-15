use super::lexer::Lexer;
use super::token::{Keyword, Operator, Separator, Token};

use std::error;
use std::fmt;

use std::iter::Peekable;

pub struct AST<'a> {
    pub expressions: Vec<Expr<'a>>,
}

#[derive(Debug)]
pub enum Expr<'a> {
    Assignment(Box<AssignmentExpr<'a>>),
    Function(Box<FunctionExpr<'a>>),
    DoUntil(Box<DoUntilExpr<'a>>),
    For(Box<ForExpr<'a>>),
    Group(Vec<Expr<'a>>),
    If(Box<IfExpr<'a>>),
    Repeat(Box<RepeatExpr<'a>>),
    Switch(Box<SwitchExpr<'a>>),
    Var(Box<VarExpr<'a>>),
    With(Box<WithExpr<'a>>),
    While(Box<WhileExpr<'a>>),

    Nop,
}

#[derive(Debug)]
pub enum Evaluable<'a> {
    LiteralReal(f64),
    LiteralString(String),
    Identifier(Box<IdentifierExpr<'a>>),
    Function(Box<FunctionExpr<'a>>),
    Unary(Box<UnaryExpr<'a>>),
    Binary(Box<BinaryExpr<'a>>),
}


#[derive(Debug)]
pub struct AssignmentExpr<'a> {
    pub op: Operator,
    pub lhs: IdentifierExpr<'a>,
}

#[derive(Debug)]
pub struct UnaryExpr<'a> {
    pub op: Operator,
    pub child: Expr<'a>,
}

#[derive(Debug)]
pub struct BinaryExpr<'a> {
    pub op: Operator,
    pub left: Expr<'a>,
    pub right: Expr<'a>,
}

#[derive(Debug)]
pub struct IdentifierExpr<'a> {
    pub variable: &'a str,
    pub owner: Option<Expr<'a>>,
    pub array_accessor: Vec<Expr<'a>>,
}

#[derive(Debug)]
pub struct FunctionExpr<'a> {
    pub name: &'a str,
    pub params: Vec<Expr<'a>>,
}

#[derive(Debug)]
pub struct DoUntilExpr<'a> {
    pub cond: Expr<'a>,
    pub body: Expr<'a>,
}

#[derive(Debug)]
pub struct ForExpr<'a> {
    pub start: Expr<'a>,
    pub cond: Expr<'a>,
    pub step: Expr<'a>,

    pub body: Expr<'a>,
}

#[derive(Debug)]
pub struct IfExpr<'a> {
    pub cond: Expr<'a>,
    pub body: Expr<'a>,
    pub else_body: Option<Expr<'a>>,
}

#[derive(Debug)]
pub struct RepeatExpr<'a> {
    pub count: Expr<'a>,
    pub body: Expr<'a>,
}

#[derive(Debug)]
pub struct SwitchExpr<'a> {
    pub value: Expr<'a>,
    pub cases: Vec<(Expr<'a>, Expr<'a>)>,
}

#[derive(Debug)]
pub struct VarExpr<'a> {
    pub vars: Vec<&'a str>,
}

#[derive(Debug)]
pub struct WithExpr<'a> {
    pub target: Expr<'a>,
    pub body: Expr<'a>,
}

#[derive(Debug)]
pub struct WhileExpr<'a> {
    pub cond: Expr<'a>,
    pub body: Expr<'a>,
}

#[derive(Debug)]
pub struct Error {
    message: String,
}

impl Error {
    pub fn new(message: String) -> Self {
        Error { message }
    }
}

impl error::Error for Error {}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl<'a> AST<'a> {
    pub fn new(source: &'a str) -> Result<Self, Error> {
        let mut lex = Lexer::new(source).peekable();
        let mut expressions = Vec::new();
        let mut line: usize = 1;

        loop {
            // Get the first token from the iterator, or exit the loop if there are no more
            match AST::read_line(&mut lex, &mut line) {
                Ok(Some(expr)) => {
                    // Filter top-level NOPs
                    if !expr.is_nop() {
                        expressions.push(expr);
                    }
                }
                Ok(None) => break,
                Err(e) => return Err(e),
            }
        }

        Ok(AST { expressions })
    }

    fn read_line(
        lex: &mut Peekable<Lexer<'a>>,
        line: &mut usize,
    ) -> Result<Option<Expr<'a>>, Error> {
        let token = match lex.next() {
            Some(t) => t,
            None => return Ok(None), // EOF
        };

        // Use token type to determine what logic we should apply here
        match token {
            Token::Keyword(key) => {
                match key {
                    Keyword::Var => {
                        // Read var identifiers
                        let mut vars = Vec::with_capacity(1);

                        loop {
                            // Read one identifier and store it as a var name
                            let next_token = match lex.next() {
                                Some(t) => t,
                                None => {
                                    return Err(Error::new(format!(
                                        "Expected var name, found EOF (line {})",
                                        line
                                    )))
                                }
                            };
                            let var_name = match next_token {
                                Token::Identifier(id) => id,
                                _ => {
                                    return Err(Error::new(format!(
                                        "Invalid token, expected var name (line {}): {:?}",
                                        line, next_token,
                                    )))
                                }
                            };
                            vars.push(var_name);

                            // Check if next token is a comma, if so, we expect another var name afterwards
                            let next_token = match lex.peek() {
                                Some(t) => t,
                                None => break,
                            };
                            match next_token {
                                Token::Separator(ref sep) if *sep == Separator::Comma => {
                                    lex.next(); // skip the comma
                                }
                                _ => break,
                            }
                        }

                        Ok(Some(Expr::Var(Box::new(VarExpr { vars }))))
                    }

                    Keyword::Do => {
                        // TODO: do-until
                        Ok(None)
                    }

                    Keyword::If => {
                        // TODO: if-else
                        Ok(None)
                    }

                    Keyword::For => {
                        // TODO: for
                        Ok(None)
                    }

                    Keyword::Repeat => {
                        // TODO: repeat
                        Ok(None)
                    }

                    Keyword::Switch => {
                        // TODO: switch
                        Ok(None)
                    }

                    Keyword::With => {
                        // TODO: with
                        Ok(None)
                    }

                    Keyword::While => {
                        // TODO: while
                        Ok(None)
                    }

                    _ => {
                        return Err(Error::new(format!(
                            "Invalid Keyword at beginning of expression on line {}: {:?}",
                            line, key
                        )))
                    }
                }
            }

            Token::Identifier(id) => {
                // An expression starting with an identifier may be either an assignment or script/function.
                // This is determined by what type of token immediately follows it.
                let next_token = match lex.next() {
                    Some(t) => t,
                    None => return Err(Error::new(format!("Stray identifier at EOF: {:?}", id))),
                };
                match next_token {
                    Token::Separator(ref sep) if *sep == Separator::ParenLeft => {
                        // TODO: parse a script/function call
                        Ok(None)
                    }
                    _ => {
                        // TODO: parse an assignment
                        // (note next token may be one of 8 assignment operators, period, or '[' )
                        Ok(None)
                    }
                }
            }

            Token::Separator(sep) => {
                match sep {
                    // Code contained in {} is treated here as one single expression, called a Group.
                    Separator::BraceLeft => {
                        let mut inner_expressions = Vec::new();
                        loop {
                            match lex.peek() {
                                Some(Token::Separator(Separator::BraceRight)) => {
                                    lex.next();
                                    break;
                                }
                                _ => match AST::read_line(lex, line) {
                                    Ok(Some(e)) => inner_expressions.push(e),
                                    Ok(None) => {
                                        return Err(Error::new("Unclosed brace at EOF".to_string()))
                                    }
                                    Err(e) => return Err(e),
                                },
                            }
                        }
                        Ok(Some(Expr::Group(inner_expressions)))
                    }

                    // An assignment may start with an open-parenthesis, eg: (1).x = 400;
                    Separator::ParenLeft => {
                        // TODO: parse an assignment
                        Ok(None)
                    }

                    // A semicolon is treated as a line of code which does nothing.
                    Separator::Semicolon => Ok(Some(Expr::Nop)),

                    // Default
                    _ => {
                        return Err(Error::new(format!(
                            "Invalid Separator at beginning of expression on line {}: {:?}",
                            line, sep
                        )))
                    }
                }
            }

            Token::Comment(_) => Ok(Some(Expr::Nop)),

            Token::LineHint(l) => {
                *line = l;
                Ok(Some(Expr::Nop))
            }

            _ => {
                return Err(Error::new(format!(
                    "Invalid token at beginning of expression on line {}: {:?}",
                    line, token
                )))
            }
        }
    }
}

impl<'a> Expr<'a> {
    pub fn is_nop(&self) -> bool {
        if let Expr::Nop = self {
            true
        } else {
            false
        }
    }
}
