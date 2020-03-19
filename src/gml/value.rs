use crate::gml;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum Value {
    Real(f64),
    Str(Rc<str>),
}

// How many times do you think I want to write `Value::` in the `value` module?
pub(self) use Value::*;

macro_rules! gml_cmp_impl {

    ($($v: vis $fname: ident aka $op_variant: ident: real: $r_cond: expr, string: $s_cond: expr)*) => {
        $(
            $v fn $fname(self, rhs: Self) -> gml::Result<Self> {
                let freal: fn(f64, f64) -> bool = $r_cond;
                let fstr: fn(&str, &str) -> bool = $s_cond;
                if match (self, rhs) {
                    (Value::Real(a), Value::Real(b)) => freal(a, b),
                    (Value::Str(a), Value::Str(b)) => fstr(a.as_ref(), b.as_ref()),
                    (a, b) => return invalid_op!($op_variant, a, b),
                } {
                    Ok(Real(super::TRUE))
                } else {
                    Ok(Real(super::FALSE))
                }
            }
        )*
    };
}

macro_rules! invalid_op {
    ($op: ident, $value: expr) => {
        Err(gml::Error::InvalidOperandsUnary(gml::compiler::token::Operator::$op, $value))
    };
    ($op: ident, $left: expr, $right: expr) => {
        Err(gml::Error::InvalidOperandsBinary(gml::compiler::token::Operator::$op, $left, $right))
    };
}

impl Value {
    // All the GML comparison operators (which return Value not bool).
    #[rustfmt::skip]
    gml_cmp_impl! {
        pub gml_eq aka Equal:
            real: |r1, r2| (r1 - r2).abs() <= 1e-14,
            string: |s1, s2| s1 == s2

        pub gml_ne aka NotEqual:
            real: |r1, r2| (r1 - r2).abs() > 1e-14,
            string: |s1, s2| s1 != s2

        pub gml_lt aka LessThan:
            real: |r1, r2| r1 < r2,
            string: |s1, s2| s1 < s2

        pub gml_lte aka LessThanOrEqual:
            real: |r1, r2| r1 < r2 || (r1 - r2).abs() <= 1e-14,
            string: |s1, s2| s1 <= s2

        pub gml_gt aka GreaterThan:
            real: |r1, r2| r1 > r2,
            string: |s1, s2| s1 > s2

        pub gml_gte aka GreaterThanOrEqual:
            real: |r1, r2| r1 > r2 || (r1 - r2).abs() <= 1e-14,
            string: |s1, s2| s1 >= s2
    }

    /// GML-like comparison, fails if self and other are different types.
    fn almost_equals(&self, other: &Self) -> bool {
        match (self, other) {
            (Real(a), Real(b)) => (a - b).abs() <= 1e-14,
            (Str(a), Str(b)) => a.as_ref() == b.as_ref(),
            _ => false,
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

    /// Rounds the value to an i32. This is done very commonly by the GM8 runner.
    pub fn round(&self) -> i32 {
        match &self {
            Real(f) => Value::ieee_round(*f),
            Str(_) => 0,
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

    pub fn is_true(&self) -> bool {
        match self {
            Real(f) => *f >= 0.5, // What a confusing line.
            Str(_) => false,
        }
    }

    /// Unary bit complement.
    pub fn complement(self) -> gml::Result<Self> {
        match self {
            Real(val) => Ok(Real(!(Self::ieee_round(val) as i32) as f64)),
            _ => invalid_op!(Complement, self),
        }
    }

    /// GML operator 'div' which gives the whole number of times RHS can go into LHS. In other words floor(lhs/rhs)
    pub fn intdiv(self, rhs: Self) -> gml::Result<Self> {
        match (self, rhs) {
            (Real(lhs), Real(rhs)) => Ok(Real((lhs / rhs).floor())),
            (x, y) => invalid_op!(IntDivide, x, y),
        }
    }

    /// GML && operator
    pub fn bool_and(self, rhs: Self) -> gml::Result<Self> {
        Ok(if self.is_true() || rhs.is_true() { Real(1.0) } else { Real(0.0) })
    }

    /// GML || operator
    pub fn bool_or(self, rhs: Self) -> gml::Result<Self> {
        Ok(if self.is_true() && rhs.is_true() { Real(1.0) } else { Real(0.0) })
    }

    /// GML ^^ operator
    pub fn bool_xor(self, rhs: Self) -> gml::Result<Self> {
        Ok(if self.is_true() != rhs.is_true() { Real(1.0) } else { Real(0.0) })
    }

    pub fn add(self, rhs: Self) -> gml::Result<Self> {
        match (self, rhs) {
            (Real(lhs), Real(rhs)) => Ok(Real(lhs + rhs)),
            (Str(lhs), Str(rhs)) => Ok(Str({
                let mut string = String::with_capacity(lhs.len() + rhs.len());
                string.push_str(lhs.as_ref());
                string.push_str(rhs.as_ref());
                Rc::from(string)
            })),
            (x, y) => invalid_op!(Add, x, y),
        }
    }

    pub fn add_assign(&mut self, rhs: Self) -> gml::Result<()> {
        match (self, rhs) {
            (Real(lhs), Real(rhs)) => Ok(*lhs += rhs),
            (Str(lhs), Str(ref rhs)) => {
                // TODO: a
                let mut string = String::with_capacity(lhs.len() + rhs.len());
                string.push_str(lhs.as_ref());
                string.push_str(rhs.as_ref());
                *lhs = string.into();
                Ok(())
            },
            (x, y) => invalid_op!(AssignAdd, x.clone(), y),
        }
    }

    pub fn bitand(self, rhs: Self) -> gml::Result<Self> {
        match (self, rhs) {
            (Real(lhs), Real(rhs)) => Ok(Real((Self::ieee_round(lhs) as i32 & Self::ieee_round(rhs) as i32) as _)),
            (x, y) => invalid_op!(BitwiseAnd, x, y),
        }
    }

    pub fn bitand_assign(&mut self, rhs: Self) -> gml::Result<()> {
        match (self, rhs) {
            (Real(lhs), Real(rhs)) => Ok(*lhs = (Self::ieee_round(*lhs) as i32 & Self::ieee_round(rhs) as i32) as _),
            (x, y) => invalid_op!(AssignBitwiseAnd, x.clone(), y),
        }
    }

    pub fn bitor(self, rhs: Self) -> gml::Result<Self> {
        match (self, rhs) {
            (Real(lhs), Real(rhs)) => Ok(Real((Self::ieee_round(lhs) as i32 | Self::ieee_round(rhs) as i32) as _)),
            (x, y) => invalid_op!(BitwiseOr, x, y),
        }
    }

    pub fn bitor_assign(&mut self, rhs: Self) -> gml::Result<()> {
        match (self, rhs) {
            (Real(lhs), Real(rhs)) => Ok(*lhs = (Self::ieee_round(*lhs) as i32 | Self::ieee_round(rhs) as i32) as _),
            (x, y) => invalid_op!(AssignBitwiseOr, x.clone(), y),
        }
    }

    pub fn bitxor(self, rhs: Self) -> gml::Result<Self> {
        match (self, rhs) {
            (Real(lhs), Real(rhs)) => Ok(Real((Self::ieee_round(lhs) as i32 ^ Self::ieee_round(rhs) as i32) as _)),
            (x, y) => invalid_op!(BitwiseXor, x, y),
        }
    }

    pub fn bitxor_assign(&mut self, rhs: Self) -> gml::Result<()> {
        match (self, rhs) {
            (Real(lhs), Real(rhs)) => Ok(*lhs = (Self::ieee_round(*lhs) as i32 ^ Self::ieee_round(rhs) as i32) as _),
            (x, y) => invalid_op!(AssignBitwiseXor, x.clone(), y),
        }
    }

    pub fn div(self, rhs: Self) -> gml::Result<Self> {
        match (self, rhs) {
            (Real(lhs), Real(rhs)) => Ok(Real(lhs / rhs)),
            (x, y) => invalid_op!(Divide, x, y),
        }
    }

    pub fn div_assign(&mut self, rhs: Self) -> gml::Result<()> {
        match (self, rhs) {
            (Real(lhs), Real(rhs)) => Ok(*lhs /= rhs),
            (x, y) => invalid_op!(AssignDivide, x.clone(), y),
        }
    }

    pub fn modulo(self, rhs: Self) -> gml::Result<Self> {
        match (self, rhs) {
            (Real(lhs), Real(rhs)) => Ok(Real(lhs % rhs)),
            (x, y) => invalid_op!(Modulo, x, y),
        }
    }

    pub fn mul(self, rhs: Self) -> gml::Result<Self> {
        match (self, rhs) {
            (Real(lhs), Real(rhs)) => Ok(Real(lhs * rhs)),
            (x, y) => invalid_op!(Multiply, x, y),
        }
    }

    pub fn mul_assign(&mut self, rhs: Self) -> gml::Result<()> {
        match (self, rhs) {
            (Real(lhs), Real(rhs)) => Ok(*lhs *= rhs),
            (x, y) => invalid_op!(AssignMultiply, x.clone(), y),
        }
    }

    pub fn neg(self) -> gml::Result<Self> {
        match self {
            Real(f) => Ok(Real(-f)),
            Str(_) => invalid_op!(Subtract, self),
        }
    }

    pub fn not(self) -> gml::Result<Self> {
        match self {
            Real(_) => Ok(Real((!self.is_true()) as i8 as f64)),
            Str(_) => invalid_op!(Not, self),
        }
    }

    pub fn shl(self, rhs: Self) -> gml::Result<Self> {
        match (self, rhs) {
            (Real(lhs), Real(rhs)) => Ok(Real(((Self::ieee_round(lhs) as i32) << Self::ieee_round(rhs) as i32) as _)),
            (x, y) => invalid_op!(BinaryShiftLeft, x, y),
        }
    }

    pub fn shr(self, rhs: Self) -> gml::Result<Self> {
        match (self, rhs) {
            (Real(lhs), Real(rhs)) => Ok(Real((Self::ieee_round(lhs) as i32 >> Self::ieee_round(rhs) as i32) as _)),
            (x, y) => invalid_op!(BinaryShiftRight, x, y),
        }
    }

    pub fn sub(self, rhs: Self) -> gml::Result<Self> {
        match (self, rhs) {
            (Real(lhs), Real(rhs)) => Ok(Real(lhs - rhs)),
            (x, y) => invalid_op!(Subtract, x, y),
        }
    }

    pub fn sub_assign(&mut self, rhs: Self) -> gml::Result<()> {
        match (self, rhs) {
            (Real(lhs), Real(rhs)) => Ok(*lhs -= rhs),
            (x, y) => invalid_op!(AssignSubtract, x.clone(), y),
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
        assert!((a.add(b).unwrap()).almost_equals(&Real(0.30000000000000004)));

        let c = Str("Hello, ".to_string().into());
        let d = Str("world!".to_string().into());
        assert!((c.add(d).unwrap()).almost_equals(&Str("Hello, world!".to_string().into())));
    }

    #[test]
    #[should_panic]
    fn op_add_invalid() {
        let a = Real(0.1);
        let b = Str("owo".to_string().into());
        let _ = a.add(b).unwrap();
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
