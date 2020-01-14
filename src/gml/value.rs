use std::{ops::Add, rc::Rc};

#[derive(Debug)]
pub enum Value {
    Real(f64),
    Str(Rc<str>),
}

pub(self) use Value::*;

impl Add for Value {
    type Output = Result<Self, &'static str>;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Real(lhs), Real(rhs)) => Ok(Real(lhs + rhs)),
            (Str(lhs), Str(rhs)) => Ok(Str({
                let mut string = String::with_capacity(lhs.len() + rhs.len());
                string.push_str(lhs.as_ref());
                string.push_str(rhs.as_ref());
                Rc::from(string)
            })),
            (Real(_), Str(_)) => Err("invalid operands to add (real + string)"),
            (Str(_), Real(_)) => Err("invalid operands to add (string + real)"),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Real(a), Real(b)) => a == b,
            (Str(a), Str(b)) => a.as_ref() == b.as_ref(),
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn op_add() {
        let a = Real(0.1);
        let b = Real(0.2);
        assert_eq!(a + b, Ok(Real(0.30000000000000004)));

        let c = Str("Hello, ".to_string().into());
        let d = Str("world!".to_string().into());
        assert_eq!(c + d, Ok(Str("Hello, world!".to_string().into())));

        assert!((Real(0.1) + Str("hi".to_string().into())).is_err());
        assert!((Str("hi".to_string().into()) + Real(0.1)).is_err());
    }
}
