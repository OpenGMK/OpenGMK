mod windows;

#[cfg(windows)]
pub use windows::{disk_free, disk_size, display_colour_depth, display_frequency, display_height, display_width};
