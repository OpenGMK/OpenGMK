//! Custom wrappers for dear imgui.

// Note to self: ImGui's popup API is bugged and doesn't do anything, don't use it. Make your own.
// Current hours wasted trying to use popup API in this file: 4

use crate::types::Colour;
use cimgui_sys as c;
use std::{
    ops,
    ptr::{self, NonNull},
    slice,
};

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

    pub fn frame_height(&self) -> f32 {
        unsafe { c::igGetFrameHeight() }
    }

    pub fn window_padding(&self) -> Vec2<f32> {
        unsafe { (*c::igGetStyle()).FramePadding.into() }
    }

    pub fn window_border_size(&self) -> f32 {
        unsafe { (*c::igGetStyle()).WindowBorderSize }
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

    fn _begin(&mut self, name: &str, is_open: Option<&mut bool>, flags: u32) {
        self.cstr_store(name);
        unsafe {
            c::igBegin(
                self.cstr(),
                match is_open {
                    Some(p) => p as _,
                    None => std::ptr::null_mut(),
                },
                flags as i32,
            );
        }
    }

    pub fn setup_next_window(
        &mut self,
        default_pos: Vec2<f32>,
        default_size: Option<Vec2<f32>>,
        min_size: Option<Vec2<f32>>,
    ) {
        unsafe {
            if let Some(min) = min_size {
                c::igSetNextWindowSizeConstraints(min.into(), (*c::igGetIO()).DisplaySize, None, std::ptr::null_mut());
            }
            if let Some(size) = default_size {
                c::igSetNextWindowSize(size.into(), 4);
            }
            c::igSetNextWindowPos(default_pos.into(), 4, c::ImVec2 { x: 0.0, y: 0.0 });
        }
    }

    pub fn begin_window(
        &mut self,
        name: &str,
        size: Option<Vec2<f32>>,
        resizable: bool,
        menu_bar: bool,
        is_open: Option<&mut bool>,
    ) {
        if let Some(size) = size {
            unsafe { c::igSetNextWindowSize(size.into(), 0) };
        }
        self._begin(
            name,
            is_open,
            if menu_bar { c::ImGuiWindowFlags__ImGuiWindowFlags_MenuBar } else { 0 }
                | if !resizable { c::ImGuiWindowFlags__ImGuiWindowFlags_NoResize } else { 0 },
        )
    }

    pub fn begin_context_menu(&mut self, pos: Vec2<f32>) {
        unsafe {
            c::igBegin("__popup\0".as_ptr() as _, std::ptr::null_mut(), 0b1_0111_1111);
            let mut size = std::mem::MaybeUninit::uninit();
            c::igGetWindowSize(size.as_mut_ptr());
            let size = size.assume_init();
            c::igSetWindowPosStr(
                "__popup\0".as_ptr() as _,
                c::ImVec2 {
                    x: pos.0.min((*c::igGetIO()).DisplaySize.x - size.x),
                    y: if pos.1 + size.y > (*c::igGetIO()).DisplaySize.y && pos.1 >= size.y {
                        pos.1 - size.y
                    } else {
                        pos.1
                    },
                },
                0,
            );
            if c::igIsWindowAppearing() {
                c::igSetWindowFocusNil();
            }
        }
    }

    pub fn begin_tree_node(&mut self, label: &str) -> bool {
        self.cstr_store(label);
        unsafe { c::igTreeNodeStr(self.cstr()) }
    }

    pub fn pop_tree_node(&mut self) {
        unsafe { c::igTreePop() }
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

    pub fn window_size(&self) -> Vec2<f32> {
        unsafe {
            let mut size = std::mem::MaybeUninit::uninit();
            c::igGetWindowSize(size.as_mut_ptr());
            size.assume_init().into()
        }
    }

    pub fn content_position(&self) -> Vec2<f32> {
        unsafe {
            let mut pos = std::mem::MaybeUninit::uninit();
            c::igGetWindowContentRegionMin(pos.as_mut_ptr());
            pos.assume_init().into()
        }
    }

    pub fn window_focused(&self) -> bool {
        unsafe { c::igIsWindowFocused(0) }
    }

    pub fn window_collapsed(&self) -> bool {
        unsafe { c::igIsWindowCollapsed() }
    }

    pub fn button(&mut self, name: &str, size: Vec2<f32>, position: Option<Vec2<f32>>) -> bool {
        self.cstr_store(name);
        unsafe {
            if let Some(pos) = position {
                c::igSetCursorPos(pos.into());
            }
            c::igButton(self.cstr(), size.into())
        }
    }

    pub fn invisible_button(&mut self, name: &str, size: Vec2<f32>, position: Option<Vec2<f32>>) -> bool {
        self.cstr_store(name);
        unsafe {
            if let Some(pos) = position {
                c::igSetCursorPos(pos.into());
            }
            c::igInvisibleButton(self.cstr(), size.into(), c::ImGuiButtonFlags__ImGuiButtonFlags_None as _)
        }
    }

    pub fn text(&mut self, text: &str) {
        self.cstr_store(text);
        unsafe { c::igText(self.cstr()) };
    }

    pub fn coloured_text(&mut self, text: &str, col: Colour) {
        self.cstr_store(text);
        unsafe { c::igTextColored(c::ImVec4 { x: col.r as _, y: col.g as _, z: col.b as _, w: 1.0 }, self.cstr()) }
    }

    pub fn text_centered(&mut self, text: &str, center: Vec2<f32>) {
        self.cstr_store(text);
        unsafe {
            let mut size = std::mem::MaybeUninit::uninit();
            c::igCalcTextSize(size.as_mut_ptr(), self.cstr(), std::ptr::null(), false, -1.0);
            let size = size.assume_init();
            let size = Vec2(size.x / 2.0, size.y / 2.0);
            c::igSetCursorPos((center - size).into());
            c::igText(self.cstr())
        }
    }

    pub fn menu_item(&mut self, label: &str) -> bool {
        self.cstr_store(label);
        unsafe { cimgui_sys::igMenuItemBool(self.cstr(), std::ptr::null(), false, true) }
    }

    pub fn callback<T>(
        &mut self,
        callback: unsafe extern "C" fn(*const c::ImDrawList, *const c::ImDrawCmd),
        data_ptr: &mut T,
    ) {
        unsafe {
            c::ImDrawList_AddCallback(c::igGetWindowDrawList(), Some(callback), data_ptr as *mut T as *mut _);
        }
    }

    pub fn key_pressed(&self, code: u8) -> bool {
        unsafe { c::igIsKeyPressed(code.into(), true) }
    }

    pub fn key_released(&self, code: u8) -> bool {
        unsafe { c::igIsKeyReleased(code.into()) }
    }

    pub fn mouse_pos(&self) -> Vec2<f32> {
        unsafe {
            let mut pos = std::mem::MaybeUninit::uninit();
            c::igGetMousePos(pos.as_mut_ptr());
            pos.assume_init().into()
        }
    }

    pub fn left_clicked(&self) -> bool {
        unsafe { c::igIsMouseClicked(0, false) }
    }

    pub fn right_clicked(&self) -> bool {
        unsafe { c::igIsMouseClicked(1, false) }
    }

    pub fn middle_clicked(&self) -> bool {
        unsafe { c::igIsMouseClicked(2, false) }
    }

    pub fn window_hovered(&self) -> bool {
        unsafe { c::igIsWindowHovered(0) }
    }

    pub fn item_hovered(&self) -> bool {
        unsafe { c::igIsItemHovered(0) }
    }

    pub fn rect(&mut self, min: Vec2<f32>, max: Vec2<f32>, colour: Colour, alpha: u8) {
        unsafe {
            c::ImDrawList_AddRectFilled(
                c::igGetWindowDrawList(),
                min.into(),
                max.into(),
                colour.as_decimal() | (u32::from(alpha) << 24),
                0.0,
                0,
            )
        }
    }

    pub fn rect_outline(&mut self, min: Vec2<f32>, max: Vec2<f32>, colour: Colour, alpha: u8) {
        unsafe {
            c::ImDrawList_AddRect(
                c::igGetWindowDrawList(),
                min.into(),
                max.into(),
                colour.as_decimal() | (u32::from(alpha) << 24),
                0.0,
                0,
                1.0,
            )
        }
    }

    pub fn begin_screen_cover(&mut self) {
        unsafe {
            c::igSetNextWindowFocus();
            c::igSetNextWindowSize((*c::igGetIO()).DisplaySize, 0);
            c::igSetNextWindowPos(Vec2(0.0, 0.0).into(), 0, Vec2(0.0, 0.0).into());
            c::igBegin("__cover\0".as_ptr() as _, std::ptr::null_mut(), 0b0001_0011_1111);
        }
    }

    pub fn popup(&mut self, message: &str) -> bool {
        unsafe {
            self.begin_screen_cover();
            if self.window_hovered() && self.left_clicked() {
                c::igEnd();
                false
            } else {
                c::igEnd();
                let screen_size = (*c::igGetIO()).DisplaySize;
                c::igSetNextWindowFocus();
                c::igSetNextWindowPos(
                    Vec2(f32::from(screen_size.x) / 2.0, f32::from(screen_size.y) / 2.0).into(),
                    0,
                    Vec2(0.5, 0.5).into(),
                );
                c::igBegin("Information\0".as_ptr() as _, std::ptr::null_mut(), 0b0001_0111_1110);
                self.text(message);
                c::igEnd();
                true
            }
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
            c::ImFontAtlas_GetTexDataAsRGBA32(self.0.Fonts, &mut data, &mut width, &mut height, &mut bpp);
            assert!(!data.is_null());
            assert!(width >= 0);
            assert!(height >= 0);
            assert!(bpp > 0);
            FontData {
                data: slice::from_raw_parts(data, width as usize * height as usize * bpp as usize),
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

    pub fn framerate(&self) -> f32 {
        self.0.Framerate
    }

    pub fn clear_inputs(&mut self) {
        self.0.KeysDown = [false; 512];
        self.0.MouseDown = [false; 5];
    }
}

#[derive(Clone, Copy)]
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

impl<T, O> std::ops::Add for Vec2<T>
where
    T: std::ops::Add<Output = O>,
{
    type Output = Vec2<O>;

    fn add(self, rhs: Self) -> Self::Output {
        Vec2(self.0 + rhs.0, self.1 + rhs.1)
    }
}

impl<T, O> std::ops::Sub for Vec2<T>
where
    T: std::ops::Sub<Output = O>,
{
    type Output = Vec2<O>;

    fn sub(self, rhs: Self) -> Self::Output {
        Vec2(self.0 - rhs.0, self.1 - rhs.1)
    }
}
