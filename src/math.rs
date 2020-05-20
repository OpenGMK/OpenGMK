use std::{
    hint::black_box,
    ops::{Add, Sub},
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

cfg_if::cfg_if! {
    if #[cfg(target_arch = "x86_64")] {
        #[rustfmt::skip]
        macro_rules! fpu_binary_op {
            ($op: literal, $op1_rmut: expr, $op2_r: expr) => {{
                let out: f64;
                unsafe {
                    llvm_asm! {
                        concat!(
                            "fldl ($2)
                            fldl ($1)
                            f", $op, "p %st, %st(1)
                            fstpl ($1)
                            movq ($1), $0",
                        )
                        : "=r"(out)
                        : "r"($op1_rmut), "r"($op2_r)
                    }
                }
                black_box(out).into()
            }};
        }

        impl Add for Real {
            type Output = Self;

            #[inline(always)]
            fn add(mut self, other: Self) -> Self {
                fpu_binary_op!("add", &mut self, &other)
            }
        }

        impl Sub for Real {
            type Output = Self;

            #[inline(always)]
            fn sub(mut self, other: Self) -> Self {
                fpu_binary_op!("sub", &mut self, &other)
            }
        }
    }
}

impl PartialEq for Real {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        (*self - *other).0.abs() < CMP_EPSILON
    }
}
impl Eq for Real {}

impl Real {
    #[inline]
    pub fn round(self) -> i32 {
        (self.round64() & u32::max_value() as i64) as i32
    }

    #[inline(always)]
    pub fn round64(self) -> i64 {
        unsafe {
            let out: i64;
            llvm_asm! {
                "fldl ($1)
                fistpq ($1)
                movq ($1), $0"

                : "=r"(out)
                : "r"(&self)
            }
            black_box(out)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Real;

    #[test]
    fn add() {
        assert_eq!(3.0, (Real(1.0) + Real(2.0)).0);
    }

    #[test]
    fn sub() {
        assert_eq!(1.0, (Real(3.0) - Real(2.0)).0);
    }

    #[test]
    fn round() {
        for i in 0..1000 {
            assert_eq!(0, Real(f64::from(i) + 0.5).round() % 2);
        }
    }
}
