pub mod dnd;
pub mod lexer;
pub mod token;
pub mod ast;

#[derive(Debug)]
pub enum Value {
    Real(f64),
    String(String),
}
