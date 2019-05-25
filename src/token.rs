#[derive(Debug)]
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

#[derive(Debug)]
pub enum Operator {}

#[derive(Debug)]
pub enum Separator {}

