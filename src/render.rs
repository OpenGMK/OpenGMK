//! Game rendering functionality

mod opengl;

use crate::{atlas::AtlasBuilder, game::window::Window, types::Colour};
use std::{any::Any, io, path::PathBuf};

// Re-export for more logical module pathing
pub use crate::atlas::AtlasRef;

pub struct Renderer(Box<dyn RendererTrait>);
pub trait RendererTrait {
    fn as_any(&self) -> &dyn Any;
    fn max_texture_size(&self) -> u32;

    fn set_clear_colour(&mut self, colour: Colour);

    fn swap_buffers(&self);
    fn set_swap_interval(&self, n: Option<u32>) -> bool;
}

pub struct RendererOptions {
    pub size: (u32, u32),
    pub vsync: bool,

    pub clear_colour: Colour,
}

impl Renderer {
    pub fn new(backend: (), options: &RendererOptions, window: &Window) -> Result<Self, String> {
        Ok(Self(Box::new(match backend {
            () => opengl::RendererImpl::new(options, window)?,
        })))
    }

    pub fn swap_buffers(&self) {
        self.0.swap_buffers();
    }

    pub fn swap_interval(&self, n: Option<u32>) -> bool {
        self.0.set_swap_interval(n)
    }
}

// pub trait Renderer {
//     /// Stores & uploads atlases to the GPU.
//     /// This function is for initializing, and should be called only once.
//     ///
//     /// Returns a handle to each inserted texture (in insertion order).
//     fn upload_atlases(&mut self, atl: AtlasBuilder) -> Result<(), String>;

//     /// Dumps atlases to filepaths provided by `Fn(index: usize) -> PathBuf`.
//     fn dump_atlases(&self, path: fn(usize) -> PathBuf) -> io::Result<()>;

//     /// Makes the renderer current. OpenGL global mutable spam C-nile design bullshit. Holy fuck.
//     fn set_current(&self) -> bool;
//     fn is_current(&self) -> bool;

//     /// Updates the view (source rectangle, angle and viewport) to use when drawing things.
//     fn set_view(
//         &mut self,
//         width: u32,
//         height: u32,
//         unscaled_width: u32,
//         unscaled_height: u32,
//         src_x: i32,
//         src_y: i32,
//         src_w: i32,
//         src_h: i32,
//         src_angle: f64,
//         port_x: i32,
//         port_y: i32,
//         port_w: i32,
//         port_h: i32,
//     );

//     /// Draws a sprite to the screen. Parameters are similar to those of GML's draw_sprite_ext.
//     fn draw_sprite(
//         &mut self,
//         texture: &AtlasRef,
//         x: i32,
//         y: i32,
//         xscale: f64,
//         yscale: f64,
//         angle: f64,
//         colour: i32,
//         alpha: f64,
//     );

//     /// Draws part of a sprite to a screen. Useful for drawing background tiles, font characters etc.
//     fn draw_sprite_partial(
//         &mut self,
//         texture: &AtlasRef,
//         part_x: i32,
//         part_y: i32,
//         part_w: i32,
//         part_h: i32,
//         x: i32,
//         y: i32,
//         xscale: f64,
//         yscale: f64,
//         angle: f64,
//         colour: i32,
//         alpha: f64,
//     );

//     /// Draws a sprite to the screen, tiled rightwards and downwards, starting just below 0,0 and
//     /// continuing until the given boundaries are exceeded.
//     fn draw_sprite_tiled(
//         &mut self,
//         texture: &AtlasRef,
//         x: f64,
//         y: f64,
//         xscale: f64,
//         yscale: f64,
//         colour: i32,
//         alpha: f64,
//         tile_end_x: Option<f64>,
//         tile_end_y: Option<f64>,
//     );

//     /// Updates the screen. Should be called only after drawing everything that should be in the current frame.
//     fn finish(&mut self, width: u32, height: u32);
// }
