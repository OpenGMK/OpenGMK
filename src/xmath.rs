// xmath: Contains unconventional math functions that don't behave as you'd expect in GameMaker 8.
// * These could be optimized with some clever usage of asm!()

/// Mimics the FPU Integer STore (FIST) instruction - rounds to nearest even number.
#[inline(always)]
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
    }
}
