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
    Function(Box<FunctionExpr<'a>>),

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
pub struct FunctionExpr<'a> {
    pub name: &'a str,
    pub args: Vec<Expr<'a>>,
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
                        let binary_tree = AST::read_binary_tree(lex, line, Some(token))?;
                        Ok(Some(binary_tree))
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
                        let binary_tree = AST::read_binary_tree(lex, line, None)?;
                        Ok(Some(binary_tree))
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

    fn read_binary_tree(
        lex: &mut Peekable<Lexer<'a>>,
        line: &mut usize,
        first_token: Option<Token<'a>>,
    ) -> Result<Expr<'a>, Error> {
        // player.alarm[0]
        // ([] (. player alarm) 0)
        // (1).x = 5;
        // (= (. 1 x) 5)

        // Get the very first token in this exp value
        let mut lhs = match first_token {
            Some(t) => Expr::Literal(t),
            None => match lex.next() {
                Some(Token::Separator(ref sep)) if *sep == Separator::ParenLeft => {
                    let binary_tree = AST::read_binary_tree(lex, line, None)?;
                    if lex.next() != Some(Token::Separator(Separator::ParenRight)) {
                        return Err(Error::new(format!(
                            "Unclosed parenthesis in binary tree on line {}",
                            line
                        )));
                    }
                    binary_tree
                }
                Some(Token::Identifier(t)) => Expr::Literal(Token::Identifier(t)),
                Some(t) => {
                    return Err(Error::new(format!(
                        "Invalid token while scanning binary tree on line {}: {:?}",
                        line, t
                    )))
                }
                None => {
                    return Err(Error::new(format!(
                        "Found EOF unexpectedly while reading binary tree (line {})",
                        line
                    )))
                }
            },
        };

        // Do we need to amend this LHS at all?
        match lex.peek() {
            Some(Token::Separator(ref sep)) if *sep == Separator::BracketLeft => {
                lex.next();
                let mut dimensions = Vec::new();
                loop {
                    match lex.peek() {
                        Some(Token::Separator(ref sep)) if *sep == Separator::BracketRight => {
                            lex.next();
                            break;
                        }
                        Some(Token::Separator(ref sep)) if *sep == Separator::Comma => {
                            lex.next();
                        }
                        None => {
                            return Err(Error::new(format!(
                                "Found EOF unexpectedly while reading binary tree (line {})",
                                line
                            )))
                        }
                        _ => {
                            let binary_tree = AST::read_binary_tree(lex, line, None)?;
                            dimensions.push(binary_tree);
                        }
                    }
                }
                lhs = Expr::Binary(Box::new(BinaryExpr {
                    op: Operator::ArrayAccessor,
                    left: lhs,
                    right: Expr::Group(dimensions),
                }));
            }

            Some(Token::Separator(ref sep)) if *sep == Separator::Period => {
                lex.next();
                lhs = Expr::Binary(Box::new(BinaryExpr {
                    op: Operator::Period,
                    left: lhs,
                    right: Expr::Literal(lex.next().ok_or_else(|| {
                        Error::new(format!(
                            "Found EOF unexpectedly while reading binary tree (line {})",
                            line
                        ))
                    })?),
                }));
            }
            Some(_) => {}
            None => {
                return Err(Error::new(format!(
                    "Found EOF unexpectedly while reading binary tree (line {})",
                    line
                )))
            }
        }

        // Check if the next token is an operator
        // TODO: we don't do precedence
        let next_token = lex.peek();
        match next_token {
            Some(&Token::Operator(_)) => {
                if let Some(Token::Operator(op)) = lex.next() {
                    Ok(Expr::Binary(Box::new(BinaryExpr {
                        op: op,
                        left: lhs,
                        right: AST::read_binary_tree(lex, line, None)?,
                    })))
                } else {
                    unreachable!()
                }
            }
            _ => Ok(lhs),
        }
    }

    fn get_op_precedence(op: Operator) -> i8 {
        match op {
            Operator::Add => 4,
            Operator::Subtract => 4,
            Operator::Multiply => 5,
            Operator::Divide => 5,
            Operator::IntDivide => 5,
            Operator::BinaryAnd => 2,
            Operator::BinaryOr => 2,
            Operator::BinaryXor => 2,
            Operator::Assign => 6,
            Operator::Not => 7,
            Operator::LessThan => 1,
            Operator::GreaterThan => 1,
            Operator::AssignAdd => 6,
            Operator::AssignSubtract => 6,
            Operator::AssignMultiply => 6,
            Operator::AssignDivide => 6,
            Operator::AssignBinaryAnd => 6,
            Operator::AssignBinaryOr => 6,
            Operator::AssignBinaryXor => 6,
            Operator::Equal => 1,
            Operator::NotEqual => 1,
            Operator::LessThanOrEqual => 1,
            Operator::GreaterThanOrEqual => 1,
            Operator::Modulo => 5,
            Operator::And => 0,
            Operator::Or => 0,
            Operator::Xor => 0,
            Operator::BinaryShiftLeft => 3,
            Operator::BinaryShiftRight => 3,
            Operator::Complement => 7,
            Operator::Period => 9,
            Operator::ArrayAccessor => 8,
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
