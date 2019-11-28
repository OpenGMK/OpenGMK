//! OpenGL bindings & functions
//! 
//! The raw bindings are generated at build time, see build.rs

#![allow(clippy::all)]
pub mod gl {
    include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));
}
