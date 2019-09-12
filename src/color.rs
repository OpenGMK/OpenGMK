#[derive(Copy, Clone, Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    /// Constructs a new Color from RGB values.
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Color {
        Color { r, g, b, a }
    }

    /// Creates a Color from an ABGR decimal value.
    pub fn from_abgr_packed(val: u32) -> Color {
Color {
            r: (val & 0xFF) as u8 ,
            g: ((val >> 8) & 0xFF) as u8,
            b: ((val >> 16) & 0xFF) as u8,
            a: ((val >> 24) & 0xFF) as u8,
        }
    }

    /// Converts the color to an ABGR decimal value.
    pub fn as_decimal(&self) -> u32 {
        ((self.a as u32) << 24 | (self.b as u32) << 16 | (self.g as u32) << 8 | self.r  as u32)
    }

    /// Creates a tuple of (r, g, b) values from self.
    pub fn as_rgba(&self) -> (u8, u8, u8, u8) {
        
            (self.r, self.g, self.b, self.a)
        
    }

    /// Formats self as an RGBA hexadecimal value.
    pub fn as_hexstring(&self) -> String {
        format!("{:0>8X}", self.as_decimal().swap_bytes())
    }
}

impl From<(u8, u8, u8, u8)> for Color {
    fn from(rgb: (u8, u8, u8, u8)) -> Color {
        Color::new(rgb.0, rgb.1, rgb.2, rgb.3)
    }
}

impl From<u32> for Color {
    fn from(n: u32) -> Color {
        Color::from_abgr_packed(n)
    }
}

impl From<Color> for u32 {
    fn from(col: Color) -> u32 {
        col.as_decimal()
    }
}

impl From<Color> for (u8, u8, u8, u8) {
    fn from(col: Color) -> (u8, u8, u8, u8) {
        col.as_rgba()
    }
}
