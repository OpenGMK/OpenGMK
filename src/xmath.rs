// xmath: Contains unconventional math functions that don't behave as you'd expect in GameMaker 8.

#[inline(always)]
/// Compares two real numbers.
/// Both are interpreted as a 32-bit float to avoid pitfalls when comparing.
pub fn equals(val1: f64, val2: f64) -> bool {
    val1 as f32 == val2 as f32
}

#[inline(always)]
/// Mimics the FPU Integer STore (FIST) instruction -
///   rounds f64 to i32, preferring even numbers when the decimal part is .5
pub fn round(val: f64) -> i32 {
    let floor = val.floor();
    let floori = floor as i32;
    let diff = val - floor;
    if diff < 0.5 {
        floori
    } else if diff > 0.5 {
        floori + 1
    } else {
        floori + (floori & 1)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn round() {
        assert_eq!(super::round(0.5), 0);
        assert_eq!(super::round(1.5), 2);
        assert_eq!(super::round(2.5), 2);
        assert_eq!(super::round(3.5), 4);
        for i in 0..1000 {
            assert_eq!(super::round(i as f64 + 0.5) % 2, 0);
        }
    }
}
