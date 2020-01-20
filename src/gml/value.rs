use std::{
    ops::{
        Add,
        Neg,
        Not,
    },
    rc::Rc
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

    fn is_true(&self) -> bool {
        match self {
            Real(f) => *f >= 0.5, // What a confusing line.
            Str(_) => false,
        }
    }

    pub fn complement(self) -> Self {
        // TODO: no FISTP round yet?
        unimplemented!()
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
            (Real(_), Str(_)) => gml_panic!("invalid operands to add (real + string)"),
            (Str(_), Real(_)) => gml_panic!("invalid operands to add (string + real)"),
        }
    }
}

impl Neg for Value {
    type Output = Self;

    fn neg(self) -> Self::Output {
        match self {
            Real(f) => Real(-f),
            Str(_) => gml_panic!("invalid operand to neg"),
        }
    }
}

impl Not for Value {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Real(_) => Real(self.is_true() as i8 as f64),
            Str(_) => gml_panic!("invalid operand to neg"),
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
