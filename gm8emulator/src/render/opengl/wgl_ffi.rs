// We don't name things around here.
#![allow(bad_style)]

macro_rules! def_handle {
    ($name: ident, $private_name: ident $(,)?) => {
        #[doc(hidden)]
        pub enum $private_name {}
        pub type $name = *mut $private_name;
    };
}

def_handle!(HDC, HDC__);
def_handle!(HINSTANCE, HINSTANCE__);
def_handle!(HMODULE, HMODULE__);
def_handle!(HGLRC, HGLRC__);
def_handle!(PROC, __some_function);

pub use core::ffi::c_void;
pub type c_char = i8;
pub type c_schar = i8;
pub type c_uchar = u8;
pub type c_short = i16;
pub type c_ushort = u16;
pub type c_int = i32;
pub type c_uint = u32;
pub type c_long = i32;
pub type c_ulong = u32;
pub type c_longlong = i64;
pub type c_ulonglong = u64;

pub type BOOL = c_int;
pub type BYTE = c_uchar;
pub type CHAR = c_char;
pub type DWORD = c_ulong;
pub type LPCVOID = *const c_void;
pub type LPVOID = *mut c_void;
pub type LPCSTR = *const CHAR;
pub type LPWSTR = *mut WCHAR;
pub type WCHAR = WORD;
pub type WORD = c_ushort;

pub const FORMAT_MESSAGE_IGNORE_INSERTS: DWORD = 0x00000200;
pub const FORMAT_MESSAGE_FROM_STRING: DWORD = 0x00000400;
pub const FORMAT_MESSAGE_FROM_HMODULE: DWORD = 0x00000800;
pub const FORMAT_MESSAGE_FROM_SYSTEM: DWORD = 0x00001000;
pub const FORMAT_MESSAGE_ARGUMENT_ARRAY: DWORD = 0x00002000;
pub const FORMAT_MESSAGE_MAX_WIDTH_MASK: DWORD = 0x000000FF;
pub const FORMAT_MESSAGE_ALLOCATE_BUFFER: DWORD = 0x00000100;

#[repr(C)]
pub struct PIXELFORMATDESCRIPTOR {
    nSize: WORD,
    nVersion: WORD,
    dwFlags: DWORD,
    iPixelType: BYTE,
    cColorBits: BYTE,
    cRedBits: BYTE,
    cRedShift: BYTE,
    cGreenBits: BYTE,
    cGreenShift: BYTE,
    cBlueBits: BYTE,
    cBlueShift: BYTE,
    cAlphaBits: BYTE,
    cAlphaShift: BYTE,
    cAccumBits: BYTE,
    cAccumRedBits: BYTE,
    cAccumGreenBits: BYTE,
    cAccumBlueBits: BYTE,
    cAccumAlphaBits: BYTE,
    cDepthBits: BYTE,
    cStencilBits: BYTE,
    cAuxBuffers: BYTE,
    iLayerType: BYTE,
    bReserved: BYTE,
    dwLayerMask: DWORD,
    dwVisibleMask: DWORD,
    dwDamageMask: DWORD,
}

pub const LANG_NEUTRAL: c_ushort = 0x00;
pub const PFD_DOUBLEBUFFER: DWORD = 0x00000001;
pub const PFD_DRAW_TO_WINDOW: DWORD = 0x00000004;
pub const PFD_MAIN_PLANE: BYTE = 0;
pub const PFD_SUPPORT_OPENGL: DWORD = 0x00000020;
pub const PFD_TYPE_RGBA: BYTE = 0;
pub const SUBLANG_DEFAULT: c_ushort = 0x01;

pub fn MAKELANGID(p: c_ushort, s: c_ushort) -> c_ushort { (s << 10) | p }

#[link(name = "opengl32")]
extern "system" {
    pub fn wglCreateContext(hdc: HDC) -> HGLRC;
    pub fn wglDeleteContext(hglrc: HGLRC) -> BOOL;
    pub fn wglGetCurrentContext() -> HGLRC;
    pub fn wglGetCurrentDC() -> HDC;
    pub fn wglGetProcAddress(lpszProc: LPCSTR) -> PROC;
    pub fn wglMakeCurrent(hdc: HDC, hglrc: HGLRC) -> BOOL;
    pub fn wglChoosePixelFormat(
        hdc: HDC,
        ppfd: *const PIXELFORMATDESCRIPTOR,
    ) -> c_int;
    pub fn wglSetPixelFormat(
        hdc: HDC,
        iPixelFormat: c_int,
        ppfd: *const PIXELFORMATDESCRIPTOR,
    ) -> BOOL;
    pub fn wglSwapBuffers(hdc: HDC) -> BOOL;
}

#[link(name = "kernel32")]
extern "system" {
    pub fn LoadLibraryA(lpLibFileName: LPCSTR) -> HMODULE;
    pub fn GetProcAddress(hModule: HMODULE, lpModuleName: LPCSTR) -> *const c_void;
    pub fn FormatMessageW(
        dwFlags: DWORD,
        lpSource: LPCVOID,
        dwMessageId: DWORD,
        dwLanguageId: DWORD,
        lpBuffer: LPWSTR,
        nSize: DWORD,
        Arguments: *mut c_void, // va_list
    ) -> DWORD;
    pub fn GetLastError() -> DWORD;
    pub fn LocalFree(hMem: LPVOID) -> LPVOID;
}

#[link(name = "user32")]
extern "system" {
    pub fn GetDC(hWnd: ramen::platform::win32::HWND) -> HDC;
}
