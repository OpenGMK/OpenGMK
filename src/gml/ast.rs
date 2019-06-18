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
    pub default_case: Option<Expr<'a>>,
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

impl<'a> fmt::Display for Expr<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Expr::Literal(tok) => match tok {
                Token::Identifier(id) => write!(f, "{}", id),
                Token::Real(r) => write!(f, "{}", r),
                Token::String(s) => write!(f, "\"{}\"", s),
                _ => panic!("adam, fix this!"),
            },

            Expr::Unary(unary) => write!(f, "({} {})", unary.op, unary.child),
            Expr::Binary(binary) => write!(f, "({} {} {})", binary.op, binary.left, binary.right),

            Expr::DoUntil(dountil) => write!(f, "(do {} until {})", dountil.cond, dountil.body),
            Expr::For(for_ex) => write!(
                f,
                "(for ({}, {}, {}) {})",
                for_ex.start, for_ex.cond, for_ex.step, for_ex.body
            ),
            Expr::Function(call) => write!(
                f,
                "(@{} {})",
                call.name,
                call.params
                    .iter()
                    .fold(String::new(), |acc, fnname| acc + &format!("{} ", fnname))
                    .trim_end()
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
            Expr::Switch(switch) => write!(
                f,
                "(switch {} {})",
                switch.value,
                switch
                    .cases
                    .iter()
                    .fold(String::new(), |acc, (val, body)| acc + &format!("({} {}) ", val, body))
                    .trim_end()
            ),
            Expr::Var(var) => write!(
                f,
                "(var {})",
                var.vars
                    .iter()
                    .fold(String::new(), |acc, varname| acc + &format!("{} ", varname))
                    .trim_end()
            ),
            Expr::With(with) => write!(f, "(with {} {})", with.target, with.body),
            Expr::While(while_ex) => write!(f, "(while {} {})", while_ex.cond, while_ex.body),

            Expr::Nop => write!(f, ""),
        }
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

    fn read_line(lex: &mut Peekable<Lexer<'a>>, line: &mut usize) -> Result<Option<Expr<'a>>, Error> {
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
                                    return Err(Error::new(format!("Expected var name, found EOF (line {})", line)));
                                }
                            };
                            let var_name = match next_token {
                                Token::Identifier(id) => id,
                                _ => {
                                    return Err(Error::new(format!(
                                        "Invalid token, expected var name (line {}): {:?}",
                                        line, next_token,
                                    )));
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
                        )));
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
                                    Ok(None) => return Err(Error::new("Unclosed brace at EOF".to_string())),
                                    Err(e) => return Err(e),
                                },
                            }
                        }
                        Ok(Some(Expr::Group(inner_expressions)))
                    }

                    // An assignment may start with an open-parenthesis, eg: (1).x = 400;
                    Separator::ParenLeft => {
                        let binary_tree =
                            AST::read_binary_tree(lex, line, Some(Token::Separator(Separator::ParenLeft)), true, 0)?;
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
                        )));
                    }
                }
            }

            _ => {
                return Err(Error::new(format!(
                    "Invalid token at beginning of expression on line {}: {:?}",
                    line, token
                )));
            }
        }
    }

    fn read_binary_tree(
        lex: &mut Peekable<Lexer<'a>>,
        line: &mut usize,
        first_token: Option<Token<'a>>, // Sometimes we've already parsed the first token, so it should be put here.
        expect_assignment: bool,        // Do we expect the first op to be an assignment?
        lowest_prec: u8,                // We are not allowed to go below this operator precedence in this tree.
                                        // If we do, we'll return the next op.
    ) -> Result<(Expr<'a>, Option<Operator>), Error> {
        // Get the very first token in this exp value
        let mut lhs = match if first_token.is_some() { first_token } else { lex.next() } {
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
                )));
            }
            None => {
                return Err(Error::new(format!(
                    "Found EOF unexpectedly while reading binary tree (line {})",
                    line
                )));
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
                                )));
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
                        op: Operator::Index,
                        left: lhs,
                        right: Expr::Group(dimensions),
                    }));
                }

                Some(Token::Separator(ref sep)) if *sep == Separator::Period => {
                    lex.next();
                    lhs = Expr::Binary(Box::new(BinaryExpr {
                        op: Operator::Deref,
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
                        // Here, we get the precedence of the operator we found.
                        // If this returns None, it's probably an assignment,
                        // so we use that in conjunction with an if-let to check its validity.
                        if let Some(precedence) = AST::get_op_precedence(&op) {
                            // this op is invalid if an assignment is expected
                            if expect_assignment {
                                break Err(Error::new(format!(
                                    "Invalid operator {:?} found, expected assignment (line {})",
                                    op, line,
                                )));
                            } else {
                                // If this op has lower prec than we're allowed to read, we have to return it here.
                                if precedence < lowest_prec {
                                    break Ok((lhs, Some(op)));
                                } else {
                                    // We're allowed to use the next operator. Let's read an RHS to put on after it.
                                    // You might be thinking "precedence + 1" is counter-intuitive -
                                    // "precedence" would make more sense, right?
                                    // Well, the difference is left-to-right vs right-to-left construction.
                                    // This way, 1/2/3 is correctly built as (1/2)/3 rather than 1/(2/3).
                                    let rhs = AST::read_binary_tree(lex, line, None, false, precedence + 1)?;
                                    if let Some(next_op) = rhs.1 {
                                        // There's another operator even after the RHS.
                                        if let Some(next_prec) = AST::get_op_precedence(&next_op) {
                                            if next_prec < lowest_prec {
                                                // This next op is lower than we're allowed to go, so we must return it
                                                break Ok((
                                                    Expr::Binary(Box::new(BinaryExpr {
                                                        op: op,
                                                        left: lhs,
                                                        right: rhs.0,
                                                    })),
                                                    Some(next_op),
                                                ));
                                            } else {
                                                // Update LHS by sticking RHS onto it,
                                                // set op to the new operator, and go round again.
                                                lhs = Expr::Binary(Box::new(BinaryExpr {
                                                    op: op,
                                                    left: lhs,
                                                    right: rhs.0,
                                                }));
                                                op = next_op;
                                            }
                                        } else {
                                            // Precedence would already have been checked by the returning function.
                                            unreachable!()
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
                            // this op is invalid if assignment not expected, OR if it's a unary operator
                            // (those have no precedence so they pass the previous test.)
                            if !expect_assignment || op == Operator::Not || op == Operator::Complement {
                                break Err(Error::new(format!(
                                    "Invalid operator {:?} found, expected evaluable (line {})",
                                    op, line,
                                )));
                            } else {
                                // No need to do precedence on an assignment, so just grab RHS and return
                                let rhs = AST::read_binary_tree(lex, line, None, false, lowest_prec)?;
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
            Operator::Deref => None,
            Operator::Index => None,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nothing() {
        // Empty string
        assert_ast("", None)
    }

    #[test]
    fn test_assignment_op_assign() {
        assert_ast(
            // Simple assignment - Assign
            "a = 1",
            Some(vec![Expr::Binary(Box::new(BinaryExpr {
                op: Operator::Assign,
                left: Expr::Literal(Token::Identifier("a")),
                right: Expr::Literal(Token::Real(1.0)),
            }))]),
        )
    }

    #[test]
    fn test_assignment_op_add() {
        assert_ast(
            // Simple assignment - AssignAdd
            "b += 2",
            Some(vec![Expr::Binary(Box::new(BinaryExpr {
                op: Operator::AssignAdd,
                left: Expr::Literal(Token::Identifier("b")),
                right: Expr::Literal(Token::Real(2.0)),
            }))]),
        )
    }

    #[test]
    fn test_assignment_op_subtract() {
        assert_ast(
            // Simple assignment - AssignSubtract
            "c -= 3",
            Some(vec![Expr::Binary(Box::new(BinaryExpr {
                op: Operator::AssignSubtract,
                left: Expr::Literal(Token::Identifier("c")),
                right: Expr::Literal(Token::Real(3.0)),
            }))]),
        )
    }

    #[test]
    fn test_assignment_op_multiply() {
        assert_ast(
            // Simple assignment - AssignMultiply
            "d *= 4",
            Some(vec![Expr::Binary(Box::new(BinaryExpr {
                op: Operator::AssignMultiply,
                left: Expr::Literal(Token::Identifier("d")),
                right: Expr::Literal(Token::Real(4.0)),
            }))]),
        )
    }

    #[test]
    fn test_assignment_op_divide() {
        assert_ast(
            // Simple assignment - AssignDivide
            "e /= 5",
            Some(vec![Expr::Binary(Box::new(BinaryExpr {
                op: Operator::AssignDivide,
                left: Expr::Literal(Token::Identifier("e")),
                right: Expr::Literal(Token::Real(5.0)),
            }))]),
        )
    }

    #[test]
    fn test_assignment_op_and() {
        assert_ast(
            // Simple assignment - AssignBinaryAnd
            "f &= 6",
            Some(vec![Expr::Binary(Box::new(BinaryExpr {
                op: Operator::AssignBinaryAnd,
                left: Expr::Literal(Token::Identifier("f")),
                right: Expr::Literal(Token::Real(6.0)),
            }))]),
        )
    }

    #[test]
    fn test_assignment_op_or() {
        assert_ast(
            // Simple assignment - AssignBinaryOr
            "g |= 7",
            Some(vec![Expr::Binary(Box::new(BinaryExpr {
                op: Operator::AssignBinaryOr,
                left: Expr::Literal(Token::Identifier("g")),
                right: Expr::Literal(Token::Real(7.0)),
            }))]),
        )
    }

    #[test]
    fn test_assignment_op_xor() {
        assert_ast(
            // Simple assignment - AssignBinaryXor
            "h ^= 8",
            Some(vec![Expr::Binary(Box::new(BinaryExpr {
                op: Operator::AssignBinaryXor,
                left: Expr::Literal(Token::Identifier("h")),
                right: Expr::Literal(Token::Real(8.0)),
            }))]),
        )
    }

    #[test]
    #[should_panic]
    fn test_assignment_op_invalid() {
        // Assignment syntax - Multiply - should fail
        // Note: chose "Multiply" specifically as it cannot be unary, unlike Add or Subtract
        assert_ast("i * 9", None);
    }

    #[test]
    #[should_panic]
    fn test_assignment_op_not() {
        // Assignment syntax - Not - should fail
        assert_ast("j ! 10", None);
    }

    #[test]
    #[should_panic]
    fn test_assignment_op_complement() {
        // Assignment syntax - Complement - should fail
        assert_ast("k ~ 11", None);
    }

    #[test]
    fn test_assignment_lhs() {
        assert_ast(
            // Assignment with deref and index on lhs
            "a.b[c] += d;",
            Some(vec![Expr::Binary(Box::new(BinaryExpr {
                op: Operator::AssignAdd,
                left: Expr::Binary(Box::new(BinaryExpr {
                    op: Operator::Index,
                    left: Expr::Binary(Box::new(BinaryExpr {
                        op: Operator::Deref,
                        left: Expr::Literal(Token::Identifier("a")),
                        right: Expr::Literal(Token::Identifier("b")),
                    })),
                    right: Expr::Group(vec![Expr::Literal(Token::Identifier("c"))]),
                })),
                right: Expr::Literal(Token::Identifier("d")),
            }))]),
        );
    }

    #[test]
    fn test_assignment_2d_index() {
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
                                    left: Expr::Literal(Token::Identifier("a")),
                                    right: Expr::Literal(Token::Identifier("b")),
                                })),
                                right: Expr::Group(vec![Expr::Literal(Token::Identifier("c"))]),
                            })),
                            right: Expr::Literal(Token::Identifier("d")),
                        })),
                        right: Expr::Literal(Token::Identifier("e")),
                    })),
                    right: Expr::Group(vec![
                        Expr::Literal(Token::Identifier("f")),
                        Expr::Literal(Token::Identifier("g")),
                    ]),
                })),
                right: Expr::Binary(Box::new(BinaryExpr {
                    op: Operator::Deref,
                    left: Expr::Binary(Box::new(BinaryExpr {
                        op: Operator::Index,
                        left: Expr::Literal(Token::Identifier("h")),
                        right: Expr::Group(vec![
                            Expr::Literal(Token::Identifier("i")),
                            Expr::Literal(Token::Identifier("j")),
                        ]),
                    })),
                    right: Expr::Literal(Token::Identifier("k")),
                })),
            }))]),
        );
    }

    #[test]
    fn test_assignment_lhs_expression() {
        assert_ast(
            // Assignment whose LHS is an expression-deref
            "(a + 1).x = 400;",
            Some(vec![Expr::Binary(Box::new(BinaryExpr {
                op: Operator::Assign,
                left: Expr::Binary(Box::new(BinaryExpr {
                    op: Operator::Deref,
                    left: Expr::Binary(Box::new(BinaryExpr {
                        op: Operator::Add,
                        left: Expr::Literal(Token::Identifier("a")),
                        right: Expr::Literal(Token::Real(1.0)),
                    })),
                    right: Expr::Literal(Token::Identifier("x")),
                })),
                right: Expr::Literal(Token::Real(400.0)),
            }))]),
        );
    }

    #[test]
    fn test_assignment_assign_equal() {
        assert_ast(
            // Differentiation between usages of '=' - simple
            "a=b=c",
            Some(vec![Expr::Binary(Box::new(BinaryExpr {
                op: Operator::Assign,
                left: Expr::Literal(Token::Identifier("a")),
                right: Expr::Binary(Box::new(BinaryExpr {
                    op: Operator::Equal,
                    left: Expr::Literal(Token::Identifier("b")),
                    right: Expr::Literal(Token::Identifier("c")),
                })),
            }))]),
        );
    }

    #[test]
    fn test_assignment_assign_equal_complex() {
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
                            left: Expr::Literal(Token::Identifier("a")),
                            right: Expr::Literal(Token::Identifier("b")),
                        })),
                        right: Expr::Literal(Token::Identifier("c")),
                    })),
                    right: Expr::Group(vec![Expr::Binary(Box::new(BinaryExpr {
                        op: Operator::Equal,
                        left: Expr::Literal(Token::Identifier("d")),
                        right: Expr::Literal(Token::Identifier("e")),
                    }))]),
                })),
                right: Expr::Binary(Box::new(BinaryExpr {
                    op: Operator::Equal,
                    left: Expr::Binary(Box::new(BinaryExpr {
                        op: Operator::Index,
                        left: Expr::Literal(Token::Identifier("f")),
                        right: Expr::Group(vec![Expr::Binary(Box::new(BinaryExpr {
                            op: Operator::Equal,
                            left: Expr::Literal(Token::Identifier("g")),
                            right: Expr::Literal(Token::Identifier("h")),
                        }))]),
                    })),
                    right: Expr::Literal(Token::Identifier("i")),
                })),
            }))]),
        );
    }

    #[test]
    fn test_btree_unary_positive() {
        assert_ast(
            // Binary tree format - unary operator - positive
            "a=+1",
            Some(vec![Expr::Binary(Box::new(BinaryExpr {
                op: Operator::Assign,
                left: Expr::Literal(Token::Identifier("a")),
                right: Expr::Unary(Box::new(UnaryExpr {
                    op: Operator::Add,
                    child: Expr::Literal(Token::Real(1.0)),
                })),
            }))]),
        )
    }

    #[test]
    fn test_btree_unary_subtract() {
        assert_ast(
            // Binary tree format - unary operator - negative
            "a=-1",
            Some(vec![Expr::Binary(Box::new(BinaryExpr {
                op: Operator::Assign,
                left: Expr::Literal(Token::Identifier("a")),
                right: Expr::Unary(Box::new(UnaryExpr {
                    op: Operator::Subtract,
                    child: Expr::Literal(Token::Real(1.0)),
                })),
            }))]),
        )
    }

    #[test]
    fn test_btree_unary_complement() {
        assert_ast(
            // Binary tree format - unary operator - complement
            "a=~1",
            Some(vec![Expr::Binary(Box::new(BinaryExpr {
                op: Operator::Assign,
                left: Expr::Literal(Token::Identifier("a")),
                right: Expr::Unary(Box::new(UnaryExpr {
                    op: Operator::Complement,
                    child: Expr::Literal(Token::Real(1.0)),
                })),
            }))]),
        )
    }

    #[test]
    fn test_btree_unary_not() {
        assert_ast(
            // Binary tree format - unary operator - negative
            "a=!1",
            Some(vec![Expr::Binary(Box::new(BinaryExpr {
                op: Operator::Assign,
                left: Expr::Literal(Token::Identifier("a")),
                right: Expr::Unary(Box::new(UnaryExpr {
                    op: Operator::Not,
                    child: Expr::Literal(Token::Real(1.0)),
                })),
            }))]),
        )
    }

    #[test]
    fn test_btree_unary_syntax() {
        assert_ast(
            // Binary tree format - unary operators - syntax parse test
            "a = 1+!~-b.c[+d]-2--3", // (- (- (+ 1 2) 3) 4)
            Some(vec![Expr::Binary(Box::new(BinaryExpr {
                op: Operator::Assign,
                left: Expr::Literal(Token::Identifier("a")),
                right: Expr::Binary(Box::new(BinaryExpr {
                    op: Operator::Subtract,
                    left: Expr::Binary(Box::new(BinaryExpr {
                        op: Operator::Subtract,
                        left: Expr::Binary(Box::new(BinaryExpr {
                            op: Operator::Add,
                            left: Expr::Literal(Token::Real(1.0)),
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
                                                left: Expr::Literal(Token::Identifier("b")),
                                                right: Expr::Literal(Token::Identifier("c")),
                                            })),
                                            right: Expr::Group(vec![Expr::Unary(Box::new(UnaryExpr {
                                                op: Operator::Add,
                                                child: Expr::Literal(Token::Identifier("d")),
                                            }))]),
                                        })),
                                    })),
                                })),
                            })),
                        })),
                        right: Expr::Literal(Token::Real(2.0)),
                    })),
                    right: Expr::Unary(Box::new(UnaryExpr {
                        op: Operator::Subtract,
                        child: Expr::Literal(Token::Real(3.0)),
                    })),
                })),
            }))]),
        )
    }

    //#[test]
    //fn test_btree_unary_grouping() {
    // TODO: "a = ~(b + 1)"
    //}

    fn assert_ast(input: &str, expected_output: Option<Vec<Expr>>) {
        match AST::new(input) {
            Ok(ast) => {
                if let Some(e) = expected_output {
                    assert_eq!(ast.expressions, e);
                }
            }
            Err(e) => panic!("AST test encountered error: '{}' for input: {}", e, input),
        }
    }
}
