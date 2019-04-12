// Don't import this, obviously

// Requires:
#![feature(asm)]

#[inline(always)]
#[cfg(target_arch = "x86_64")]
/// Direct binding to the FPU Integer STore (FIST) instruction -
///   rounds f64 to i32, preferring even numbers when the decimal part is .5
pub fn round(val: f64) -> i32 {
    let out: i32;
    unsafe {
        asm!(
            "sub $$0x08, %rsp   # allocate 8 bytes (x)
            mov $1, (%rsp)      # val -> x
            fldl (%rsp)         # x -> ST(0)
            fistpl (%rsp)       # FIST(ST(0)) -> x, pop ST(0)
            movl (%rsp), $0     # x -> out
            add $$0x08, %rsp"

            : "=r"(out)
            : "r"(val)
        );
    }
    out
}