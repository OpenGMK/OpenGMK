//! Game rendering functionality

pub mod opengl;

use crate::atlas::AtlasBuilder;

pub trait Renderer {
    /// Stores & uploads atlases to the GPU.
    /// This function is for initializing, and should be called only once.
    ///
    /// Returns a handle to each inserted texture (in insertion order).
    fn process_atlases(&mut self, atl: AtlasBuilder) -> Vec<Texture>;
}
pub struct Texture(usize);
