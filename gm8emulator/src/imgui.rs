//! Custom wrappers for dear imgui.

use cimgui_sys as c;
use std::{marker::PhantomData, ops, ptr::{self, NonNull}, slice};

#[repr(transparent)]
pub struct Context(NonNull<c::ImGuiContext>);
#[repr(transparent)]
pub struct IO(c::ImGuiIO);
pub struct Frame<'a>(PhantomData<&'a Context>);

pub struct FontData<'a> {
    pub data: &'a [u8],
    pub size: (u32, u32),
}

impl Context {
    pub fn new() -> Self {
        match NonNull::new(unsafe { c::igCreateContext(ptr::null_mut()) }) {
            Some(ctx) => Self(ctx),
            None => panic!("`ImGui::CreateContext` returned `nullptr`"),
        }
    }

    pub fn draw_data(&self) -> &c::ImDrawData {
        unsafe { &*c::igGetDrawData() }
    }

    pub fn make_current(&mut self) {
        unsafe { c::igSetCurrentContext(self.0.as_ptr()) };
    }

    pub fn new_frame(&self) -> Frame<'_> {
        unsafe { c::igNewFrame() };
        Frame(PhantomData)
    }

    pub fn io(&self) -> &mut IO {
        unsafe { &mut *(c::igGetIO() as *mut IO) }
    }
}

impl ops::Drop for Context {
    fn drop(&mut self) {
        unsafe {
            c::igDestroyContext(self.0.as_ptr());
        }
    }
}

impl Frame<'_> {
    pub fn begin(&self, name: &[u8], is_open: &mut bool) {
        debug_assert!(matches!(name.last().copied(), Some(0)));
        unsafe {
            c::igBegin(
                name.as_ptr().cast(),
                is_open,
                c::ImGuiWindowFlags__ImGuiWindowFlags_MenuBar as _,
            );
        }
    }

    pub fn end(&self) {
        unsafe { c::igEnd() };
    }

    pub fn button(&self, name: &[u8], size: Vec2<f32>) -> bool {
        debug_assert!(matches!(name.last().copied(), Some(0)));
        unsafe { c::igButton(name.as_ptr().cast(), size.into()) }
    }

    pub fn text(&self, name: &[u8]) {
        debug_assert!(matches!(name.last().copied(), Some(0)));
        unsafe { c::igText(name.as_ptr().cast()) };
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
        let mut map = self.0.KeysDown;
        if key < map.len() {
            map[key] = state;
        }
    }

    pub fn set_mouse(&mut self, pos: Vec2<f32>) {
        self.0.MousePos = pos.into();
    }

    pub fn set_mouse_button(&mut self, btn: usize, state: bool) {
        let mut map = self.0.MouseDown;
        if btn < map.len() {
            map[btn] = state;
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
