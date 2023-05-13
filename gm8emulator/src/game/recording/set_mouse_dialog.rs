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

const BUFFER_LENGTH: usize = 15;

pub struct SetMouseDialog {
    is_open: bool,
    x_text_buffer: Vec<u8>,
    y_text_buffer: Vec<u8>,

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
            frame.input_text(&"##xpos", self.x_text_buffer.as_mut_ptr(), self.x_text_buffer.len(), 0);
            
            Self::draw_text_sameline_validated(frame, self.y_valid, &"Y Position");
            frame.input_text(&"##ypos", self.y_text_buffer.as_mut_ptr(), self.y_text_buffer.len(), 0);

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
            x_text_buffer: vec![0 as u8; BUFFER_LENGTH],
            y_text_buffer: vec![0 as u8; BUFFER_LENGTH],

            x_valid: false,
            y_valid: false,

            x: 0,
            y: 0,

            result: None,
        }
    }

    fn update_inputs(&mut self) {
        // X
        match String::from_utf8(self.x_text_buffer.iter().take_while(|x| **x != 0u8).copied().collect()) {
            Ok(x_string) => {
                match x_string.parse::<i32>() {
                    Ok(new_x) => {
                        self.x_valid = true;
                        self.x = new_x;
                    },
                    Err(_) => self.x_valid = false,
                }
            },
            Err(_) => self.x_valid = false,
        }

        // Y
        match String::from_utf8(self.y_text_buffer.iter().take_while(|x| **x != 0u8).copied().collect()) {
            Ok(y_string) => {
                match y_string.parse::<i32>() {
                    Ok(new_y) => {
                        self.y_valid = true;
                        self.y = new_y;
                    },
                    Err(_) => self.y_valid = false,
                }
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
            self.x_text_buffer = self.x.to_string().as_bytes().to_vec();
            self.x_text_buffer.push(0);
            while self.x_text_buffer.len() < BUFFER_LENGTH {
                self.x_text_buffer.push(0);
            }
            self.x_valid = true;

            // Y
            self.y_text_buffer = self.y.to_string().as_bytes().to_vec();
            self.y_text_buffer.push(0);
            while self.y_text_buffer.len() < BUFFER_LENGTH {
                self.y_text_buffer.push(0);
            }
            self.y_valid = true;
        }
    }

    pub fn get_result(&self) -> Option<MouseDialogResult> {
        self.result
    }
}
