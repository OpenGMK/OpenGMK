//! Game rendering functionality

pub mod opengl;

use crate::{atlas::AtlasBuilder, types::Colour};
use std::{io, path::PathBuf};

// Re-export for more logical module pathing
pub use crate::atlas::AtlasRef;

pub trait Renderer {
    /// Stores & uploads atlases to the GPU.
    /// This function is for initializing, and should be called only once.
    ///
    /// Returns a handle to each inserted texture (in insertion order).
    fn upload_atlases(&mut self, atl: AtlasBuilder) -> Result<(), String>;

    /// Dumps atlases to filepaths provided by `Fn(index: usize) -> PathBuf`.
    fn dump_atlases(&self, path: fn(usize) -> PathBuf) -> io::Result<()>;

    /// Returns the max texture size the GPU can hold.
    fn max_gpu_texture_size(&self) -> usize;

    /// Used to set the swap interval. Pass 0 to turn V-Sync off.
    fn swap_interval(&mut self, n: u32);

    /// Sets the colour (RGB) which will be used to clear the background rectangle after using set_view().
    /// If None is provided, the background will not be cleared at all.
    fn set_background_colour(&mut self, colour: Option<Colour>);

    /// Updates the view (source rectangle, angle and viewport) to use when drawing things.
    fn set_view(
        &mut self,
        width: u32,
        height: u32,
        unscaled_width: u32,
        unscaled_height: u32,
        src_x: i32,
        src_y: i32,
        src_w: i32,
        src_h: i32,
        src_angle: f64,
        port_x: i32,
        port_y: i32,
        port_w: i32,
        port_h: i32,
    );

    /// Draws a sprite to the screen. Parameters are similar to those of GML's draw_sprite_ext.
    fn draw_sprite(
        &mut self,
        texture: &AtlasRef,
        x: i32,
        y: i32,
        xscale: f64,
        yscale: f64,
        angle: f64,
        colour: i32,
        alpha: f64,
    );

    /// Draws part of a sprite to a screen. Useful for drawing background tiles, font characters etc.
    fn draw_sprite_partial(
        &mut self,
        texture: &AtlasRef,
        part_x: i32,
        part_y: i32,
        part_w: i32,
        part_h: i32,
        x: i32,
        y: i32,
        xscale: f64,
        yscale: f64,
        angle: f64,
        colour: i32,
        alpha: f64,
    );

    /// Draws a sprite to the screen, tiled rightwards and downwards, starting just below 0,0 and
    /// continuing until the given boundaries are exceeded.
    fn draw_sprite_tiled(
        &mut self,
        texture: &AtlasRef,
        x: f64,
        y: f64,
        xscale: f64,
        yscale: f64,
        colour: i32,
        alpha: f64,
        tile_end_x: f64,
        tile_end_y: f64,
    );

    /// Updates the screen. Should be called only after drawing everything that should be in the current frame.
    fn finish(&mut self, width: u32, height: u32);
}

pub struct RendererOptions<'a> {
    pub title: &'a str,
    pub size: (u32, u32),
    pub icons: Vec<(Vec<u8>, u32, u32)>,
    pub global_clear_colour: Colour,
    pub resizable: bool,
    pub on_top: bool,
    pub decorations: bool,
    pub fullscreen: bool,
    pub vsync: bool,
}
