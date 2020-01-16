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

// How many times do you think I want to write `Value::` in the `value` module?
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
    
    /// The default way to round as defined by IEEE 754 - nearest, ties to even. Fuck yourself.
    fn ieee_round(real: f64) -> i32 {
        let floor = real.floor();
        let floori = floor as i32;
        let diff = real - floor;
        if diff < 0.5 {
            floori
        } else if diff > 0.5 {
            floori + 1
        } else {
            floori + (floori & 1)
        }
    }
    
    /// Formats the value as a number or a string with quotes around it so you can see that it is.
    /// Used in generating error messages.
    fn log_fmt(&self) -> String {
        match self {
            Real(real) => real.to_string(),
            Str(string) => format!("\"{}\"", string),
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

    #[test]
    fn ieee_round() {
        assert_eq!(Value::ieee_round(-3.5), -4);
        assert_eq!(Value::ieee_round(-2.5), -2);
        assert_eq!(Value::ieee_round(-1.5), -2);
        assert_eq!(Value::ieee_round(-0.5), 0);
        assert_eq!(Value::ieee_round(0.5), 0);
        assert_eq!(Value::ieee_round(1.5), 2);
        assert_eq!(Value::ieee_round(2.5), 2);
        assert_eq!(Value::ieee_round(3.5), 4);
        for i in 0..1000 {
            assert_eq!(Value::ieee_round(i as f64 + 0.5) % 2, 0);
        }
    }
}
