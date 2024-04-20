use crate::{game::{recording::window::DisplayInformation, replay::FrameRng}, imgui};

use super::{DialogState, Dialog};

pub struct StringInputPopup {
    name: &'static str,
    input_buffer: String,
    char_limit: Option<usize>,
    is_open: bool,
}

pub struct RNGSelect {
    input: StringInputPopup,
    result: Option<FrameRng>
}

impl Dialog for StringInputPopup {
    fn show(&mut self, info: &mut DisplayInformation) -> DialogState {
        let DisplayInformation {
            frame,
            keybindings,
            ..
        } = info;
        let mut state: DialogState = DialogState::Closed;

        if frame.begin_popup_modal(&self.name) {
            state = DialogState::Open;

            if !self.is_open {
                self.is_open = true;
                frame.set_keyboard_focus_here(0); // Auto-focus textbox if we just opened the dialog
            }

            let submitted = frame.input_text(&"##textinput", &mut self.input_buffer, cimgui_sys::ImGuiInputTextFlags__ImGuiInputTextFlags_EnterReturnsTrue as _, self.char_limit);
            if frame.is_item_focused() {
                keybindings.disable_bindings();
            }
            if submitted || frame.button("Submit", imgui::Vec2(50.0, 20.0), None) {
                frame.close_current_popup();
                state = DialogState::Submit;
            }

            frame.same_line(0.0, 5.0);
            if frame.button("Cancel", imgui::Vec2(50.0, 20.0), None) {
                frame.close_current_popup();
                state = DialogState::Cancelled;
            }
            frame.end_popup();
        }

        state
    }

    fn get_name(&self) -> &'static str {
        self.name
    }

    fn reset(&mut self) {
        self.input_buffer.clear();
        self.is_open = false;
    }
}

impl StringInputPopup {
    pub fn new(name: &'static str, char_limit: Option<usize>) -> StringInputPopup {
        StringInputPopup {
            name,
            input_buffer: String::with_capacity(char_limit.unwrap_or(128)),
            char_limit,
            is_open: false,
        }
    }

    pub fn get_string(&self) -> &String {
        &self.input_buffer
    }
}

impl Dialog for RNGSelect {
    fn show(&mut self, info: &mut DisplayInformation) -> DialogState {
        match self.input.show(info) {
            DialogState::Submit => {
                let str = self.input.get_string();
                if str.len() == 0 { // If we leave the textbox empty, assume we want to unset the RNG
                    self.result = None;
                    DialogState::Submit
                } else {
                    match str.parse::<i32>() {
                        Ok(value) => {
                            if str.starts_with("+") {
                                // If the number is prefixed by a +, the RNG should be incremented that many times
                                self.result = Some(FrameRng::Increment(value));
                            } else {
                                // Otherwise just set the seed to that
                                self.result = Some(FrameRng::Override(value));
                            }
                            DialogState::Submit
                        },
                        Err(_) => DialogState::Invalid
                    }
                }
            },
            state => state
        }
    }

    fn get_name(&self) -> &'static str {
        self.input.get_name()
    }

    fn reset(&mut self) {
        self.input.reset();
        self.result = None;
    }
}

impl RNGSelect {
    pub fn new(name: &'static str, ) -> RNGSelect {
        RNGSelect {
            input: StringInputPopup::new(name, Some(11)), // 2^32 is 10 characters in decimal + a potential sign for the number makes 11
            result: None,
        }
    }

    /// Function returns the FrameRng that was entered into the box.
    /// Only calls this if show() returned DialogResult::Submit, otherwise it may panic if no result is stored.
    pub fn get_result(&self) -> Option<FrameRng> {
        self.result.clone()
    }
}
