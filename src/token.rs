pub enum Token<'a> {
    Identifier(&'a str),
    Keyword(Keyword),

    Operator(Operator),
    Separator(Separator),

    Real(f64),
    String(&'a str),
}

pub enum Keyword {}
pub enum Operator {}
pub enum Separator {}

