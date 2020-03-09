/// An instance of a view in a room
#[derive(Clone, Copy)]
pub struct View {
    /// Whether to draw this view
    pub visible: bool,

    /// Region of the room this view is looking at - x coordinate
    pub source_x: i32,

    /// Region of the room this view is looking at - y coordinate
    pub source_y: i32,

    /// Region of the room this view is looking at - width
    pub source_w: u32,

    /// Region of the room this view is looking at - height
    pub source_h: u32,

    /// Port on screen to draw this view to - x coordinate
    pub port_x: i32,

    /// Port on screen to draw this view to - y coordinate
    pub port_y: i32,

    /// Port on screen to draw this view to - width
    pub port_w: u32,

    /// Port on screen to draw this view to - height
    pub port_h: u32,

    /// Target object ID this view should follow
    pub follow_target: i32,

    /// Horizontal border within which to follow an instance
    pub follow_hborder: i32,

    /// Vertical border within which to follow an instance
    pub follow_vborder: i32,

    /// Horizontal speed with which to follow an instance
    pub follow_hspeed: i32,

    /// Vertical speed with which to follow an instance
    pub follow_vspeed: i32,
}