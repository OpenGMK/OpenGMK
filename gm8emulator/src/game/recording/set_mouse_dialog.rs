use crate::{
    imgui,
    game::recording::window::{
        Window,
        DisplayInformation
    },
};

#[derive(Copy, Clone)]
pub enum MouseDialogResult {
    Ok(Option<(i32, i32)>),
    Cancel,
}

const TEXT_LENGTH: usize = 11; // How long the input string is allowed to be (2^31 in decimal is 10 characters + a potential sign)

pub struct SetMouseDialog {
    is_open: bool,
    x_text: String,
    y_text: String,

    x_valid: bool,
    y_valid: bool,

    x: i32,
    y: i32,

    result: Option<MouseDialogResult>,
}

// Set mouse dialog
impl Window for SetMouseDialog {
    fn name(&self) -> String {
        "Set Mouse".to_owned()
    }

    fn show_window(&mut self, info: &mut DisplayInformation) {
        let DisplayInformation {
            frame,
            ..
        } = info;

        frame.begin_window("Set Mouse", Some(imgui::Vec2(400.0, 105.0)), false, false, Some(&mut self.is_open));
        if self.is_open && !frame.window_collapsed() {
            Self::draw_text_sameline_validated(frame, self.x_valid, &"X Position");
            frame.input_text(&"##xpos", &mut self.x_text, 0, Some(TEXT_LENGTH));
            
            Self::draw_text_sameline_validated(frame, self.y_valid, &"Y Position");
            frame.input_text(&"##ypos", &mut self.y_text, 0, Some(TEXT_LENGTH));

            self.update_inputs();

            let button_size = imgui::Vec2(125.0, 20.0);
            if frame.button("Set", button_size, None) && self.x_valid && self.y_valid {
                self.result = Some(MouseDialogResult::Ok(Some((self.x, self.y))));
                self.is_open = false;
            }
            frame.same_line(0.0, -1.0);
            if frame.button("Unset", button_size, None) {
                self.result = Some(MouseDialogResult::Ok(None));
                self.is_open = false;
            }
            frame.same_line(0.0, -1.0);
            if frame.button("Cancel", button_size, None) {
                self.result = Some(MouseDialogResult::Cancel);
                self.is_open = false;
            }
        } else {
            self.is_open = false; // close window when minifying
            self.result = Some(MouseDialogResult::Cancel);
        }
        frame.end();
    }

    fn is_open(&self) -> bool { self.is_open }
}

impl SetMouseDialog {
    pub fn new() -> Self {
        Self {
            is_open: false,
            x_text: String::with_capacity(TEXT_LENGTH),
            y_text: String::with_capacity(TEXT_LENGTH),

            x_valid: false,
            y_valid: false,

            x: 0,
            y: 0,

            result: None,
        }
    }

    fn update_inputs(&mut self) {
        // X
        match self.x_text.parse::<i32>() {
            Ok(new_x) => {
                self.x_valid = true;
                self.x = new_x;
            },
            Err(_) => self.x_valid = false,
        }

        // Y
        match self.y_text.parse::<i32>() {
            Ok(new_y) => {
                self.y_valid = true;
                self.y = new_y;
            },
            Err(_) => self.y_valid = false,
        }
    }

    fn draw_text_sameline_validated(frame: &mut imgui::Frame, valid: bool, text: &str) {
        if !valid {
            unsafe { cimgui_sys::igPushStyleColorVec4(cimgui_sys::ImGuiCol__ImGuiCol_Text as _, cimgui_sys::ImVec4 { x: 1.0, y: 0.5, z: 0.5, w: 1.0 }); }
        }
        frame.text(text);
        if !valid {
            unsafe { cimgui_sys::igPopStyleColor(1); }
        }
        frame.same_line(0.0, -1.0);
    }

    pub fn init_if_closed(&mut self, new_mouse_pos: (i32, i32)) {
        if !self.is_open {
            self.is_open = true;
            //self.x_text_buffer.fill(0);
            //self.y_text_buffer.fill(0);
            self.x = new_mouse_pos.0;
            self.y = new_mouse_pos.1;
            self.result = None;

            // X
            self.x_text = self.x.to_string();
            self.x_valid = true;
            
            // Y
            self.y_text = self.y.to_string();
            self.y_valid = true;
        }
    }

    pub fn get_result(&self) -> Option<MouseDialogResult> {
        self.result
    }
}
