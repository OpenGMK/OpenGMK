//! Windows-specific OpenGL loading.

#![cfg(target_os = "windows")]

use crate::{
    render::opengl::gl::{self, types::GLint},
    window::win32::WindowImpl,
};
use std::{
    ffi::CStr,
    mem::{self, size_of},
    ops::Drop,
    os::raw::{c_char, c_int, c_void},
    ptr,
};
use winapi::{
    Interface,
    shared::{
        minwindef::HINSTANCE,
        windef::{HDC, HGLRC},
        dxgi::{CreateDXGIFactory, IDXGIFactory, IDXGIOutput},
    },
    um::{
        libloaderapi::{GetProcAddress, LoadLibraryA},
        wingdi::{
            wglCreateContext, wglDeleteContext, wglGetCurrentContext, wglGetCurrentDC, wglGetProcAddress,
            wglMakeCurrent, ChoosePixelFormat, SetPixelFormat, SwapBuffers, PFD_DOUBLEBUFFER, PFD_DRAW_TO_WINDOW,
            PFD_MAIN_PLANE, PFD_SUPPORT_OPENGL, PFD_TYPE_RGBA, PIXELFORMATDESCRIPTOR,
        },
        winuser::GetDC,
    },
};

pub mod wgl {
    #![allow(clippy::all)]

    include!(concat!(env!("OUT_DIR"), "/wgl_bindings.rs"));
}

pub struct PlatformImpl {
    context: HGLRC,
    device: HDC,
    version: (u8, u8),
    dxgi_output: *mut IDXGIOutput,
}

/// Global buffer to make fucking gl_generator not need one alloc per query.
/// Don't forget to manually mem::replace & drop as this is global.
/// Remind me to make a better library.
static mut GLGEN_BUF: Vec<u8> = Vec::new();
unsafe fn glgen_loader(name: &str, gl32_dll: HINSTANCE) -> *const c_void {
    GLGEN_BUF.clear();
    GLGEN_BUF.extend_from_slice(name.as_bytes());
    GLGEN_BUF.push(0);
    load_function(GLGEN_BUF.as_ptr() as *const c_char, gl32_dll)
}

/// Configuration for querying device pixel format.
/// Half of these fields are zero because it'll see RGBA and not care.
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

/// Flags for wglCreateContextAttribsARB
#[rustfmt::skip]
static WGL_CCTX_ATTR_ARB: &[u32] = &[
    wgl::CONTEXT_MAJOR_VERSION_ARB, 3,
    wgl::CONTEXT_MINOR_VERSION_ARB, 3,
    wgl::CONTEXT_LAYER_PLANE_ARB,   0,
    wgl::CONTEXT_FLAGS_ARB,         wgl::CONTEXT_FORWARD_COMPATIBLE_BIT_ARB,
    wgl::CONTEXT_PROFILE_MASK_ARB,  wgl::CONTEXT_CORE_PROFILE_BIT_ARB,
    0, // END
];

// TODO: move this stuff to a reusable location
use std::{ffi::OsString, os::windows::ffi::OsStringExt, slice};
use winapi::{
    shared::ntdef::{LANG_NEUTRAL, MAKELANGID, SUBLANG_DEFAULT, WCHAR},
    um::{
        errhandlingapi::GetLastError,
        winbase::{
            FormatMessageW, LocalFree, FORMAT_MESSAGE_ALLOCATE_BUFFER, FORMAT_MESSAGE_FROM_SYSTEM,
            FORMAT_MESSAGE_IGNORE_INSERTS,
        },
    },
};
macro_rules! wapi_call {
    ($ex: expr) => {{
        match $ex {
            x if x as u64 != 0 => Ok(x),
            _ => Err(format!("[{} @ line {}: {}] {}", file!(), line!(), stringify!($ex), wapi_error_string(),)),
        }
    }};
}
unsafe fn wapi_error_string() -> String {
    let mut buf_ptr: *mut WCHAR = ptr::null_mut();
    let char_count = FormatMessageW(
        FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_IGNORE_INSERTS,
        ptr::null(),
        GetLastError(),
        MAKELANGID(LANG_NEUTRAL, SUBLANG_DEFAULT).into(),
        (&mut buf_ptr as *mut *mut WCHAR) as *mut _, // ugh
        0,
        ptr::null_mut(),
    );
    assert!(!buf_ptr.is_null());
    let wchars = slice::from_raw_parts(buf_ptr, char_count as usize);
    let os_message = OsString::from_wide(wchars);
    let message = os_message.to_string_lossy().into_owned();
    LocalFree(buf_ptr.cast());
    message
}

unsafe fn create_dxgi_output() -> Result<*mut IDXGIOutput, String> {
    let mut factory: *mut IDXGIFactory = ptr::null_mut();
    let factory_ptr: *mut *mut IDXGIFactory = &mut factory;
    match CreateDXGIFactory(&IDXGIFactory::uuidof(), factory_ptr.cast()) {
        0 => (),
        e => return Err(format!("Could not create DXGIFactory (code {:#X})", e)),
    }
    let mut adapter = ptr::null_mut();
    match (&*factory).EnumAdapters(0, &mut adapter) {
        0 => (),
        e => return Err(format!("Could not get first DXGIAdapter (code {:#X})", e)),
    }
    let mut output = ptr::null_mut();
    match (&*adapter).EnumOutputs(0, &mut output) {
        0 => Ok(output),
        e => Err(format!("Could not get first DXGIOutput (code {:#X})", e)),
    }
}

unsafe fn create_context_basic(device: HDC) -> Result<HGLRC, String> {
    let saved_context = wglGetCurrentContext();
    let saved_device = wglGetCurrentDC();

    match wapi_call!(wglCreateContext(device))
        .and_then(|handle| wapi_call!(wglMakeCurrent(device, handle)).map(|_| handle))
    {
        Ok(handle) => Ok(handle),
        Err(err) => {
            wglMakeCurrent(saved_device, saved_context);
            Err(err)
        },
    }
}

unsafe fn create_context_attribs(device: HDC) -> Result<HGLRC, String> {
    let saved_context = wglGetCurrentContext();
    let saved_device = wglGetCurrentDC();
    let context = wapi_call!(wgl::CreateContextAttribsARB(
        device as *const _ as *const c_void,
        ptr::null_mut(),
        WGL_CCTX_ATTR_ARB as *const _ as *const c_int,
    ))? as HGLRC;
    match wapi_call!(wglMakeCurrent(device, context)) {
        Ok(_) => Ok(context),
        Err(err) => {
            wglMakeCurrent(saved_device, saved_context);
            Err(err)
        },
    }
}

/// Loads an OpenGL function pointer.
/// Only works if there is a current OpenGL context.
unsafe fn load_function(name: *const c_char, gl32_dll: HINSTANCE) -> *const c_void {
    let addr = wglGetProcAddress(name);
    match addr as isize {
        // All of these return values mean failure, as much as the docs say it's just NULL.
        // You load some of them like this, but only if wglGetProcAddress failed.
        // The ones that would are the 1.1 functions because they're in opengl32.dll.
        -1 | 0 | 1 | 2 | 3 => GetProcAddress(gl32_dll, name).cast(),
        _ => addr as *const c_void,
    }
}

impl PlatformImpl {
    pub unsafe fn new(window: &WindowImpl) -> Result<Self, String> {
        static mut GL_LOADED: bool = false;
        static mut WGL_LOADED: bool = false;
        static mut OPENGL32_DLL: HINSTANCE = ptr::null_mut();

        // our device context
        let device = wapi_call!(GetDC(window.get_hwnd()))?;

        // set up pixel format
        let pixel_format = wapi_call!(ChoosePixelFormat(device, &PIXEL_FORMAT))?;
        wapi_call!(SetPixelFormat(device, pixel_format, &PIXEL_FORMAT))?;

        // gl 1.1 functions are located in here then they decided to not do it that way
        let gl32 = match OPENGL32_DLL {
            x if x.is_null() => wapi_call!(LoadLibraryA(b"opengl32.dll\0".as_ptr() as *const c_char))?,
            x => x,
        };
        OPENGL32_DLL = gl32;

        // basic context we can work with
        let mut context = create_context_basic(device)?;

        // load wgl function pointers
        if !WGL_LOADED {
            wgl::load_with(|s: &'static str| glgen_loader(s, gl32));
            WGL_LOADED = true;
        }

        if wgl::CreateContextAttribsARB::is_loaded() {
            let ex_context = create_context_attribs(device)?;
            wglDeleteContext(context);
            context = ex_context;
        }

        // opengl function pointers
        if !GL_LOADED {
            gl::load_with(|s: &'static str| glgen_loader(s, gl32));
            GL_LOADED = true;
        }

        // debug print
        let ver_str = CStr::from_ptr(gl::GetString(gl::VERSION).cast()).to_str().unwrap();
        println!("OpenGL Version: {}", ver_str);
        let vendor_str = CStr::from_ptr(gl::GetString(gl::VENDOR).cast()).to_str().unwrap();
        println!("OpenGL Vendor: {}", vendor_str);

        // don't leak memory
        let _ = mem::replace(&mut GLGEN_BUF, Vec::new());

        let mut ver1: GLint = 0;
        gl::GetIntegerv(gl::MAJOR_VERSION, &mut ver1);
        let mut ver2: GLint = 0;
        gl::GetIntegerv(gl::MINOR_VERSION, &mut ver2);

        Ok(Self { context, device, version: (ver1.min(255) as u8, ver2.min(255) as u8), dxgi_output: create_dxgi_output()? })
    }

    pub fn version(&self) -> (u8, u8) {
        self.version
    }

    pub unsafe fn swap_buffers(&self) {
        SwapBuffers(self.device);
    }

    pub unsafe fn set_swap_interval(&self, n: u32) -> bool {
        if wgl::SwapIntervalEXT::is_loaded() {
            wgl::SwapIntervalEXT(n as i32);
            true
        } else {
            false
        }
    }

    pub unsafe fn wait_vsync(&self) {
        (&*self.dxgi_output).WaitForVBlank();
    }

    // pub unsafe fn make_current(&self) -> bool {
    //     wglMakeCurrent(self.device, self.context) != 0
    // }

    // pub unsafe fn is_current(&self) -> bool {
    //     wglGetCurrentContext() == self.context
    // }
}

impl Drop for PlatformImpl {
    fn drop(&mut self) {
        unsafe {
            // unset if we're the current context
            if wglGetCurrentContext() == self.context {
                wglMakeCurrent(ptr::null_mut(), ptr::null_mut());
            }

            wglDeleteContext(self.context);
        }
    }
}
