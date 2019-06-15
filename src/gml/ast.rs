use super::lexer::Lexer;
use super::token::{Keyword, Operator, Separator, Token};

use std::error;
use std::fmt;

use std::iter::Peekable;

pub struct AST<'a> {
    pub expressions: Vec<Expr<'a>>,
}

pub enum Expr<'a> {
    Literal(Token<'a>),

    Unary(Box<UnaryExpr<'a>>),
    Binary(Box<BinaryExpr<'a>>),

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

pub struct UnaryExpr<'a> {
    pub op: Operator,
    pub child: Expr<'a>,
}

pub struct BinaryExpr<'a> {
    pub op: Operator,
    pub left: Expr<'a>,
    pub right: Expr<'a>,
}

pub struct DoUntilExpr<'a> {
    pub cond: Expr<'a>,
    pub body: Expr<'a>,
}

pub struct ForExpr<'a> {
    pub start: Expr<'a>,
    pub cond: Expr<'a>,
    pub step: Expr<'a>,

    pub body: Expr<'a>,
}

pub struct IfExpr<'a> {
    pub cond: Expr<'a>,
    pub body: Expr<'a>,
    pub else_body: Option<Expr<'a>>,
}

pub struct RepeatExpr<'a> {
    pub count: Expr<'a>,
    pub body: Expr<'a>,
}

pub struct SwitchExpr<'a> {
    pub value: Expr<'a>,
    pub cases: Vec<(Expr<'a>, Expr<'a>)>,
}

pub struct VarExpr<'a> {
    pub vars: Vec<&'a str>,
}

pub struct WithExpr<'a> {
    pub target: Expr<'a>,
    pub body: Expr<'a>,
}

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

        loop {
            // Get the first token from the iterator, or exit the loop if there are no more
            let expr = AST::read_line(&mut lex);
            match expr {
                Ok(Some(expr)) => {
                    expressions.push(expr);
                }
                Ok(None) => break,
                Err(e) => return Err(e),
            }
        }

        Ok(AST { expressions })
    }

    fn read_line(lex: &mut Peekable<Lexer<'a>>) -> Result<Option<Expr<'a>>, Error> {
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
                                    return Err(Error::new(
                                        "Expected var name, found EOF".to_string(),
                                    ))
                                }
                            };
                            let var_name = match next_token {
                                Token::Identifier(id) => id,
                                _ => {
                                    return Err(Error::new(format!(
                                        "Invalid token, expected var name: {:?}",
                                        next_token
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
                            "Invalid Keyword at beginning of expression: {:?}",
                            key
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
                                _ => {
                                    let inner_exp = AST::read_line(lex);
                                    match inner_exp {
                                        Ok(Some(e)) => inner_expressions.push(e),
                                        Ok(None) => {
                                            return Err(Error::new(
                                                "Unclosed brace at EOF".to_string(),
                                            ))
                                        }
                                        Err(e) => return Err(e),
                                    }
                                }
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
                            "Invalid Separator at beginning of expression: {:?}",
                            sep
                        )))
                    }
                }
            }

            Token::Comment(_) => Ok(Some(Expr::Nop)),

            Token::LineHint(_) => Ok(Some(Expr::Nop)), // TODO: store the line number somewhere

            _ => {
                return Err(Error::new(format!(
                    "Invalid token at beginning of expression: {:?}",
                    token
                )))
            }
        }
    }
}
