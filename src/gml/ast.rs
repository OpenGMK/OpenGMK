use super::lexer::Lexer;
use super::token::{Keyword, Operator, Separator, Token};

use std::error;
use std::fmt;

use std::iter::Peekable;

#[derive(Debug, PartialEq)]
pub struct AST<'a> {
    pub expressions: Vec<Expr<'a>>,
}

#[derive(Debug, PartialEq)]
pub enum Expr<'a> {
    Literal(Token<'a>),

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
    With(Box<WithExpr<'a>>),
    While(Box<WhileExpr<'a>>),

    Nop,
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
pub struct IdentifierExpr<'a> {
    pub variable: &'a str,
    pub owner: Option<Expr<'a>>,
    pub array_accessor: Vec<Expr<'a>>,
}

#[derive(Debug, PartialEq)]
pub struct FunctionExpr<'a> {
    pub name: &'a str,
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
    pub value: Expr<'a>,
    pub cases: Vec<(Expr<'a>, Expr<'a>)>,
}

#[derive(Debug, PartialEq)]
pub struct VarExpr<'a> {
    pub vars: Vec<&'a str>,
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
                let next_token = match lex.peek() {
                    Some(t) => t,
                    None => return Err(Error::new(format!("Stray identifier at EOF: {:?}", id))),
                };
                match next_token {
                    Token::Separator(ref sep) if *sep == Separator::ParenLeft => {
                        // TODO: parse a script/function call
                        Ok(None)
                    }
                    _ => {
                        let binary_tree = AST::read_binary_tree(lex, line, Some(token), true, 0)?;
                        if let Some(op) = binary_tree.1 {
                            Err(Error::new(format!(
                                "Stray operator {:?} in expression on line {}",
                                op, line,
                            )))
                        } else {
                            Ok(Some(binary_tree.0))
                        }
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
                        let binary_tree = AST::read_binary_tree(lex, line, Some(Token::Separator(Separator::ParenLeft)), true, 0)?;
                        if let Some(op) = binary_tree.1 {
                            Err(Error::new(format!(
                                "Stray operator {:?} in expression on line {}",
                                op, line,
                            )))
                        } else {
                            Ok(Some(binary_tree.0))
                        }
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
        first_token: Option<Token<'a>>, // Sometimes we've already parsed the first token, so it should be put here.
        expect_assignment: bool,        // Do we expect the first op to be an assignment?
        lowest_prec: u8, // We are not allowed to go below this operator precedence in this tree. If we do, we'll return the next op.
    ) -> Result<(Expr<'a>, Option<Operator>), Error> {
        // Get the very first token in this exp value
        let mut lhs = match if first_token.is_some() {
            first_token
        } else {
            lex.next()
        } {
            Some(Token::Separator(ref sep)) if *sep == Separator::ParenLeft => {
                let binary_tree = AST::read_binary_tree(lex, line, None, false, 0)?;
                if lex.next() != Some(Token::Separator(Separator::ParenRight)) {
                    return Err(Error::new(format!(
                        "Unclosed parenthesis in binary tree on line {}",
                        line
                    )));
                } else if let Some(op) = binary_tree.1 {
                    return Err(Error::new(format!(
                        "Stray operator {:?} in expression on line {}",
                        op, line,
                    )));
                }
                binary_tree.0
            }
            Some(Token::Identifier(t)) => Expr::Literal(Token::Identifier(t)),
            Some(Token::Real(t)) => Expr::Literal(Token::Real(t)),
            Some(Token::String(t)) => Expr::Literal(Token::String(t)),
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
        };

        // Do we need to amend this LHS at all?
        loop {
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
                                let binary_tree = AST::read_binary_tree(lex, line, None, false, 0)?;
                                if let Some(op) = binary_tree.1 {
                                    return Err(Error::new(format!(
                                        "Stray operator {:?} in expression on line {}",
                                        op, line,
                                    )));
                                }
                                dimensions.push(binary_tree.0);
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
                _ => break,
            }
        }

        // Check if the next token is an operator
        let next_token = lex.peek();
        match next_token {
            Some(&Token::Operator(_)) => {
                if let Some(Token::Operator(mut op)) = lex.next() {
                    // '=' can be either an assignment or equality check (==) in GML.
                    // So if we're not expecting an assignment operator, it should be seen as a comparator instead.
                    if (op == Operator::Assign) && (!expect_assignment) {
                        op = Operator::Equal;
                    }

                    // Now, loop until there are no more buffered operators.
                    loop {
                        // Here, we get the precedence of the operator we found. If this returns None, it's probably an assignment,
                        // so we use that in conjunction with an if-let to check its validity.
                        // TODO: probably don't do this, it assumes any non-precedence op is an assignment, eg "1 ! 2;" gets compiled to (not 1 2)
                        if let Some(precedence) = AST::get_op_precedence(&op) {
                            // this op is invalid if assignment expected
                            if expect_assignment {
                                break Err(Error::new(format!(
                                    "Invalid operator {:?} found, expected assignment (line {})",
                                    op, line,
                                )));
                            } else {
                                // If this op has lower precedence than we're allowed to read, we have to return it here.
                                if precedence < lowest_prec {
                                    break Ok((lhs, Some(op)));
                                } else {
                                    // We're allowed to use the next operator. Let's read an RHS to put on after it.
                                    // You might be thinking "precedence + 1" is counter-intuitive- "precedence" would make more sense. Well, the difference
                                    // is left-to-right vs right-to-left construction. This way, 1/2/3 is correctly built as (1/2)/3 rather than 1/(2/3).
                                    let rhs = AST::read_binary_tree(
                                        lex,
                                        line,
                                        None,
                                        false,
                                        precedence + 1,
                                    )?;
                                    if let Some(next_op) = rhs.1 {
                                        // There's another operator even after the RHS.
                                        if let Some(next_prec) = AST::get_op_precedence(&next_op) {
                                            if next_prec < lowest_prec {
                                                // This next op is lower than we're allowed to go, so we have to return it
                                                break Ok((
                                                    Expr::Binary(Box::new(BinaryExpr {
                                                        op: op,
                                                        left: lhs,
                                                        right: rhs.0,
                                                    })),
                                                    Some(next_op),
                                                ));
                                            } else {
                                                // Update LHS by sticking RHS onto it, set op to the new operator, and go round again.
                                                lhs = Expr::Binary(Box::new(BinaryExpr {
                                                    op: op,
                                                    left: lhs,
                                                    right: rhs.0,
                                                }));
                                                op = next_op;
                                            }
                                        } else {
                                            unreachable!() // Precedence would already have been checked by the returning function.
                                        }
                                    } else {
                                        // No more operators so let's put our lhs and rhs together.
                                        break Ok((
                                            Expr::Binary(Box::new(BinaryExpr {
                                                op: op,
                                                left: lhs,
                                                right: rhs.0,
                                            })),
                                            None,
                                        ));
                                    }
                                }
                            }
                        } else {
                            // this op is invalid if assignment not expected
                            if !expect_assignment {
                                break Err(Error::new(format!(
                                    "Invalid operator {:?} found, expected evaluable (line {})",
                                    op, line,
                                )));
                            } else {
                                // No need to do precedence on an assignment, so just grab RHS and return
                                let rhs =
                                    AST::read_binary_tree(lex, line, None, false, lowest_prec)?;
                                if let Some(op) = rhs.1 {
                                    break Err(Error::new(format!(
                                        "Stray operator {:?} in expression on line {}",
                                        op, line,
                                    )));
                                } else {
                                    break Ok((
                                        Expr::Binary(Box::new(BinaryExpr {
                                            op: op,
                                            left: lhs,
                                            right: rhs.0,
                                        })),
                                        None,
                                    ));
                                }
                            }
                        }
                    }
                } else {
                    unreachable!()
                }
            }
            _ => {
                if expect_assignment {
                    Err(Error::new(format!(
                        "Invalid token {:?} when expecting assignment operator on line {}",
                        next_token, line,
                    )))
                } else {
                    Ok((lhs, None))
                }
            }
        }
    }

    fn get_op_precedence(op: &Operator) -> Option<u8> {
        match op {
            Operator::Add => Some(4),
            Operator::Subtract => Some(4),
            Operator::Multiply => Some(5),
            Operator::Divide => Some(5),
            Operator::IntDivide => Some(5),
            Operator::BinaryAnd => Some(2),
            Operator::BinaryOr => Some(2),
            Operator::BinaryXor => Some(2),
            Operator::Assign => None,
            Operator::Not => None,
            Operator::LessThan => Some(1),
            Operator::GreaterThan => Some(1),
            Operator::AssignAdd => None,
            Operator::AssignSubtract => None,
            Operator::AssignMultiply => None,
            Operator::AssignDivide => None,
            Operator::AssignBinaryAnd => None,
            Operator::AssignBinaryOr => None,
            Operator::AssignBinaryXor => None,
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
            Operator::Period => None,
            Operator::ArrayAccessor => None,
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
