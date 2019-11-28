pub mod ast;
pub mod lexer;
pub mod rand;
pub mod token;

#[derive(Debug)]
pub enum Value {
    Real(f64),
    String(String),
}
