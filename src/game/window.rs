//! Windowing magic.

pub mod win32;

use crate::input::{Key, MouseButton};
use std::slice;

#[cfg(target_os = "windows")]
use win32 as platform;

#[derive(Debug)]
pub enum Event {
    Resize(u32, u32),
    KeyboardDown(Key),
    KeyboardUp(Key),
    MouseButtonDown(MouseButton),
    MouseButtonUp(MouseButton),
    MouseWheelUp,
    MouseWheelDown,
    MouseMove(i32, i32),
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
    fn close_requested(&self) -> bool;
    fn request_close(&mut self);

    fn process_events<'a>(&'a mut self) -> slice::Iter<'a, Event>;

    fn set_style(&mut self, style: Style);
    fn set_visible(&mut self, visible: bool);
}

impl Window {
    /// Creates a new Window, invisible by default.
    pub fn new(width: u32, height: u32, title: &str) -> Result<Self, String> {
        Ok(Self(Box::new(platform::WindowImpl::new(width, height, title)?)))
    }

    pub fn close_requested(&self) -> bool {
        self.0.close_requested()
    }

    pub fn request_close(&mut self) {
        self.0.request_close()
    }

    pub fn process_events<'a>(&'a mut self) -> slice::Iter<'a, Event> {
        self.0.process_events()
    }

    pub fn set_style(&mut self, style: Style) {
        self.0.set_style(style)
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.0.set_visible(visible)
    }
}
