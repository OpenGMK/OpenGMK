#[derive(Debug)]
pub enum Token<'a> {
    Identifier(&'a str),
    Keyword(Keyword),

    Operator(Operator),
    Separator(Separator),

    Real(f64),
    String(&'a str),
}

#[derive(Debug)]
pub enum Keyword {}

#[derive(Debug)]
pub enum Operator {}

#[derive(Debug)]
pub enum Separator {}

