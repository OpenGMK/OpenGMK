// xmath: Contains unconventional math functions that don't behave as you'd expect in GameMaker 8.

#[inline(always)]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
/// Direct binding to the FPU Integer STore (FIST) instruction - rounds to nearest even number.
pub fn round(val: f64) -> i32 {
    let val = val as f32;
    let out: i32;
    unsafe {
        asm!(
            "sub $$0x04, %rsp   # allocate 4 bytes (x)
            mov ${0}, (%rsp)    # val -> x
            flds (%rsp)         # x -> ST(0)
            fistpl (%rsp)       # FIST(ST(0)) -> x, pop ST(0)
            movl (%rsp), $0     # x -> out
            add $$0x04, %rsp"

            : "=r"(out)
            : "r"(val)
        );
    }
    out
}

#[inline(always)]
#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
/// Mimics the FPU Integer STore (FIST) instruction - rounds to nearest even number.
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
            assert!(super::round(i as f64 + 0.5) % 2 == 0);
        }
    }
}
