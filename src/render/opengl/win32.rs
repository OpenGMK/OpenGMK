//! what do I put here

use super::gl;
use crate::game::window::win32::WindowImpl;
use std::{
    ffi::CStr,
    mem::{self, size_of, transmute},
    os::raw::{c_char, c_int, c_void},
    ptr,
};
use winapi::{
    shared::{
        minwindef::{BOOL, HINSTANCE},
        windef::{HDC, HGLRC},
    },
    um::{
        libloaderapi::{GetProcAddress, LoadLibraryA},
        wingdi::{
            wglCreateContext, wglDeleteContext, wglGetCurrentContext, wglGetProcAddress, wglMakeCurrent,
            ChoosePixelFormat, SetPixelFormat, SwapBuffers, PFD_DOUBLEBUFFER, PFD_DRAW_TO_WINDOW, PFD_MAIN_PLANE,
            PFD_SUPPORT_OPENGL, PFD_TYPE_RGBA, PIXELFORMATDESCRIPTOR,
        },
        winuser::GetDC,
    },
};

#[allow(clippy::all)]
pub mod wgl {
    include!(concat!(env!("OUT_DIR"), "/wgl_bindings.rs"));
}

pub struct PlatformGL {
    hdc: HDC,
    hglrc: HGLRC,
}

static PIXEL_FORMAT: PIXELFORMATDESCRIPTOR = PIXELFORMATDESCRIPTOR {
    nSize: size_of::<PIXELFORMATDESCRIPTOR>() as _,
    nVersion: 1,
    dwFlags: PFD_DRAW_TO_WINDOW | PFD_SUPPORT_OPENGL | PFD_DOUBLEBUFFER,
    iPixelType: PFD_TYPE_RGBA,
    cColorBits: 32,
    cRedBits: 0,
    cRedShift: 0,
    cGreenBits: 0,
    cGreenShift: 0,
    cBlueBits: 0,
    cBlueShift: 0,
    cAlphaBits: 0,
    cAlphaShift: 0,
    cAccumBits: 0,
    cAccumRedBits: 0,
    cAccumGreenBits: 0,
    cAccumBlueBits: 0,
    cAccumAlphaBits: 0,
    cDepthBits: 24,
    cStencilBits: 8,
    cAuxBuffers: 0,
    iLayerType: PFD_MAIN_PLANE,
    bReserved: 0,
    dwLayerMask: 0,
    dwVisibleMask: 0,
    dwDamageMask: 0,
};

#[rustfmt::skip]
/// Flags for wglCreateContextAttribsARB
static WGL_CCTX_ATTR_ARB: &[u32] = &[
    wgl::CONTEXT_MAJOR_VERSION_ARB, 3,
    wgl::CONTEXT_MINOR_VERSION_ARB, 3,
    wgl::CONTEXT_LAYER_PLANE_ARB,   0,
    wgl::CONTEXT_FLAGS_ARB,         wgl::CONTEXT_FORWARD_COMPATIBLE_BIT_ARB,
    wgl::CONTEXT_PROFILE_MASK_ARB,  wgl::CONTEXT_CORE_PROFILE_BIT_ARB,
    0, // END
];

/// HGLRC wglCreateContextAttribsARB(HDC hDC, HGLRC hShareContext, const int *attribList)
type WglCreateContextAttribsARBTy = extern "C" fn(HDC, HGLRC, *const c_int) -> HGLRC;

/// Loads an OpenGL function pointer.
/// Only works if there is a current OpenGL context.
unsafe fn load_gl_function(name: *const c_char, gl32_hi: HINSTANCE) -> *const c_void {
    let addr = wglGetProcAddress(name);
    match addr as isize {
        // All of these return values mean failure, as much as the docs say it's just NULL.
        // You load some of them like this, but only if wglGetProcAddress failed.
        // The ones that would are the legacy (2.0) functions, usually.
        -1 | 0 | 1 | 2 | 3 => GetProcAddress(gl32_hi, name) as *const c_void,
        _ => addr as *const c_void,
    }
}

impl PlatformGL {
    pub unsafe fn new(window: &WindowImpl) -> Self {
        // query device context, set up pixel format
        let device = GetDC(window.get_hwnd());
        let format = ChoosePixelFormat(device, &PIXEL_FORMAT);
        assert_ne!(format, 0, "couldn't find pixel format");
        let result = SetPixelFormat(device, format, &PIXEL_FORMAT);
        assert_ne!(result, 0, "couldn't set pixel format");

        // for 1.1 functions
        let gl32 = LoadLibraryA(b"opengl32.dll\0".as_ptr() as *const c_char);

        // create OpenGL context for opening ARB context
        let context = wglCreateContext(device);
        let result = wglMakeCurrent(device, context);
        assert_ne!(result, 0, "couldn't make new context current");

        static mut FFI_BUF: Vec<u8> = Vec::new();
        FFI_BUF.reserve(64);
        let loader_fn = |name: &str| {
            FFI_BUF.clear();
            FFI_BUF.extend_from_slice(name.as_bytes());
            FFI_BUF.push(0);
            load_gl_function(FFI_BUF.as_ptr() as *const c_char, gl32)
        };

        // create the real context! (TODO: this should be fine to avoid if it's already 3.2?)
        wgl::load_with(loader_fn);
        if !wgl::CreateContextAttribsARB::is_loaded() {
            panic!("wglCreateContextAttribsARB not loaded! Uh oh...");
        }
        let context2 = wgl::CreateContextAttribsARB(
            device as *const c_void,
            ptr::null_mut(),
            WGL_CCTX_ATTR_ARB as *const _ as *const c_int,
        ) as HGLRC;
        assert!(!context2.is_null(), "wglCreateContextAttribsARB returned null");
        wglMakeCurrent(device, context2);
        wglDeleteContext(context);

        // load gl function pointers
        gl::load_with(loader_fn);

        // debug print, whatever
        let ver_str = CStr::from_ptr(gl::GetString(gl::VERSION) as *const _).to_str().unwrap();
        println!("OpenGL Version: {}", ver_str);
        let vendor_str = CStr::from_ptr(gl::GetString(gl::VENDOR) as *const _).to_str().unwrap();
        println!("OpenGL Vendor: {}", vendor_str);

        // don't leak memory
        let _ = mem::replace(&mut FFI_BUF, Vec::new());

        PlatformGL { hdc: device, hglrc: context2 }
    }

    #[inline(always)]
    pub unsafe fn swap_buffers(&self) {
        SwapBuffers(self.hdc);
    }

    pub unsafe fn swap_interval(&self, n: u32) {
        if wgl::SwapIntervalEXT::is_loaded() {
            wgl::SwapIntervalEXT(n as i32);
        } else {
            eprintln!("wglSwapIntervalEXT missing!");
        }
    }

    #[must_use]
    pub unsafe fn cleanup(&self) -> bool {
        if !self.hdc.is_null() && wglGetCurrentContext() == self.hglrc {
            wglMakeCurrent(self.hdc, ptr::null_mut());
            wglDeleteContext(self.hglrc);
            true
        } else {
            false
        }
    }
}
