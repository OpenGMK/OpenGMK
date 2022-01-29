use crate::{math::Real, types::ID};
use serde::{Deserialize, Serialize};
use std::cell::Cell;

/// An instance of a background tile.
#[derive(Clone, Serialize, Deserialize)]
pub struct Tile {
    /// The tile's x position in the room
    pub x: Cell<Real>,

    /// The tile's y position in the room
    pub y: Cell<Real>,

    /// Index of the background which this tile will draw a section of
    pub background_index: Cell<i32>,

    /// The x coordinate to draw from in the background image
    pub tile_x: Cell<i32>,

    /// The y coordinate to draw from in the background image
    pub tile_y: Cell<i32>,

    /// Width of the tile, in both the background image and the room
    pub width: Cell<i32>,

    /// Height of the tile, in both the background image and the room
    pub height: Cell<i32>,

    /// Depth of this tile in the room
    pub depth: Cell<Real>,

    /// Unique ID of this tile - tile IDs are above 10,000,000
    pub id: Cell<ID>,

    /// Alpha value of this tile, from 0.0 (invisible) to 1.0 (opaque)
    pub alpha: Cell<Real>,

    /// Colour blend value of this tile
    pub blend: Cell<i32>,

    /// xscale with which to draw this tile
    pub xscale: Cell<Real>,

    /// yscale with which to draw this tile
    pub yscale: Cell<Real>,

    /// Whether this tile will be drawn
    pub visible: Cell<bool>,
}
