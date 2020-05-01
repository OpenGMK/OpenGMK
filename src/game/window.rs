//! Windowing magic.

pub mod win32;

#[cfg(target_os = "windows")]
use win32 as platform;

pub struct Window(pub platform::WindowImpl);
