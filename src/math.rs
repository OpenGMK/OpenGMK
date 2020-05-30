use std::{
    cmp::Ordering,
    fmt,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign},
};

/// A transparent wrapper for f64 with extended precision (80-bit) arithmetic.
#[derive(Copy, Clone, Default)]
#[repr(transparent)]
pub struct Real(f64);

/// The lenience between values when compared.
const CMP_EPSILON: f64 = 1e-13;

impl From<i32> for Real {
    fn from(i: i32) -> Self {
        Real(f64::from(i))
    }
}

impl From<f64> for Real {
    fn from(f: f64) -> Self {
        Real(f)
    }
}

impl fmt::Debug for Real {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Display for Real {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[rustfmt::skip]
macro_rules! fpu_binary_op {
    ($code: literal, $op1: expr, $op2: expr) => {{
        let out: f64;
        unsafe {
            asm! {
                concat!(
                    "fld qword ptr [{0}]
                    fld qword ptr [{1}]
                    f", $code, "p st, st(1)
                    fstp qword ptr [{0}]
                    mov {2}, qword ptr [{0}]",
                ),
                in(reg) &mut $op1,
                in(reg) &$op2,
                lateout(reg) out,
            }
        }
        out.into()
    }};
}

impl Add for Real {
    type Output = Self;

    #[inline(always)]
    fn add(mut self, other: Self) -> Self {
        fpu_binary_op!("add", self, other)
    }
}

impl Sub for Real {
    type Output = Self;

    #[inline(always)]
    fn sub(mut self, other: Self) -> Self {
        fpu_binary_op!("sub", self, other)
    }
}

impl Mul for Real {
    type Output = Self;

    #[inline(always)]
    fn mul(mut self, other: Self) -> Self {
        fpu_binary_op!("mul", self, other)
    }
}

impl Div for Real {
    type Output = Self;

    #[inline(always)]
    fn div(mut self, other: Self) -> Self {
        fpu_binary_op!("div", self, other)
    }
}

impl AddAssign for Real {
    #[inline(always)]
    fn add_assign(&mut self, other: Self) {
        *self = *self + other
    }
}

impl SubAssign for Real {
    #[inline(always)]
    fn sub_assign(&mut self, other: Self) {
        *self = *self - other
    }
}

impl MulAssign for Real {
    #[inline(always)]
    fn mul_assign(&mut self, other: Self) {
        *self = *self * other
    }
}

impl DivAssign for Real {
    #[inline(always)]
    fn div_assign(&mut self, other: Self) {
        *self = *self / other
    }
}

impl PartialEq for Real {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        (*self - *other).0.abs() < CMP_EPSILON
    }
}
impl Eq for Real {}

impl PartialOrd for Real {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let sub = *self - *other;
        if sub.0 > CMP_EPSILON {
            Some(Ordering::Greater)
        } else if sub.0 < -CMP_EPSILON {
            Some(Ordering::Less)
        } else {
            Some(Ordering::Equal)
        }
    }
}

impl Real {
    #[inline]
    pub fn round(self) -> i32 {
        (self.round64() & u32::max_value() as i64) as i32
    }

    #[inline(always)]
    pub fn round64(mut self) -> i64 {
        unsafe {
            let out: i64;
            asm! {
                "fld qword ptr [{1}]
                fistp qword ptr [{1}]
                mov {0}, [{1}]",
                out(reg) out,
                in(reg) &mut self,
            }
            out
        }
    }

    #[inline(always)]
    pub fn sin(mut self) -> Self {
        unsafe {
            let out: f64;
            asm! {
                "fld qword ptr [{1}]
                fsin
                fstp qword ptr [{1}]
                mov {0}, [{1}]",
                out(reg) out,
                in(reg) &mut self,
            }
            out.into()
        }
    }

    #[inline(always)]
    pub fn cos(mut self) -> Self {
        unsafe {
            let out: f64;
            asm! {
                "fld qword ptr [{1}]
                fcos
                fstp qword ptr [{1}]
                mov {0}, [{1}]",
                out(reg) out,
                in(reg) &mut self,
            }
            out.into()
        }
    }

    #[inline(always)]
    pub fn tan(mut self) -> Self {
        unsafe {
            let out: f64;
            asm! {
                "fld qword ptr [{1}]
                fptan
                fstp st(0)
                fstp qword ptr [{1}]
                mov {0}, [{1}]",
                out(reg) out,
                in(reg) &mut self,
            }
            out.into()
        }
    }

    #[inline(always)]
    pub fn abs(self) -> Self {
        self.0.abs().into()
    }
}

#[cfg(test)]
mod tests {
    use super::Real;
    use std::f64::consts::PI;

    #[test]
    fn add() {
        assert_eq!(Real(3.0), Real(1.0) + Real(2.0));
    }

    #[test]
    fn sub() {
        assert_eq!(Real(1.0), Real(3.0) - Real(2.0));
    }

    #[test]
    fn mul() {
        assert_eq!(Real(6.0), Real(3.0) * Real(2.0));
        assert_eq!(Real(-2.0), Real(2.0) * Real(-1.0));
    }

    #[test]
    fn div() {
        assert_eq!(Real(3.0), Real(6.0) / Real(2.0));
        assert_eq!(Real(-1.0), Real(2.0) / Real(-2.0));
    }

    #[test]
    fn prec_19() {
        const INCREMENT: Real = Real(0.2);
        let mut x = INCREMENT;
        let target = Real(19.0);
        loop {
            x += INCREMENT;
            if x == target {
                break
            } else if x > target {
                panic!();
            }
        }
    }

    #[test]
    fn lt() {
        assert_eq!(Real(3.0) < Real(4.0), true);
        assert_eq!(Real(3.0) < Real(3.0), false);
        assert_eq!(Real(-3.0) < Real(-4.0), false);
        assert_eq!(Real(0.3) < Real(0.1) + Real(0.2), false);
    }

    #[test]
    fn le() {
        assert_eq!(Real(3.0) <= Real(4.0), true);
        assert_eq!(Real(3.0) <= Real(3.0), true);
        assert_eq!(Real(-3.0) <= Real(-4.0), false);
        assert_eq!(Real(0.3) <= Real(0.1) + Real(0.2), true);
    }

    #[test]
    fn gt() {
        assert_eq!(Real(4.0) > Real(3.0), true);
        assert_eq!(Real(3.0) > Real(3.0), false);
        assert_eq!(Real(-4.0) > Real(-3.0), false);
        assert_eq!(Real(0.1) + Real(0.2) > Real(0.3), false);
    }

    #[test]
    fn ge() {
        assert_eq!(Real(4.0) >= Real(3.0), true);
        assert_eq!(Real(3.0) >= Real(3.0), true);
        assert_eq!(Real(-4.0) >= Real(-3.0), false);
        assert_eq!(Real(0.1) + Real(0.2) >= Real(0.3), true);
    }

    #[test]
    fn round() {
        for i in 0..1000 {
            assert_eq!(0, Real(f64::from(i) + 0.5).round() % 2);
        }
    }

    #[test]
    fn sin() {
        assert_eq!(Real(PI / 2.0).sin(), Real(1.0));
    }

    #[test]
    fn cos() {
        assert_eq!(Real(PI).cos(), Real(-1.0));
    }

    #[test]
    fn tan() {
        assert_eq!(Real(PI).tan(), Real(0.0));
    }
}
