use crate::{
    input::Button,
    game::recording::{
            KeyState,
            keybinds::Binding,
            window::{Window, Openable, DisplayInformation},
        },
};

pub struct MacroWindow {
    macro_string: String,
    start_frame: usize,
    last_frame: usize,
    info_text: String,
    is_open: bool,
    input_frames: Vec<Vec<StateChange>>,

    repeat_macro: bool,
    run_macro: bool,

    id: usize,
}

#[derive(Copy, Clone)]
enum StateChange {
    // Click(keyboard_state index)
    Click(usize),
    // ChangeTo(keyboard_state index, target keystate)
    ChangeTo(usize, KeyState),
}

// Macro window
impl Window for MacroWindow {
    fn stored_kind(&self) -> Option<super::WindowKind> {
        Some(super::WindowKind::Macro(self.id))
    }
    
    fn window_id(&self) -> usize {
        self.id
    }

    fn name(&self) -> String {
        format!("Macro {}", self.id+1)
    }

    fn show_window(&mut self, info: &mut DisplayInformation) {
        self.show_macro_windows(info);
    }

    fn is_open(&self) -> bool { self.is_open }
}
impl Openable<Self> for MacroWindow {
    fn window_name() -> &'static str {
        "Macro Window"
    }

    fn open(id: usize) -> Self {
        let mut window = Self::new();
        window.id = id;

        window
    }
}

// Macro window
impl MacroWindow {
    pub fn new() -> Self {
        MacroWindow {
            macro_string: String::with_capacity(256),
            start_frame: 0,
            last_frame: 0,
            info_text: String::from("Not running"),
            input_frames: Vec::new(),
            is_open: true,
            run_macro: false,
            repeat_macro: false,
            id: 0,
        }
    }

    fn show_macro_windows(&mut self, info: &mut DisplayInformation) {
        let keybind_pressed = info.keybind_pressed(Binding::ToggleMacros);
        let DisplayInformation {
            keyboard_state,
            keybindings,
            config,
            frame,
            ..
        } = info;

        frame.begin_window(&self.name(), None, true, false, Some(&mut self.is_open));

        let mut flags: i32 = 0;
        if self.run_macro {
            flags = cimgui_sys::ImGuiInputTextFlags__ImGuiInputTextFlags_ReadOnly as _;
        }
        
        let window_size = frame.window_size();
        let content_position = frame.content_position();
        frame.set_next_item_width(window_size.0 - content_position.0 * 2.0);
        frame.input_text(&"##macroinput", &mut self.macro_string, flags, None);
        if frame.is_item_focused() {
            keybindings.disable_bindings();
        }

        let pressed = frame.checkbox("Run Macro", &mut self.run_macro) || keybind_pressed;
        if keybind_pressed {
            self.run_macro = !self.run_macro;
        }
        frame.same_line(0.0, -1.0);
        frame.checkbox("Repeat Macro", &mut self.repeat_macro);
        if pressed {
            if self.run_macro {
                self.start_frame = config.current_frame;
                self.update_macro();
            } else {
                self.info_text = String::from("Not running");
            }
        }
        
        frame.text(&self.info_text);
        
        // Apply the current frame of the macro if the frame has changed
        if self.run_macro && self.last_frame != config.current_frame {
            self.last_frame = config.current_frame;

            let current_frame = config.current_frame.checked_sub(self.start_frame).unwrap_or(0);
            let index = if !self.repeat_macro && current_frame >= self.input_frames.len() {
                self.input_frames.len() - 1
            } else {
                current_frame % self.input_frames.len()
            };

            // if the start frame is before the current frame
            if self.start_frame <= config.current_frame {
                self.info_text = format!("Macro Frame {}/{}", index, self.input_frames.len() - 1);

                // and we repeat the macro or are still at the first iteration
                if self.repeat_macro || config.current_frame - self.start_frame < self.input_frames.len() {
                    let current_frame = self.input_frames.get(index).unwrap();
                    
                    for entry in current_frame {
                        match entry {
                            StateChange::Click(index) => keyboard_state.get_mut(*index).unwrap().click(),
                            StateChange::ChangeTo(index, state) => keyboard_state.get_mut(*index).unwrap().reset_to_state(*state),
                        }
                    }
                }
            } else {
                self.info_text = format!("Macro Frame -{}/{}", self.start_frame - config.current_frame, self.input_frames.len() - 1);
            }
        }

        frame.end();
    }

    fn update_macro(&mut self) {
        self.last_frame = 0;
        self.input_frames.clear();

        self.parse_macro_string();
    }

    fn parse_macro_string(&mut self) {
        let mut token = String::new();
        let mut last_keycode: Option<usize> = None;
        let mut changes: Vec<StateChange> = Vec::new();
        let mut state_change: Option<StateChange> = None;

        macro_rules! unexpected_token {
            () => {{
                self.info_text = format!("Unexpected token '{}'", token);
                self.input_frames.clear();
                self.run_macro = false;
                return;
            }};
            ($str:expr) => {{
                self.info_text = format!($str, token);
                self.input_frames.clear();
                self.run_macro = false;
                return;
            }};
        }

        let mut chars = self.macro_string.chars().peekable();
        while let Some(next_tt) = self.next_token(&mut chars, &mut token) {
            match next_tt {
                TokenType::Key => {
                    if state_change.is_some() {
                        unexpected_token!();
                    } else if let Some(btn) = Button::try_from_str(token.as_str()) {
                        last_keycode = Some(btn as _);
                        state_change = Some(StateChange::Click(last_keycode.unwrap()));
                    } else {
                        unexpected_token!("Couldn't identify button '{}'");
                    };
                },
                TokenType::PreviousKey => {
                    if state_change.is_some() {
                        unexpected_token!();
                    } else if last_keycode.is_some() {
                        state_change = Some(StateChange::Click(last_keycode.unwrap()));
                    } else {
                        unexpected_token!("Cannot use token '{}' as no button was previously defined");
                    }
                },
                TokenType::Modifier => {
                    if matches!(state_change, None | Some(StateChange::ChangeTo(..))) || last_keycode.is_none() {
                        unexpected_token!();
                    }
                    match token.as_str() {
                        "(N)" => state_change = Some(StateChange::ChangeTo(last_keycode.unwrap(), KeyState::Neutral)),
                        "(R)" => state_change = Some(StateChange::ChangeTo(last_keycode.unwrap(), KeyState::NeutralWillCactus)),
                        "(RP)" => state_change = Some(StateChange::ChangeTo(last_keycode.unwrap(), KeyState::HeldWillDouble)),
                        "(RPR)" => state_change = Some(StateChange::ChangeTo(last_keycode.unwrap(), KeyState::HeldWillTriple)),
                        "(H)" => state_change = Some(StateChange::ChangeTo(last_keycode.unwrap(), KeyState::Held)),
                        "(P)" => state_change = Some(StateChange::ChangeTo(last_keycode.unwrap(), KeyState::NeutralWillPress)),
                        "(PR)" => state_change = Some(StateChange::ChangeTo(last_keycode.unwrap(), KeyState::NeutralWillDouble)),
                        "(PRP)" => state_change = Some(StateChange::ChangeTo(last_keycode.unwrap(), KeyState::NeutralWillTriple)),
                        _ => { unexpected_token!("Unknown modifier token '{}'") },
                    }
                },
                TokenType::KeySeparator => {
                    if state_change.is_none() {
                        unexpected_token!();
                    }
                    changes.push(state_change.unwrap());
                    state_change = None;
                },
                TokenType::NextFrame => {
                    if state_change.is_some() {
                        changes.push(state_change.unwrap());
                        state_change = None;
                    }

                    self.input_frames.push(changes.clone());
                    changes.clear();
                },
                TokenType::Number => {
                    unexpected_token!();
                },
            };
        }
        if state_change.is_some() {
            changes.push(state_change.unwrap());
        }

        self.input_frames.push(changes.clone());
        changes.clear();
    }

    fn next_token(&self, chars: &mut std::iter::Peekable<std::str::Chars<'_>>, token: &mut String) -> Option<TokenType> {
        let mut tt: Option<TokenType> = None;
        let mut token_valid = true;
        macro_rules! check_no_tokentype {
            () => {
                if tt.is_some() {
                    if !(matches!(tt, Some(TokenType::Key) | Some(TokenType::Number))) {
                        tt = None;
                    }
                    break;
                }
            };
        }

        token.clear();
        while let Some(chr) = chars.peek() {
            macro_rules! push_char {
                () => {
                    token.push(chars.next().unwrap())
                };
            }
            if chr.is_alphanumeric() {
                match tt {
                    None => {
                        tt = Some(if chr.is_alphabetic() { TokenType::Key } else { TokenType::Number });
                        push_char!();
                    },
                    Some(TokenType::Key) => {
                        push_char!();
                    },
                    Some(TokenType::Number) => {
                        if chr.is_numeric() {
                            push_char!();
                        } else {
                            tt = None;
                            break;
                        }
                    },
                    Some(TokenType::Modifier) => {
                        if *chr == 'P' || *chr == 'R' {
                            push_char!();
                        } else {
                            tt = None;
                            break;
                        }
                    }
                    _ => {
                        tt = None;
                        break;
                    }
                }
            } else {
                match chr {
                    '.' => {
                        check_no_tokentype!();
                        tt = Some(TokenType::PreviousKey);
                        push_char!();
                        break;
                    },
                    '>' => {
                        check_no_tokentype!();
                        push_char!();
                        tt = Some(TokenType::NextFrame);
                        break;
                    },
                    ',' => {
                        check_no_tokentype!();
                        push_char!();
                        tt = Some(TokenType::KeySeparator);
                        break;
                    },
                    '(' => {
                        check_no_tokentype!();
                        push_char!();
                        tt = Some(TokenType::Modifier);
                        token_valid = false;
                    },
                    ')' => {
                        if tt == Some(TokenType::Modifier) {
                            push_char!();
                            token_valid = true;
                            break;
                        } else {
                            tt = None;
                            break;
                        }
                    }
                    _ => {
                        tt = None;
                        break;
                    }
                }
            }
        };

        if token_valid { tt } else { None }
    }
}

#[derive(Eq, PartialEq)]
enum TokenType {
    // A key (ex: A, B, C, LeftShift, etc)
    Key,
    // Refers to the most recently identified key
    PreviousKey,
    // A key modifier (ex: (R), (PR), (P) etc)
    Modifier,
    // A key separator (,)
    KeySeparator,
    // Next frame
    NextFrame,
    // A number (a decimal number)
    Number,
}

impl Button {
    pub fn try_from_str(value: &str) -> Option<Button> {
        match value {
            "MouseLeft" => Some(Self::MouseLeft),
            "MouseRight" => Some(Self::MouseRight),
            "MouseMiddle" => Some(Self::MouseMiddle),
            "MouseX1" => Some(Self::MouseX1),
            "MouseX2" => Some(Self::MouseX2),
            "Backspace" => Some(Self::Backspace),
            "Tab" => Some(Self::Tab),
            "Clear" => Some(Self::Clear),
            "Return" => Some(Self::Return),
            "Shift" => Some(Self::Shift),
            "Control" => Some(Self::Control),
            "Alt" => Some(Self::Alt),
            "Pause" => Some(Self::Pause),
            "CapsLock" => Some(Self::CapsLock),
            "ImeKanaOrHangul" => Some(Self::ImeKanaOrHangul),
            "ImeOn" => Some(Self::ImeOn),
            "ImeJunja" => Some(Self::ImeJunja),
            "ImeFinal" => Some(Self::ImeFinal),
            "ImeHanjaOrKanji" => Some(Self::ImeHanjaOrKanji),
            "ImeOff" => Some(Self::ImeOff),
            "Escape" => Some(Self::Escape),
            "ImeConvert" => Some(Self::ImeConvert),
            "ImeNonConvert" => Some(Self::ImeNonConvert),
            "ImeAccept" => Some(Self::ImeAccept),
            "ImeModeChangeRequest" => Some(Self::ImeModeChangeRequest),
            "Space" => Some(Self::Space),
            "PageUp" => Some(Self::PageUp),
            "PageDown" => Some(Self::PageDown),
            "End" => Some(Self::End),
            "Home" => Some(Self::Home),
            "LeftArrow" => Some(Self::LeftArrow),
            "UpArrow" => Some(Self::UpArrow),
            "RightArrow" => Some(Self::RightArrow),
            "DownArrow" => Some(Self::DownArrow),
            "Select" => Some(Self::Select),
            "Print" => Some(Self::Print),
            "Execute" => Some(Self::Execute),
            "PrintScreen" => Some(Self::PrintScreen),
            "Insert" => Some(Self::Insert),
            "Delete" => Some(Self::Delete),
            "Help" => Some(Self::Help),
            "Alpha0" => Some(Self::Alpha0),
            "Alpha1" => Some(Self::Alpha1),
            "Alpha2" => Some(Self::Alpha2),
            "Alpha3" => Some(Self::Alpha3),
            "Alpha4" => Some(Self::Alpha4),
            "Alpha5" => Some(Self::Alpha5),
            "Alpha6" => Some(Self::Alpha6),
            "Alpha7" => Some(Self::Alpha7),
            "Alpha8" => Some(Self::Alpha8),
            "Alpha9" => Some(Self::Alpha9),
            "A" => Some(Self::A),
            "B" => Some(Self::B),
            "C" => Some(Self::C),
            "D" => Some(Self::D),
            "E" => Some(Self::E),
            "F" => Some(Self::F),
            "G" => Some(Self::G),
            "H" => Some(Self::H),
            "I" => Some(Self::I),
            "J" => Some(Self::J),
            "K" => Some(Self::K),
            "L" => Some(Self::L),
            "M" => Some(Self::M),
            "N" => Some(Self::N),
            "O" => Some(Self::O),
            "P" => Some(Self::P),
            "Q" => Some(Self::Q),
            "R" => Some(Self::R),
            "S" => Some(Self::S),
            "T" => Some(Self::T),
            "U" => Some(Self::U),
            "V" => Some(Self::V),
            "W" => Some(Self::W),
            "X" => Some(Self::X),
            "Y" => Some(Self::Y),
            "Z" => Some(Self::Z),
            "LeftWindows" => Some(Self::LeftWindows),
            "RightWindows" => Some(Self::RightWindows),
            "Applications" => Some(Self::Applications),
            "Sleep" => Some(Self::Sleep),
            "Keypad0" => Some(Self::Keypad0),
            "Keypad1" => Some(Self::Keypad1),
            "Keypad2" => Some(Self::Keypad2),
            "Keypad3" => Some(Self::Keypad3),
            "Keypad4" => Some(Self::Keypad4),
            "Keypad5" => Some(Self::Keypad5),
            "Keypad6" => Some(Self::Keypad6),
            "Keypad7" => Some(Self::Keypad7),
            "Keypad8" => Some(Self::Keypad8),
            "Keypad9" => Some(Self::Keypad9),
            "KeypadMultiply" => Some(Self::KeypadMultiply),
            "KeypadAdd" => Some(Self::KeypadAdd),
            "KeypadSeparator" => Some(Self::KeypadSeparator),
            "KeypadSubtract" => Some(Self::KeypadSubtract),
            "KeypadDecimal" => Some(Self::KeypadDecimal),
            "KeypadDivide" => Some(Self::KeypadDivide),
            "F1" => Some(Self::F1),
            "F2" => Some(Self::F2),
            "F3" => Some(Self::F3),
            "F4" => Some(Self::F4),
            "F5" => Some(Self::F5),
            "F6" => Some(Self::F6),
            "F7" => Some(Self::F7),
            "F8" => Some(Self::F8),
            "F9" => Some(Self::F9),
            "F10" => Some(Self::F10),
            "F11" => Some(Self::F11),
            "F12" => Some(Self::F12),
            "F13" => Some(Self::F13),
            "F14" => Some(Self::F14),
            "F15" => Some(Self::F15),
            "F16" => Some(Self::F16),
            "F17" => Some(Self::F17),
            "F18" => Some(Self::F18),
            "F19" => Some(Self::F19),
            "F20" => Some(Self::F20),
            "F21" => Some(Self::F21),
            "F22" => Some(Self::F22),
            "F23" => Some(Self::F23),
            "F24" => Some(Self::F24),
            "NumLock" => Some(Self::NumLock),
            "ScrollLock" => Some(Self::ScrollLock),
            "LeftShift" => Some(Self::LeftShift),
            "RightShift" => Some(Self::RightShift),
            "LeftControl" => Some(Self::LeftControl),
            "RightControl" => Some(Self::RightControl),
            "LeftAlt" => Some(Self::LeftAlt),
            "RightAlt" => Some(Self::RightAlt),
            "BrowserBack" => Some(Self::BrowserBack),
            "BrowserForward" => Some(Self::BrowserForward),
            "BrowserRefresh" => Some(Self::BrowserRefresh),
            "BrowserStop" => Some(Self::BrowserStop),
            "BrowserSearch" => Some(Self::BrowserSearch),
            "BrowserFavourites" => Some(Self::BrowserFavourites),
            "BrowserHome" => Some(Self::BrowserHome),
            "MediaVolumeDown" => Some(Self::MediaVolumeDown),
            "MediaVolumeMute" => Some(Self::MediaVolumeMute),
            "MediaVolumeUp" => Some(Self::MediaVolumeUp),
            "MediaNextTrack" => Some(Self::MediaNextTrack),
            "MediaPreviousTrack" => Some(Self::MediaPreviousTrack),
            "MediaStop" => Some(Self::MediaStop),
            "MediaPlayPause" => Some(Self::MediaPlayPause),
            "LaunchMail" => Some(Self::LaunchMail),
            "LaunchMedia" => Some(Self::LaunchMedia),
            "LaunchApp1" => Some(Self::LaunchApp1),
            "LaunchApp2" => Some(Self::LaunchApp2),
            "Oem1" => Some(Self::Oem1),
            "OemPlus" => Some(Self::OemPlus),
            "OemComma" => Some(Self::OemComma),
            "OemMinus" => Some(Self::OemMinus),
            "OemPeriod" => Some(Self::OemPeriod),
            "Oem2" => Some(Self::Oem2),
            "Oem3" => Some(Self::Oem3),
            "GamepadA" => Some(Self::GamepadA),
            "GamepadB" => Some(Self::GamepadB),
            "GamepadX" => Some(Self::GamepadX),
            "GamepadY" => Some(Self::GamepadY),
            "GamepadR1" => Some(Self::GamepadR1),
            "GamepadL1" => Some(Self::GamepadL1),
            "GamepadL2" => Some(Self::GamepadL2),
            "GamepadR2" => Some(Self::GamepadR2),
            "GamepadDpadUp" => Some(Self::GamepadDpadUp),
            "GamepadDpadDown" => Some(Self::GamepadDpadDown),
            "GamepadDpadLeft" => Some(Self::GamepadDpadLeft),
            "GamepadDpadRight" => Some(Self::GamepadDpadRight),
            "GamepadMenu" => Some(Self::GamepadMenu),
            "GamepadView" => Some(Self::GamepadView),
            "GamepadL3" => Some(Self::GamepadL3),
            "GamepadR3" => Some(Self::GamepadR3),
            "GamepadLUp" => Some(Self::GamepadLUp),
            "GamepadLDown" => Some(Self::GamepadLDown),
            "GamepadLRight" => Some(Self::GamepadLRight),
            "GamepadLLeft" => Some(Self::GamepadLLeft),
            "GamepadRUp" => Some(Self::GamepadRUp),
            "GamepadRDown" => Some(Self::GamepadRDown),
            "GamepadRRight" => Some(Self::GamepadRRight),
            "GamepadRLeft" => Some(Self::GamepadRLeft),
            "Oem4" => Some(Self::Oem4),
            "Oem5" => Some(Self::Oem5),
            "Oem6" => Some(Self::Oem6),
            "Oem7" => Some(Self::Oem7),
            "Oem8" => Some(Self::Oem8),
            "OemAx" => Some(Self::OemAx),
            "Oem102" => Some(Self::Oem102),
            "IcoHelp" => Some(Self::IcoHelp),
            "Ico00" => Some(Self::Ico00),
            "ImeProcess" => Some(Self::ImeProcess),
            "IcoClear" => Some(Self::IcoClear),
            "OemReset" => Some(Self::OemReset),
            "OemJump" => Some(Self::OemJump),
            "OemPa1" => Some(Self::OemPa1),
            "OemPa2" => Some(Self::OemPa2),
            "OemPa3" => Some(Self::OemPa3),
            "OemWsCtrl" => Some(Self::OemWsCtrl),
            "OemCuSel" => Some(Self::OemCuSel),
            "OemAttn" => Some(Self::OemAttn),
            "OemFinish" => Some(Self::OemFinish),
            "OemCopy" => Some(Self::OemCopy),
            "OemAuto" => Some(Self::OemAuto),
            "OemEnlw" => Some(Self::OemEnlw),
            "OemBackTab" => Some(Self::OemBackTab),
            "Attn" => Some(Self::Attn),
            "CrSel" => Some(Self::CrSel),
            "ExSel" => Some(Self::ExSel),
            "EraseEof" => Some(Self::EraseEof),
            "MediaPlay" => Some(Self::MediaPlay),
            "Zoom" => Some(Self::Zoom),
            "Pa1" => Some(Self::Pa1),
            "OemClear" => Some(Self::OemClear),
            _ => None,            
        }
    }
}