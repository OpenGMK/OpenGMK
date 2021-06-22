//! Custom wrappers for dear imgui.

use cimgui_sys as c;
use std::{ops, ptr::{self, NonNull}, slice};

pub struct Context {
    cbuf: Vec<u8>,
    ctx: NonNull<c::ImGuiContext>,
}

pub struct Frame<'a>(&'a mut Context);

#[repr(transparent)]
pub struct IO(c::ImGuiIO);

pub struct FontData<'a> {
    pub data: &'a [u8],
    pub size: (u32, u32),
}

impl Context {
    pub fn new() -> Self {
        match NonNull::new(unsafe { c::igCreateContext(ptr::null_mut()) }) {
            Some(ctx) => Self { cbuf: Vec::with_capacity(128), ctx },
            None => panic!("`ImGui::CreateContext` returned `nullptr`"),
        }
    }

    pub fn draw_data(&self) -> &c::ImDrawData {
        unsafe { &*c::igGetDrawData() }
    }

    pub fn make_current(&mut self) {
        unsafe { c::igSetCurrentContext(self.ctx.as_ptr()) };
    }

    pub fn new_frame(&mut self) -> Frame<'_> {
        unsafe { c::igNewFrame() };
        Frame(self)
    }

    pub fn io(&self) -> &mut IO {
        unsafe { &mut *(c::igGetIO() as *mut IO) }
    }
}

impl ops::Drop for Context {
    fn drop(&mut self) {
        unsafe {
            c::igDestroyContext(self.ctx.as_ptr());
        }
    }
}

impl Frame<'_> {
    fn cstr_store(&mut self, s: &str) {
        self.0.cbuf.clear();
        self.0.cbuf.extend_from_slice(s.as_bytes());
        self.0.cbuf.push(0);
    }

    fn cstr(&self) -> *const i8 {
        self.0.cbuf.as_ptr().cast()
    }

    fn _begin(&mut self, name: &str, is_open: &mut bool, flags: u32) {
        self.cstr_store(name);
        unsafe {
            c::igBegin(
                self.cstr(),
                is_open,
                flags as i32,
            );
        }
    }

    pub fn begin_window(
        &mut self,
        name: &str,
        size: Option<Vec2<f32>>,
        resizable: bool,
        menu_bar: bool,
        is_open: &mut bool,
    ) {
        if let Some(size) = size {
            unsafe { c::igSetNextWindowSize(size.into(), 0) };
        }
        self._begin(
            name,
            is_open,
            if menu_bar { c::ImGuiWindowFlags__ImGuiWindowFlags_MenuBar } else { 0 } |
                if !resizable { c::ImGuiWindowFlags__ImGuiWindowFlags_NoResize } else { 0 }
        )
    }

    pub fn end(&self) {
        unsafe { c::igEnd() };
    }

    pub fn window_position(&self) -> Vec2<f32> {
        unsafe {
            let mut pos = std::mem::MaybeUninit::uninit();
            c::igGetWindowPos(pos.as_mut_ptr());
            pos.assume_init().into()
        }
    }

    pub fn window_collapsed(&self) -> bool {
        unsafe {
            c::igIsWindowCollapsed()
        }
    }

    pub fn button(&mut self, name: &str, size: Vec2<f32>) -> bool {
        self.cstr_store(name);
        unsafe { c::igButton(self.cstr(), size.into()) }
    }

    pub fn text(&mut self, text: &str) {
        self.cstr_store(text);
        unsafe { c::igText(self.cstr()) };
    }

    pub fn callback<T>(&mut self, callback: unsafe extern "C" fn(*const c::ImDrawList, *const c::ImDrawCmd), data_ptr: &mut T) {
        unsafe {
            c::ImDrawList_AddCallback(c::igGetWindowDrawList(), Some(callback), data_ptr as *mut T as *mut _);
        }
    }

    pub fn render(self) {
        unsafe { c::igRender() };
    }
}

impl IO {
    pub fn font_data(&self) -> FontData<'_> {
        unsafe {
            let mut data: *mut u8 = ptr::null_mut();
            let mut width = 0;
            let mut height = 0;
            let mut bpp = 0;
            c::ImFontAtlas_GetTexDataAsRGBA32(
                self.0.Fonts,
                &mut data, &mut width, &mut height, &mut bpp,
            );
            assert!(!data.is_null());
            assert!(width >= 0);
            assert!(height >= 0);
            assert!(bpp > 0);
            FontData {
                data: slice::from_raw_parts(
                    data,
                    width as usize * height as usize * bpp as usize,
                ),
                size: (width as u32, height as u32),
            }
        }
    }

    pub fn set_delta_time(&mut self, delta: f32) {
        self.0.DeltaTime = delta;
    }

    pub fn set_display_size(&mut self, size: Vec2<f32>) {
        self.0.DisplaySize = size.into();
    }

    pub fn set_key(&mut self, key: usize, state: bool) {
        if let Some(entry) = self.0.KeysDown.get_mut(key) {
            *entry = state;
        }
    }

    pub fn set_ctrl(&mut self, state: bool) {
        self.0.KeyCtrl = state;
    }

    pub fn set_shift(&mut self, state: bool) {
        self.0.KeyShift = state;
    }

    pub fn set_alt(&mut self, state: bool) {
        self.0.KeyAlt = state;
    }

    pub fn set_mouse(&mut self, pos: Vec2<f32>) {
        self.0.MousePos = pos.into();
    }

    pub fn set_mouse_button(&mut self, btn: usize, state: bool) {
        if let Some(entry) = self.0.MouseDown.get_mut(btn) {
            *entry = state;
        }
    }

    pub fn set_mouse_wheel(&mut self, delta: f32) {
        self.0.MouseWheel = delta;
    }

    pub fn set_texture_id(&mut self, ptr: *mut ::std::ffi::c_void) {
        unsafe { (*self.0.Fonts).TexID = ptr };
    }
}

pub struct Vec2<T>(pub T, pub T);

impl From<Vec2<f32>> for c::ImVec2 {
    fn from(vec2: Vec2<f32>) -> Self {
        let Vec2(x, y) = vec2;
        Self { x, y }
    }
}

impl From<c::ImVec2> for Vec2<f32> {
    fn from(cvec2: c::ImVec2) -> Self {
        let c::ImVec2 { x, y } = cvec2;
        Self(x, y)
    }
}
