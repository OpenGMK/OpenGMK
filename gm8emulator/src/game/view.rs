use crate::{math::Real, util};
use serde::{Deserialize, Serialize};

/// An instance of a view in a room
#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct View {
    /// Whether to draw this view
    pub visible: bool,

    /// Region of the room this view is looking at - x coordinate
    pub source_x: i32,

    /// Region of the room this view is looking at - y coordinate
    pub source_y: i32,

    /// Region of the room this view is looking at - width
    pub source_w: i32,

    /// Region of the room this view is looking at - height
    pub source_h: i32,

    /// Port on screen to draw this view to - x coordinate
    pub port_x: i32,

    /// Port on screen to draw this view to - y coordinate
    pub port_y: i32,

    /// Port on screen to draw this view to - width
    pub port_w: u32,

    /// Port on screen to draw this view to - height
    pub port_h: u32,

    /// Angle to which this view is rotated on the screen
    pub angle: Real,

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

impl View {
    /// Returns bool indicating whether the given point is inside this view's port on screen
    pub fn contains_point(&self, x: i32, y: i32) -> bool {
        x >= self.port_x
            && y >= self.port_y
            && x < (self.port_x + self.port_w as i32)
            && y < (self.port_y + self.port_h as i32)
    }

    /// Transforms a point on screen to a point relative to this view in room-space
    pub fn transform_point(&self, x: i32, y: i32) -> (i32, i32) {
        let src_x = f64::from(self.source_x);
        let src_y = f64::from(self.source_y);
        let src_w = f64::from(self.source_w);
        let src_h = f64::from(self.source_h);
        let mut x = src_x + (src_w * f64::from(x - self.port_x) / f64::from(self.port_w));
        let mut y = src_y + (src_h * f64::from(y - self.port_y) / f64::from(self.port_h));
        let angle = self.angle.to_radians();
        util::rotate_around(
            &mut x,
            &mut y,
            src_x + (src_w / 2.0),
            src_y + (src_h / 2.0),
            angle.sin().into(),
            angle.cos().into(),
        );
        (Real::from(x).round().to_i32(), Real::from(y).round().to_i32())
    }
}
