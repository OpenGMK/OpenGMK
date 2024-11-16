//! Simple implementation of pseudo-random numbers with a Linear Congruential Generator (LCG) algorithm.
//!
//! The modulus is 32, the increment & multiplier are exposed in constants.

use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Random(i32);

/// Increment value in the LCG algorithm.
pub const INCREMENT: i32 = 1;

/// Multiplier value in the LCG algorithm.
pub const MULTIPLIER: i32 = 0x0808_8405;

/// Constant representing 1/2^32, used in distributing the seed onto a random float.
/// This is the f64 value represented in raw bits for maximum accuracy.
pub const INT_STEP: u64 = 0x3DF0_0000_0000_0000;

impl Random {
    /// Creates a new LCG with a random seed.
    #[inline]
    pub fn new() -> Self {
        Self(rand_int())
    }

    /// Creates a new LCG with a given seed.
    #[inline]
    pub const fn with_seed(seed: i32) -> Self {
        Self(seed)
    }

    /// Equivalent to GML random_get_seed().
    ///
    /// Returns the current LCG seed.
    #[inline]
    pub const fn seed(&self) -> i32 {
        self.0
    }

    /// Equivalent to random_set_seed(n).
    ///
    /// Sets the current LCG seed.
    #[inline]
    pub fn set_seed(&mut self, seed: i32) {
        self.0 = seed;
    }

    /// Implementation of GML randomize().
    ///
    /// Randomizes the LCG seed.
    #[inline]
    pub fn randomize(&mut self) {
        self.set_seed(rand_int());
    }

    /// Cycles the randomizer seed. This is done automatically every call to next/next_int.
    #[inline]
    pub fn cycle(&mut self) {
        self.0 = self.0.wrapping_mul(MULTIPLIER).wrapping_add(INCREMENT);
    }

    /// Equivalent to GML random(n).
    ///
    /// Returns a random float between 0 and n (exclusive).
    #[inline]
    pub fn next(&mut self, bound: f64) -> f64 {
        self.cycle(); // cycle seed
        (self.0 as u32 as f64) * f64::from_bits(INT_STEP) * bound
    }

    /// Equivalent to GML irandom(n).
    ///
    /// Returns a random integer between 0 and n (inclusive).
    ///
    /// The input needs to be cast to unsigned because of weird UB with negative integers.
    /// The output can still be signed, if the input was a signed number (but cast to unsigned).
    #[inline]
    pub fn next_int(&mut self, bound: u32) -> i32 {
        self.cycle(); // cycle seed
        let ls = (self.0 as u64) & 0xFFFF_FFFF;
        let lb = u64::from(bound.wrapping_add(1));
        ((ls.wrapping_mul(lb)) >> 32) as _
    }
}

// Makes a pseudorandom integer. Only used for seeding, such as in randomize().
fn rand_int() -> i32 {
    // TODO: Use uninit_array() and array_assume_init() when stabilized (will require v1.78.X+ tho)
    let mut bytes = std::mem::MaybeUninit::<[u8; 4]>::uninit();
    let _ = unsafe { getrandom::getrandom(&mut *bytes.as_mut_ptr()) };
    i32::from_le_bytes(unsafe { bytes.assume_init() })
}
