//! Game rendering functionality

mod opengl;

use crate::{atlas::AtlasBuilder, game::window::Window, types::Colour};
use std::any::Any;

// Re-export for more logical module pathing
pub use crate::atlas::AtlasRef;

pub struct Renderer(Box<dyn RendererTrait>);

pub trait RendererTrait {
    fn as_any(&self) -> &dyn Any;
    fn max_texture_size(&self) -> u32;
    fn push_atlases(&mut self, atl: AtlasBuilder) -> Result<(), String>;

    fn set_background_colour(&mut self, colour: Option<Colour>);
    fn set_swap_interval(&self, n: Option<u32>) -> bool;

    fn draw_sprite(&mut self, tex: &AtlasRef, x: f64, y: f64, xs: f64, ys: f64, ang: f64, col: i32, alpha: f64);
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
    fn flush_queue(&mut self);
    fn finish(&mut self, width: u32, height: u32);

    fn get_pixels(&self, w: i32, h: i32) -> Box<[u8]>;
    fn draw_pixels(&mut self, rgb: Box<[u8]>, w: i32, h: i32);

    fn draw_sprite_partial(
        &mut self,
        texture: &AtlasRef,
        mut part_x: i32,
        mut part_y: i32,
        part_w: i32,
        part_h: i32,
        mut x: f64,
        mut y: f64,
        xscale: f64,
        yscale: f64,
        angle: f64,
        colour: i32,
        alpha: f64,
    ) {
        if part_x < 0 {
            x -= f64::from(part_x);
            part_x = 0;
        }
        if part_y < 0 {
            y -= f64::from(part_y);
            part_y = 0;
        }
        let part_w = (part_x + part_w).min(texture.w) - part_x;
        let part_h = (part_y + part_h).min(texture.h) - part_y;

        if part_w >= 0 && part_h >= 0 {
            self.draw_sprite(
                &AtlasRef {
                    atlas_id: texture.atlas_id,
                    w: part_w,
                    h: part_h,
                    x: texture.x + part_x,
                    y: texture.y + part_y,
                    origin_x: 0.0,
                    origin_y: 0.0,
                },
                x,
                y,
                xscale,
                yscale,
                angle,
                colour,
                alpha,
            )
        }
    }
    fn draw_sprite_tiled(
        &mut self,
        texture: &AtlasRef,
        mut x: f64,
        mut y: f64,
        xscale: f64,
        yscale: f64,
        colour: i32,
        alpha: f64,
        tile_end_x: Option<f64>,
        tile_end_y: Option<f64>,
    ) {
        let width = f64::from(texture.w) * xscale;
        let height = f64::from(texture.h) * yscale;

        if tile_end_x.is_some() {
            x = x.rem_euclid(width);
            if x > 0.0 {
                x -= width;
            }
        }
        if tile_end_y.is_some() {
            y = y.rem_euclid(height);
            if y > 0.0 {
                y -= height;
            }
        }

        let start_x = x;

        loop {
            loop {
                self.draw_sprite(texture, x, y, xscale, yscale, 0.0, colour, alpha);
                x += width;
                match tile_end_x {
                    Some(end_x) if x < end_x => (),
                    _ => break,
                }
            }
            x = start_x;
            y += height;
            match tile_end_y {
                Some(end_y) if y < end_y => (),
                _ => break,
            }
        }
    }
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

    pub fn max_texture_size(&self) -> u32 {
        self.0.max_texture_size()
    }

    pub fn push_atlases(&mut self, atl: AtlasBuilder) -> Result<(), String> {
        self.0.push_atlases(atl)
    }

    pub fn set_background_colour(&mut self, colour: Option<Colour>) {
        self.0.set_background_colour(colour)
    }

    pub fn set_swap_interval(&self, n: Option<u32>) -> bool {
        self.0.set_swap_interval(n)
    }

    pub fn draw_sprite(
        &mut self,
        texture: &AtlasRef,
        x: f64,
        y: f64,
        xscale: f64,
        yscale: f64,
        angle: f64,
        colour: i32,
        alpha: f64,
    ) {
        self.0.draw_sprite(texture, x, y, xscale, yscale, angle, colour, alpha)
    }

    pub fn set_view(
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
    ) {
        self.0.set_view(
            width,
            height,
            unscaled_width,
            unscaled_height,
            src_x,
            src_y,
            src_w,
            src_h,
            src_angle,
            port_x,
            port_y,
            port_w,
            port_h,
        )
    }

    pub fn draw_sprite_partial(
        &mut self,
        texture: &AtlasRef,
        part_x: i32,
        part_y: i32,
        part_w: i32,
        part_h: i32,
        x: f64,
        y: f64,
        xscale: f64,
        yscale: f64,
        angle: f64,
        colour: i32,
        alpha: f64,
    ) {
        self.0.draw_sprite_partial(texture, part_x, part_y, part_w, part_h, x, y, xscale, yscale, angle, colour, alpha)
    }

    pub fn draw_sprite_tiled(
        &mut self,
        texture: &AtlasRef,
        x: f64,
        y: f64,
        xscale: f64,
        yscale: f64,
        colour: i32,
        alpha: f64,
        tile_end_x: Option<f64>,
        tile_end_y: Option<f64>,
    ) {
        self.0.draw_sprite_tiled(texture, x, y, xscale, yscale, colour, alpha, tile_end_x, tile_end_y)
    }

    fn get_pixels(&self, w: i32, h: i32) -> Box<[u8]> {
        self.0.get_pixels(w, h)
    }

    fn draw_pixels(&mut self, rgb: Box<[u8]>, w: i32, h: i32) {
        self.0.draw_pixels(rgb, w, h)
    }

    pub fn flush_queue(&mut self) {
        self.0.flush_queue()
    }

    pub fn finish(&mut self, width: u32, height: u32) {
        self.0.finish(width, height)
    }
}

/// Multiply two mat4's together
fn mat4mult(m1: [f32; 16], m2: [f32; 16]) -> [f32; 16] {
    [
        (m1[0] * m2[0]) + (m1[1] * m2[4]) + (m1[2] * m2[8]) + (m1[3] * m2[12]),
        (m1[0] * m2[1]) + (m1[1] * m2[5]) + (m1[2] * m2[9]) + (m1[3] * m2[13]),
        (m1[0] * m2[2]) + (m1[1] * m2[6]) + (m1[2] * m2[10]) + (m1[3] * m2[14]),
        (m1[0] * m2[3]) + (m1[1] * m2[7]) + (m1[2] * m2[11]) + (m1[3] * m2[15]),
        (m1[4] * m2[0]) + (m1[5] * m2[4]) + (m1[6] * m2[8]) + (m1[7] * m2[12]),
        (m1[4] * m2[1]) + (m1[5] * m2[5]) + (m1[6] * m2[9]) + (m1[7] * m2[13]),
        (m1[4] * m2[2]) + (m1[5] * m2[6]) + (m1[6] * m2[10]) + (m1[7] * m2[14]),
        (m1[4] * m2[3]) + (m1[5] * m2[7]) + (m1[6] * m2[11]) + (m1[7] * m2[15]),
        (m1[8] * m2[0]) + (m1[9] * m2[4]) + (m1[10] * m2[8]) + (m1[11] * m2[12]),
        (m1[8] * m2[1]) + (m1[9] * m2[5]) + (m1[10] * m2[9]) + (m1[11] * m2[13]),
        (m1[8] * m2[2]) + (m1[9] * m2[6]) + (m1[10] * m2[10]) + (m1[11] * m2[14]),
        (m1[8] * m2[3]) + (m1[9] * m2[7]) + (m1[10] * m2[11]) + (m1[11] * m2[15]),
        (m1[12] * m2[0]) + (m1[13] * m2[4]) + (m1[14] * m2[8]) + (m1[15] * m2[12]),
        (m1[12] * m2[1]) + (m1[13] * m2[5]) + (m1[14] * m2[9]) + (m1[15] * m2[13]),
        (m1[12] * m2[2]) + (m1[13] * m2[6]) + (m1[14] * m2[10]) + (m1[15] * m2[14]),
        (m1[12] * m2[3]) + (m1[13] * m2[7]) + (m1[14] * m2[11]) + (m1[15] * m2[15]),
    ]
}
