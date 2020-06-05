#![cfg(target_os = "windows")]

use super::{Cursor, Event, Style, WindowBuilder, WindowTrait};
use crate::input::{Key, MouseButton};
use std::{
    any::Any,
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
        minwindef::{ATOM, DWORD, FALSE, HINSTANCE, HIWORD, LOWORD, LPARAM, LRESULT, TRUE, UINT, WPARAM},
        windef::{HBRUSH, HCURSOR, HWND, POINT, RECT},
        windowsx::{GET_X_LPARAM, GET_Y_LPARAM},
    },
    um::{
        commctrl::_TrackMouseEvent,
        errhandlingapi::GetLastError,
        winnt::IMAGE_DOS_HEADER,
        winuser::{
            AdjustWindowRect, CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetCursorPos,
            GetSystemMetrics, GetWindowLongPtrW, GetWindowRect, LoadImageW, PeekMessageW, RegisterClassExW,
            ReleaseCapture, SetCapture, SetCursor, SetWindowLongPtrW, SetWindowPos, SetWindowTextW, ShowWindow,
            TranslateMessage, UnregisterClassW, COLOR_BACKGROUND, CS_OWNDC, GET_WHEEL_DELTA_WPARAM, GWLP_USERDATA,
            GWL_STYLE, HWND_TOP, IDC_APPSTARTING, IDC_ARROW, IDC_CROSS, IDC_HAND, IDC_IBEAM, IDC_SIZEALL, IDC_SIZENESW,
            IDC_SIZENS, IDC_SIZENWSE, IDC_SIZEWE, IDC_UPARROW, IDC_WAIT, IMAGE_CURSOR, LR_DEFAULTSIZE, LR_SHARED, MSG,
            PM_REMOVE, SM_CXSCREEN, SM_CYSCREEN, SWP_NOMOVE, SWP_SHOWWINDOW, SW_HIDE, SW_SHOW, TME_LEAVE,
            TRACKMOUSEEVENT, WM_CLOSE, WM_ERASEBKGND, WM_KEYDOWN, WM_KEYUP, WM_LBUTTONDOWN, WM_LBUTTONUP,
            WM_MBUTTONDOWN, WM_MBUTTONUP, WM_MOUSELEAVE, WM_MOUSEMOVE, WM_MOUSEWHEEL, WM_RBUTTONDOWN, WM_RBUTTONUP,
            WM_SETCURSOR, WM_SIZE, WM_SIZING, WNDCLASSEXW, WS_CAPTION, WS_MAXIMIZEBOX, WS_MINIMIZEBOX, WS_POPUP,
            WS_SYSMENU, WS_THICKFRAME,
        },
    },
};

extern "C" {
    static __ImageBase: IMAGE_DOS_HEADER;
}
unsafe fn this_hinstance() -> HINSTANCE {
    &__ImageBase as *const _ as HINSTANCE
}

// re-registering a window class is an error.
static WINDOW_CLASS_ATOM: AtomicU16 = AtomicU16::new(0);

// so multiple windows don't destroy each other's window classes on drop
static WINDOW_COUNT: AtomicUsize = AtomicUsize::new(0);

pub struct WindowImpl {
    cursor: Cursor,
    hwnd: HWND,
    user_data: Box<WindowUserData>,
    style: Style,
    title: String,
    visible: bool,
}

struct WindowUserData {
    /// how much to add to client width and height to get full outer bounds
    border_offset: (c_int, c_int),

    /// the inner size of the window
    client_size: (c_int, c_int),

    /// whether closing the window was requested (X button)
    close_requested: bool,

    /// stored as to not re-emit stale mouse coordinates tracked outside the window
    mouse_cache: Option<(c_int, c_int)>,

    /// whether the mouse is tracked, as in is inside the window yielding events
    mouse_tracked: bool,

    /// whether the mouse is being tracked by the OS with _TrackMouseEvent()
    mouse_os_tracked: bool,

    /// event queue (cleared per-poll)
    events: Vec<Event>,

    /// yeah
    cursor_handle: HCURSOR,
}

impl Default for WindowUserData {
    fn default() -> Self {
        Self {
            border_offset: (0, 0),
            client_size: (0, 0),
            close_requested: false,
            mouse_cache: None,
            mouse_tracked: false,
            mouse_os_tracked: false,

            cursor_handle: ptr::null_mut(),
            events: Vec::with_capacity(8),
        }
    }
}

#[inline(always)]
unsafe fn hwnd_windowdata<'a>(hwnd: HWND) -> Option<&'a mut WindowUserData> {
    let lptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA);
    if lptr != 0 { Some(&mut *(lptr as *mut WindowUserData)) } else { None }
}

unsafe fn register_window_class() -> Result<ATOM, DWORD> {
    // can we get utf16 literals in rust please? i mean this isn't EXACTLY utf16 but it'd work
    static WINDOW_CLASS_WNAME: &[u8] = b"\0G\0M\08\0E\0m\0u\0l\0a\0t\0o\0r\0\0";

    let class = WNDCLASSEXW {
        cbSize: mem::size_of::<WNDCLASSEXW>() as UINT,
        style: CS_OWNDC,
        lpfnWndProc: Some(wnd_proc),
        hInstance: this_hinstance(),
        hCursor: ptr::null_mut(),
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

fn get_window_style(style: Style) -> DWORD {
    match style {
        Style::Regular => WS_CAPTION | WS_MINIMIZEBOX | WS_SYSMENU,
        Style::Resizable => WS_CAPTION | WS_SYSMENU | WS_THICKFRAME | WS_MINIMIZEBOX | WS_MAXIMIZEBOX,
        Style::Undecorated => WS_CAPTION,
        Style::Borderless => WS_POPUP,
        Style::BorderlessFullscreen => WS_POPUP,
    }
}

fn adjust_window_rect(width: c_int, height: c_int, style: Style) -> RECT {
    let mut rect = RECT { top: 0, left: 0, right: width, bottom: height };
    unsafe { AdjustWindowRect(&mut rect, get_window_style(style), FALSE) };
    rect
}

fn window_rect_wh(rect: RECT) -> (c_int, c_int) {
    let RECT { top, left, right, bottom } = rect;
    ((-left) + right, (-top) + bottom)
}

unsafe fn center_coords_primary_monitor(width: c_int, height: c_int) -> (c_int, c_int) {
    ((GetSystemMetrics(SM_CXSCREEN) / 2) - (width / 2), (GetSystemMetrics(SM_CYSCREEN) / 2) - (height / 2))
}

unsafe fn load_cursor(cursor: Cursor) -> HCURSOR {
    let name = match cursor {
        Cursor::Arrow => IDC_ARROW,
        Cursor::AppStart => IDC_APPSTARTING,
        Cursor::Beam => IDC_IBEAM,
        Cursor::Cross => IDC_CROSS,
        Cursor::Hand => IDC_HAND,
        Cursor::Hourglass => IDC_WAIT,
        Cursor::Invisible => return ptr::null_mut() as HCURSOR,
        Cursor::SizeNESW => IDC_SIZENESW,
        Cursor::SizeNS => IDC_SIZENS,
        Cursor::SizeNWSE => IDC_SIZENWSE,
        Cursor::SizeWE => IDC_SIZEWE,
        Cursor::SizeAll => IDC_SIZEALL,
        Cursor::Up => IDC_UPARROW,
    };
    LoadImageW(ptr::null_mut(), name, IMAGE_CURSOR, 0, 0, LR_DEFAULTSIZE | LR_SHARED) as HCURSOR
}

fn window_title_wstr(title: &str) -> Vec<wchar_t> {
    OsStr::new(title).encode_wide().chain(Some(0x00)).collect()
}

impl WindowImpl {
    pub fn new(builder: &WindowBuilder) -> Result<Self, String> {
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
        let mut client_width = builder.size.0.min(c_int::max_value() as u32) as c_int;
        let mut client_height = builder.size.1.min(c_int::max_value() as u32) as c_int;
        let title = window_title_wstr(&builder.title);
        unsafe {
            let (width, height, x_pos, y_pos) = {
                match builder.style {
                    Style::BorderlessFullscreen => {
                        let screen_width = GetSystemMetrics(SM_CXSCREEN);
                        let screen_height = GetSystemMetrics(SM_CYSCREEN);
                        client_width = screen_width;
                        client_height = screen_height;
                        (screen_width, screen_height, 0, 0)
                    },
                    _ => {
                        let rect = adjust_window_rect(client_width, client_height, builder.style);
                        let (width, height) = window_rect_wh(rect);
                        let (x_pos, y_pos) = center_coords_primary_monitor(width, height);
                        (width, height, x_pos, y_pos)
                    },
                }
            };
            let hwnd = CreateWindowExW(
                0,                               // dwExStyle
                class_atom as _,                 // lpClassName
                title.as_ptr(),                  // lpWindowName
                get_window_style(builder.style), // dwStyle
                x_pos,                           // X
                y_pos,                           // Y
                width,                           // nWidth
                height,                          // nHeight
                ptr::null_mut(),                 // hWndParent
                ptr::null_mut(),                 // hMenu
                this_hinstance(),                // hInstance
                ptr::null_mut(),                 // lpParam
            );
            if hwnd.is_null() {
                let code = GetLastError();
                return Err(format!("Failed to create window! (Code: {:#X})", code))
            }
            let mut user_data = Box::new(WindowUserData::default());
            user_data.border_offset = (width - client_width, height - client_height);
            user_data.client_size = (client_width, client_height);
            user_data.cursor_handle = load_cursor(builder.cursor);
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, user_data.as_ref() as *const _ as LONG_PTR as _);
            WINDOW_COUNT.fetch_add(1, atomic::Ordering::AcqRel);
            println!("Window Created! Window Count: {}", WINDOW_COUNT.load(atomic::Ordering::Acquire));
            Ok(Self {
                cursor: builder.cursor,
                style: builder.style,
                title: builder.title.to_owned(),
                visible: false,

                hwnd,
                user_data,
            })
        }
    }

    pub fn get_hwnd(&self) -> HWND {
        self.hwnd
    }
}

impl WindowTrait for WindowImpl {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn get_inner_size(&self) -> (u32, u32) {
        let (width, height) = self.user_data.client_size;
        (width as u32, height as u32)
    }

    fn close_requested(&self) -> bool {
        self.user_data.close_requested
    }

    fn set_close_requested(&mut self, value: bool) {
        self.user_data.close_requested = value;
    }

    fn process_events<'a>(&'a mut self) -> slice::Iter<'a, Event> {
        unsafe {
            self.user_data.events.clear();
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

            // if mouse out of bounds, calculate mouse pos, emit if changed
            if !self.user_data.mouse_tracked {
                let mut wrect: RECT = mem::zeroed();
                GetWindowRect(self.hwnd, &mut wrect);
                let mut mpoint: POINT = mem::zeroed();
                GetCursorPos(&mut mpoint);
                let (border_x, border_y) = self.user_data.border_offset;
                let window_x = wrect.left + border_x;
                let window_y = wrect.top + border_y;
                let x = mpoint.x - window_x;
                let y = mpoint.y - window_y;
                match self.user_data.mouse_cache {
                    Some((cx, cy)) if cx == x && cy == y => (),
                    _ => {
                        self.user_data.mouse_cache = Some((x, y));
                        self.user_data.events.push(Event::MouseMove(x, y));
                    },
                }
            }
        }
        self.user_data.events.iter()
    }

    fn resize(&mut self, width: u32, height: u32) {
        // TODO: does gamemaker adjust the X/Y to make sense for the new window?
        if let Style::BorderlessFullscreen = self.style {
            return
        }

        let (border_x, border_y) = self.user_data.border_offset;
        let width = border_x + (width as i32).max(0);
        let height = border_y + (height as i32).max(0);
        unsafe {
            SetWindowPos(self.hwnd, ptr::null_mut(), 0, 0, width, height, SWP_NOMOVE);
        }
    }

    fn get_cursor(&self) -> Cursor {
        self.cursor
    }

    fn set_cursor(&mut self, cursor: Cursor) {
        unsafe {
            // gm8 instantly sets the cursor when modified AND inside the game window
            self.cursor = cursor;
            self.user_data.cursor_handle = load_cursor(cursor);
            if self.user_data.mouse_tracked {
                SetCursor(self.user_data.cursor_handle);
            }
        }
    }

    fn set_style(&mut self, style: Style) {
        // they're casted from i32 in get_inner_size so this is fine
        let inner_size = self.get_inner_size();
        let (cwidth, cheight) = (inner_size.0 as i32, inner_size.1 as i32);
        unsafe {
            let window_rect = adjust_window_rect(cwidth as i32, cheight as i32, style);
            let (width, height) = window_rect_wh(window_rect);
            SetWindowLongPtrW(self.hwnd, GWL_STYLE, get_window_style(style) as LONG_PTR as _);
            SetWindowPos(self.hwnd, HWND_TOP, 0, 0, width, height, SWP_NOMOVE | SWP_SHOWWINDOW);
            self.user_data.border_offset = (width - cwidth, height - cheight);
            self.style = style;
        }
    }

    fn get_title(&self) -> &str {
        &self.title
    }

    fn set_title(&mut self, title: &str) {
        let wide_title = window_title_wstr(title);
        self.title = title.to_owned();
        unsafe {
            SetWindowTextW(self.hwnd, wide_title.as_ptr());
        }
    }

    fn get_visible(&self) -> bool {
        self.visible
    }

    fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
        let flag = if visible { SW_SHOW } else { SW_HIDE };
        unsafe {
            ShowWindow(self.hwnd, flag);
        }
    }

    fn window_handle(&self) -> usize {
        self.hwnd as _
    }
}

impl Drop for WindowImpl {
    fn drop(&mut self) {
        WINDOW_COUNT.fetch_sub(1, atomic::Ordering::AcqRel);
        if WINDOW_COUNT.load(atomic::Ordering::Acquire) == 0 {
            let atom = WINDOW_CLASS_ATOM.swap(0, atomic::Ordering::AcqRel);
            unsafe {
                UnregisterClassW(atom as _, this_hinstance());
            }
        }
        unsafe {
            DestroyWindow(self.hwnd);
        }
    }
}

unsafe extern "system" fn wnd_proc(hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_ERASEBKGND => return TRUE as LRESULT,
        WM_CLOSE => {
            if let Some(window_data) = hwnd_windowdata(hwnd) {
                window_data.close_requested = true;
            }
            return 0
        },

        // keyboard events
        WM_KEYDOWN => {
            if let Some(key) = Key::from_winapi(wparam as u8) {
                if let Some(window_data) = hwnd_windowdata(hwnd) {
                    window_data.events.push(Event::KeyboardDown(key));
                }
            }
            return 0
        },
        WM_KEYUP => {
            if let Some(key) = Key::from_winapi(wparam as u8) {
                if let Some(window_data) = hwnd_windowdata(hwnd) {
                    window_data.events.push(Event::KeyboardUp(key));
                }
            }
            return 0
        },

        // mouse events (yes, this is disgusting)
        WM_LBUTTONDOWN => {
            if let Some(window_data) = hwnd_windowdata(hwnd) {
                window_data.events.push(Event::MouseButtonDown(MouseButton::Left));
                SetCapture(hwnd);
            }
            return 0
        },
        WM_LBUTTONUP => {
            if let Some(window_data) = hwnd_windowdata(hwnd) {
                window_data.events.push(Event::MouseButtonUp(MouseButton::Left));
                ReleaseCapture();
            }
            return 0
        },
        WM_RBUTTONDOWN => {
            if let Some(window_data) = hwnd_windowdata(hwnd) {
                window_data.events.push(Event::MouseButtonDown(MouseButton::Right));
                SetCapture(hwnd);
            }
            return 0
        },
        WM_RBUTTONUP => {
            if let Some(window_data) = hwnd_windowdata(hwnd) {
                window_data.events.push(Event::MouseButtonUp(MouseButton::Right));
                ReleaseCapture();
            }
            return 0
        },
        WM_MBUTTONDOWN => {
            if let Some(window_data) = hwnd_windowdata(hwnd) {
                window_data.events.push(Event::MouseButtonDown(MouseButton::Middle));
                SetCapture(hwnd);
            }
            return 0
        },
        WM_MBUTTONUP => {
            if let Some(window_data) = hwnd_windowdata(hwnd) {
                window_data.events.push(Event::MouseButtonUp(MouseButton::Middle));
                ReleaseCapture();
            }
            return 0
        },
        WM_MOUSEWHEEL => {
            if let Some(window_data) = hwnd_windowdata(hwnd) {
                let delta = GET_WHEEL_DELTA_WPARAM(wparam);
                if delta < 0 {
                    window_data.events.push(Event::MouseWheelDown);
                } else if delta > 0 {
                    window_data.events.push(Event::MouseWheelUp);
                }
            }
            return 0
        },

        // mouse movements
        WM_MOUSEMOVE => {
            if let Some(window_data) = hwnd_windowdata(hwnd) {
                if !window_data.mouse_os_tracked {
                    window_data.mouse_os_tracked = true;
                    _TrackMouseEvent(&TRACKMOUSEEVENT {
                        cbSize: mem::size_of::<TRACKMOUSEEVENT>() as DWORD,
                        dwFlags: TME_LEAVE,
                        hwndTrack: hwnd,
                        dwHoverTime: 0,
                    } as *const _ as *mut _);
                }
                let x = GET_X_LPARAM(lparam);
                let y = GET_Y_LPARAM(lparam);
                window_data.mouse_tracked = true;
                window_data.events.push(Event::MouseMove(x, y));
            }
            return 0
        },
        WM_MOUSELEAVE => {
            if let Some(window_data) = hwnd_windowdata(hwnd) {
                window_data.mouse_tracked = false;
                window_data.mouse_os_tracked = false;
            }
            return 0
        },

        // window resizing
        WM_SIZE => {
            if let Some(window_data) = hwnd_windowdata(hwnd) {
                let width = u32::from(LOWORD(lparam as DWORD));
                let height = u32::from(HIWORD(lparam as DWORD));
                println!("WM_SIZE @ w: {}, h: {}", width, height);
                match window_data.events.last_mut() {
                    Some(Event::Resize(w, h)) => {
                        *w = width;
                        *h = height;
                    },
                    _ => window_data.events.push(Event::Resize(width, height)),
                }
                window_data.client_size = (width as i32, height as i32);
            }
        },
        WM_SIZING => {
            // We'd only use this if the window had its own thread.

            // let rect = &*(lparam as *const RECT);
            // let width = (rect.right - rect.left).max(0) as u32;
            // let height = (rect.bottom - rect.top).max(0) as u32;
            // let window_data = hwnd_windowdata(hwnd); <-- ACTUALLY CHECKED THOUGH
            // match window_data.events.last_mut() {
            //     Some(Event::Resize(w, h)) => {
            //         *w = width;
            //         *h = height;
            //     },
            //     _ => window_data.events.push(Event::Resize(width, height)),
            // }
            // window_data.inner_size = (width, height);
        },

        // cursor handling
        WM_SETCURSOR => {
            if let Some(window_data) = hwnd_windowdata(hwnd) {
                SetCursor(window_data.cursor_handle);
            } else {
                // I mean this shouldn't happen, but
                SetCursor(ptr::null_mut());
            }
            return TRUE as LRESULT
        },

        _ => (),
    }
    DefWindowProcW(hwnd, msg, wparam, lparam)
}
