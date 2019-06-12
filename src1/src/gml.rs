pub mod dnd;

#[derive(Debug)]
pub enum Value {
    Real(f64),
    String(String),
}
