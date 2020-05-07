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
        minwindef::{ATOM, DWORD, FALSE, HINSTANCE, HIWORD, LOWORD, LPARAM, LRESULT, TRUE, UINT, WPARAM},
        windef::{HBRUSH, HWND, POINT, RECT},
        windowsx::{GET_X_LPARAM, GET_Y_LPARAM},
    },
    um::{
        commctrl::_TrackMouseEvent,
        errhandlingapi::GetLastError,
        winnt::IMAGE_DOS_HEADER,
        winuser::{
            AdjustWindowRect, BeginPaint, CreateWindowExW, DefWindowProcW, DispatchMessageW, EndPaint, GetCursorPos,
            GetSystemMetrics, GetWindowLongPtrW, GetWindowRect, LoadCursorW, PeekMessageW, RegisterClassExW,
            ReleaseCapture, SetCapture, SetWindowLongPtrW, SetWindowPos, ShowWindow, TranslateMessage,
            UnregisterClassW, COLOR_BACKGROUND, CS_OWNDC, GET_WHEEL_DELTA_WPARAM, GWLP_USERDATA, GWL_STYLE, IDC_ARROW,
            MSG, PAINTSTRUCT, PM_REMOVE, SM_CXSCREEN, SM_CYSCREEN, SWP_NOMOVE, SW_HIDE, SW_SHOW, TME_LEAVE,
            TRACKMOUSEEVENT, WM_CLOSE, WM_ERASEBKGND, WM_KEYDOWN, WM_KEYUP, WM_LBUTTONDOWN, WM_LBUTTONUP,
            WM_MBUTTONDOWN, WM_MBUTTONUP, WM_MOUSELEAVE, WM_MOUSEMOVE, WM_MOUSEWHEEL, WM_PAINT, WM_RBUTTONDOWN,
            WM_RBUTTONUP, WM_SIZE, WM_SIZING, WNDCLASSEXW, WS_CAPTION, WS_MAXIMIZEBOX, WS_MINIMIZEBOX, WS_OVERLAPPED,
            WS_POPUP, WS_SYSMENU, WS_THICKFRAME,
        },
    },
};

// TODO: This might not work on MinGW (GCC)? Time for CI to find out!
extern "C" {
    static __ImageBase: IMAGE_DOS_HEADER;
}

#[inline(always)]
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
        Style::Undecorated => WS_CAPTION,
        Style::Borderless => WS_POPUP,
        Style::BorderlessFullscreen => unimplemented!("no fullscreen yet"),
    }
}

struct WindowData {
    close_requested: bool,
    events: Vec<Event>,

    border_offset: (i32, i32), // for adjusting outside client area mouse coordinates
    mouse_tracked: bool,       // whether it's inside the client area
    mouse_os_tracked: bool,    // whether the OS is tracking for a window leave event
    mouse_cache: (i32, i32),   // for outside area updates

    inner_size: (u32, u32),
}

impl Default for WindowData {
    fn default() -> Self {
        Self {
            border_offset: (0, 0),
            close_requested: false,
            events: Vec::new(),
            mouse_tracked: false,
            mouse_os_tracked: false,
            mouse_cache: (i32::min_value(), i32::min_value()),
            inner_size: (0, 0),
        }
    }
}

#[inline(always)]
unsafe fn hwnd_windowdata<'a>(hwnd: HWND) -> &'a mut WindowData {
    let lptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA);
    &mut *(lptr as *mut WindowData)
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
            // TODO: is this even necessary? I don't think so?
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

        // keyboard events
        WM_KEYDOWN => {
            if let Some(key) = Key::from_winapi(wparam as u8) {
                hwnd_windowdata(hwnd).events.push(Event::KeyboardDown(key));
            }
            return 0
        },
        WM_KEYUP => {
            if let Some(key) = Key::from_winapi(wparam as u8) {
                hwnd_windowdata(hwnd).events.push(Event::KeyboardUp(key));
            }
            return 0
        },

        // mouse events (yes, this is disgusting)
        WM_LBUTTONDOWN => {
            hwnd_windowdata(hwnd).events.push(Event::MouseButtonDown(MouseButton::Left));
            SetCapture(hwnd);
            return 0
        },
        WM_LBUTTONUP => {
            hwnd_windowdata(hwnd).events.push(Event::MouseButtonUp(MouseButton::Left));
            ReleaseCapture();
            return 0
        },
        WM_RBUTTONDOWN => {
            hwnd_windowdata(hwnd).events.push(Event::MouseButtonDown(MouseButton::Right));
            SetCapture(hwnd);
            return 0
        },
        WM_RBUTTONUP => {
            hwnd_windowdata(hwnd).events.push(Event::MouseButtonUp(MouseButton::Right));
            ReleaseCapture();
            return 0
        },
        WM_MBUTTONDOWN => {
            hwnd_windowdata(hwnd).events.push(Event::MouseButtonDown(MouseButton::Middle));
            SetCapture(hwnd);
            return 0
        },
        WM_MBUTTONUP => {
            hwnd_windowdata(hwnd).events.push(Event::MouseButtonUp(MouseButton::Middle));
            ReleaseCapture();
            return 0
        },
        WM_MOUSEWHEEL => {
            let delta = GET_WHEEL_DELTA_WPARAM(wparam);
            let window_data = hwnd_windowdata(hwnd);
            if delta < 0 {
                window_data.events.push(Event::MouseWheelDown);
            } else {
                window_data.events.push(Event::MouseWheelUp);
            }
            return 0
        },

        // mouse movements
        WM_MOUSEMOVE => {
            let window_data = hwnd_windowdata(hwnd);
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
            return 0
        },
        WM_MOUSELEAVE => {
            let window_data = hwnd_windowdata(hwnd);
            window_data.mouse_tracked = false;
            window_data.mouse_os_tracked = false;
            return 0
        },

        // window resizing
        WM_SIZE => {
            let window_data = hwnd_windowdata(hwnd);
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
            window_data.inner_size = (width, height);
        },
        WM_SIZING => {
            // We'd only use this if the window had its own thread.

            // let rect = &*(lparam as *const RECT);
            // let width = (rect.right - rect.left).max(0) as u32;
            // let height = (rect.bottom - rect.top).max(0) as u32;
            // let window_data = hwnd_windowdata(hwnd);
            // match window_data.events.last_mut() {
            //     Some(Event::Resize(w, h)) => {
            //         *w = width;
            //         *h = height;
            //     },
            //     _ => window_data.events.push(Event::Resize(width, height)),
            // }
            // window_data.inner_size = (width, height);
        },

        _ => (),
    }
    DefWindowProcW(hwnd, msg, wparam, lparam)
}

unsafe fn window_borders(width: c_int, height: c_int, style: DWORD) -> RECT {
    let mut rect = RECT { top: 0, left: 0, right: width, bottom: height };
    AdjustWindowRect(&mut rect, style, FALSE);
    rect
}

fn window_rect_flat(rect: &RECT) -> (i32, i32) {
    let RECT { top, left, right, bottom } = rect;
    ((-left) + right, (-top) + bottom)
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
        let client_width = width.min(i32::max_value() as u32) as i32;
        let client_height = height.min(i32::max_value() as u32) as i32;
        let style = get_window_style(Style::Regular);
        let title = OsStr::new(title).encode_wide().chain(Some(0x00)).collect::<Vec<wchar_t>>();
        let (extra, hwnd) = unsafe {
            let window_rect = window_borders(client_width, client_height, style);
            let (width, height) = window_rect_flat(&window_rect);
            let hwnd = CreateWindowExW(
                0,                                                  // dwExStyle
                class_atom as _,                                    // lpClassName
                title.as_ptr(),                                     // lpWindowName
                style,                                              // dwStyle
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
            let mut extra = Box::new(WindowData::default());
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, extra.as_ref() as *const _ as LONG_PTR);
            extra.border_offset = (-window_rect.left, -window_rect.top);
            extra.inner_size = (client_width as u32, client_height as u32);
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

    fn get_inner_size(&self) -> (u32, u32) {
        self.extra.inner_size
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
            if !window_data.mouse_tracked {
                let mut wrect: RECT = mem::zeroed();
                GetWindowRect(self.hwnd, &mut wrect);
                let mut mpoint: POINT = mem::zeroed();
                GetCursorPos(&mut mpoint);

                // calculate mouse pos
                let window_x = wrect.left + window_data.border_offset.0;
                let window_y = wrect.top + window_data.border_offset.1;
                let x = mpoint.x - window_x;
                let y = mpoint.y - window_y;
                if (x, y) != window_data.mouse_cache {
                    window_data.mouse_cache = (x, y);
                    window_data.events.push(Event::MouseMove(x, y));
                }
            }

            window_data.events.iter()
        }
    }

    fn resize(&mut self, width: u32, height: u32) {
        unsafe {
            // TODO: does gamemaker adjust the X/Y to make sense for the new window?
            SetWindowPos(self.hwnd, ptr::null_mut(), 0, 0, (width as i32).max(0), (height as i32).max(0), SWP_NOMOVE);
        }
    }

    fn set_style(&mut self, style: Style) {
        unsafe {
            let wstyle = get_window_style(style);
            let (cwidth, cheight) = self.get_inner_size();
            let window_rect = window_borders(cwidth as i32, cheight as i32, wstyle);
            let (width, height) = window_rect_flat(&window_rect);
            SetWindowLongPtrW(self.hwnd, GWL_STYLE, wstyle as LONG_PTR);
            SetWindowPos(self.hwnd, ptr::null_mut(), 0, 0, width, height, SWP_NOMOVE);
            self.extra.border_offset = (-window_rect.left, -window_rect.top);
        }
    }

    fn set_visible(&mut self, visible: bool) {
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
        let count = WINDOW_COUNT.fetch_sub(1, atomic::Ordering::AcqRel);
        if count == 0 {
            let atom = WINDOW_CLASS_ATOM.swap(0, atomic::Ordering::AcqRel);
            unsafe {
                UnregisterClassW(atom as _, get_hinstance());
            }
        }
    }
}
