//! what do I put here

use super::gl;
use std::{
    mem::{size_of, transmute},
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
            wglCreateContext, wglDeleteContext, wglGetProcAddress, wglMakeCurrent, ChoosePixelFormat, SetPixelFormat,
            SwapBuffers, PFD_DOUBLEBUFFER, PFD_DRAW_TO_WINDOW, PFD_MAIN_PLANE, PFD_SUPPORT_OPENGL, PFD_TYPE_RGBA,
            PIXELFORMATDESCRIPTOR,
        },
        winuser::GetDC,
    },
};
use winit::{platform::windows::WindowExtWindows, window::Window};

pub struct PlatformGL(HDC, HGLRC);

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

const WGL_CONTEXT_MAJOR_VERSION_ARB: c_int = 0x2091;
const WGL_CONTEXT_MINOR_VERSION_ARB: c_int = 0x2092;
const WGL_CONTEXT_LAYER_PLANE_ARB: c_int = 0x2093;
const WGL_CONTEXT_FLAGS_ARB: c_int = 0x2094;
const WGL_CONTEXT_PROFILE_MASK_ARB: c_int = 0x9126;
const WGL_CONTEXT_FORWARD_COMPATIBLE_BIT_ARB: c_int = 0x0002;
const WGL_CONTEXT_CORE_PROFILE_BIT_ARB: c_int = 0x0001;

#[rustfmt::skip]
/// Flags for wglCreateContextAttribsARB
static WGL_CCTX_ATTR_ARB: &[c_int] = &[
    WGL_CONTEXT_MAJOR_VERSION_ARB, 3,
    WGL_CONTEXT_MINOR_VERSION_ARB, 3,
    WGL_CONTEXT_LAYER_PLANE_ARB,   0,
    WGL_CONTEXT_FLAGS_ARB,         WGL_CONTEXT_FORWARD_COMPATIBLE_BIT_ARB,
    WGL_CONTEXT_PROFILE_MASK_ARB,  WGL_CONTEXT_CORE_PROFILE_BIT_ARB,
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

pub fn setup(window: &Window) -> PlatformGL {
    unsafe {
        // query device context, set up pixel format
        let device = GetDC(window.hwnd() as *mut _);
        let format = ChoosePixelFormat(device, &PIXEL_FORMAT);
        assert_ne!(format, 0, "couldn't find pixel format");
        let result = SetPixelFormat(device, format, &PIXEL_FORMAT);
        assert_ne!(result, 0, "couldn't set pixel format");

        // for 2.0+ functions
        let gl32 = LoadLibraryA(b"opengl32.dll\0".as_ptr() as *const c_char);

        // create OpenGL context for opening ARB context
        let context = wglCreateContext(device);
        let result = wglMakeCurrent(device, context);
        assert_ne!(result, 0, "couldn't make new context current");

        // create the real context! (TODO: this should be fine to avoid if it's already 3.2 Core?)
        let wgl_cctx = b"wglCreateContextAttribsARB\0".as_ptr() as *const c_char;
        let wgl_cctx_fn = load_gl_function(wgl_cctx, gl32);
        assert!(!wgl_cctx_fn.is_null(), "can't load wglCCtxARB");
        let context2 = transmute::<_, WglCreateContextAttribsARBTy>(wgl_cctx_fn)(
            device,
            ptr::null_mut(),
            WGL_CCTX_ATTR_ARB as *const _ as *const c_int,
        );
        assert!(!context2.is_null(), "wglCCtxARB returned null");
        wglMakeCurrent(device, context2);
        wglDeleteContext(context);

        // load function pointers
        static mut FFI_BUF: Vec<u8> = Vec::new();
        FFI_BUF.reserve(64);
        gl::load_with(|name| {
            FFI_BUF.clear();
            FFI_BUF.extend_from_slice(name.as_bytes());
            FFI_BUF.push(0);
            load_gl_function(FFI_BUF.as_ptr() as *const c_char, gl32)
        });

        let ver_str = std::ffi::CStr::from_ptr(gl::GetString(gl::VERSION) as *const _).to_str().unwrap();
        println!("OpenGL Version: {}", ver_str);
        let vendor_str = std::ffi::CStr::from_ptr(gl::GetString(gl::VENDOR) as *const _).to_str().unwrap();
        println!("OpenGL Vendor: {}", vendor_str);

        PlatformGL(device, context2)
    }
}

pub fn swap_buffers(plat: &PlatformGL) {
    unsafe {
        SwapBuffers(plat.0);
    }
}

pub fn swap_interval(n: u32) {
    unsafe {
        let gl32 = LoadLibraryA(b"opengl32.dll\0".as_ptr() as *const c_char);
        let fp = load_gl_function(b"wglSwapIntervalEXT\0".as_ptr() as *const c_char, gl32);
        assert!(!fp.is_null(), "wglSwapIntervalEXT was not found");
        transmute::<_, extern "C" fn(c_int) -> BOOL>(fp)(n as c_int);
    }
}

pub fn cleanup(plat: &PlatformGL) {
    unsafe {
        if !plat.0.is_null() {
            wglMakeCurrent(plat.0, ptr::null_mut());
            if !plat.1.is_null() {
                wglDeleteContext(plat.1);
            }
        }
    }
}
