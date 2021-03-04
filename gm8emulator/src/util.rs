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
    let floori = floor as i64 as i32;
    let diff = real - floor;
    if diff < 0.5 {
        floori
    } else if diff > 0.5 {
        floori + 1
    } else {
        floori + (floori & 1)
    }
}

// Helper fn: rotate mutable x and y around a center point, given sin and cos of the angle to rotate by
pub fn rotate_around(x: &mut f64, y: &mut f64, center_x: f64, center_y: f64, sin: f64, cos: f64) {
    *x -= center_x;
    *y -= center_y;
    rotate_around_center(x, y, sin, cos);
    *x += center_x;
    *y += center_y;
}

pub fn rotate_around_center(x: &mut f64, y: &mut f64, sin: f64, cos: f64) {
    let x_new = (*x * cos) - (*y * sin);
    let y_new = (*x * sin) + (*y * cos);
    *x = x_new;
    *y = y_new;
}

#[cfg(test)]
mod tests {
    use super::{bgra2rgba, ieee_round, rgba2bgra};

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

    #[test]
    fn round() {
        assert_eq!(ieee_round(-3.5), -4);
        assert_eq!(ieee_round(-2.5), -2);
        assert_eq!(ieee_round(-1.5), -2);
        assert_eq!(ieee_round(-0.5), 0);
        assert_eq!(ieee_round(0.5), 0);
        assert_eq!(ieee_round(1.5), 2);
        assert_eq!(ieee_round(2.5), 2);
        assert_eq!(ieee_round(3.5), 4);
        for i in 0..1000 {
            assert_eq!(ieee_round(i as f64 + 0.5) % 2, 0);
        }
    }
}
