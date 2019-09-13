pub mod ast;
// pub mod dnd;
pub mod lexer;
pub mod token;

#[derive(Debug)]
pub enum Value {
    Real(f64),
    String(String),
}
