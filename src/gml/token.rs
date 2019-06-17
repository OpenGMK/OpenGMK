use std::fmt;

#[derive(Debug, PartialEq)]
pub enum Token<'a> {
    Identifier(&'a str),
    Keyword(Keyword),

    Operator(Operator),
    Separator(Separator),

    Real(f64),
    String(&'a str),

    InvalidChar(usize, u8),
}

#[derive(Debug, PartialEq)]
pub enum Keyword {
    Var,
    If,
    Else,
    With,
    Repeat,
    Do,
    Until,
    While,
    For,
    Switch,
    Case,
    Default,
    Break,
    Continue,
    Return,
    Exit,
}

#[derive(Debug, PartialEq)]
pub enum Operator {
    /// `+` Add or unary positive (redundant)
    Add,

    /// `-` Subtract or unary negative
    Subtract,

    /// `*` Multiply
    Multiply,

    /// `/` Divide
    Divide,

    /// `div` Integer Divide (Divide, Floor)
    IntDivide,

    /// `&` Binary AND
    BinaryAnd,

    /// `|` Binary OR
    BinaryOr,

    /// `^` Binary XOR
    BinaryXor,

    /// `=` Assign
    /// NOTE: This operator means Equal (`==`) if read in an expression
    Assign,

    /// `!` Boolean NOT
    Not,

    /// `<` Less Than (RHS)
    LessThan,

    /// `>` Greater Than (RHS)
    GreaterThan,

    /// `+=` Assignment Add
    AssignAdd,

    /// `-=` Assignment Subtract
    AssignSubtract,

    /// `*=` Assignment Multiply
    AssignMultiply,

    /// `/=` Assignment Divide
    AssignDivide,

    /// `&=` Assignment Binary AND
    AssignBinaryAnd,

    /// `|=` Assignment Binary OR
    AssignBinaryOr,

    /// `^=` Assignment Binary XOR
    AssignBinaryXor,

    /// `==` Equal
    Equal,

    /// `!=` Not Equal
    NotEqual,

    /// `<=` Less Than or Equal
    LessThanOrEqual,

    /// `>=` Greater Than or Equal
    GreaterThanOrEqual,

    /// `mod` Modulo
    Modulo,

    /// `&&` Boolean AND
    And,

    /// `||` Boolean OR
    Or,

    /// `^^` Boolean XOR
    Xor,

    /// `<<` Binary Shift Left
    BinaryShiftLeft,

    /// `>>` Binary Shift Right
    BinaryShiftRight,

    /// `~` Binary Complement (Unary)
    Complement,

    /// `.` Dereference Operator
    Deref,

    /// `[]` Array Accessor
    Index,
}

#[derive(Debug, PartialEq)]
pub enum Separator {
    /// `(` Parentheses Open
    ParenLeft,

    /// `)` Parentheses Close
    ParenRight,

    /// `{` Braces Open
    BraceLeft,

    /// `}` Braces Close
    BraceRight,

    /// `[` Bracket Open
    BracketLeft,

    /// `]` Bracket Close
    BracketRight,

    /// `;` Semicolon
    Semicolon,

    /// `:` Colon
    Colon,

    /// `,` Comma
    Comma,

    /// `.` Period
    Period,

    /// `then` (Legacy)
    Then,
}

impl fmt::Display for Keyword {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Keyword::Var => write!(f, "var"),
            Keyword::If => write!(f, "if"),
            Keyword::Else => write!(f, "else"),
            Keyword::With => write!(f, "with"),
            Keyword::Repeat => write!(f, "repeat"),
            Keyword::Do => write!(f, "do"),
            Keyword::Until => write!(f, "until"),
            Keyword::While => write!(f, "while"),
            Keyword::For => write!(f, "for"),
            Keyword::Switch => write!(f, "switch"),
            Keyword::Case => write!(f, "case"),
            Keyword::Default => write!(f, "default"),
            Keyword::Break => write!(f, "break"),
            Keyword::Continue => write!(f, "continue"),
            Keyword::Return => write!(f, "return"),
            Keyword::Exit => write!(f, "exit"),
        }
    }
}

impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Operator::Add => write!(f, "+"),
            Operator::Subtract => write!(f, "-"),
            Operator::Multiply => write!(f, "*"),
            Operator::Divide => write!(f, "/"),
            Operator::IntDivide => write!(f, "div"),
            Operator::BinaryAnd => write!(f, "&"),
            Operator::BinaryOr => write!(f, "|"),
            Operator::BinaryXor => write!(f, "^"),
            Operator::Assign => write!(f, "="),
            Operator::Not => write!(f, "!"),
            Operator::LessThan => write!(f, "<"),
            Operator::GreaterThan => write!(f, ">"),
            Operator::AssignAdd => write!(f, "+="),
            Operator::AssignSubtract => write!(f, "-="),
            Operator::AssignMultiply => write!(f, "*="),
            Operator::AssignDivide => write!(f, "/="),
            Operator::AssignBinaryAnd => write!(f, "&="),
            Operator::AssignBinaryOr => write!(f, "|="),
            Operator::AssignBinaryXor => write!(f, "^="),
            Operator::Equal => write!(f, "=="),
            Operator::NotEqual => write!(f, "!="),
            Operator::LessThanOrEqual => write!(f, "<="),
            Operator::GreaterThanOrEqual => write!(f, ">="),
            Operator::Modulo => write!(f, "mod"),
            Operator::And => write!(f, "&&"),
            Operator::Or => write!(f, "||"),
            Operator::Xor => write!(f, "^^"),
            Operator::BinaryShiftLeft => write!(f, "<<"),
            Operator::BinaryShiftRight => write!(f, ">>"),
            Operator::Complement => write!(f, "~"),
            Operator::Deref => write!(f, "."),
            Operator::Index => write!(f, "[]"),
        }
    }
}

impl fmt::Display for Separator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Separator::ParenLeft => write!(f, "("),
            Separator::ParenRight => write!(f, ")"),
            Separator::BraceLeft => write!(f, "{{"),
            Separator::BraceRight => write!(f, "}}"),
            Separator::BracketLeft => write!(f, "["),
            Separator::BracketRight => write!(f, "]"),
            Separator::Semicolon => write!(f, ";"),
            Separator::Colon => write!(f, ":"),
            Separator::Comma => write!(f, ","),
            Separator::Period => write!(f, "."),
            Separator::Then => write!(f, "then"),
        }
    }
}
