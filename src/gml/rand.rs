pub struct Random(i32);

pub const INCREMENT: i32 = 1;
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
        (self.0 as u32 as f64) * f64::from_bits(INT_STEP) * bound
    }
}
