// Converts BGRA pixeldata to RGBA pixeldata in-place.
pub fn bgra2rgba(data: &mut [u8]) {
    let len = data.len();
    let mut tmp: u8; // blue <> red buffer
    for i in (0..len).step_by(4) {
        tmp = data[i]; // buf = blue
        data[i] = data[i + 2]; // blue = red
        data[i + 2] = tmp; // red = buf
    }
}
