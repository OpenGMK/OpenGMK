//! Custom wrappers for dear imgui.

// Note to self: ImGui's popup API is bugged and doesn't do anything, don't use it. Make your own.
// Current hours wasted trying to use popup API in this file: 4

use crate::{
    input,
    types::Colour,
};
use cimgui_sys as c;
use std::{
    ops, os::raw::c_void, ptr::{self, NonNull}, slice
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

struct UserData<'a> {
    string: &'a mut String,
    char_limit: Option<usize>,
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

    fn _begin(&mut self, name: &str, is_open: Option<&mut bool>, flags: u32) -> bool {
        self.cstr_store(name);
        unsafe {
            c::igBegin(
                self.cstr(),
                match is_open {
                    Some(p) => p as _,
                    None => std::ptr::null_mut(),
                },
                flags as i32,
            )
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

    pub fn set_next_window_focus(&self) {
        unsafe { c::igSetNextWindowFocus(); }
    }

    pub fn begin_window(
        &mut self,
        name: &str,
        size: Option<Vec2<f32>>,
        resizable: bool,
        menu_bar: bool,
        is_open: Option<&mut bool>,
    ) -> bool {
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

    pub fn set_scroll_here_y(&self, center_y_ratio: f32) {
        unsafe { c::igSetScrollHereY(center_y_ratio) }
    }

    pub fn set_next_item_width(&self, width: f32) {
        unsafe { c::igSetNextItemWidth(width) }
    }

    pub fn begin_listbox(&mut self, label: &str, size: Vec2<f32>) -> bool {
        self.cstr_store(label);
        unsafe { c::igBeginListBox(self.cstr(), size.into()) }
    }

    pub fn end_listbox(&self) {
        unsafe { c::igEndListBox() }
    }

    pub fn begin_table(&mut self, label: &str, column: i32, flags: c::ImGuiTableFlags, outer_size: Vec2<f32>, inner_width: f32) -> bool {
        self.cstr_store(label);
        unsafe { c::igBeginTable(self.cstr(), column, flags, outer_size.into(), inner_width) }
    }

    pub fn end_table(&self) {
        unsafe { c::igEndTable() };
    }

    pub fn table_setup_scroll_freeze(&self, columns: i32, rows: i32) {
        unsafe { c::igTableSetupScrollFreeze(columns, rows); }
    }

    pub fn table_headers_row(&self) {
        unsafe { c::igTableHeadersRow(); }
    }

    pub fn table_next_row(&self, row_flags: c::ImGuiTableRowFlags, min_row_height: f32) {
        unsafe { c::igTableNextRow(row_flags, min_row_height) };
    }

    pub fn table_next_column(&self) -> bool {
        unsafe { c::igTableNextColumn() }
    }

    pub fn table_set_column_index(&self, column_n: i32) -> bool {
        unsafe { c::igTableSetColumnIndex(column_n) }
    }

    pub fn table_setup_column(&mut self, label: &str, flags: c::ImGuiTableColumnFlags, init_width_or_weight: f32) {
        self.cstr_store(label);
        unsafe { c::igTableSetupColumn(self.cstr(), flags, init_width_or_weight, 0) };
    }

    pub fn same_line(&self, offset_from_start_x: f32, spacing: f32) {
        unsafe { c::igSameLine(offset_from_start_x, spacing) };
    }

    pub fn set_scroll_y(&self, position: f32) {
        unsafe { c::igSetScrollYFloat(position); }
    }

    pub fn get_scroll_x(&self) -> f32 {
        unsafe { c::igGetScrollX() }
    }

    pub fn get_scroll_max_x(&self) -> f32 {
        unsafe { c::igGetScrollMaxX() }
    }

    pub fn get_scroll_y(&self) -> f32 {
        unsafe { c::igGetScrollY() }
    }

    pub fn get_scroll_max_y(&self) -> f32 {
        unsafe { c::igGetScrollMaxY() }
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
    pub fn content_position_max(&self) -> Vec2<f32> {
        unsafe {
            let mut pos = std::mem::MaybeUninit::uninit();
            c::igGetWindowContentRegionMax(pos.as_mut_ptr());
            pos.assume_init().into()
        }
    }

    pub fn get_content_size(&self) -> Vec2<f32> {
        self.content_position_max()-self.content_position()
    }

    pub fn is_item_focused(&self) -> bool {
        unsafe { c::igIsItemFocused() }
    }

    pub fn set_keyboard_focus_here(&self, offset: i32) {
        unsafe { c::igSetKeyboardFocusHere(offset) }
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

    extern "C" fn input_string_callback(callbackdata: *mut c::ImGuiInputTextCallbackData) -> i32 {
        let data = unsafe { &mut *callbackdata };
        let userdata = unsafe { &mut *(data.UserData as *mut UserData) };
        if data.EventFlag == c::ImGuiInputTextFlags__ImGuiInputTextFlags_CallbackResize as _ {
            // The text changed, check if we need to resize the buffer
            assert!(userdata.string.as_mut_ptr() == data.Buf as _);

            if userdata.string.capacity() <= data.BufTextLen as _ {
                // If capacity == text length we are missing 1 byte for the null terminator, resize our buffer
                userdata.string.reserve(data.BufTextLen as usize - userdata.string.len() + 1);
                data.Buf = userdata.string.as_mut_ptr() as _;
            }
        } else if data.EventFlag == c::ImGuiInputTextFlags__ImGuiInputTextFlags_CallbackEdit as _ {
            // The text changed (but we aren't applying it yet), check if we have a character limit and limit the new string to that.
            // We can't do that in resize because while it does have the new text length in bytes, it does not have any way to check the amount of actual characters in the new string, which might differ if the user entered non-ascii characters
            if let Some(char_limit) = userdata.char_limit {
                if char_limit < data.BufTextLen as _ { // data.BufTextLen might not be accurate to the amount of characters but it gives a good estimate for whether or not we need to count
                    let str = unsafe { std::str::from_utf8(std::slice::from_raw_parts(data.Buf as _, data.BufTextLen as _)).unwrap() }; // imgui should always return a valid utf8 string
                    if let Some((_index, (byte_pos, _char))) = str.char_indices().enumerate().find(|(index, (_, _))| *index >= char_limit) {
                        unsafe { *data.Buf.add(byte_pos) = 0; }
                        data.BufTextLen = byte_pos as _;
                        data.BufDirty = true;
                    }
                }
            }
        }

        0 // I don't know what this is for but the imgui code seems to indicate that it's not even read so just return 0
    }

    pub fn input_text(&mut self, label: &str, string: &mut String, flags: c::ImGuiInputTextFlags, char_limit: Option<usize>) -> bool {
        self.cstr_store(label);
        unsafe {
            let buffer = string.as_mut_ptr();
            let len = string.len();
            let capacity = string.capacity();

            if *buffer.add(len) != 0 {
                *buffer.add(len) = 0; // String must be null terminated, if the current string does not end in a null byte, add it. This should be fine since len() will always point to char boundary and be <= capacity, thus be valid for adding a new \0 character into the buffer without breaking utf-8 or going past it's managed bounds
            }

            let mut userdata = UserData {
                string,
                char_limit,
            };
            let callback_flags: i32 = if char_limit.is_some() {
                c::ImGuiInputTextFlags__ImGuiInputTextFlags_CallbackResize |
                c::ImGuiInputTextFlags__ImGuiInputTextFlags_CallbackEdit
            } else {
                c::ImGuiInputTextFlags__ImGuiInputTextFlags_CallbackResize
            } as _;
            let result = c::igInputText(
                self.cstr(),
                buffer as *mut i8,
                capacity,
                flags | callback_flags,
                 Some(Self::input_string_callback),
                 &mut userdata as *mut _ as *mut c_void
            );

            // Buffer and capacity might have been changed at this point, don't assume they're still the same
            let slice = slice::from_raw_parts(string.as_mut_ptr(), string.capacity());
            if let Some(new_len) = slice.iter().position(|x| *x == 0) {
                string.as_mut_vec().set_len(new_len);
            }

            result
        }
    }

    pub fn checkbox(&mut self, label: &str, value: &mut bool) -> bool {
        self.cstr_store(label);
        unsafe { c::igCheckbox(self.cstr(), value as _) }
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

    pub fn text_centered_float(&mut self, text: &str, center: Vec2<f32>) {
        self.cstr_store(text);
        unsafe {
            let mut size = std::mem::MaybeUninit::uninit();
            c::igCalcTextSize(size.as_mut_ptr(), self.cstr(), std::ptr::null(), false, -1.0);
            let size = size.assume_init();
            let size = Vec2((size.x+16.0) / 2.0, (size.y+16.0) / 2.0);
            let pos: c::ImVec2 = (center - size).into();
            c::igBegin("__float\0".as_ptr() as _, std::ptr::null_mut(), 0b110_0000_0011_1111_1111);
            c::igSetWindowPosStr(
                "__float\0".as_ptr() as _,
                pos,
                0,
            );
            if c::igIsWindowAppearing() {
                c::igSetWindowFocusNil();
            }
            //c::igSetCursorPos();
            c::igText(self.cstr());
            c::igEnd();
        }
    }

    pub fn begin_menu_main_bar(&self) -> bool {
        unsafe { c::igBeginMainMenuBar() }
    }

    pub fn end_menu_main_bar(&self) {
        unsafe { c::igEndMainMenuBar(); }
    }

    pub fn begin_menu(&mut self, label: &str, enabled: bool) -> bool {
        self.cstr_store(label);
        unsafe { c::igBeginMenu(self.cstr(), enabled) }
    }

    pub fn end_menu(&self) {
        unsafe { c::igEndMenu(); }
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

    pub fn ctrl_down(&self) -> bool {
        self.0.io().get_ctrl()
    }

    pub fn shift_down(&self) -> bool {
        self.0.io().get_shift()
    }

    pub fn alt_down(&self) -> bool {
        self.0.io().get_alt()
    }

    pub fn get_keys(&self) -> Vec<u8> {
        self.0.io()
            .get_keys()
            .into_iter()
            .take(255)
            .enumerate()
            .filter_map(|(key, &pressed)| {
                if pressed {
                    Some(key as u8)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn key_down(&self, code: u8) -> bool {
        unsafe { c::igIsKeyDown(code.into()) }
    }

    pub fn key_pressed_norepeat(&self, code: u8) -> bool {
        unsafe { c::igIsKeyPressed(code.into(), false) }
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

    pub fn left_released(&self) -> bool {
        unsafe { c::igIsMouseReleased(0) }
    }

    pub fn right_released(&self) -> bool {
        unsafe { c::igIsMouseReleased(1) }
    }

    pub fn left_down(&self) -> bool {
        unsafe { c::igIsMouseDown(0) }
    }

    pub fn right_down(&self) -> bool {
        unsafe { c::igIsMouseDown(1) }
    }

    pub fn mouse_down(&self) -> bool {
        unsafe { c::igIsAnyMouseDown() }
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

    pub fn get_item_rect_min(&self) -> Vec2<f32> {
        let mut min = c::ImVec2 { x: 0.0, y: 0.0 };
        unsafe { c::igGetItemRectMin(&mut min as *mut c::ImVec2); }

        min.into()
    }

    pub fn get_item_rect_max(&self) -> Vec2<f32> {
        let mut max = c::ImVec2 { x: 0.0, y: 0.0 };
        unsafe { c::igGetItemRectMax(&mut max as *mut c::ImVec2); }

        max.into()
    }

    pub fn get_item_rect_size(&self) -> Vec2<f32> {
        let mut size = c::ImVec2 { x: 0.0, y: 0.0 };
        unsafe { c::igGetItemRectSize(&mut size as *mut c::ImVec2); }

        size.into()
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
                c::igSetNextWindowFocus();
                self.center_next_window();
                c::igBegin("Information\0".as_ptr() as _, std::ptr::null_mut(), 0b0001_0111_1110);
                self.text(message);
                c::igEnd();
                true
            }
        }
    }

    pub fn open_popup(&mut self, name: &str) {
        self.cstr_store(name);
        unsafe {
            c::igOpenPopup(self.cstr(), 0)
        }
    }

    pub fn begin_popup_modal(&mut self, name: &str) -> bool {
        self.cstr_store(name);
        let mut open: bool = true;
        unsafe { c::igBeginPopupModal(self.cstr(), &mut open as _, (c::ImGuiWindowFlags__ImGuiWindowFlags_AlwaysAutoResize|c::ImGuiWindowFlags__ImGuiWindowFlags_NoMove) as _) }
    }

    pub fn center_next_window(&self) {
        unsafe {
            let screen_size = (*c::igGetIO()).DisplaySize;
            c::igSetNextWindowPos(
                Vec2(f32::from(screen_size.x) / 2.0, f32::from(screen_size.y) / 2.0).into(),
                0,
                Vec2(0.5, 0.5).into(),
            );
        }
    }

    pub fn close_current_popup(&self) {
        unsafe { c::igCloseCurrentPopup() }
    }

    pub fn end_popup(&self) {
        unsafe { c::igEndPopup() }
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

    pub fn get_keys(&self) -> &[bool; 512] {
        &self.0.KeysDown
    }

    pub fn set_ctrl(&mut self, state: bool) {
        self.0.KeyCtrl = state;
    }

    pub fn get_ctrl(&self) -> bool {
        self.0.KeyCtrl
    }

    pub fn set_shift(&mut self, state: bool) {
        self.0.KeyShift = state;
    }

    pub fn get_shift(&self) -> bool {
        self.0.KeyShift
    }

    pub fn set_alt(&mut self, state: bool) {
        self.0.KeyAlt = state;
    }

    pub fn get_alt(&self) -> bool {
        self.0.KeyAlt
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

    pub fn add_input_character(&mut self, chr: char) {
        let mut b = [0u16; 2];
        let section = chr.encode_utf16(&mut b);
        unsafe {
            for chars in section {
                c::ImGuiIO_AddInputCharacterUTF16(&mut self.0, *chars);
            }
        }
    }

    pub fn setup_default_keymap(&mut self) {
        for i in 0..c::ImGuiKey__ImGuiKey_COUNT {
            self.0.KeyMap[i as usize] = match i {
                c::ImGuiKey__ImGuiKey_Tab => input::Button::Tab as _,
                c::ImGuiKey__ImGuiKey_LeftArrow => input::Button::LeftArrow as _,
                c::ImGuiKey__ImGuiKey_RightArrow => input::Button::RightArrow as _,
                c::ImGuiKey__ImGuiKey_UpArrow => input::Button::UpArrow as _,
                c::ImGuiKey__ImGuiKey_DownArrow => input::Button::DownArrow as _,
                c::ImGuiKey__ImGuiKey_PageUp => input::Button::PageUp as _,
                c::ImGuiKey__ImGuiKey_PageDown => input::Button::PageDown as _,
                c::ImGuiKey__ImGuiKey_Home => input::Button::Home as _,
                c::ImGuiKey__ImGuiKey_End => input::Button::End as _,
                c::ImGuiKey__ImGuiKey_Insert => input::Button::Insert as _,
                c::ImGuiKey__ImGuiKey_Delete => input::Button::Delete as _,
                c::ImGuiKey__ImGuiKey_Backspace => input::Button::Backspace as _,
                c::ImGuiKey__ImGuiKey_Space => input::Button::Space as _,
                c::ImGuiKey__ImGuiKey_Enter => input::Button::Return as _,
                c::ImGuiKey__ImGuiKey_Escape => input::Button::Escape as _,
                c::ImGuiKey__ImGuiKey_KeyPadEnter => input::Button::Return as _,
                c::ImGuiKey__ImGuiKey_A => input::Button::A as _,
                c::ImGuiKey__ImGuiKey_C => input::Button::C as _,
                c::ImGuiKey__ImGuiKey_V => input::Button::V as _,
                c::ImGuiKey__ImGuiKey_X => input::Button::X as _,
                c::ImGuiKey__ImGuiKey_Y => input::Button::Y as _,
                c::ImGuiKey__ImGuiKey_Z => input::Button::Z as _,
                _ => -1,
            }
        }
    }

    pub fn framerate(&self) -> f32 {
        self.0.Framerate
    }

    pub fn clear_inputs(&mut self) {
        self.0.KeysDown = [false; 512];
        self.0.MouseDown = [false; 5];
        self.0.KeyCtrl = false;
        self.0.KeyAlt = false;
        self.0.KeyShift = false;
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
