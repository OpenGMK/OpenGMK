use cfg_if::cfg_if;
use std::{
    cmp::Ordering,
    fmt,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Rem, Sub, SubAssign},
};

/// A transparent wrapper for f64 with extended precision (80-bit) arithmetic.
#[derive(Copy, Clone, Default)]
#[repr(transparent)]
pub struct Real(f64);

/// The lenience between values when compared.
const CMP_EPSILON: f64 = 1e-13;

// Platform-specific implementation of the arithmetic. Should provide:
// Add, Sub, Mul, Div, sin, cos, tan, round64
cfg_if! {
    if #[cfg(any(target_arch = "x86", target_arch = "x86_64"))] {
        macro_rules! fpu_unary_op {
            ($code: literal, $op: expr) => {
                unsafe {
                    let out: f64;
                    asm! {
                        concat!(
                            "fld qword ptr [{1}]
                            ", $code, "
                            fstp qword ptr [{1}]
                            movsd {0}, [{1}]"
                        ),
                        lateout(xmm_reg) out,
                        in(reg) &mut $op,
                    }
                    out.into()
                }
            };
        }

        macro_rules! fpu_binary_op {
            ($code: literal, $op1: expr, $op2: expr) => {{
                let out: f64;
                unsafe {
                    asm! {
                        concat!(
                            "fld qword ptr [{0}]
                            fld qword ptr [{1}]
                            ", $code, " st, st(1)
                            fstp qword ptr [{0}]
                            movsd {2}, qword ptr [{0}]",
                        ),
                        in(reg) &mut $op1,
                        in(reg) &$op2,
                        lateout(xmm_reg) out,
                    }
                }
                out.into()
            }};
        }

        impl Real {
            #[inline(always)]
            pub fn round64(mut self) -> i64 {
                unsafe {
                    let out: i64;
                    if cfg!(target_arch = "x86_64") {
                        asm! {
                            "fld qword ptr [{1}]
                            fistp qword ptr [{1}]
                            mov {0}, qword ptr [{1}]",
                            lateout(reg) out,
                            in(reg) &mut self,
                        }
                    } else {
                        // OPTIMIZE: Using an SSE register here probably isn't the fastest?
                        // How the fuck do I specify "stack value"?
                        asm! {
                            "fld qword ptr [{1}]
                            fistp qword ptr [{1}]
                            movsd {0}, [{1}]",
                            lateout(xmm_reg) out,
                            in(reg) &mut self,
                        }
                    }
                    out
                }
            }

            #[inline(always)]
            pub fn sin(mut self) -> Self {
                fpu_unary_op!("fsin", self)
            }

            #[inline(always)]
            pub fn cos(mut self) -> Self {
                fpu_unary_op!("fcos", self)
            }

            #[inline(always)]
            pub fn tan(mut self) -> Self {
                fpu_unary_op!(
                    "fptan
                    fstp st(0)",
                    self
                )
            }

            pub fn arcsin(mut self) -> Self {
                unsafe {
                    let out: f64;
                    asm! {
                        "fld qword ptr [{1}]
                        fld1
                        fadd st(0),st(1)
                        fld1
                        fsub st(0),st(2)
                        fmulp
                        fsqrt
                        fpatan
                        fstp qword ptr [{1}]
                        movsd {0}, qword ptr [{1}]",
                        lateout(xmm_reg) out,
                        in(reg) &mut self,
                    }
                    out.into()
                }
            }

            pub fn arccos(mut self) -> Self {
                unsafe {
                    let out: f64;
                    asm! {
                        "fld1
                        fld qword ptr [{1}]
                        fsub st(1),st(0)
                        fld1
                        faddp
                        fmulp
                        fsqrt
                        fld qword ptr [{1}]
                        fpatan
                        fstp qword ptr [{1}]
                        movsd {0}, qword ptr [{1}]",
                        lateout(xmm_reg) out,
                        in(reg) &mut self,
                    }
                    out.into()
                }
            }

            pub fn arctan(mut self) -> Self {
                fpu_unary_op!(
                    "fld1
                    fpatan",
                    self
                )
            }

            pub fn arctan2(mut self, mut other: Real) -> Self {
                unsafe {
                    let out: f64;
                    asm! {
                        "fld qword ptr [{1}]
                        fld qword ptr [{2}]
                        fpatan
                        fstp qword ptr [{1}]
                        movsd {0}, qword ptr [{1}]",
                        lateout(xmm_reg) out,
                        in(reg) &mut self,
                        in(reg) &mut other,
                    }
                    out.into()
                }
            }

            pub fn exp(mut self) -> Self {
                unsafe {
                    let out: f64;
                    asm! {
                        "fld qword ptr [{1}]
                        fldl2e
                        fmulp
                        fld st(0)
                        frndint
                        fsub st(1),st(0)
                        fxch
                        f2xm1
                        fld1
                        faddp
                        fscale
                        fstp qword ptr [{1}]
                        fstp st(0)
                        movsd {0}, qword ptr [{1}]",
                        lateout(xmm_reg) out,
                        in(reg) &mut self,
                    }
                    out.into()
                }
            }

            pub fn ln(mut self) -> Self {
                unsafe {
                    let out: f64;
                    asm! {
                        "fldln2
                        fld qword ptr [{1}]
                        fyl2x
                        fstp qword ptr [{1}]
                        movsd {0}, qword ptr [{1}]",
                        lateout(xmm_reg) out,
                        in(reg) &mut self,
                    }
                    out.into()
                }
            }

            pub fn log2(mut self) -> Self {
                unsafe {
                    let out: f64;
                    asm! {
                        "fld1
                        fld qword ptr [{1}]
                        fyl2x
                        fstp qword ptr [{1}]
                        movsd {0}, qword ptr [{1}]",
                        lateout(xmm_reg) out,
                        in(reg) &mut self,
                    }
                    out.into()
                }
            }

            pub fn log10(mut self) -> Self {
                unsafe {
                    let out: f64;
                    asm! {
                        "fldlg2
                        fld qword ptr [{1}]
                        fyl2x
                        fstp qword ptr [{1}]
                        movsd {0}, qword ptr [{1}]",
                        lateout(xmm_reg) out,
                        in(reg) &mut self,
                    }
                    out.into()
                }
            }

            pub fn logn(mut self, mut other: Real) -> Self {
                unsafe {
                    let out: f64;
                    asm! {
                        "fld1
                        fld qword ptr [{1}]
                        fyl2x
                        fld1
                        fld qword ptr [{2}]
                        fyl2x
                        fdivp
                        fstp qword ptr [{1}]
                        movsd {0}, qword ptr [{1}]",
                        lateout(xmm_reg) out,
                        in(reg) &mut self,
                        in(reg) &mut other,
                    }
                    out.into()
                }
            }

            pub fn sqrt(mut self) -> Self {
                fpu_unary_op!(
                    "fsqrt",
                    self
                )
            }
        }

        impl Add for Real {
            type Output = Self;

            #[inline(always)]
            fn add(mut self, other: Self) -> Self {
                fpu_binary_op!("faddp", self, other)
            }
        }

        impl Sub for Real {
            type Output = Self;

            #[inline(always)]
            fn sub(mut self, other: Self) -> Self {
                fpu_binary_op!("fsubp", self, other)
            }
        }

        impl Mul for Real {
            type Output = Self;

            #[inline(always)]
            fn mul(mut self, other: Self) -> Self {
                fpu_binary_op!("fmulp", self, other)
            }
        }

        impl Div for Real {
            type Output = Self;

            #[inline(always)]
            fn div(mut self, other: Self) -> Self {
                fpu_binary_op!("fdivp", self, other)
            }
        }
    }
}

impl From<i32> for Real {
    #[inline(always)]
    fn from(i: i32) -> Self {
        Real(f64::from(i))
    }
}

impl From<u32> for Real {
    #[inline(always)]
    fn from(i: u32) -> Self {
        Real(f64::from(i))
    }
}

impl From<f64> for Real {
    #[inline(always)]
    fn from(f: f64) -> Self {
        Real(f)
    }
}

impl From<Real> for f64 {
    #[inline(always)]
    fn from(real: Real) -> Self {
        real.0
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

impl Rem for Real {
    type Output = Self;

    #[inline(always)]
    fn rem(self, other: Self) -> Self {
        Real(self.0 % other.0)
    }
}

impl Neg for Real {
    type Output = Self;

    #[inline(always)]
    fn neg(self) -> Self {
        Real(-self.0)
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
        if sub.0 >= CMP_EPSILON {
            Some(Ordering::Greater)
        } else if sub.0 <= -CMP_EPSILON {
            Some(Ordering::Less)
        } else {
            Some(Ordering::Equal)
        }
    }
}

impl Real {
    #[inline(always)]
    pub fn abs(self) -> Self {
        self.0.abs().into()
    }

    #[inline(always)]
    pub fn into_inner(self) -> f64 {
        self.0
    }

    #[inline(always)]
    pub fn round(self) -> i32 {
        (self.round64() & u32::max_value() as i64) as i32
    }

    #[inline(always)]
    pub fn floor(self) -> Self {
        Self(self.0.floor())
    }

    #[inline(always)]
    pub fn ceil(self) -> Self {
        Self(self.0.ceil())
    }

    #[inline(always)]
    pub fn fract(self) -> Self {
        Self(self.0.fract())
    }

    #[inline(always)]
    pub fn to_radians(self) -> Self {
        // TODO: once asm! can process tword ptrs, multiply input with this instead:
        // [AE C8 E9 94 12 35 FA 8E F9 3F]
        Self(self.0.to_radians())
    }

    #[inline(always)]
    pub fn to_degrees(self) -> Self {
        // TODO: once asm! can process tword ptrs, multiply input with this instead:
        // [C3 BD 0F 1E D3 E0 2E E5 04 40]
        Self(self.0.to_degrees())
    }

    #[inline(always)]
    pub fn min(self, other: Real) -> Self {
        self.0.min(other.0).into()
    }

    #[inline(always)]
    pub fn max(self, other: Real) -> Self {
        self.0.max(other.0).into()
    }

    #[inline(always)]
    pub fn as_ref(&self) -> &f64 {
        &self.0
    }

    #[inline(always)]
    pub fn as_mut_ref(&mut self) -> &mut f64 {
        &mut self.0
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
    fn rem() {
        assert_eq!(Real(1.0), Real(3.0) / Real(7.0));
        assert_eq!(Real(0.75), Real(2.25) / Real(160.5));
    }

    #[test]
    fn neg() {
        assert_eq!(-Real(3.0), Real(-3.0));
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

    #[test]
    fn arcsin() {
        assert_eq!(Real(0.8).arcsin(), Real(0.9272952180016123));
    }

    #[test]
    fn arccos() {
        assert_eq!(Real(0.8).arccos(), Real(0.6435011087932844));
    }

    #[test]
    fn arctan() {
        assert_eq!(Real(123.4).arctan(), Real(1.5626927764648464));
    }

    #[test]
    fn arctan2() {
        assert_eq!(Real(5.0).arctan2(Real(8.1)), Real(0.5530314441506405));
        assert_eq!(Real(8.1).arctan2(Real(5.0)), Real(1.0177648826442560));
    }

    #[test]
    fn exp() {
        assert_eq!(Real(3.1).exp(), Real(22.19795128144164));
    }

    #[test]
    fn ln() {
        assert_eq!(Real(std::f64::consts::E).ln(), Real(1.0));
    }

    #[test]
    fn log2() {
        assert_eq!(Real(2.0).log2(), Real(1.0));
        assert_eq!(Real(8.0).log2(), Real(3.0));
    }

    #[test]
    fn log10() {
        assert_eq!(Real(10.0).log10(), Real(1.0));
        assert_eq!(Real(1000.0).log10(), Real(3.0));
    }

    #[test]
    fn logn() {
        assert_eq!(Real(343.0).logn(Real(7.0)), Real(3.0));
    }

    #[test]
    fn sqrt() {
        assert_eq!(Real(9.0).sqrt(), Real(3.0));
    }
}
