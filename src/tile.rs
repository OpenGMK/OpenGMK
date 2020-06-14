use serde::{Deserialize, Serialize};

/// An instance of a background tile.
#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct Tile {
    /// The tile's x position in the room
    pub x: f64,

    /// The tile's y position in the room
    pub y: f64,

    /// Index of the background which this tile will draw a section of
    pub background_index: i32,

    /// The x coordinate to draw from in the background image
    pub tile_x: u32,

    /// The y coordinate to draw from in the background image
    pub tile_y: u32,

    /// Width of the tile, in both the background image and the room
    pub width: u32,

    /// Height of the tile, in both the background image and the room
    pub height: u32,

    /// Depth of this tile in the room
    pub depth: i32,

    /// Unique ID of this tile - tile IDs are above 10,000,000
    pub id: usize,

    /// Alpha value of this tile, from 0.0 (invisible) to 1.0 (opaque)
    pub alpha: f64,

    /// Colour blend value of this tile
    pub blend: i32,

    /// xscale with which to draw this tile
    pub xscale: f64,

    /// yscale with which to draw this tile
    pub yscale: f64,

    /// Whether this tile will be drawn
    pub visible: bool,
}
