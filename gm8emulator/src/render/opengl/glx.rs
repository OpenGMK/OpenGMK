#![cfg(unix)]
#![allow(bad_style, overflowing_literals)]

use ramen::{connection::Connection, platform::linux::Display, window::Window};
use std::{ffi::c_void, os::raw::{c_char, c_int, c_uint}, sync::Once};

pub mod glx {
    #![allow(clippy::all)]
    include!(concat!(env!("OUT_DIR"), "/glx_bindings.rs"));
}

#[link(name = "GL")]
extern "C" {
    fn glXQueryVersion(dpy: *mut Display, major: *mut c_int, minor: *mut c_int) -> Bool;
    fn glXQueryExtensionsString(dpy: *mut Display, screen: c_int) -> *const c_char;
    fn glXChooseFBConfig(dpy: *mut Display, screen: c_int, attr_list: *const c_int, elements: *mut c_int) -> *mut GLXFBConfig;
    fn glXGetFBConfigAttrib(dpy: *mut Display, config: GLXFBConfig, attribute: c_int, value: *mut c_int) -> c_int;
    fn glXGetProcAddressARB(name: *const u8) -> *mut c_void;
    fn glXCreateWindow(dpy: *mut Display, config: GLXFBConfig, window: u32, attrib_list: *const c_int) -> GLXWindow;
    fn glXMakeContextCurrent(dpy: *mut Display, drawable: GLXDrawable, read: GLXDrawable, ctx: GLXContext) -> c_int;
    fn glXQueryDrawable(dpy: *mut Display, drawable: GLXDrawable, prop: c_int, value: *mut c_uint) -> c_int;
    fn glXSwapBuffers(dpy: *mut Display, drawable: GLXDrawable);
    fn glXDestroyWindow(dpy: *mut Display, win: GLXWindow);
}

type GLXDrawable = u32;
type GLXContext = *const c_void;
type GLXWindow = u32;
type GLXFBConfig = *const c_void;
type Bool = c_int;

static GLX_INIT: Once = Once::new();
pub static mut GLX: Option<GlxGuts> = None;

const GLX_FB_ATTRIBS: &[c_int] = &[
    /* has X visual a.k.a. can render to window/pixmap? */
    X_RENDERABLE,   1,
    /* can render to a window in specific? */
    DRAWABLE_TYPE,  WINDOW_BIT,
    /* can support rgba mode? (there's no rgb, or other colour modes) */
    RENDER_TYPE,    RGBA_BIT,
    /* we don't deal with fake colours around here */
    X_VISUAL_TYPE,  TRUE_COLOR,
    /* _at least_ 8 bits per channel (when omitted, does not care) */
    RED_SIZE,       8,
    GREEN_SIZE,     8,
    BLUE_SIZE,      8,
    ALPHA_SIZE,     8,
    /* _at least_ a depth of 8+8+8 (when omitted, asks for no depth at all) */
    DEPTH_SIZE,     24,
    /* yeah i admit i don't know stencil buffers */
    STENCIL_SIZE,   8,
    /* must double buffer! (GLX_DONT_CARE would be okay too, probably) */
    DOUBLEBUFFER,   1,
    /* none-terminator >w< */
    0
];
const CONTEXT_MAJOR_VERSION_ARB: c_int = 0x2091;
const CONTEXT_MINOR_VERSION_ARB: c_int = 0x2092;
const CONTEXT_FLAGS_ARB: c_int = 0x2094;
const CONTEXT_PROFILE_MASK_ARB: c_int = 0x9126;
const CONTEXT_FORWARD_COMPATIBLE_BIT_ARB: c_int = 0x0002;
const CONTEXT_CORE_PROFILE_BIT_ARB: c_int = 0x00000001;
const SWAP_INTERVAL_EXT: c_int = 0x20F1;

const GLX_CC_ATTRIBS: &[c_int] = &[
    CONTEXT_MAJOR_VERSION_ARB, 3,
    CONTEXT_MINOR_VERSION_ARB, 3,
    CONTEXT_FLAGS_ARB,         CONTEXT_FORWARD_COMPATIBLE_BIT_ARB,
    CONTEXT_PROFILE_MASK_ARB,  CONTEXT_CORE_PROFILE_BIT_ARB,
    0
];

pub struct GlxGuts {
    xlib: *mut Display,
    screen: c_int,
    pub depth: u8,
    pub visual: u32,
    fb_config: GLXFBConfig,
    glXCreateContextAttribsARB: unsafe extern "C" fn(*mut Display, GLXFBConfig, GLXContext, c_int, *const c_int) -> GLXContext,
    glXSwapIntervalEXT: unsafe extern "C" fn(*mut Display, GLXDrawable, interval: c_int),
}

pub fn glx_init(display: *mut Display, screen: u32) {
    GLX_INIT.call_once(move || unsafe {
        let mut major: c_int = 0;
        let mut minor: c_int = 0;
        if glXQueryVersion(display, &mut major, &mut minor) == 0 {
            panic!("failed to request opengl version");
        }
        if major == 0 || (major == 1 && minor < 4) {
            panic!("glx >=v1.4 required");
        }
        let mut n_configs: c_int = 0;
        let configs = glXChooseFBConfig(display, screen as c_int, &GLX_FB_ATTRIBS[0], &mut n_configs);
        if configs.is_null() || n_configs < 1 { panic!("couldn't get fb configs :("); }
        let mut best_sc: c_int = -10_000_000; // return to tradition
        let mut best_sc_idx: c_int = 0;
        for i in 0..(n_configs as usize) {
            let config = *configs.add(i);
            let mut sc: c_int = 0;
            let mut msb: c_int = 0;
            if glXGetFBConfigAttrib(display, config, SAMPLES, &mut sc) != 0 { continue; }
            if glXGetFBConfigAttrib(display, config, SAMPLE_BUFFERS, &mut msb) != 0 { continue; }
            if msb != 0 { continue; } // prevent glBlitFramebuffer crash due to MSB mismatch with FBO
            if sc > best_sc {
                best_sc = sc;
                best_sc_idx = i as _;
            }
        }
        let fb_config = *configs.add(best_sc_idx as _);
        let e = glXQueryExtensionsString(display, screen as _);
        if e.is_null() { panic!("bruh"); }
        let extensions = std::ffi::CStr::from_ptr(e).to_str().unwrap();
        let mut has_cc = false;
        let mut has_gpa = false;
        let mut has_swc = false;
        for extension in extensions.split(" ") {
            match extension {
                "GLX_ARB_create_context" => has_cc = true,
                "GLX_ARB_get_proc_address" => has_gpa = true,
                "GLX_EXT_swap_control" => has_swc = true,
                _ => (),
            }
        }
        if !(has_cc && has_gpa && has_swc) { panic!("upgrade your drivers Thanks"); }
        let glXCreateContextAttribsARB = std::mem::transmute::<_, unsafe extern "C" fn(*mut Display, GLXFBConfig, GLXContext, c_int, *const c_int) -> GLXContext>(glXGetProcAddressARB("glXCreateContextAttribsARB\0".as_ptr().cast()));
        let glXSwapIntervalEXT = std::mem::transmute::<_, unsafe extern "C" fn(*mut Display, GLXDrawable, interval: c_int)>(glXGetProcAddressARB("glXSwapIntervalEXT\0".as_ptr().cast()));
        let mut depth: c_int = 0;
        _ = glXGetFBConfigAttrib(display, fb_config, DEPTH_SIZE, &mut depth);
        let mut visual: c_int = 0;
        _ = glXGetFBConfigAttrib(display, fb_config, VISUAL_ID, &mut visual);
        GLX = Some(GlxGuts {
            xlib: display,
            screen: screen as _,
            depth: depth as _,
            visual: visual as _,
            fb_config,
            glXCreateContextAttribsARB,
            glXSwapIntervalEXT,
        });
    })
}

static mut LOAD_BUF: Vec<u8> = Vec::new();

pub struct PlatformImpl {
    glx_window: GLXWindow,
    window: u32, // XID
}

impl PlatformImpl {
    pub unsafe fn new(connection: &Connection, window: &Window) -> Result<Self, String> {
        let glx = GLX.as_ref().unwrap();
        let context = (glx.glXCreateContextAttribsARB)(
            glx.xlib,
            glx.fb_config,
            std::ptr::null_mut(), // shared context
            1,                    // direct rendering (don't go through X server)
            &GLX_CC_ATTRIBS[0],
        );
        if context.is_null() { return Err("yeah sorry couldn't create that context".into()); }
        let glx_window = glXCreateWindow(glx.xlib, glx.fb_config, window.xid(), std::ptr::null_mut());
        let success = glXMakeContextCurrent(glx.xlib, window.xid(), window.xid(), context);
        if success == 0 { panic!("failed to set context :( sorry"); }
        Ok(PlatformImpl { glx_window, window: window.xid() })
    }

    pub unsafe fn get_function_loader() -> Result<Box<dyn FnMut(&'static str) -> *const std::os::raw::c_void>, String> {
        let glx = GLX.as_ref().unwrap();
        let mut libgl = libc::dlopen("libGL.so.1\0".as_ptr().cast(), libc::RTLD_GLOBAL | libc::RTLD_NOW);
        if libgl.is_null() {
            libgl = libc::dlopen("libGL.so\0".as_ptr().cast(), libc::RTLD_GLOBAL | libc::RTLD_NOW);
        }
        if libgl.is_null() { panic!("libGL missing, you might not have drivers"); }
        Ok(Box::new(move |name: &'static str| unsafe {
            LOAD_BUF.clear();
            LOAD_BUF.extend_from_slice(name.as_bytes());
            LOAD_BUF.push(0);
            let mut s = libc::dlsym(libgl, LOAD_BUF.as_ptr().cast());
            if s.is_null() {
                s = glXGetProcAddressARB(LOAD_BUF.as_ptr().cast());
            }
            s as *const _ as *const c_void
        }) as _)
    }

    pub unsafe fn clean_function_loader() {
        LOAD_BUF = Vec::new();
    }

    pub unsafe fn swap_buffers(&self) {
        let glx = GLX.as_ref().unwrap();
        glXSwapBuffers(glx.xlib, self.window);
    }

    pub unsafe fn set_swap_interval(&self, n: u32) -> bool {
        let glx = GLX.as_ref().unwrap();
        (glx.glXSwapIntervalEXT)(glx.xlib, self.window, n as _);
        true
    }

    pub unsafe fn get_swap_interval(&self) -> u32 {
        let glx = GLX.as_ref().unwrap();
        let mut v: c_uint = 0;
        glXQueryDrawable(glx.xlib, self.window, SWAP_INTERVAL_EXT, &mut v);
        v as _
    }

    pub unsafe fn wait_vsync(&self) {
        //(&*self.dxgi_output).WaitForVBlank();
        panic!("let's GOOOOOOOOOOOOOOOOOOOO");
    }

    // pub unsafe fn make_current(&self) -> bool {
    // }

    // pub unsafe fn is_current(&self) -> bool {
    // }
}

impl Drop for PlatformImpl {
    fn drop(&mut self) {
        unsafe {
            let glx = GLX.as_ref().unwrap();
            glXMakeContextCurrent(glx.xlib, 0, 0, std::ptr::null_mut());
            glXDestroyWindow(glx.xlib, self.glx_window);
        }
    }
}


// do not scroll down



































// please


// stop this right now















//



//



#[allow(dead_code, non_upper_case_globals)] pub const ACCUM_ALPHA_SIZE: c_int = 17;
#[allow(dead_code, non_upper_case_globals)] pub const ACCUM_BLUE_SIZE: c_int = 16;
#[allow(dead_code, non_upper_case_globals)] pub const ACCUM_BUFFER_BIT: c_int = 0x00000080;
#[allow(dead_code, non_upper_case_globals)] pub const ACCUM_GREEN_SIZE: c_int = 15;
#[allow(dead_code, non_upper_case_globals)] pub const ACCUM_RED_SIZE: c_int = 14;
#[allow(dead_code, non_upper_case_globals)] pub const ALPHA_SIZE: c_int = 11;
#[allow(dead_code, non_upper_case_globals)] pub const AUX0_EXT: c_int = 0x20E2;
#[allow(dead_code, non_upper_case_globals)] pub const AUX1_EXT: c_int = 0x20E3;
#[allow(dead_code, non_upper_case_globals)] pub const AUX2_EXT: c_int = 0x20E4;
#[allow(dead_code, non_upper_case_globals)] pub const AUX3_EXT: c_int = 0x20E5;
#[allow(dead_code, non_upper_case_globals)] pub const AUX4_EXT: c_int = 0x20E6;
#[allow(dead_code, non_upper_case_globals)] pub const AUX5_EXT: c_int = 0x20E7;
#[allow(dead_code, non_upper_case_globals)] pub const AUX6_EXT: c_int = 0x20E8;
#[allow(dead_code, non_upper_case_globals)] pub const AUX7_EXT: c_int = 0x20E9;
#[allow(dead_code, non_upper_case_globals)] pub const AUX8_EXT: c_int = 0x20EA;
#[allow(dead_code, non_upper_case_globals)] pub const AUX9_EXT: c_int = 0x20EB;
#[allow(dead_code, non_upper_case_globals)] pub const AUX_BUFFERS: c_int = 7;
#[allow(dead_code, non_upper_case_globals)] pub const AUX_BUFFERS_BIT: c_int = 0x00000010;
#[allow(dead_code, non_upper_case_globals)] pub const BACK_EXT: c_int = 0x20E0;
#[allow(dead_code, non_upper_case_globals)] pub const BACK_LEFT_BUFFER_BIT: c_int = 0x00000004;
#[allow(dead_code, non_upper_case_globals)] pub const BACK_LEFT_EXT: c_int = 0x20E0;
#[allow(dead_code, non_upper_case_globals)] pub const BACK_RIGHT_BUFFER_BIT: c_int = 0x00000008;
#[allow(dead_code, non_upper_case_globals)] pub const BACK_RIGHT_EXT: c_int = 0x20E1;
#[allow(dead_code, non_upper_case_globals)] pub const BAD_ATTRIBUTE: c_int = 2;
#[allow(dead_code, non_upper_case_globals)] pub const BAD_CONTEXT: c_int = 5;
#[allow(dead_code, non_upper_case_globals)] pub const BAD_ENUM: c_int = 7;
#[allow(dead_code, non_upper_case_globals)] pub const BAD_SCREEN: c_int = 1;
#[allow(dead_code, non_upper_case_globals)] pub const BAD_VALUE: c_int = 6;
#[allow(dead_code, non_upper_case_globals)] pub const BAD_VISUAL: c_int = 4;
#[allow(dead_code, non_upper_case_globals)] pub const BIND_TO_MIPMAP_TEXTURE_EXT: c_int = 0x20D2;
#[allow(dead_code, non_upper_case_globals)] pub const BIND_TO_TEXTURE_RGBA_EXT: c_int = 0x20D1;
#[allow(dead_code, non_upper_case_globals)] pub const BIND_TO_TEXTURE_RGB_EXT: c_int = 0x20D0;
#[allow(dead_code, non_upper_case_globals)] pub const BIND_TO_TEXTURE_TARGETS_EXT: c_int = 0x20D3;
#[allow(dead_code, non_upper_case_globals)] pub const BLUE_SIZE: c_int = 10;
#[allow(dead_code, non_upper_case_globals)] pub const BUFFER_SIZE: c_int = 2;
#[allow(dead_code, non_upper_case_globals)] pub const BufferSwapComplete: c_int = 1;
#[allow(dead_code, non_upper_case_globals)] pub const COLOR_INDEX_BIT: c_int = 0x00000002;
#[allow(dead_code, non_upper_case_globals)] pub const COLOR_INDEX_TYPE: c_int = 0x8015;
#[allow(dead_code, non_upper_case_globals)] pub const CONFIG_CAVEAT: c_int = 0x20;
#[allow(dead_code, non_upper_case_globals)] pub const DAMAGED: c_int = 0x8020;
#[allow(dead_code, non_upper_case_globals)] pub const DEPTH_BUFFER_BIT: c_int = 0x00000020;
#[allow(dead_code, non_upper_case_globals)] pub const DEPTH_SIZE: c_int = 12;
#[allow(dead_code, non_upper_case_globals)] pub const DIRECT_COLOR: c_int = 0x8003;
#[allow(dead_code, non_upper_case_globals)] pub const DONT_CARE: c_int = 0xFFFFFFFF;
#[allow(dead_code, non_upper_case_globals)] pub const DOUBLEBUFFER: c_int = 5;
#[allow(dead_code, non_upper_case_globals)] pub const DRAWABLE_TYPE: c_int = 0x8010;
#[allow(dead_code, non_upper_case_globals)] pub const EVENT_MASK: c_int = 0x801F;
#[allow(dead_code, non_upper_case_globals)] pub const EXTENSIONS: c_int = 0x3;
#[allow(dead_code, non_upper_case_globals)] pub const EXTENSION_NAME: &'static str = "GLX";
#[allow(dead_code, non_upper_case_globals)] pub const FBCONFIG_ID: c_int = 0x8013;
#[allow(dead_code, non_upper_case_globals)] pub const FRONT_EXT: c_int = 0x20DE;
#[allow(dead_code, non_upper_case_globals)] pub const FRONT_LEFT_BUFFER_BIT: c_int = 0x00000001;
#[allow(dead_code, non_upper_case_globals)] pub const FRONT_LEFT_EXT: c_int = 0x20DE;
#[allow(dead_code, non_upper_case_globals)] pub const FRONT_RIGHT_BUFFER_BIT: c_int = 0x00000002;
#[allow(dead_code, non_upper_case_globals)] pub const FRONT_RIGHT_EXT: c_int = 0x20DF;
#[allow(dead_code, non_upper_case_globals)] pub const GRAY_SCALE: c_int = 0x8006;
#[allow(dead_code, non_upper_case_globals)] pub const GREEN_SIZE: c_int = 9;
#[allow(dead_code, non_upper_case_globals)] pub const HEIGHT: c_int = 0x801E;
#[allow(dead_code, non_upper_case_globals)] pub const LARGEST_PBUFFER: c_int = 0x801C;
#[allow(dead_code, non_upper_case_globals)] pub const LEVEL: c_int = 3;
#[allow(dead_code, non_upper_case_globals)] pub const MAX_PBUFFER_HEIGHT: c_int = 0x8017;
#[allow(dead_code, non_upper_case_globals)] pub const MAX_PBUFFER_PIXELS: c_int = 0x8018;
#[allow(dead_code, non_upper_case_globals)] pub const MAX_PBUFFER_WIDTH: c_int = 0x8016;
#[allow(dead_code, non_upper_case_globals)] pub const MIPMAP_TEXTURE_EXT: c_int = 0x20D7;
#[allow(dead_code, non_upper_case_globals)] pub const NONE: c_int = 0x8000;
#[allow(dead_code, non_upper_case_globals)] pub const NON_CONFORMANT_CONFIG: c_int = 0x800D;
#[allow(dead_code, non_upper_case_globals)] pub const NO_EXTENSION: c_int = 3;
#[allow(dead_code, non_upper_case_globals)] pub const PBUFFER: c_int = 0x8023;
#[allow(dead_code, non_upper_case_globals)] pub const PBUFFER_BIT: c_int = 0x00000004;
#[allow(dead_code, non_upper_case_globals)] pub const PBUFFER_CLOBBER_MASK: c_int = 0x08000000;
#[allow(dead_code, non_upper_case_globals)] pub const PBUFFER_HEIGHT: c_int = 0x8040;
#[allow(dead_code, non_upper_case_globals)] pub const PBUFFER_WIDTH: c_int = 0x8041;
#[allow(dead_code, non_upper_case_globals)] pub const PIXMAP_BIT: c_int = 0x00000002;
#[allow(dead_code, non_upper_case_globals)] pub const PRESERVED_CONTENTS: c_int = 0x801B;
#[allow(dead_code, non_upper_case_globals)] pub const PSEUDO_COLOR: c_int = 0x8004;
#[allow(dead_code, non_upper_case_globals)] pub const PbufferClobber: c_int = 0;
#[allow(dead_code, non_upper_case_globals)] pub const RED_SIZE: c_int = 8;
#[allow(dead_code, non_upper_case_globals)] pub const RENDER_TYPE: c_int = 0x8011;
#[allow(dead_code, non_upper_case_globals)] pub const RGBA: c_int = 4;
#[allow(dead_code, non_upper_case_globals)] pub const RGBA_BIT: c_int = 0x00000001;
#[allow(dead_code, non_upper_case_globals)] pub const RGBA_TYPE: c_int = 0x8014;
#[allow(dead_code, non_upper_case_globals)] pub const SAMPLES: c_int = 100001;
#[allow(dead_code, non_upper_case_globals)] pub const SAMPLE_BUFFERS: c_int = 100000;
#[allow(dead_code, non_upper_case_globals)] pub const SAVED: c_int = 0x8021;
#[allow(dead_code, non_upper_case_globals)] pub const SCREEN: c_int = 0x800C;
#[allow(dead_code, non_upper_case_globals)] pub const SLOW_CONFIG: c_int = 0x8001;
#[allow(dead_code, non_upper_case_globals)] pub const STATIC_COLOR: c_int = 0x8005;
#[allow(dead_code, non_upper_case_globals)] pub const STATIC_GRAY: c_int = 0x8007;
#[allow(dead_code, non_upper_case_globals)] pub const STENCIL_BUFFER_BIT: c_int = 0x00000040;
#[allow(dead_code, non_upper_case_globals)] pub const STENCIL_SIZE: c_int = 13;
#[allow(dead_code, non_upper_case_globals)] pub const STEREO: c_int = 6;
#[allow(dead_code, non_upper_case_globals)] pub const TEXTURE_1D_BIT_EXT: c_int = 0x00000001;
#[allow(dead_code, non_upper_case_globals)] pub const TEXTURE_1D_EXT: c_int = 0x20DB;
#[allow(dead_code, non_upper_case_globals)] pub const TEXTURE_2D_BIT_EXT: c_int = 0x00000002;
#[allow(dead_code, non_upper_case_globals)] pub const TEXTURE_2D_EXT: c_int = 0x20DC;
#[allow(dead_code, non_upper_case_globals)] pub const TEXTURE_FORMAT_EXT: c_int = 0x20D5;
#[allow(dead_code, non_upper_case_globals)] pub const TEXTURE_FORMAT_NONE_EXT: c_int = 0x20D8;
#[allow(dead_code, non_upper_case_globals)] pub const TEXTURE_FORMAT_RGBA_EXT: c_int = 0x20DA;
#[allow(dead_code, non_upper_case_globals)] pub const TEXTURE_FORMAT_RGB_EXT: c_int = 0x20D9;
#[allow(dead_code, non_upper_case_globals)] pub const TEXTURE_RECTANGLE_BIT_EXT: c_int = 0x00000004;
#[allow(dead_code, non_upper_case_globals)] pub const TEXTURE_RECTANGLE_EXT: c_int = 0x20DD;
#[allow(dead_code, non_upper_case_globals)] pub const TEXTURE_TARGET_EXT: c_int = 0x20D6;
#[allow(dead_code, non_upper_case_globals)] pub const TRANSPARENT_ALPHA_VALUE: c_int = 0x28;
#[allow(dead_code, non_upper_case_globals)] pub const TRANSPARENT_BLUE_VALUE: c_int = 0x27;
#[allow(dead_code, non_upper_case_globals)] pub const TRANSPARENT_GREEN_VALUE: c_int = 0x26;
#[allow(dead_code, non_upper_case_globals)] pub const TRANSPARENT_INDEX: c_int = 0x8009;
#[allow(dead_code, non_upper_case_globals)] pub const TRANSPARENT_INDEX_VALUE: c_int = 0x24;
#[allow(dead_code, non_upper_case_globals)] pub const TRANSPARENT_RED_VALUE: c_int = 0x25;
#[allow(dead_code, non_upper_case_globals)] pub const TRANSPARENT_RGB: c_int = 0x8008;
#[allow(dead_code, non_upper_case_globals)] pub const TRANSPARENT_TYPE: c_int = 0x23;
#[allow(dead_code, non_upper_case_globals)] pub const TRUE_COLOR: c_int = 0x8002;
#[allow(dead_code, non_upper_case_globals)] pub const USE_GL: c_int = 1;
#[allow(dead_code, non_upper_case_globals)] pub const VENDOR: c_int = 0x1;
#[allow(dead_code, non_upper_case_globals)] pub const VERSION: c_int = 0x2;
#[allow(dead_code, non_upper_case_globals)] pub const VISUAL_ID: c_int = 0x800B;
#[allow(dead_code, non_upper_case_globals)] pub const WIDTH: c_int = 0x801D;
#[allow(dead_code, non_upper_case_globals)] pub const WINDOW: c_int = 0x8022;
#[allow(dead_code, non_upper_case_globals)] pub const WINDOW_BIT: c_int = 0x00000001;
#[allow(dead_code, non_upper_case_globals)] pub const X_RENDERABLE: c_int = 0x8012;
#[allow(dead_code, non_upper_case_globals)] pub const X_VISUAL_TYPE: c_int = 0x22;
#[allow(dead_code, non_upper_case_globals)] pub const Y_INVERTED_EXT: c_int = 0x20D4;
