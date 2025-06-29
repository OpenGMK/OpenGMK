use imgui;

#[derive(Clone, Copy)]
pub struct Vec2<T>(pub T, pub T);

impl<T> Vec2<T> {
    pub fn new(x: T, y: T) -> Self {
        Self (x, y)
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

use crate::types::Colour;

pub trait UiCustomFunction {
    fn setup_next_window(
        &mut self,
        default_pos: Vec2<f32>,
        default_size: Option<Vec2<f32>>,
        min_size: Option<Vec2<f32>>,
    );
    fn begin_context_menu(&mut self, pos: Vec2<f32>);
    fn end(&self);
    fn button_with_size_and_pos(&self, name: &str, size: Vec2<f32>, position: Vec2<f32>) -> bool;
    fn invisible_button_with_size_and_pos(&self, name: &str, size: Vec2<f32>, position: Vec2<f32>) -> bool;
    fn coloured_text(&self, text: &str, col: Colour);
    fn text_centered(&self, text: &str, center: Vec2<f32>);
    fn callback<T>(
        &self,
        callback: unsafe extern "C" fn(*const c::ImDrawList, *const c::ImDrawCmd),
        data_ptr: &mut T,
    );
    fn key_pressed(&self, code: u8) -> bool;
    fn key_released(&self, code: u8) -> bool;
    fn mouse_pos(&self) -> Vec2<f32>;
    fn rect(&self, min: Vec2<f32>, max: Vec2<f32>, colour: Colour, alpha: u8);
    fn rect_outline(&self, min: Vec2<f32>, max: Vec2<f32>, colour: Colour, alpha: u8);
    fn begin_screen_cover(&mut self);
    fn popup(&mut self, message: &str) -> bool;
}

use imgui::sys as c;

impl UiCustomFunction for imgui::Ui {
    fn setup_next_window(
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

    fn begin_context_menu(&mut self, pos: Vec2<f32>) {
        unsafe {
            c::igBegin("__popup\0".as_ptr() as _, std::ptr::null_mut(), 0b1_0111_1111);
            let mut size = std::mem::MaybeUninit::uninit();
            c::igGetWindowSize(size.as_mut_ptr());
            let size = size.assume_init();
            c::igSetWindowPos_Str(
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
                c::igSetWindowFocus_Nil();
            }
        }
    }

    fn end(&self) {
        unsafe { c::igEnd() };
    }

    fn button_with_size_and_pos(&self, name: &str, size: Vec2<f32>, position: Vec2<f32>) -> bool
    {
        self.set_cursor_pos([position.0, position.1]);
        self.button_with_size(name, [size.0, size.1])
    }
    
    fn invisible_button_with_size_and_pos(&self, name: &str, size: Vec2<f32>, position: Vec2<f32>) -> bool
    {
        self.set_cursor_pos([position.0, position.1]);
        self.invisible_button(name, [size.0, size.1])
    }
    
    fn coloured_text(&self, text: &str, col: Colour) {
        self.text_colored(
            [col.r as f32, col.g as f32, col.b as f32, 1.0],
            text
        )
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

    fn key_pressed(&self, code: u8) -> bool {
        unsafe { c::igIsKeyPressed(code.into(), true) }
    }

    fn key_released(&self, code: u8) -> bool {
        unsafe { c::igIsKeyReleased(code.into()) }
    }

    fn mouse_pos(&self) -> Vec2<f32> {
        unsafe {
            let mut pos = std::mem::MaybeUninit::uninit();
            c::igGetMousePos(pos.as_mut_ptr());
            pos.assume_init().into()
        }
    }

    fn rect(&self, min: Vec2<f32>, max: Vec2<f32>, colour: Colour, alpha: u8) {
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

    fn rect_outline(&self, min: Vec2<f32>, max: Vec2<f32>, colour: Colour, alpha: u8) {
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

    fn begin_screen_cover(&mut self) {
        unsafe {
            c::igSetNextWindowFocus();
            c::igSetNextWindowSize((*c::igGetIO()).DisplaySize, 0);
            c::igSetNextWindowPos(Vec2(0.0, 0.0).into(), 0, Vec2(0.0, 0.0).into());
            c::igBegin("__cover\0".as_ptr() as _, std::ptr::null_mut(), 0b0001_0011_1111);
        }
    }

    fn popup(&mut self, message: &str) -> bool {
        unsafe {
            self.begin_screen_cover();
            if self.is_window_hovered() && self.is_mouse_clicked(imgui::MouseButton::Left) {
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