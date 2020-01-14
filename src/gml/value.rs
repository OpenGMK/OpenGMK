
use std::{rc::Rc};

#[derive(Debug)]
pub enum Value {
    Real(f64),
    String(Rc<str>),
}
