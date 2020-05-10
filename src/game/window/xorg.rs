#![cfg(target_os = "linux")]

use super::{Cursor, Event, Style, WindowBuilder, WindowTrait};
use std::{ffi, ptr, slice};
use x11::xlib;

pub struct WindowImpl {
    pub close_requested: bool,
    pub inner_size: (u32, u32),
}

impl WindowImpl {
    pub fn new(builder: &WindowBuilder) -> Result<Self, String> {
        unsafe {
            let display = xlib::XOpenDisplay(ptr::null());
            if display.is_null() {
                return Err("xlib::XOpenDisplay failed".into())
            }

            let screen = xlib::XDefaultScreen(display);
            let root = xlib::XRootWindow(display, screen);

            let mut attributes: xlib::XSetWindowAttributes = std::mem::MaybeUninit::uninit().assume_init();
            //attributes.background_pixel = xlib::XWhitePixel(display, screen);

            let window = xlib::XCreateWindow(
                display,
                root,
                100,
                100,
                builder.size.0,
                builder.size.1,
                0,
                0,
                xlib::InputOutput as _,
                ptr::null_mut(),
                xlib::CWBackPixel,
                &mut attributes,
            );

            let title = ffi::CString::new(builder.title.clone()).unwrap();
            xlib::XStoreName(display, window, title.as_ptr() as *mut _);

            xlib::XMapWindow(display, window);

            Ok(Self { close_requested: false, inner_size: builder.size })
        }
    }
}

impl WindowTrait for WindowImpl {
    fn close_requested(&self) -> bool {
        self.close_requested
    }

    fn set_close_requested(&mut self, value: bool) {
        self.close_requested = value
    }

    fn get_inner_size(&self) -> (u32, u32) {
        self.inner_size
    }

    fn process_events<'a>(&'a mut self) -> slice::Iter<'a, Event> {
        todo!()
    }

    fn resize(&mut self, width: u32, height: u32) {
        todo!()
    }

    fn get_cursor(&self) -> Cursor {
        todo!()
    }

    fn set_cursor(&mut self, cursor: Cursor) {
        todo!()
    }

    fn set_style(&mut self, style: Style) {
        todo!()
    }

    fn get_title(&self) -> &str {
        todo!()
    }

    fn set_title(&mut self, title: &str) {
        todo!()
    }

    fn get_visible(&self) -> bool {
        todo!()
    }

    fn set_visible(&mut self, visible: bool) {
        todo!()
    }

    fn window_handle(&self) -> usize {
        todo!()
    }
}
