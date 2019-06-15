#[derive(Debug, PartialEq)]
pub enum Token<'a> {
    Identifier(&'a str),
    Keyword(Keyword),

    Operator(Operator),
    Separator(Separator),

    Real(f64),
    String(&'a str),

    Comment(&'a str),
    LineHint(usize),
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

    /// `[]` Array Indexing (AST)
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
