/// An instance of a background in a room
pub struct Background {
    /// Whether to draw this background
    pub visible: bool,

    /// Whether this background is a foreground (yep)
    /// Backgrounds are drawn before everything else, foregrounds are drawn after everything else
    pub is_foreground: bool,

    /// ID of Background asset to draw
    pub background_id: i32,

    /// X offset from 0 at which to draw this background
    pub x_offset: f64,

    /// Y offset from 0 at which to draw this background
    pub y_offset: f64,

    /// Whether to draw this background repeatedly to cover the whole screen in X axis
    pub tile_horizontal: bool,

    /// Whether to draw this background repeatedly to cover the whole screen in Y axis
    pub tile_vertical: bool,

    /// Speed of x_offset increase per frame
    pub hspeed: f64,

    /// Speed of y_offset increase per frame
    pub vspeed: f64,

    /// Whether to stretch the background image to fit the screen
    pub stretch: bool,
}