use std::{
    ops::{
        Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Div, DivAssign, Mul, MulAssign,
        Neg, Not, Rem, RemAssign, Shl, ShlAssign, Shr, ShrAssign, Sub, SubAssign,
    },
    rc::Rc,
};

#[derive(Debug)]
pub enum Value {
    Real(f64),
    Str(Rc<str>),
}

pub(self) use Value::*;

impl Value {
    /// GML-like comparison, fails if self and other are different types.
    fn almost_equals(&self, other: &Self) -> bool {
        match (self, other) {
            (Real(a), Real(b)) => (a - b).abs() <= 1e-14,
            (Str(a), Str(b)) => a.as_ref() == b.as_ref(),
            (Real(_), Str(_)) => gml_panic!("cannot compare arguments (real == string)"),
            (Str(_), Real(_)) => gml_panic!("cannot compare arguments (string == real)"),
        }
    }
}

impl Add for Value {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Real(lhs), Real(rhs)) => Real(lhs + rhs),
            (Str(lhs), Str(rhs)) => Str({
                let mut string = String::with_capacity(lhs.len() + rhs.len());
                string.push_str(lhs.as_ref());
                string.push_str(rhs.as_ref());
                Rc::from(string)
            }),
            (Real(_), Str(_)) => gml_panic!("invalid arguments to + (real + string)"),
            (Str(_), Real(_)) => gml_panic!("invalid arguments to + (string + real)"),
        }
    }
}

impl AddAssign for Value {
    fn add_assign(&mut self, rhs: Self) {
        match (self, rhs) {
            (Real(lhs), Real(rhs)) => *lhs += rhs,
            (Str(lhs), Str(ref rhs)) => {
                let mut string = String::with_capacity(lhs.len() + rhs.len());
                string.push_str(lhs.as_ref());
                string.push_str(rhs.as_ref());
                *lhs = string.into();
            },
            (Real(_), Str(_)) => gml_panic!("invalid arguments to += (real += string)"),
            (Str(_), Real(_)) => gml_panic!("invalid arguments to += (string += real)"),
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
        assert!((a + b).almost_equals(&Real(0.30000000000000004)));

        let c = Str("Hello, ".to_string().into());
        let d = Str("world!".to_string().into());
        assert!((c + d).almost_equals(&Str("Hello, world!".to_string().into())));
    }

    #[test]
    #[should_panic]
    fn op_add_invalid() {
        let a = Real(0.1);
        let b = Str("owo".to_string().into());
        let _ = a + b;
    }
}
