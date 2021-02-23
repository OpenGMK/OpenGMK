use crate::util;
use cfg_if::cfg_if;
use serde::{Deserialize, Serialize};
use std::{
    cmp::Ordering,
    fmt,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Rem, Sub, SubAssign},
};

/// A transparent wrapper for f64 with extended precision (80-bit) arithmetic.
#[derive(Copy, Clone, Default, Serialize, Deserialize)]
#[repr(transparent)]
pub struct Real(f64);

/// The lenience between values when compared.
const CMP_EPSILON: f64 = 1e-13;

// Platform-specific implementation of the arithmetic. Should provide:
// Add, Sub, Mul, Div, sin, cos, tan, round64
cfg_if! {
    if #[cfg(any(target_arch = "x86", target_arch = "x86_64"))] {

        impl Real {

            #[inline(always)]
            pub fn sin(self) -> Self {
                Self(self.0.sin())
            }

            #[inline(always)]
            pub fn cos(self) -> Self {
                Self(self.0.cos())
            }

            #[inline(always)]
            pub fn tan(self) -> Self {
                Self(self.0.tan())
            }

            pub fn arcsin(self) -> Self {
                Self(self.0.asin())
            }

            pub fn arccos(self) -> Self {
                Self(self.0.acos())
            }

            pub fn arctan(self) -> Self {
                Self(self.0.atan())
            }

            pub fn arctan2(self, other: Real) -> Self {
                Self(self.0.atan2(other.0))
            }

            pub fn exp(self) -> Self {
                Self(self.0.exp())
            }

            pub fn ln(self) -> Self {
                Self(self.0.ln())
            }

            pub fn log2(self) -> Self {
                Self(self.0.log2())
            }

            pub fn log10(self) -> Self {
                Self(self.0.log10())
            }

            pub fn logn(self, other: Real) -> Self {
                Self(self.0.log(other.0))
            }

            pub fn sqrt(self) -> Self {
                Self(self.0.sqrt())
            }
        }

        impl Add for Real {
            type Output = Self;

            #[inline(always)]
            fn add(self, other: Self) -> Self {
                Self(self.0 + other.0)
            }
        }

        impl Sub for Real {
            type Output = Self;

            #[inline(always)]
            fn sub(self, other: Self) -> Self {
                Self(self.0 - other.0)
            }
        }

        impl Mul for Real {
            type Output = Self;

            #[inline(always)]
            fn mul(self, other: Self) -> Self {
                Self(self.0 * other.0)
            }
        }

        impl Div for Real {
            type Output = Self;

            #[inline(always)]
            fn div(self, other: Self) -> Self {
                Self(self.0 / other.0)
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

impl std::iter::Sum<Real> for Real {
    fn sum<I>(iter: I) -> Real
    where
        I: Iterator<Item = Real>,
    {
        iter.fold(Real(0.0), Real::add)
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
        util::ieee_round(self.0)
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
    pub fn trunc(self) -> Self {
        Self(self.0.trunc())
    }

    #[inline(always)]
    pub fn fract(self) -> Self {
        Self(self.0.fract())
    }

    #[inline(always)]
    pub fn to_radians(self) -> Self {
        Self(self.0.to_radians())
    }

    #[inline(always)]
    pub fn to_degrees(self) -> Self {
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
    pub fn rem_euclid(self, other: Real) -> Self {
        self.0.rem_euclid(other.0).into()
    }

    #[inline(always)]
    pub fn as_ref(&self) -> &f64 {
        &self.0
    }

    #[inline(always)]
    pub fn as_mut_ref(&mut self) -> &mut f64 {
        &mut self.0
    }

    #[inline(always)]
    pub fn cmp_nan_first(&self, other: &Self) -> Ordering {
        if self.0.is_nan() {
            if other.0.is_nan() {
                return Ordering::Equal
            }
            return Ordering::Less
        }
        if other.0.is_nan() {
            return Ordering::Greater
        }
        self.partial_cmp(other).unwrap()
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
        assert_eq!(Real(1.0), Real(7.0) % Real(3.0));
        assert_eq!(Real(0.75), Real(160.5) % Real(2.25));
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

    #[test]
    fn to_degrees() {
        assert_eq!(Real(std::f64::consts::PI).to_degrees(), Real(180.0));
    }

    #[test]
    fn to_radians() {
        assert_eq!(Real(180.0).to_radians(), Real(std::f64::consts::PI));
    }
}
