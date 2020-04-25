/// Represents an object, instance, tile or special values.
///
/// When positive:
/// - Asset Index in range 0..100_000
/// - Instance Index in range 100_000..10_000_000
/// - Tile Index in range 10_000_000..= (Undefined Behaviour)
///
/// When negative:
/// - `self` / -1, referring to the context of the executing object
/// - `other` / -2, referring to the context of other instances in special events (ex: collision with other instance)
/// - `all` / -3, referring to the context of every instance
/// - `noone` / -4, representing to a nonexistant instance
/// - `global` / -5, referring to a global dummy object
/// - `local` / -7, referring to the context of a dummy object
/// that holds variables of the current script
///
/// Regarding local, `var x; x = 10` is equivalent to `local.x = 10`.
pub type ID = i32;

// -- OLD STUFF --

pub struct BoundingBox {
    pub width: u32,
    pub height: u32,

    pub top: u32,
    pub bottom: u32,
    pub left: u32,
    pub right: u32,
}

pub const RGB_RANGE: f64 = 255.0;

#[derive(Copy, Clone, Debug)]
pub struct Colour {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

impl Colour {
    /// Constructs a new Colour from RGB values.
    pub fn new(r: f64, g: f64, b: f64) -> Self {
        Self { r, g, b }
    }

    /// Converts the colour to an ABGR decimal value.
    pub fn as_decimal(&self) -> u32 {
        ((self.b * RGB_RANGE) as u32) << 16 | ((self.g * RGB_RANGE) as u32) << 8 | (self.r * RGB_RANGE) as u32
    }

    /// Creates a tuple of (r, g, b) values from self.
    pub fn as_rgb(&self) -> (u8, u8, u8) {
        ((self.r * RGB_RANGE) as u8, (self.g * RGB_RANGE) as u8, (self.b * RGB_RANGE) as u8)
    }

    /// Formats self as an RGBA hexadecimal value.
    pub fn as_hexstring(&self) -> String {
        format!("{:0>8X}", self.as_decimal().swap_bytes())
    }
}

impl From<(f64, f64, f64)> for Colour {
    /// Identical to Colour::new()
    fn from(rgb: (f64, f64, f64)) -> Self {
        Self::new(rgb.0, rgb.1, rgb.2)
    }
}

impl From<u32> for Colour {
    /// Creates a Colour from an ABGR decimal value.
    fn from(n: u32) -> Self {
        Self {
            r: (n & 0xFF) as f64 / RGB_RANGE,
            g: ((n >> 8) & 0xFF) as f64 / RGB_RANGE,
            b: ((n >> 16) & 0xFF) as f64 / RGB_RANGE,
        }
    }
}

impl From<Colour> for u32 {
    /// Converts self to an ABGR decimal value.
    fn from(col: Colour) -> u32 {
        col.as_decimal()
    }
}

impl From<(u8, u8, u8)> for Colour {
    fn from(rgb: (u8, u8, u8)) -> Self {
        Self { r: rgb.0 as f64 / RGB_RANGE, g: rgb.1 as f64 / RGB_RANGE, b: rgb.2 as f64 / RGB_RANGE }
    }
}

impl From<Colour> for (u8, u8, u8) {
    fn from(col: Colour) -> (u8, u8, u8) {
        col.as_rgb()
    }
}

#[derive(Debug)]
pub struct Dimensions {
    pub width: u32,
    pub height: u32,
}

pub struct Point {
    pub x: i32,
    pub y: i32,
}

pub struct Rectangle {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

pub type Version = u32;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colour_test() {
        let cool_colour = 0xFECAAB; // RGBA #abcafe
        let col: Colour = cool_colour.into();

        let tup: (u8, u8, u8) = col.into();

        assert_eq!(tup, (171, 202, 254));
        assert_eq!(col.as_decimal(), 0xFECAAB);
        // TODO: test the rest of the From<> impls
    }
}
