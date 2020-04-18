use winit::{
    event_loop::EventLoop,
    platform::unix::{EventLoopExtUnix, WindowBuilderExtUnix, WindowExtUnix},
    window::{Window, WindowBuilder},
};

#[allow(clippy::all)]
pub mod glx {
    include!(concat!(env!("OUT_DIR"), "/glx_bindings.rs"));
}

pub struct PlatformGL {
    dpy: *mut xlib::Display,
    window_id: c_ulong,
}

#[link(name = "GL")]
extern "C" {
    fn glXChooseFBConfig(
        display: *mut xlib::Display,
        screen: c_int,
        attr: *const c_int,
        el: *mut c_int,
    ) -> *const glx::types::GLXFBConfig;
    fn glXGetProcAddressARB(name: *const c_char) -> *const c_void;
    fn glXMakeCurrent(display: *mut xlib::Display, draw: c_ulong, ctx: *const c_void) -> xlib::Bool;
}

use super::gl;
use std::{mem, os::raw::*, ptr};
use x11::xlib;

pub fn setup(window: &Window) -> PlatformGL {
    let dpy = window.xlib_display().unwrap();
    let window_id = window.xlib_window().unwrap();
    let screen_id = window.xlib_screen_id().unwrap();
    unsafe {
        let mut nelements = 0i32;
        let fb_config = glXChooseFBConfig(dpy as *mut _, screen_id, ptr::null(), &mut nelements);
        let cctx = glXGetProcAddressARB(b"glXCreateContextAttribsARB\0".as_ptr() as *const _);

        #[rustfmt::skip]
        static CCTX_ATTRIBS: &[u32] = &[
            glx::CONTEXT_MAJOR_VERSION_ARB, 3,
            glx::CONTEXT_MINOR_VERSION_ARB, 3,
            glx::CONTEXT_FLAGS_ARB,         glx::CONTEXT_FORWARD_COMPATIBLE_BIT_ARB,
            glx::CONTEXT_PROFILE_MASK_ARB,  glx::CONTEXT_CORE_PROFILE_BIT_ARB,
            0, // END
        ];

        let cctx: extern "C" fn(
            *mut xlib::Display,
            glx::types::GLXFBConfig,
            glx::types::GLXContext,
            xlib::Bool,
            *const u32,
        ) -> *const c_void = mem::transmute(cctx);
        let ctx = cctx(dpy as *mut xlib::Display, *fb_config, ptr::null(), 1, &CCTX_ATTRIBS[0] as *const _);
        assert!(!ctx.is_null());
        let res = glXMakeCurrent(dpy as *mut xlib::Display, window_id, ctx);
        assert_ne!(res, xlib::False);

        static mut FFI_BUF: Vec<u8> = Vec::new();
        FFI_BUF.reserve(64);
        gl::load_with(|name| {
            FFI_BUF.clear();
            FFI_BUF.extend_from_slice(name.as_bytes());
            FFI_BUF.push(0);

            // big todo here
            glXGetProcAddressARB(FFI_BUF.as_ptr() as *const c_char)
        });

        glx::load_with(|name| {
            FFI_BUF.clear();
            FFI_BUF.extend_from_slice(name.as_bytes());
            FFI_BUF.push(0);

            // big todo here
            glXGetProcAddressARB(FFI_BUF.as_ptr() as *const c_char)
        });

        PlatformGL { dpy: dpy as *mut _, window_id }
    }
}

pub fn swap_interval(plat: &PlatformGL, n: u32) {
    unsafe {
        glx::SwapIntervalEXT(plat.dpy as *mut _, plat.window_id, n as _);
    }
}

pub fn swap_buffers(plat: &PlatformGL) {
    unsafe {
        glx::SwapBuffers(plat.dpy as *mut _, plat.window_id);
    }
}
