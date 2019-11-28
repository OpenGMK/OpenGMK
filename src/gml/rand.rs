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
    pub fn new(seed: i32) -> Self {
        Self(seed)
    }

    pub fn cycle(&mut self) {
        self.0 = self.0.wrapping_mul(MULTIPLIER).wrapping_add(INCREMENT);
    }

    pub fn next(&mut self, bound: f64) -> f64 {
        self.cycle(); // cycle seed
        (self.0 as u32 as f64) * f64::from_bits(INT_STEP) * bound
    }
}
