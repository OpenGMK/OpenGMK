use crate::math::Real;
use serde::{Deserialize, Serialize};

/// An instance of a background in a room
#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct Background {
    /// Whether to draw this background
    pub visible: bool,

    /// Whether this background is a foreground (yep)
    /// Backgrounds are drawn before everything else, foregrounds are drawn after everything else
    pub is_foreground: bool,

    /// ID of Background asset to draw
    pub background_id: i32,

    /// X offset from 0 at which to draw this background
    pub x_offset: Real,

    /// Y offset from 0 at which to draw this background
    pub y_offset: Real,

    /// Whether to draw this background repeatedly to cover the whole screen in X axis
    pub tile_horizontal: bool,

    /// Whether to draw this background repeatedly to cover the whole screen in Y axis
    pub tile_vertical: bool,

    /// Speed of x_offset increase per frame
    pub hspeed: Real,

    /// Speed of y_offset increase per frame
    pub vspeed: Real,

    /// X-axis scale factor
    pub xscale: Real,

    /// Y-axis scale factor
    pub yscale: Real,

    /// Colour blend value to draw this background with
    pub blend: i32,

    /// Alpha-blend value to draw this background with
    pub alpha: Real,
}
