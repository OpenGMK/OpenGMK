pub mod ast;
pub mod compiler;
pub mod lexer;
pub mod rand;
pub mod token;

pub use compiler::Compiler;

pub const SELF: i32 = -1;
pub const OTHER: i32 = -2;
pub const NOONE: i32 = -3;
pub const ALL: i32 = -4;
pub const GLOBAL: i32 = -5;
pub const LOCAL: i32 = -7;

pub mod ev {
    pub const CREATE: usize = 0;
    pub const DESTROY: usize = 1;
    pub const ALARMS: usize = 2;
    pub const STEP: usize = 3;
    pub const COLLISION: usize = 4;
    pub const KEYBOARD: usize = 5;
    pub const MOUSE: usize = 6;
    pub const OTHER: usize = 7;
    pub const DRAW: usize = 8;
    pub const KEYPRESS: usize = 9;
    pub const KEYRELEASE: usize = 10;
    pub const TRIGGER: usize = 11;
}

#[derive(Debug)]
pub enum Value {
    Real(f64),
    String(String),
}

#[derive(Debug)]
pub enum Instruction {
    // TODO: GML runtime instructions
    InterpretationError { error: String },
}
