//! Simple implementation of pseudo-random numbers with a Linear Congruential Generator (LCG) algorithm.
//!
//! The modulus is 32, the increment & multiplier are exposed in constants.

pub struct Random(i32);

/// Increment value in the LCG algorithm.
pub const INCREMENT: i32 = 1;

/// Multiplier value in the LCG algorithm.
pub const MULTIPLIER: i32 = 0x8088405;

/// Constant representing 1/2^32, used in distributing the seed onto a random float.
/// This is the f64 value represented in raw bits for maximum accuracy.
pub const INT_STEP: u64 = 0x3DF0000000000000;

impl Random {
    #[inline]
    pub const fn new(seed: i32) -> Self {
        Self(seed)
    }

    #[inline]
    pub fn cycle(&mut self) {
        self.0 = self.0.wrapping_mul(MULTIPLIER).wrapping_add(INCREMENT);
    }

    #[inline]
    pub fn next(&mut self, bound: f64) -> f64 {
        self.cycle(); // cycle seed
        (self.0 as u32 as f64) * f64::from_bits(INT_STEP) * bound
    }

    #[inline]
    pub fn next_int(&mut self, bound: i32) -> i32 {
        self.cycle(); // cycle seed
        let ls = (self.0 as u64) & 0xFFFFFFFF;
        let lb = i64::from(bound).wrapping_add(1);
        ((ls.wrapping_mul(lb as u64)) >> 32) as i32
    }
}
