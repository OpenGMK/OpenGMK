use std::{
    ops::{
        Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Div, DivAssign, Mul, MulAssign,
        Neg, Not, Rem, RemAssign, Shl, ShlAssign, Shr, ShrAssign, Sub, SubAssign,
    },
    rc::Rc,
};

#[derive(Debug, Clone)]
pub enum Value {
    Real(f64),
    Str(Rc<str>),
}

macro_rules! gml_cmp_impl {
    ($($v: vis $fname: ident aka $op_str: literal: real: $r_cond: expr, string: $s_cond: expr)*) => {
        $(
            $v fn $fname(self, rhs: Self) -> Self {
                let freal: fn(f64, f64) -> bool = $r_cond;
                let fstr: fn(&str, &str) -> bool = $s_cond;
                if match (self, rhs) {
                    (Value::Real(a), Value::Real(b)) => freal(a, b),
                    (Value::Str(a), Value::Str(b)) => fstr(a.as_ref(), b.as_ref()),
                    (a, b) => gml_panic!(
                        concat!(
                            "invalid arguments to ",
                            $op_str,
                            " operator ({} ",
                            $op_str,
                            " {})",
                        ),
                        a.log_fmt(),
                        b.log_fmt()
                    )
                } {
                    Real(super::TRUE)
                } else {
                    Real(super::FALSE)
                }
            }
        )*
    };
}

// How many times do you think I want to write `Value::` in the `value` module?
pub(self) use Value::*;

impl Value {
    /// GML-like comparison, fails if self and other are different types.
    fn almost_equals(&self, other: &Self) -> bool {
        match (self, other) {
            (Real(a), Real(b)) => (a - b).abs() <= 1e-14,
            (Str(a), Str(b)) => a.as_ref() == b.as_ref(),
            (x, y) => gml_panic!("invalid arguments to == operator ({} == {})", x.log_fmt(), y.log_fmt()),
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

    fn is_true(&self) -> bool {
        match self {
            Real(f) => *f >= 0.5, // What a confusing line.
            Str(_) => false,
        }
    }

    /// Unary bit complement. Has its own operator in GML as ! is always boolean.
    pub fn complement(self) -> Self {
        match self {
            Real(val) => Real(!(Self::ieee_round(val) as i32) as f64),
            _ => gml_panic!("invalid arguments to ~ (complement)"),
        }
    }

    /// GML operator 'div' which gives the whole number of times RHS can go into LHS. In other words floor(lhs/rhs)
    pub fn intdiv(self, rhs: Self) -> Self {
        match (self, rhs) {
            (Real(lhs), Real(rhs)) => Real((lhs / rhs).floor()),
            (x, y) => gml_panic!("invalid arguments to div operator ({} & {})", x.log_fmt(), y.log_fmt()),
        }
    }

    // All the GML comparison operators (which return Value not bool).
    #[rustfmt::skip] gml_cmp_impl! {
        pub gml_eq aka "==":
            real: |r1, r2| (r1 - r2).abs() <= 1e-14,
            string: |s1, s2| s1 == s2

        pub gml_ne aka "!=":
            real: |r1, r2| (r1 - r2).abs() > 1e-14,
            string: |s1, s2| s1 != s2

        pub gml_lt aka "<":
            real: |r1, r2| r1 < r2,
            string: |s1, s2| s1 < s2
        
        pub gml_lte aka "<=":
            real: |r1, r2| r1 < r2 || (r1 - r2).abs() <= 1e-14,
            string: |s1, s2| s1 <= s2
        
        pub gml_gt aka ">":
            real: |r1, r2| r1 > r2,
            string: |s1, s2| s1 > s2
        
        pub gml_gte aka ">=":
            real: |r1, r2| r1 > r2 || (r1 - r2).abs() <= 1e-14,
            string: |s1, s2| s1 >= s2
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
            (x, y) => gml_panic!("invalid arguments to + operator ({} + {})", x.log_fmt(), y.log_fmt()),
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
            }
            (x, y) => gml_panic!("invalid arguments to += operator ({} += {})", x.log_fmt(), y.log_fmt()),
        }
    }
}

impl BitAnd for Value {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Real(lhs), Real(rhs)) => Real((Self::ieee_round(lhs) as i32 & Self::ieee_round(rhs) as i32) as _),
            (x, y) => gml_panic!("invalid arguments to & operator ({} & {})", x.log_fmt(), y.log_fmt()),
        }
    }
}

impl BitAndAssign for Value {
    fn bitand_assign(&mut self, rhs: Self) {
        match (self, rhs) {
            (Real(lhs), Real(rhs)) => *lhs = (Self::ieee_round(*lhs) as i32 & Self::ieee_round(rhs) as i32) as _,
            (x, y) => gml_panic!("invalid arguments to &= operator ({} &= {})", x.log_fmt(), y.log_fmt()),
        }
    }
}

impl BitOr for Value {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Real(lhs), Real(rhs)) => Real((Self::ieee_round(lhs) as i32 | Self::ieee_round(rhs) as i32) as _),
            (x, y) => gml_panic!("invalid arguments to | operator ({} | {})", x.log_fmt(), y.log_fmt()),
        }
    }
}

impl BitOrAssign for Value {
    fn bitor_assign(&mut self, rhs: Self) {
        match (self, rhs) {
            (Real(lhs), Real(rhs)) => *lhs = (Self::ieee_round(*lhs) as i32 | Self::ieee_round(rhs) as i32) as _,
            (x, y) => gml_panic!("invalid arguments to |= operator ({} |= {})", x.log_fmt(), y.log_fmt()),
        }
    }
}

impl BitXor for Value {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Real(lhs), Real(rhs)) => Real((Self::ieee_round(lhs) as i32 ^ Self::ieee_round(rhs) as i32) as _),
            (x, y) => gml_panic!("invalid arguments to ^ operator ({} ^ {})", x.log_fmt(), y.log_fmt()),
        }
    }
}

impl BitXorAssign for Value {
    fn bitxor_assign(&mut self, rhs: Self) {
        match (self, rhs) {
            (Real(lhs), Real(rhs)) => *lhs = (Self::ieee_round(*lhs) as i32 ^ Self::ieee_round(rhs) as i32) as _,
            (x, y) => gml_panic!("invalid arguments to ^= operator ({} ^= {})", x.log_fmt(), y.log_fmt()),
        }
    }
}

impl Div for Value {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Real(lhs), Real(rhs)) => Real(lhs / rhs),
            (x, y) => gml_panic!("invalid arguments to / operator ({} / {})", x.log_fmt(), y.log_fmt()),
        }
    }
}

impl DivAssign for Value {
    fn div_assign(&mut self, rhs: Self) {
        match (self, rhs) {
            (Real(lhs), Real(rhs)) => *lhs /= rhs,
            (x, y) => gml_panic!("invalid arguments to /= operator ({} /= {})", x.log_fmt(), y.log_fmt()),
        }
    }
}

impl Mul for Value {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Real(lhs), Real(rhs)) => Real(lhs * rhs),
            (x, y) => gml_panic!("invalid arguments to * operator ({} * {})", x.log_fmt(), y.log_fmt()),
        }
    }
}

impl MulAssign for Value {
    fn mul_assign(&mut self, rhs: Self) {
        match (self, rhs) {
            (Real(lhs), Real(rhs)) => *lhs *= rhs,
            (x, y) => gml_panic!("invalid arguments to *= operator ({} *= {})", x.log_fmt(), y.log_fmt()),
        }
    }
}

impl Neg for Value {
    type Output = Self;

    fn neg(self) -> Self::Output {
        match self {
            Real(f) => Real(-f),
            Str(_) => gml_panic!("invalid operand to neg (-)"),
        }
    }
}

impl Not for Value {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Real(_) => Real((!self.is_true()) as i8 as f64),
            Str(_) => gml_panic!("invalid operand to not (!)"),
        }
    }
}

impl Rem for Value {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Real(lhs), Real(rhs)) => Real(lhs % rhs),
            (x, y) => gml_panic!(
                "invalid arguments to mod operator ({} mod {})",
                x.log_fmt(),
                y.log_fmt()
            ),
        }
    }
}

// note: no RemAssign (%=) in GML, but I made it anyway
impl RemAssign for Value {
    fn rem_assign(&mut self, rhs: Self) {
        match (self, rhs) {
            (Real(lhs), Real(rhs)) => *lhs %= rhs,
            (x, y) => gml_panic!("invalid arguments to RemAssign ({} %= {})", x.log_fmt(), y.log_fmt()),
        }
    }
}

impl Shl for Value {
    type Output = Self;

    fn shl(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Real(lhs), Real(rhs)) => Real(((Self::ieee_round(lhs) as i32) << Self::ieee_round(rhs) as i32) as _),
            (x, y) => gml_panic!("invalid arguments to << operator ({} << {})", x.log_fmt(), y.log_fmt()),
        }
    }
}

impl ShlAssign for Value {
    fn shl_assign(&mut self, rhs: Self) {
        match (self, rhs) {
            (Real(lhs), Real(rhs)) => *lhs = ((Self::ieee_round(*lhs) as i32) << Self::ieee_round(rhs) as i32) as _,
            (x, y) => gml_panic!(
                "invalid arguments to <<= operator ({} <<= {})",
                x.log_fmt(),
                y.log_fmt()
            ),
        }
    }
}

impl Shr for Value {
    type Output = Self;

    fn shr(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Real(lhs), Real(rhs)) => Real((Self::ieee_round(lhs) as i32 >> Self::ieee_round(rhs) as i32) as _),
            (x, y) => gml_panic!("invalid arguments to >> operator ({} >> {})", x.log_fmt(), y.log_fmt()),
        }
    }
}

impl ShrAssign for Value {
    fn shr_assign(&mut self, rhs: Self) {
        match (self, rhs) {
            (Real(lhs), Real(rhs)) => *lhs = (Self::ieee_round(*lhs) as i32 >> Self::ieee_round(rhs) as i32) as _,
            (x, y) => gml_panic!(
                "invalid arguments to >>= operator ({} >>= {})",
                x.log_fmt(),
                y.log_fmt()
            ),
        }
    }
}

impl Sub for Value {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Real(lhs), Real(rhs)) => Real(lhs - rhs),
            (x, y) => gml_panic!("invalid arguments to - operator ({} - {})", x.log_fmt(), y.log_fmt()),
        }
    }
}

impl SubAssign for Value {
    fn sub_assign(&mut self, rhs: Self) {
        match (self, rhs) {
            (Real(lhs), Real(rhs)) => *lhs -= rhs,
            (x, y) => gml_panic!("invalid arguments to -= operator ({} -= {})", x.log_fmt(), y.log_fmt()),
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
