#![allow(unused_imports)]
use crate::game::window::*;
use std::{
    ffi::OsStr,
    mem,
    ops::Drop,
    os::windows::ffi::OsStrExt,
    ptr, slice,
    sync::atomic::{self, AtomicU16, AtomicUsize},
};
use winapi::{
    ctypes::{c_int, wchar_t},
    shared::{
        basetsd::LONG_PTR,
        minwindef::{ATOM, DWORD, HINSTANCE, LPARAM, LRESULT, TRUE, UINT, WPARAM},
        windef::{HBRUSH, HWND},
    },
    um::{
        errhandlingapi::GetLastError,
        winnt::IMAGE_DOS_HEADER,
        winuser::{
            BeginPaint, CreateWindowExW, DefWindowProcW, DispatchMessageW, EndPaint, GetSystemMetrics,
            GetWindowLongPtrW, LoadCursorW, PeekMessageW, RegisterClassExW, SetWindowLongPtrW, ShowWindow,
            TranslateMessage, UnregisterClassW, COLOR_BACKGROUND, CS_OWNDC, CW_USEDEFAULT, GWLP_USERDATA, GWL_STYLE,
            IDC_ARROW, MSG, PAINTSTRUCT, PM_REMOVE, SM_CXSCREEN, SM_CYSCREEN, SW_HIDE, SW_SHOW, WM_CLOSE,
            WM_ERASEBKGND, WM_NCDESTROY, WM_PAINT, WNDCLASSEXW, WS_CAPTION, WS_MAXIMIZEBOX, WS_MINIMIZEBOX,
            WS_OVERLAPPED, WS_POPUP, WS_SYSMENU, WS_THICKFRAME,
        },
    },
};

// TODO: This might not work on MinGW (GCC)? Time for CI to find out!
extern "C" {
    static __ImageBase: IMAGE_DOS_HEADER;
}
fn get_hinstance() -> HINSTANCE {
    unsafe { (&__ImageBase) as *const _ as _ }
}

// re-registering a window class is an error.
static WINDOW_CLASS_ATOM: AtomicU16 = AtomicU16::new(0);

// so multiple windows don't destroy each other's window classes on drop
static WINDOW_COUNT: AtomicUsize = AtomicUsize::new(0);

// can we get utf16 literals in rust please? i mean this isn't EXACTLY utf16 but it'd work
static WINDOW_CLASS_WNAME: &[u8] = b"\0G\0M\08\0E\0m\0u\0l\0a\0t\0o\0r\0\0";

fn get_window_style(style: Style) -> DWORD {
    match style {
        Style::Regular => WS_OVERLAPPED | WS_MINIMIZEBOX | WS_SYSMENU,
        Style::Resizable => WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_THICKFRAME | WS_MINIMIZEBOX | WS_MAXIMIZEBOX,
        Style::Undecorated => WS_OVERLAPPED,
        Style::Borderless => WS_POPUP,
        Style::BorderlessFullscreen => unimplemented!("no fullscreen yet"),
    }
}

struct WindowData {
    close_requested: bool,
    events: Vec<Event>,
}

impl Default for WindowData {
    fn default() -> Self {
        Self { close_requested: false, events: Vec::new() }
    }
}

#[inline(always)]
unsafe fn hwnd_windowdata<'a>(hwnd: HWND) -> &'a mut WindowData {
    let lptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA);
    &mut *mem::transmute::<LONG_PTR, *mut WindowData>(lptr)
}

unsafe fn register_window_class() -> Result<ATOM, DWORD> {
    let class = WNDCLASSEXW {
        cbSize: mem::size_of::<WNDCLASSEXW>() as UINT,
        style: CS_OWNDC,
        lpfnWndProc: Some(wnd_proc),
        hInstance: get_hinstance(),
        hCursor: LoadCursorW(ptr::null_mut(), IDC_ARROW),
        hbrBackground: COLOR_BACKGROUND as HBRUSH,
        lpszMenuName: ptr::null(),
        lpszClassName: WINDOW_CLASS_WNAME.as_ptr() as *const wchar_t,
        hIconSm: ptr::null_mut(),
        hIcon: ptr::null_mut(),

        // tail alloc! we don't actually use any
        cbClsExtra: 0,
        cbWndExtra: 0,
    };
    let class_atom = RegisterClassExW(&class);
    if class_atom == 0 { Err(GetLastError()) } else { Ok(class_atom) }
}

unsafe extern "system" fn wnd_proc(hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_PAINT => {
            let mut ps: PAINTSTRUCT = mem::zeroed();
            BeginPaint(hwnd, &mut ps);
            EndPaint(hwnd, &mut ps);
            return 0
        },
        WM_ERASEBKGND => return TRUE as LRESULT,
        WM_CLOSE => {
            hwnd_windowdata(hwnd).close_requested = true;
            return 0
        },
        _ => (),
    }
    DefWindowProcW(hwnd, msg, wparam, lparam)
}

pub struct WindowImpl {
    extra: Box<WindowData>,
    hwnd: HWND,
}

impl WindowImpl {
    pub fn new(width: u32, height: u32, title: &str) -> Result<Self, String> {
        let class_atom = match WINDOW_CLASS_ATOM.load(atomic::Ordering::Acquire) {
            0 => match unsafe { register_window_class() } {
                Ok(atom) => {
                    WINDOW_CLASS_ATOM.store(atom, atomic::Ordering::Release);
                    atom
                },
                Err(code) => return Err(format!("Failed to register windowclass! (Code: {:#X})", code)),
            },
            atom => atom,
        };
        let width = width.min(i32::max_value() as u32) as i32;
        let height = height.min(i32::max_value() as u32) as i32;
        let title = OsStr::new(title).encode_wide().chain(Some(0x00)).collect::<Vec<wchar_t>>();
        let (extra, hwnd) = unsafe {
            let hwnd = CreateWindowExW(
                0,                                                  // dwExStyle
                class_atom as _,                                    // lpClassName
                title.as_ptr(),                                     // lpWindowName
                get_window_style(Style::Regular),                   // dwStyle
                (GetSystemMetrics(SM_CXSCREEN) / 2) - (width / 2),  // X
                (GetSystemMetrics(SM_CYSCREEN) / 2) - (height / 2), // Y
                width,                                              // nWidth
                height,                                             // nHeight
                ptr::null_mut(),                                    // hWndParent
                ptr::null_mut(),                                    // hMenu
                get_hinstance(),                                    // hInstance
                ptr::null_mut(),                                    // lpParam
            );
            if hwnd.is_null() {
                let code = GetLastError();
                return Err(format!("Failed to create window! (Code: {:#X})", code))
            }
            let extra = Box::new(WindowData::default());
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, extra.as_ref() as *const _ as LONG_PTR);
            (extra, hwnd)
        };
        WINDOW_COUNT.fetch_add(1, atomic::Ordering::AcqRel);

        Ok(Self { extra, hwnd })
    }
}

impl WindowTrait for WindowImpl {
    fn close_requested(&self) -> bool {
        self.extra.close_requested
    }

    fn request_close(&mut self) {
        self.extra.close_requested = true;
    }

    fn process_events<'a>(&'a mut self) -> slice::Iter<'a, Event> {
        unsafe {
            let window_data = &mut *self.extra;
            window_data.events.clear();
            let mut msg: MSG = mem::zeroed();
            loop {
                match PeekMessageW(&mut msg, self.hwnd, 0, 0, PM_REMOVE) {
                    0 => break,
                    _ => {
                        TranslateMessage(&msg);
                        DispatchMessageW(&msg);
                    },
                }
            }
            window_data.events.iter()
        }
    }

    fn set_style(&self, style: Style) {
        let wstyle = get_window_style(style);
        unsafe {
            SetWindowLongPtrW(self.hwnd, GWL_STYLE, wstyle as LONG_PTR);
        }
    }

    fn set_visible(&self, visible: bool) {
        let flag = if visible { SW_SHOW } else { SW_HIDE };
        unsafe {
            ShowWindow(self.hwnd, flag);
        }
    }
}

impl Drop for WindowImpl {
    fn drop(&mut self) {
        let count = WINDOW_COUNT.fetch_sub(1, atomic::Ordering::AcqRel);
        if count == 0 {
            let atom = WINDOW_CLASS_ATOM.swap(0, atomic::Ordering::AcqRel);
            unsafe {
                UnregisterClassW(atom as _, get_hinstance());
            }
        }
    }
}
