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
    pub visible: bool,
    pub events: Vec<Event>,
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

            Ok(Self {
                display,
                window_id,
                screen_id,
                close_requested: false,
                inner_size: builder.size,
                visible: false,
                events: Vec::with_capacity(8),
            })
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
        // TODO
        self.events.iter()
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.inner_size = (width, height);
        unsafe {
            xlib::XResizeWindow(self.display, self.window_id, width, height);
        }
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

    fn set_title(&mut self, title: &str) {
        unsafe {
            let title = ffi::CString::new(title).unwrap();
            xlib::XStoreName(self.display, self.window_id, title.as_ptr() as *mut _);
        }
    }

    fn get_visible(&self) -> bool {
        self.visible
    }

    fn set_visible(&mut self, visible: bool) {
        if self.visible != visible {
            unsafe {
                if visible {
                    xlib::XMapWindow(self.display, self.window_id);
                } else {
                    xlib::XUnmapWindow(self.display, self.window_id);
                }
            }
            self.visible = visible;
        }
    }

    fn window_handle(&self) -> usize {
        todo!()
    }
}
