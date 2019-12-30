//! Game rendering functionality

pub mod opengl;

use crate::atlas::AtlasBuilder;
use std::{io, path::PathBuf};

pub trait Renderer {
    /// Stores & uploads atlases to the GPU.
    /// This function is for initializing, and should be called only once.
    ///
    /// Returns a handle to each inserted texture (in insertion order).
    fn upload_atlases(&mut self, atl: AtlasBuilder) -> Result<(), String>;

    /// Dumps atlases to filepaths provided by `Fn(index: usize) -> PathBuf`.
    fn dump_atlases(&self, path: impl Fn(usize) -> PathBuf) -> io::Result<()>;

    /// Returns the max texture size the GPU can hold.
    fn max_gpu_texture_size(&self) -> usize;

    /// Indicates whether the window wants to close.
    fn should_close(&self) -> bool;

    /// Updates the screen with new drawings for the current frame.
    fn draw(&mut self);
}

pub struct Texture(usize);

impl From<usize> for Texture {
    fn from(n: usize) -> Self {
        Texture(n)
    }
}
