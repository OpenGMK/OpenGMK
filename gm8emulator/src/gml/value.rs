use crate::{game::external::dll, gml, math::Real};
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};

#[derive(Eq, PartialEq, Debug, Clone, Serialize, Deserialize)]
pub enum Value {
    Real(Real),
    Str(gml::String),
}

impl Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Real(r) => write!(f, "{}", r),
            Self::Str(s) => write!(f, "\"{}\"", s),
        }
    }
}

macro_rules! gml_cmp_impl {
    ($($v: vis $fname: ident aka $op_variant: ident: real: $r_cond: expr, string: $s_cond: expr)*) => {
        $(
            $v fn $fname(self, rhs: Self) -> gml::Result<Self> {
                let freal: fn(Real) -> bool = $r_cond;
                let fstr: fn(&[u8], &[u8]) -> bool = $s_cond;
                Ok(if match (self, rhs) {
                    (Self::Real(a), Self::Real(b)) => freal((a - b)),
                    (Self::Str(a), Self::Str(b)) => fstr(a.as_ref(), b.as_ref()),
                    (a, b) => return invalid_op!($op_variant, a, b),
                } {
                    super::TRUE
                } else {
                    super::FALSE
                }.into())
            }
        )*
    };
}

macro_rules! invalid_op {
    ($op: ident, $value: expr) => {
        Err(gml::Error::InvalidOperandsUnary(gml_parser::token::Operator::$op, $value))
    };
    ($op: ident, $left: expr, $right: expr) => {
        Err(gml::Error::InvalidOperandsBinary(gml_parser::token::Operator::$op, $left, $right))
    };
}

impl Value {
    // All the GML comparison operators (which return Value not bool).
    #[rustfmt::skip]
    gml_cmp_impl! {
        pub gml_eq aka Equal:
            real: |diff| diff.abs() < Real::CMP_EPSILON,
            string: |s1, s2| s1 == s2

        pub gml_ne aka NotEqual:
            real: |diff| diff.abs() >= Real::CMP_EPSILON,
            string: |s1, s2| s1 != s2

        pub gml_lt aka LessThan:
            real: |diff| diff <= -Real::CMP_EPSILON,
            string: |s1, s2| s1 < s2

        pub gml_lte aka LessThanOrEqual:
            real: |diff| diff < Real::CMP_EPSILON,
            string: |s1, s2| s1 <= s2

        pub gml_gt aka GreaterThan:
            real: |diff| diff >= Real::CMP_EPSILON,
            string: |s1, s2| s1 > s2

        pub gml_gte aka GreaterThanOrEqual:
            real: |diff| diff > -Real::CMP_EPSILON,
            string: |s1, s2| s1 >= s2
    }

    pub fn max<'a>(&'a self, other: &'a Self) -> &'a Self {
        // Real never beats String on type mismatch, and String only beats Real if the Real is below 0.
        match (self, other) {
            (Value::Real(a), Value::Real(b)) => {
                if a > b {
                    self
                } else {
                    other
                }
            },
            (Value::Real(a), Value::Str(_)) => {
                if *a.as_ref() < 0.0 {
                    other
                } else {
                    self
                }
            },
            (Value::Str(_), Value::Real(_)) => self,
            (Value::Str(a), Value::Str(b)) => {
                if a > b {
                    self
                } else {
                    other
                }
            },
        }
    }

    pub fn min<'a>(&'a self, other: &'a Self) -> &'a Self {
        // Real always beats String on type mismatch, and String only beats Real if the Real is above 0.
        match (self, other) {
            (Value::Real(a), Value::Real(b)) => {
                if a > b {
                    other
                } else {
                    self
                }
            },
            (Value::Real(a), Value::Str(_)) => {
                if *a.as_ref() > 0.0 {
                    other
                } else {
                    self
                }
            },
            (Value::Str(_), Value::Real(_)) => other,
            (Value::Str(a), Value::Str(b)) => {
                if a > b {
                    other
                } else {
                    self
                }
            },
        }
    }

    pub fn ty_str(&self) -> &'static str {
        match self {
            Self::Real(_) => "real",
            Self::Str(_) => "string",
        }
    }

    /// GML-like comparison, fails if self and other are different types.
    pub fn almost_equals(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Real(a), Self::Real(b)) => *a == *b,
            (Self::Str(a), Self::Str(b)) => a.as_ref() == b.as_ref(),
            _ => false,
        }
    }

    /// Rounds the value to an i32. This is done very commonly by the GM8 runner.
    pub fn round(&self) -> i32 {
        match &self {
            Self::Real(f) => f.round().to_i32(),
            Self::Str(_) => 0,
        }
    }

    /// Formats the value as a number or a string with quotes around it so you can see that it is.
    /// Used in generating error messages.
    fn log_fmt(&self) -> String {
        match self {
            Self::Real(real) => real.to_string(),
            Self::Str(string) => format!("\"{}\"", string),
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Self::Real(f) => f.into_inner() >= 0.5,
            Self::Str(_) => false,
        }
    }

    pub fn as_real(&self) -> Option<Real> {
        match self {
            Self::Real(x) => Some(*x),
            _ => None,
        }
    }

    /// Unary bit complement.
    pub fn complement(self) -> gml::Result<Self> {
        match self {
            Self::Real(val) => Ok((!val.round().to_i32()).into()),
            _ => invalid_op!(Complement, self),
        }
    }

    /// GML operator 'div' which gives the whole number of times RHS can go into LHS. In other words floor(lhs/rhs)
    pub fn intdiv(self, rhs: Self) -> gml::Result<Self> {
        match (self, rhs) {
            (Self::Real(lhs), Self::Real(rhs)) => Ok((lhs / rhs).floor().into()),
            (x, y) => invalid_op!(IntDivide, x, y),
        }
    }

    /// GML && operator
    pub fn bool_and(self, rhs: Self) -> gml::Result<Self> {
        Ok((self.is_truthy() && rhs.is_truthy()).into())
    }

    /// GML || operator
    pub fn bool_or(self, rhs: Self) -> gml::Result<Self> {
        Ok((self.is_truthy() || rhs.is_truthy()).into())
    }

    /// GML ^^ operator
    pub fn bool_xor(self, rhs: Self) -> gml::Result<Self> {
        Ok((self.is_truthy() != rhs.is_truthy()).into())
    }

    pub fn add(self, rhs: Self) -> gml::Result<Self> {
        match (self, rhs) {
            (Self::Real(lhs), Self::Real(rhs)) => Ok((lhs + rhs).into()),
            (Self::Str(lhs), Self::Str(rhs)) => {
                let mut buf = Vec::with_capacity(lhs.as_ref().len() + rhs.as_ref().len());
                buf.extend_from_slice(lhs.as_ref());
                buf.extend_from_slice(rhs.as_ref());
                Ok(buf.into())
            },
            (x, y) => invalid_op!(Add, x, y),
        }
    }

    pub fn add_assign(&mut self, rhs: Self) -> gml::Result<()> {
        match (self, rhs) {
            (Self::Real(lhs), Self::Real(rhs)) => Ok(*lhs += rhs),
            (Self::Str(lhs), Self::Str(ref rhs)) => {
                // TODO: a
                let mut buf = Vec::with_capacity(lhs.as_ref().len() + rhs.as_ref().len());
                buf.extend_from_slice(lhs.as_ref());
                buf.extend_from_slice(rhs.as_ref());
                *lhs = buf.into();
                Ok(())
            },
            (x, y) => invalid_op!(AssignAdd, x.clone(), y),
        }
    }

    pub fn bitand(self, rhs: Self) -> gml::Result<Self> {
        match (self, rhs) {
            (Self::Real(lhs), Self::Real(rhs)) => Ok((lhs.round().to_i32() & rhs.round().to_i32()).into()),
            (x, y) => invalid_op!(BitwiseAnd, x, y),
        }
    }

    pub fn bitand_assign(&mut self, rhs: Self) -> gml::Result<()> {
        match (self, rhs) {
            (Self::Real(lhs), Self::Real(rhs)) => Ok(*lhs = (lhs.round().to_i32() & rhs.round().to_i32()).into()),
            (x, y) => invalid_op!(AssignBitwiseAnd, x.clone(), y),
        }
    }

    pub fn bitor(self, rhs: Self) -> gml::Result<Self> {
        match (self, rhs) {
            (Self::Real(lhs), Self::Real(rhs)) => Ok((lhs.round().to_i32() | rhs.round().to_i32()).into()),
            (x, y) => invalid_op!(BitwiseOr, x, y),
        }
    }

    pub fn bitor_assign(&mut self, rhs: Self) -> gml::Result<()> {
        match (self, rhs) {
            (Self::Real(lhs), Self::Real(rhs)) => Ok(*lhs = (lhs.round().to_i32() | rhs.round().to_i32()).into()),
            (x, y) => invalid_op!(AssignBitwiseOr, x.clone(), y),
        }
    }

    pub fn bitxor(self, rhs: Self) -> gml::Result<Self> {
        match (self, rhs) {
            (Self::Real(lhs), Self::Real(rhs)) => Ok((lhs.round().to_i32() ^ rhs.round().to_i32()).into()),
            (x, y) => invalid_op!(BitwiseXor, x, y),
        }
    }

    pub fn bitxor_assign(&mut self, rhs: Self) -> gml::Result<()> {
        match (self, rhs) {
            (Self::Real(lhs), Self::Real(rhs)) => Ok(*lhs = (lhs.round().to_i32() ^ rhs.round().to_i32()).into()),
            (x, y) => invalid_op!(AssignBitwiseXor, x.clone(), y),
        }
    }

    pub fn div(self, rhs: Self) -> gml::Result<Self> {
        match (self, rhs) {
            (Self::Real(lhs), Self::Real(rhs)) => Ok((lhs / rhs).into()),
            (x, y) => invalid_op!(Divide, x, y),
        }
    }

    pub fn div_assign(&mut self, rhs: Self) -> gml::Result<()> {
        match (self, rhs) {
            (Self::Real(lhs), Self::Real(rhs)) => Ok(*lhs /= rhs),
            (x, y) => invalid_op!(AssignDivide, x.clone(), y),
        }
    }

    pub fn modulo(self, rhs: Self) -> gml::Result<Self> {
        match (self, rhs) {
            (Self::Real(lhs), Self::Real(rhs)) => Ok(Self::Real(lhs - (lhs / rhs).floor_towards_zero() * rhs)),
            (x, y) => invalid_op!(Modulo, x, y),
        }
    }

    pub fn mul(self, rhs: Self) -> gml::Result<Self> {
        match (self, rhs) {
            (Self::Real(lhs), Self::Real(rhs)) => Ok((lhs * rhs).into()),
            (Self::Real(lhs), Self::Str(rhs)) => Ok({
                let repeat = lhs.round().to_i32();
                if repeat > 0 { rhs.as_ref().repeat(repeat as usize).into() } else { "".into() }
            }),
            (x, y) => invalid_op!(Multiply, x, y),
        }
    }

    pub fn mul_assign(&mut self, rhs: Self) -> gml::Result<()> {
        match (self, rhs) {
            (Self::Real(lhs), Self::Real(rhs)) => Ok(*lhs *= rhs),
            (x, y) => invalid_op!(AssignMultiply, x.clone(), y),
        }
    }

    pub fn neg(self) -> gml::Result<Self> {
        match self {
            Self::Real(f) => Ok((-f).into()),
            Self::Str(_) => invalid_op!(Subtract, self),
        }
    }

    pub fn not(self) -> gml::Result<Self> {
        match self {
            Self::Real(_) => Ok((!self.is_truthy()).into()),
            Self::Str(_) => invalid_op!(Not, self),
        }
    }

    pub fn shl(self, rhs: Self) -> gml::Result<Self> {
        match (self, rhs) {
            (Self::Real(lhs), Self::Real(rhs)) => Ok((lhs.round().to_i32() << rhs.round().to_i32()).into()),
            (x, y) => invalid_op!(BinaryShiftLeft, x, y),
        }
    }

    pub fn shr(self, rhs: Self) -> gml::Result<Self> {
        match (self, rhs) {
            (Self::Real(lhs), Self::Real(rhs)) => Ok((lhs.round().to_i32() >> rhs.round().to_i32()).into()),
            (x, y) => invalid_op!(BinaryShiftRight, x, y),
        }
    }

    pub fn sub(self, rhs: Self) -> gml::Result<Self> {
        match (self, rhs) {
            (Self::Real(lhs), Self::Real(rhs)) => Ok((lhs - rhs).into()),
            (x, y) => invalid_op!(Subtract, x, y),
        }
    }

    pub fn sub_assign(&mut self, rhs: Self) -> gml::Result<()> {
        match (self, rhs) {
            (Self::Real(lhs), Self::Real(rhs)) => Ok(*lhs -= rhs),
            (x, y) => invalid_op!(AssignSubtract, x.clone(), y),
        }
    }

    pub fn repr(&self) -> gml::String {
        match self {
            Self::Real(r) if r.fract().into_inner() == 0.0 => format!("{:.0}", r).into(),
            Self::Real(r) => format!("{:.2}", r).into(),
            Self::Str(string) => string.clone(),
        }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(16);
        match self {
            Self::Real(x) => {
                bytes.resize(4, 0);
                bytes.extend_from_slice(&f64::from(*x).to_le_bytes());
                bytes.resize(16, 0);
            },
            Self::Str(s) => {
                bytes.push(1);
                bytes.resize(12, 0);
                bytes.extend_from_slice(&(s.as_ref().len() as u32).to_le_bytes());
                bytes.extend_from_slice(s.as_ref());
            },
        }
        bytes
    }

    pub fn from_reader(reader: &mut dyn std::io::Read) -> Option<Self> {
        let mut block = [0u8; 16];
        reader.read_exact(&mut block).ok()?;
        if block.len() == 16 {
            match u32::from_le_bytes(block[0..4].try_into().unwrap()) {
                0 => Some(f64::from_le_bytes(block[4..12].try_into().unwrap()).into()),
                1 => {
                    let len = u32::from_le_bytes(block[12..16].try_into().unwrap());
                    let mut buf = vec![0; len as usize];
                    reader.read_exact(&mut buf).ok()?;
                    Some(buf.into())
                },
                _ => None,
            }
        } else {
            None
        }
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Self::Real(value.into())
    }
}

impl From<Real> for Value {
    fn from(value: Real) -> Self {
        Self::Real(value)
    }
}

impl From<i32> for Value {
    fn from(value: i32) -> Self {
        Self::Real(value.into())
    }
}

impl From<u32> for Value {
    fn from(value: u32) -> Self {
        Self::Real(value.into())
    }
}

impl From<usize> for Value {
    fn from(value: usize) -> Self {
        (value as f64).into()
    }
}

impl From<u8> for Value {
    fn from(value: u8) -> Self {
        Self::Real(value.into())
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Self::Real(if value { gml::TRUE } else { gml::FALSE }.into())
    }
}

impl From<gml::String> for Value {
    fn from(value: gml::String) -> Self {
        Self::Str(value)
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Self::Str(value.into())
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Self::Str(value.into())
    }
}

impl From<Vec<u8>> for Value {
    fn from(value: Vec<u8>) -> Self {
        Self::Str(value.into())
    }
}

impl From<&[u8]> for Value {
    fn from(value: &[u8]) -> Self {
        Self::Str(value.into())
    }
}

impl From<Value> for i32 {
    // For lazy-converting a value into an i32.
    fn from(value: Value) -> Self {
        match value {
            Value::Real(r) => r.round().to_i32(),
            Value::Str(_) => 0,
        }
    }
}

impl From<Value> for u32 {
    // For lazy-converting a value into an u32.
    fn from(value: Value) -> Self {
        match value {
            Value::Real(r) => r.round().to_u32(),
            Value::Str(_) => 0,
        }
    }
}

impl From<Value> for f64 {
    // For lazy-converting a value into a f64.
    fn from(value: Value) -> Self {
        match value {
            Value::Real(r) => r.into(),
            Value::Str(_) => 0.0,
        }
    }
}

impl From<Value> for Real {
    // For lazy-converting a value into a real.
    fn from(value: Value) -> Self {
        match value {
            Value::Real(r) => r.into(),
            Value::Str(_) => Self::from(0.0),
        }
    }
}

impl From<Value> for gml::String {
    // For lazy-converting a value into a gml::String.
    fn from(value: Value) -> Self {
        match value {
            Value::Real(_) => String::new().into(),
            Value::Str(s) => s,
        }
    }
}

impl<'a> From<&'a Value> for &'a [u8] {
    // For lazy-converting a value into bytes.
    fn from(value: &'a Value) -> Self {
        match value {
            Value::Real(_) => b"",
            Value::Str(s) => s.as_ref(),
        }
    }
}

impl Default for Value {
    fn default() -> Self {
        Self::Real(Real::from(0.0))
    }
}

impl From<Value> for dll::Value {
    fn from(v: Value) -> Self {
        match v {
            Value::Real(r) => dll::Value::Real(r.into()),
            Value::Str(s) => dll::Value::Str(dll::PascalString::new(s.as_ref())),
        }
    }
}

impl From<dll::Value> for Value {
    fn from(v: dll::Value) -> Self {
        match v {
            dll::Value::Real(r) => Value::Real(r.into()),
            dll::Value::Str(s) => s.as_slice().into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn op_add() {
        let a = Value::Real(Real::from(0.1));
        let b = Value::Real(Real::from(0.2));
        assert!((a.add(b).unwrap()).almost_equals(&Value::Real(Real::from(0.3))));

        let c = Value::Str("Hello, ".to_string().into());
        let d = Value::Str("world!".to_string().into());
        assert!((c.add(d).unwrap()).almost_equals(&Value::Str("Hello, world!".to_string().into())));
    }

    #[test]
    #[should_panic]
    fn op_add_invalid() {
        let a = Value::Real(Real::from(0.1));
        let b = Value::Str("owo".to_string().into());
        let _ = a.add(b).unwrap();
    }
}
