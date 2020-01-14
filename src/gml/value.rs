use std::{ops::Add, rc::Rc};

#[derive(Debug)]
pub enum Value {
    Real(f64),
    Str(Rc<str>),
}

pub(self) use Value::*;

impl Value {
    /// GML-like comparison, fails if self and other are different types.
    fn almost_equals(&self, other: &Self) -> Result<bool, &'static str> {
        match (self, other) {
            (Real(a), Real(b)) => Ok((a - b).abs() <= 1e-14),
            (Str(a), Str(b)) => Ok(a.as_ref() == b.as_ref()),
            (Real(_), Str(_)) => Err("cannot compare arguments (real == string)"),
            (Str(_), Real(_)) => Err("cannot compare arguments (string == real)"),
        }
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn op_add() {
        let a = Real(0.1);
        let b = Real(0.2);
        assert!(
            (a + b)
                .unwrap()
                .almost_equals(&Real(0.30000000000000004))
                .unwrap_or(false)
        );

        let c = Str("Hello, ".to_string().into());
        let d = Str("world!".to_string().into());
        assert!(
            (c + d)
                .unwrap()
                .almost_equals(&Str("Hello, world!".to_string().into()))
                .unwrap_or(false)
        );

        assert!((Real(0.1) + Str("hi".to_string().into())).is_err());
        assert!((Str("hi".to_string().into()) + Real(0.1)).is_err());
    }
}
