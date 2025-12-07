use crate::types::Colour;
use clipboard::{ClipboardContext, ClipboardProvider};
use imgui::{self, ImColor32, TableColumnSetup, WindowToken};

#[derive(Clone, Copy)]
pub struct Vec2<T>(pub T, pub T);

impl<T> Vec2<T> {
    pub fn new(x: T, y: T) -> Self {
        Self(x, y)
    }
}

impl From<Vec2<f32>> for imgui::sys::ImVec2 {
    fn from(vec2: Vec2<f32>) -> Self {
        let Vec2(x, y) = vec2;
        Self { x, y }
    }
}

impl From<imgui::sys::ImVec2> for Vec2<f32> {
    fn from(cvec2: imgui::sys::ImVec2) -> Self {
        Self(cvec2.x, cvec2.y)
    }
}

impl From<[f32; 2]> for Vec2<f32> {
    fn from(cvec2: [f32; 2]) -> Self {
        Self(cvec2[0], cvec2[1])
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

impl Into<ImColor32> for Colour {
    fn into(self) -> ImColor32 {
        imgui::ImColor32::from_rgba_f32s(self.r as f32, self.g as f32, self.b as f32, 1.0)
    }
}

pub trait UiCustomFunction {
    fn begin_context_menu(&self, pos: Vec2<f32>) -> WindowToken;
    fn button_with_size_and_pos(&self, name: &str, size: Vec2<f32>, position: Vec2<f32>) -> bool;
    fn invisible_button_with_size_and_pos(&self, name: &str, size: Vec2<f32>, position: Vec2<f32>) -> bool;
    fn coloured_text(&self, text: &str, col: Colour);
    fn text_centered(&self, text: &str, center: Vec2<f32>);
    fn callback<T>(
        &self,
        callback: unsafe extern "C" fn(*const c::ImDrawList, *const c::ImDrawCmd),
        data_ptr: &mut T,
    );
    fn mouse_pos(&self) -> Vec2<f32>;
    fn rect(&self, min: Vec2<f32>, max: Vec2<f32>, colour: Colour, alpha: u8);
    fn rect_outline(&self, min: Vec2<f32>, max: Vec2<f32>, colour: Colour, alpha: u8);
    fn get_held_keys(&self, include_mouse: bool, include_modifiers: bool) -> Vec<imgui::Key>;
    fn set_next_window_focus(&self);
    fn focus_current_window(&self);
    fn begin_screen_cover(&self, focus: bool) -> imgui::WindowToken;
    fn popup_notification(&self, message: &str) -> bool;
    fn text_centered_float(&self, text: &str, center: Vec2<f32>);
    fn item_spacing(&self) -> Vec2<f32>;
}

pub trait TableColumnSetupCustomFunction<Name: AsRef<str>> {
    fn with_flags(name: Name, flags: imgui::TableColumnFlags) -> TableColumnSetup<Name>;
    fn with_flags_and_init_width_or_weight(
        name: Name,
        flags: imgui::TableColumnFlags,
        init_width_or_weight: f32,
    ) -> TableColumnSetup<Name>;
}

use imgui::sys as c;

impl<Name: AsRef<str>> TableColumnSetupCustomFunction<Name> for TableColumnSetup<Name> {
    fn with_flags(name: Name, flags: imgui::TableColumnFlags) -> Self {
        Self {
            name,
            flags,
            init_width_or_weight: 0.0,
            user_id: imgui::Id::default(),
        }
    }
    fn with_flags_and_init_width_or_weight(name: Name, flags: imgui::TableColumnFlags, init_width_or_weight: f32) -> Self {
        Self {
            name,
            flags,
            init_width_or_weight,
            user_id: imgui::Id::default(),
        }
    }
}

fn index_to_imgui(index: usize, include_mouse: bool, include_modifiers: bool) -> Option<imgui::Key> {
    let button = if let Some(actual_index) = index.checked_sub(imgui::sys::ImGuiKey_NamedKey_BEGIN as _) {
        if actual_index < imgui::Key::COUNT {
            Some(imgui::Key::VARIANTS[actual_index])
        } else {
            None
        }
    } else {
        None
    };

    match button {
        Some(imgui::Key::MouseLeft)
        | Some(imgui::Key::MouseMiddle)
        | Some(imgui::Key::MouseRight)
        | Some(imgui::Key::MouseWheelX)
        | Some(imgui::Key::MouseWheelY)
        | Some(imgui::Key::MouseX1)
        | Some(imgui::Key::MouseX2) => {
            if include_mouse {
                button
            } else {
                None
            }
        },

        Some(imgui::Key::ModCtrl)
        | Some(imgui::Key::ModAlt)
        | Some(imgui::Key::ModShift)
        | Some(imgui::Key::ModSuper) => {
            if include_modifiers {
                button
            } else {
                None
            }
        },

        _ => button,
    }
}

impl UiCustomFunction for imgui::Ui {
    fn begin_context_menu(&self, pos: Vec2<f32>) -> WindowToken {
        let token = self
            .window("__popup")
            .title_bar(false)
            .resizable(false)
            .movable(false)
            .scroll_bar(false)
            .scrollable(false)
            .collapsible(false)
            .always_auto_resize(true)
            .save_settings(false)
            .begin()
            .expect("Failed to create context menu");

        let size: Vec2<f32> = self.window_size().into();
        let display_size: Vec2<f32> = self.io().display_size.into();
        unsafe {
            c::igSetWindowPos_Str(
                "__popup\0".as_ptr() as _,
                c::ImVec2 {
                    x: pos.0.min(display_size.0 - size.0),
                    y: if pos.1 + size.1 > display_size.1 && pos.1 >= size.1 {
                        pos.1 - size.1
                    } else {
                        pos.1
                    },
                },
                0,
            );
            if c::igIsWindowAppearing() {
                c::igSetWindowFocus_Nil();
            }
        }
        token
    }

    fn button_with_size_and_pos(&self, name: &str, size: Vec2<f32>, position: Vec2<f32>) -> bool {
        self.set_cursor_pos([position.0, position.1]);
        self.button_with_size(name, [size.0, size.1])
    }

    fn invisible_button_with_size_and_pos(&self, name: &str, size: Vec2<f32>, position: Vec2<f32>) -> bool {
        self.set_cursor_pos([position.0, position.1]);
        self.invisible_button(name, [size.0, size.1])
    }

    fn coloured_text(&self, text: &str, col: Colour) {
        self.text_colored([col.r as f32, col.g as f32, col.b as f32, 1.0], text)
    }

    fn text_centered(&self, text: &str, center: Vec2<f32>) {
        let [w, h] = self.calc_text_size(text);
        let size = Vec2(w / 2.0, h / 2.0);
        let pos = center - size;
        self.set_cursor_pos([pos.0, pos.1]);
        self.text(text);
    }

    fn callback<T>(
        &self,
        callback: unsafe extern "C" fn(*const c::ImDrawList, *const c::ImDrawCmd),
        data_ptr: &mut T,
    ) {
        unsafe {
            c::ImDrawList_AddCallback(c::igGetWindowDrawList(), Some(callback), data_ptr as *mut T as *mut _);
        }
    }

    fn mouse_pos(&self) -> Vec2<f32> {
        self.io().mouse_pos.into()
    }

    fn rect(&self, min: Vec2<f32>, max: Vec2<f32>, colour: Colour, alpha: u8) {
        let mut colour: ImColor32 = colour.into();
        colour.a = alpha;
        let min: [f32; 2] = [min.0, min.1];
        let max: [f32; 2] = [max.0, max.1];
        self.get_window_draw_list().add_rect(min, max, colour).filled(true).build();
    }

    fn rect_outline(&self, min: Vec2<f32>, max: Vec2<f32>, colour: Colour, alpha: u8) {
        let mut colour: ImColor32 = colour.into();
        colour.a = alpha;
        let min: [f32; 2] = [min.0, min.1];
        let max: [f32; 2] = [max.0, max.1];
        self.get_window_draw_list().add_rect(min, max, colour).build();
    }

    fn set_next_window_focus(&self) {
        unsafe {
            c::igSetNextWindowFocus();
        }
    }

    fn focus_current_window(&self) {
        unsafe {
            c::igSetWindowFocus_Nil();
        }
    }

    fn get_held_keys(&self, include_mouse: bool, include_modifiers: bool) -> Vec<imgui::Key> {
        self.io()
            .keys_down
            .into_iter()
            .skip(imgui::sys::ImGuiKey_NamedKey_BEGIN as _)
            .enumerate()
            .filter_map(|(index, pressed)| {
                if pressed {
                    if let Some(key) = index_to_imgui(
                        index + imgui::sys::ImGuiKey_NamedKey_BEGIN as usize,
                        include_mouse,
                        include_modifiers,
                    ) {
                        Some(key)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect()
    }

    fn begin_screen_cover(&self, focus: bool) -> imgui::WindowToken {
        self.window("__cover")
            .title_bar(false)
            .resizable(false)
            .movable(false)
            .scroll_bar(false)
            .collapsible(false)
            .save_settings(false)
            .focused(focus)
            .size(self.io().display_size, imgui::Condition::Always)
            .position([0.0, 0.0], imgui::Condition::Always)
            .begin()
            .expect("Error creating cover window")
    }

    fn popup_notification(&self, message: &str) -> bool {
        let cover = self.begin_screen_cover(true);
        if self.is_window_hovered() && self.is_mouse_clicked(imgui::MouseButton::Left) {
            false
        } else {
            cover.end();
            let screen_size = self.io().display_size;
            self.window("Information")
                .resizable(false)
                .movable(false)
                .scroll_bar(false)
                .scrollable(false)
                .collapsible(false)
                .focused(true)
                .position([screen_size[0] / 2.0, screen_size[1] / 2.0], imgui::Condition::Always)
                .position_pivot([0.5, 0.5])
                .build(|| {
                    self.text(message);
                });
            true
        }
    }

    fn text_centered_float(&self, text: &str, center: Vec2<f32>) {
        let size = self.calc_text_size(text);
        let size = Vec2((size[0] + 16.0) / 2.0, (size[1] + 16.0) / 2.0);
        let pos = [center.0 - size.0, center.1 - size.1];
        self.window("__float")
            // Note: Previous code used the 0x20000 flag which doesn't seem to exist (anymore?)
            //  I unfortunately can't remember what that was for, or whether or not that was a
            //  mistake or not but it seems to work fine without.
            .flags(
                  imgui::WindowFlags::NO_TITLE_BAR
                | imgui::WindowFlags::NO_RESIZE
                | imgui::WindowFlags::NO_MOVE
                | imgui::WindowFlags::NO_SCROLLBAR
                | imgui::WindowFlags::NO_SCROLL_WITH_MOUSE
                | imgui::WindowFlags::NO_COLLAPSE
                | imgui::WindowFlags::ALWAYS_AUTO_RESIZE
                | imgui::WindowFlags::NO_BACKGROUND
                | imgui::WindowFlags::NO_SAVED_SETTINGS
                | imgui::WindowFlags::NO_MOUSE_INPUTS
                | imgui::WindowFlags::NO_NAV_INPUTS
            )
            .position(pos, imgui::Condition::Always)
            .build(|| {
                self.text(text);
            });
    }

    fn item_spacing(&self) -> Vec2<f32> {
        unsafe { self.style().item_spacing.into() }
    }
}

pub trait IoCustomFunctions {
    fn clear_inputs(&mut self);
}

impl IoCustomFunctions for imgui::Io {
    fn clear_inputs(&mut self) {
        self.keys_down = [false; c::ImGuiKey_COUNT as usize];
        self.mouse_down = [false; 5];
    }
}
pub struct EmuClipboardProvider;
impl imgui::ClipboardBackend for EmuClipboardProvider {
    fn get(&mut self) -> Option<String> {
        if let Ok(ctx) = ClipboardProvider::new() {
            let mut ctx: ClipboardContext = ctx;
            ctx.get_contents().ok()
        } else {
            None
        }
    }

    fn set(&mut self, value: &str) {
        if let Ok(ctx) = ClipboardProvider::new() {
            let mut ctx: ClipboardContext = ctx;
            ctx.set_contents(value.to_owned()).unwrap();
        }
    }
}
