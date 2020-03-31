/// Converts BGRA pixeldata to RGBA pixeldata in-place.
pub fn bgra2rgba(data: &mut [u8]) {
    assert_eq!(data.len() % 4, 0);
    data.chunks_exact_mut(4).for_each(|chunk| chunk.swap(0, 2));
}

/// Converts RGBA pixeldata to BGRA pixeldata in-place.
pub use bgra2rgba as rgba2bgra;

/// The default way to round as defined by IEEE 754 - nearest, ties to even.
pub fn ieee_round(real: f64) -> i32 {
    let floor = real.floor();
    let floori = floor as i32;
    let diff = real - floor;
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
    use super::{bgra2rgba, rgba2bgra};

    #[test]
    fn bgra_rgba() {
        let bgra_pixels = [0u8, 1, 54, 242, 192, 24, 65, 6, 35, 98, 4, 20];
        let rgba_pixels = [54u8, 1, 0, 242, 65, 24, 192, 6, 4, 98, 35, 20];
        let mut cool_pixels = bgra_pixels.to_vec();

        bgra2rgba(&mut cool_pixels);
        assert_eq!(cool_pixels, rgba_pixels);
        rgba2bgra(&mut cool_pixels);
        assert_eq!(cool_pixels, bgra_pixels);
    }
}
