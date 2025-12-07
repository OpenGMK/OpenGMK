use imgui::StyleColor;

use crate::game::recording::window::{EmulatorContext, Window};

#[derive(Copy, Clone)]
pub enum MouseDialogResult {
    Ok(Option<(i32, i32)>),
    Cancel,
}

pub struct SetMouseDialog {
    is_open: bool,

    x: i32,
    y: i32,

    result: Option<MouseDialogResult>,
}

// Set mouse dialog
impl Window for SetMouseDialog {
    fn name(&self) -> String {
        "Set Mouse".to_owned()
    }

    fn show_window(&mut self, info: &mut EmulatorContext) {
        let frame = info.frame;

        let mut is_open = self.is_open;
        frame
            .window("Set Mouse")
            .size([400.0, 105.0], imgui::Condition::Always)
            .resizable(false)
            .opened(&mut is_open)
            .collapsible(false)
            .build(|| {
                Self::draw_text_sameline_validated(frame, true, &"X Position");
                frame.input_int("##xpos", &mut self.x).build();

                Self::draw_text_sameline_validated(frame, true, &"Y Position");
                frame.input_int("##ypos", &mut self.y).build();

                let button_size = [125.0, 20.0];
                if frame.button_with_size("Set", button_size) {
                    self.result = Some(MouseDialogResult::Ok(Some((self.x, self.y))));
                    self.is_open = false;
                }
                frame.same_line();
                if frame.button_with_size("Unset", button_size) {
                    self.result = Some(MouseDialogResult::Ok(None));
                    self.is_open = false;
                }
                frame.same_line();
                if frame.button_with_size("Cancel", button_size) {
                    self.result = Some(MouseDialogResult::Cancel);
                    self.is_open = false;
                }
            });

        if self.is_open && !is_open {
            self.result = Some(MouseDialogResult::Cancel);
            self.is_open = false;
        }
    }

    fn is_open(&self) -> bool {
        self.is_open
    }
}

impl SetMouseDialog {
    pub fn new() -> Self {
        Self {
            is_open: false,

            x: 0,
            y: 0,

            result: None,
        }
    }

    fn draw_text_sameline_validated(frame: &imgui::Ui, valid: bool, text: &str) {
        if !valid {
            let color = frame.push_style_color(StyleColor::Text, [1.0, 0.5, 0.5, 1.0]);
            frame.text(text);
            color.end();
        } else {
            frame.text(text);
        }
        frame.same_line();
    }

    pub fn init_if_closed(&mut self, new_mouse_pos: (i32, i32)) {
        if !self.is_open {
            self.is_open = true;
            self.x = new_mouse_pos.0;
            self.y = new_mouse_pos.1;
            self.result = None;
        }
    }

    pub fn get_result(&self) -> Option<MouseDialogResult> {
        self.result
    }
}
