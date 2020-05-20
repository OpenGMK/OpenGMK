use std::{hint::black_box, ops::Add};

/// A transparent wrapper for f64 with extended precision (80-bit) arithmetic.
#[derive(Copy, Clone, Default)]
#[repr(transparent)]
pub struct Real(f64);

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
                            "fldl ($1)
                            fldl ($2)
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
    }
}

#[cfg(test)]
mod tests {
    use super::Real;

    #[test]
    fn add() {
        assert_eq!(3.0, (Real(1.0) + Real(2.0)).0);
    }
}
