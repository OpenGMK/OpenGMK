mod wgl;

#[cfg(target_os = "windows")]
use wgl as imp;

use crate::{
    game::window::Window,
    render::{RendererOptions, RendererTrait},
    types::Colour,
};
use std::any::Any;

/// Auto-generated OpenGL bindings from gl_generator
pub mod gl {
    #![allow(clippy::all)]
    include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));
}
use gl::types::GLint;

pub struct RendererImpl {
    clear_colour: Colour,
    imp: imp::PlatformImpl,
}

impl RendererImpl {
    pub fn new(options: &RendererOptions, window: &Window) -> Result<Self, String> {
        todo!()
    }
}

impl RendererTrait for RendererImpl {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn max_texture_size(&self) -> u32 {
        let mut size: GLint = 0;
        unsafe {
            gl::GetIntegerv(gl::MAX_TEXTURE_SIZE, &mut size);
        }
        size.max(0) as u32
    }

    fn set_clear_colour(&mut self, colour: Colour) {
        self.clear_colour = colour;
    }

    fn swap_buffers(&self) {
        unsafe { self.imp.swap_buffers() }
    }

    fn set_swap_interval(&self, n: Option<u32>) -> bool {
        unsafe { self.imp.set_swap_interval(n.unwrap_or(0)) }
    }
}
