use super::lexer::Lexer;
use super::token::{Operator, Token, Keyword, Separator};

use std::error;
use std::fmt;

use std::collections::HashSet;

pub struct AST {}

pub enum Expr<'a> {
    Literal(Token<'a>),

    Unary(Box<UnaryExpr<'a>>),
    Binary(Box<BinaryExpr<'a>>),

    DoUntil(Box<DoUntilExpr<'a>>),
    If(Box<IfExpr<'a>>),
    For(Box<ForExpr<'a>>),
    Repeat(Box<RepeatExpr<'a>>),
    Switch(Box<SwitchExpr<'a>>),
    With(Box<WithExpr<'a>>),
    While(Box<WhileExpr<'a>>),
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

pub struct IfExpr<'a> {
    pub cond: Expr<'a>,
    pub body: Expr<'a>,
    pub else_body: Option<Expr<'a>>,
}

pub struct ForExpr<'a> {
    pub start: Expr<'a>,
    pub cond: Expr<'a>,
    pub step: Expr<'a>,

    pub body: Expr<'a>,
}

pub struct RepeatExpr<'a> {
    pub count: Expr<'a>,
    pub body: Expr<'a>,
}

pub struct SwitchExpr<'a> {
    pub value: Expr<'a>,
    pub cases: Vec<(Expr<'a>, Expr<'a>)>,
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
        Error {
            message
        }
    }
}

impl error::Error for Error {}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl AST {
    pub fn new(source: &str) -> Result<Self, Error> {
        let mut lex = Lexer::new(source).peekable();
        let mut locals: HashSet<&str> = HashSet::new();

        loop {
            // Get the first token from the iterator, or exit the loop if there are no more
            let token = match lex.next() {
                Some(t) => t,
                None => break,
            };

            // Use token type to determine what logic we should apply here
            match token {
                Token::Keyword(key) => {
                    match key {
                        Keyword::Var => {
                            // Read var identifiers
                            loop {
                                // Read one identifier and store it as a var name
                                let next_token = match lex.next() {
                                    Some(t) => t,
                                    None => return Err(Error::new("Expected var name, found EOF".to_string())),
                                };
                                let var_name = match next_token {
                                    Token::Identifier(id) => id,
                                    _ => return Err(Error::new(format!("Invalid token, expected var name: {:?}", next_token))),
                                };
                                locals.insert(var_name);

                                // Check if next token is a comma, if so, we expect another var name afterwards
                                let next_token = match lex.peek() {
                                    Some(t) => t,
                                    None => break,
                                };
                                match next_token {
                                    Token::Separator(ref sep) if *sep == Separator::Comma => {
                                        lex.next(); // skip the comma
                                    },
                                    _ => break,
                                }
                            }
                        },

                        Keyword::Do => {
                            // TODO: do-until
                        },

                        Keyword::If => {
                            // TODO: if-else
                        },

                        Keyword::For => {
                            // TODO: for
                        },

                        Keyword::Repeat => {
                            // TODO: repeat
                        },

                        Keyword::Switch => {
                            // TODO: switch
                        },

                        Keyword::With => {
                            // TODO: with
                        },

                        Keyword::While => {
                            // TODO: while
                        },

                        _ => return Err(Error::new(format!("Invalid Keyword at beginning of expression: {:?}", key))),
                    }
                },

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
                        },
                        _ => {
                            // TODO: parse an assignment
                            // (note next token may be one of 8 assignment operators, period, or '[' )
                        }
                    }
                },

                Token::Separator(sep) => {
                    // An assignment may start with an open-parenthesis, eg: (1).x = 400;
                    match sep {
                        Separator::ParenLeft => {
                            // TODO: parse an assignment
                        },
                        Separator::Semicolon => {},
                        _ => return Err(Error::new(format!("Invalid Separator at beginning of expression: {:?}", sep))),
                    }
                },
                
                Token::Comment(_) => continue,
                _ => return Err(Error::new(format!("Invalid token at beginning of expression: {:?}", token))),
            }
        }

        Ok(AST {})
    }
}
