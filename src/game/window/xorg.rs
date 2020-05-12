#![cfg(target_os = "linux")]

use super::{Cursor, Event, Style, WindowBuilder, WindowTrait};
use std::{any::Any, ffi, ptr, slice};
use x11::xlib;

pub struct WindowImpl {
    pub display: *mut xlib::Display,
    pub window_id: u64,
    pub screen_id: i32,
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

            let screen_id = xlib::XDefaultScreen(display);
            let root = xlib::XRootWindow(display, screen_id);

            let mut attributes: xlib::XSetWindowAttributes = std::mem::MaybeUninit::uninit().assume_init();
            attributes.background_pixel = xlib::XWhitePixel(display, screen_id);

            let window_id = xlib::XCreateWindow(
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
            xlib::XStoreName(display, window_id, title.as_ptr() as *mut _);

            xlib::XMapWindow(display, window_id);

            Ok(Self { display, window_id, screen_id, close_requested: false, inner_size: builder.size })
        }
    }
}

impl WindowTrait for WindowImpl {
    fn as_any(&self) -> &dyn Any {
        self
    }

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

    fn resize(&mut self, _width: u32, _height: u32) {
        todo!()
    }

    fn get_cursor(&self) -> Cursor {
        todo!()
    }

    fn set_cursor(&mut self, _cursor: Cursor) {
        todo!()
    }

    fn set_style(&mut self, _style: Style) {
        todo!()
    }

    fn get_title(&self) -> &str {
        todo!()
    }

    fn set_title(&mut self, _title: &str) {
        todo!()
    }

    fn get_visible(&self) -> bool {
        todo!()
    }

    fn set_visible(&mut self, _visible: bool) {
        todo!()
    }

    fn window_handle(&self) -> usize {
        todo!()
    }
}
