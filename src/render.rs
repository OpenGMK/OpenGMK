//! Game rendering functionality

pub mod opengl;

use crate::atlas::AtlasBuilder;

pub trait Renderer {
    /// Stores & uploads atlases to the GPU.
    /// This function is for initializing, and should be called only once.
    ///
    /// Returns a handle to each inserted texture (in insertion order).
    fn upload_atlases(&mut self, atl: AtlasBuilder) -> Result<(), String>;

    /// Returns the max texture size the GPU can hold.
    fn max_gpu_texture_size(&self) -> usize;
}

pub struct Texture(usize);

impl From<usize> for Texture {
    fn from(n: usize) -> Self {
        Texture(n)
    }
}
