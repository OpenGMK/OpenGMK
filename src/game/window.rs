//! Windowing magic.

pub mod win32;

use std::ops::Deref;

#[cfg(target_os = "windows")]
use win32 as platform;

pub enum Event {
    Nothing,
}

pub enum Style {
    /// Regular non-resizable decorated window (minimize, close buttons).
    Regular,

    /// Same as Regular but with additional resizability and a maximize button.
    Resizable,

    /// Same as Regular except no buttons in the title bar.
    Undecorated,

    /// Borderless window.
    Borderless,

    /// Borderless fullscreen mode.
    BorderlessFullscreen,
}

pub struct Window(pub Box<dyn WindowTrait>);

pub trait WindowTrait {
    fn set_style(&self, style: Style);
    fn set_visible(&self, visible: bool);
}

impl Deref for Window {
    type Target = dyn WindowTrait;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

impl Window {
    /// Creates a new Window, invisible by default.
    pub fn new(width: u32, height: u32, title: &str) -> Result<Self, String> {
        Ok(Self(Box::new(platform::WindowImpl::new(width, height, title)?)))
    }
}
