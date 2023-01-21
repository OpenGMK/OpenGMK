use crate::{
    imgui, input,
    input::Button,
    types::Colour,
    game::recording::window::{Window, Openable, DisplayInformation},
};
use ramen::input::Key;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{Display, Formatter, Error},
    convert::{ From, TryInto },
    fs::File,
    path::PathBuf,
    default::Default,
    collections::BTreeMap,
};

#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Binding {
    Advance,
    Quicksave,
    Quickload,
    SelectNext,
    SelectPrevious,
    ToggleReadOnly,
    ToggleDirect,
    ToggleKeyboard,
    NextRand,
    ExportGmtas,
    ToggleMacros,
    SetMouse,
}
impl Display for Binding {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            Self::Advance => write!(f, "Advance Frame"),
            Self::Quicksave => write!(f, "Save Quicksave"),
            Self::Quickload => write!(f, "Load Quicksave"),
            Self::SelectNext => write!(f, "Select next savestate"),
            Self::SelectPrevious => write!(f, "Select previous savestate"),
            Self::ToggleReadOnly => write!(f, "Toggle Read-Only"),
            Self::ToggleDirect => write!(f, "Toggle direct/mouse input"),
            Self::ToggleKeyboard => write!(f, "Toggle full keyboard"),
            Self::NextRand => write!(f, "Cycle RNG"),
            Self::ExportGmtas => write!(f, "Export .gmtas"),
            Self::ToggleMacros => write!(f, "Toggle \"Run Macro\""),
            Self::SetMouse => write!(f, "Set Mouse"),
            //_ => write!(f, "{:?}", self),
        }
    }
}
impl Binding {
    fn default_binding(&self) -> Option<KeyCombination> {
        match self {
            Self::Advance => Some(KeyCombination::from(vec![Button::Space])),
            Self::Quickload => Some(KeyCombination::from(vec![Button::W])),
            Self::Quicksave => Some(KeyCombination::from(vec![Button::Q])),
            Self::SelectNext => Some(KeyCombination::from(vec![Button::Shift, Button::OemPlus])),
            Self::SelectPrevious => Some(KeyCombination::from(vec![Button::Shift, Button::OemMinus])),
            Self::ToggleReadOnly => Some(KeyCombination::from(vec![Button::Shift, Button::Alpha8])),
            Self::ToggleDirect => Some(KeyCombination::from(vec![Button::Control, Button::D])),
            Self::ToggleKeyboard => Some(KeyCombination::from(vec![Button::Control, Button::K])),
            Self::NextRand => Some(KeyCombination::from(vec![Button::Control, Button::R])),
            Self::ExportGmtas => Some(KeyCombination::from(vec![Button::Control, Button::Shift, Button::E])),
            Self::ToggleMacros => Some(KeyCombination::from(vec![Button::Control, Button::Alpha1])),
            Self::SetMouse => Some(KeyCombination::from(vec![Button::Control, Button::M])),
            //_ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyCombination {
    ctrl: bool,
    alt: bool,
    shift: bool,
    keycodes: Vec<Button>,
}
impl Display for KeyCombination {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        if self.ctrl {
            f.write_str("Ctrl+")?;
        }
        if self.alt {
            f.write_str("Alt+")?;
        }
        if self.shift {
            f.write_str("Shift+")?;
        }

        for (i, code) in self.keycodes.iter().enumerate() {
            write!(f, "{}", code)?;
            if i != self.keycodes.len()-1 {
                f.write_str("+")?;
            }
        }

        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
pub struct Keybindings {
    #[serde(skip)] disable_bindings: bool,
    #[serde(skip)] bindings_disabled: bool,

    bindings: BTreeMap<Binding, Option<KeyCombination>>,
}

pub struct KeybindWindow {
    current_binding: Option<Binding>,
    current_keys: KeyCombination,
    last_keycodes: Vec<u8>,
    is_open: bool,
}

impl Keybindings {
    fn set_default_bindings(bindings: &mut BTreeMap<Binding, Option<KeyCombination>>) {
        // todo: find a way to iterate enums and automate this.
        macro_rules! insert {
            ($binding:expr) => {
                if !bindings.contains_key(&$binding) { bindings.insert($binding, $binding.default_binding()); }
            };
        }
        
        insert!(Binding::Advance);
        insert!(Binding::Quickload);
        insert!(Binding::Quicksave);
        insert!(Binding::SelectNext);
        insert!(Binding::SelectPrevious);
        insert!(Binding::ToggleReadOnly);
        insert!(Binding::ToggleDirect);
        insert!(Binding::ToggleKeyboard);
        insert!(Binding::NextRand);
        insert!(Binding::ExportGmtas);
        insert!(Binding::ToggleMacros);
        insert!(Binding::SetMouse);
    }

    pub fn keybind_pressed(&self, bind: Binding, frame: &imgui::Frame) -> bool {
        if self.bindings_disabled {
            false
        } else {
            match self.bindings.get(&bind) {
                Some(Some(keys)) => keys.pressed(frame),
                _ => false,
            }
        }
    }

    pub fn update_binding(&mut self, bind: Binding, keys: Option<KeyCombination>) {
        self.bindings.insert(bind, keys);
    }

    /// Update the bindings disabled state to ensure that bindings will be disabled until the next io update.
    pub fn update_disable_bindings(&mut self) {
        if !self.disable_bindings {
            self.bindings_disabled = false;
        }

        self.disable_bindings = false;
    }

    pub fn disable_bindings(&mut self) {
        self.bindings_disabled = true;
        self.disable_bindings = true;
    }

    pub fn from_file_or_default(path: &PathBuf) -> Self {
        let default_keybindings = Self::default();

        let mut keybindings = if path.exists() {
            match bincode::deserialize_from(File::open(&path).expect("Couldn't read key bindings")) {
                Ok(keybindings) => keybindings,
                Err(_) => {
                    println!("Warning: Couldn't parse key bindings. Using default bindings.");
                    default_keybindings
                },
            }
        } else {
            bincode::serialize_into(File::create(&path).expect("Couldn't write key bindings"), &default_keybindings)
                .expect("Couldn't serialize key bindings");
            default_keybindings
        };

        Self::set_default_bindings(&mut keybindings.bindings);
        keybindings
    }
}

impl Default for Keybindings {
    fn default() -> Self {
        let mut bindings = BTreeMap::new();
        Self::set_default_bindings(&mut bindings);
        Self {
            disable_bindings: false,
            bindings_disabled: false,
            bindings,
        }
    }
}
impl Openable<Self> for KeybindWindow {
    fn window_name() -> &'static str {
        "Keybindings"
    }

    fn open(_id: usize) -> Self {
        Self::new()
    }
}
impl Window for KeybindWindow {
    fn stored_kind(&self) -> Option<super::WindowKind> {
        Some(super::WindowKind::Keybindings)
    }

    fn name(&self) -> String {
        "Keybindings".to_owned()
    }

    fn show_window(&mut self, info: &mut DisplayInformation) {
        unsafe { cimgui_sys::igPushStyleVarVec2(cimgui_sys::ImGuiStyleVar__ImGuiStyleVar_WindowPadding.try_into().unwrap(), imgui::Vec2(0.0, 4.0).into()); }
        info.frame.begin_window("Keybindings", None, true, false, Some(&mut self.is_open));

        if info.frame.begin_table(
            "Keybindings",
            4,
            (cimgui_sys::ImGuiTableFlags__ImGuiTableFlags_RowBg
                | cimgui_sys::ImGuiTableFlags__ImGuiTableFlags_Borders) as i32,
            imgui::Vec2(0.0, 0.0),
            0.0
        ) {
            info.frame.table_setup_column("Action", 0, 0.0);
            info.frame.table_setup_column("Keybind", 0, 0.0);
            info.frame.table_setup_column("Set", cimgui_sys::ImGuiTableColumnFlags__ImGuiTableColumnFlags_WidthFixed as i32, 60.0);
            info.frame.table_setup_column("Default", cimgui_sys::ImGuiTableColumnFlags__ImGuiTableColumnFlags_WidthFixed as i32, 60.0);

            for (binding, keys) in &mut info.keybindings.bindings {
                self.binding_entry(binding, keys, info.frame);
            }

            info.frame.end_table();
        }

        info.frame.end();
        unsafe { cimgui_sys::igPopStyleVar(1); }

        // disable all bindings while recording a new one.
        if matches!(self.current_binding, Some(_)) {
            info.keybindings.disable_bindings();
            self.update_current_keys(info.frame, info.keybindings);
        }
    }

    fn is_open(&self) -> bool {
        self.is_open
    }
}

impl KeybindWindow {
    pub fn new() -> Self {
        Self {
            last_keycodes: Vec::new(),
            current_binding: None,
            current_keys: KeyCombination {
                ctrl: false,
                alt: false,
                shift: false,
                keycodes: Vec::new(),
            },
            is_open: true,
        }
    }

    fn binding_entry(&mut self, binding: &Binding, keys: &mut Option<KeyCombination>, frame: &mut imgui::Frame) {
        frame.table_next_column();
        frame.text(&format!("{}", binding));

        frame.table_next_column();
        let is_setting_binding = matches!(self.current_binding, Some(_));
        let text = if is_setting_binding && *binding == self.current_binding.unwrap() {
            frame.coloured_text(&format!("{}", self.current_keys), Colour::new(0.5, 0.6, 0.7));
            "Cancel"
        } else {
            let name = if let Some(keycombination) = keys { format!("{}", keycombination) } else { String::from("Not set") };
            frame.text(&format!("{}", name));
            "Set"
        };

        frame.table_next_column();
        if frame.button(&format!("{}###{}", text, binding), imgui::Vec2(60.0, 20.0), None) {
            if !is_setting_binding {
                self.current_binding = Some(*binding);
                self.last_keycodes.clear();
                self.current_keys = KeyCombination::from(&self.last_keycodes);
            } else {
                self.current_binding = None;
            }
        }

        frame.table_next_column();
        if frame.button(&format!("Default###default{}", binding), imgui::Vec2(60.0, 20.0), None) {
            *keys = binding.default_binding();
            self.current_binding = None;
        }
    }

    fn update_current_keys(&mut self, frame: &mut imgui::Frame, bindings: &mut Keybindings) {
        if frame.key_pressed(input::ramen2vk(Key::Escape)) {
            bindings.update_binding(self.current_binding.unwrap(), None);
            self.current_binding = None;
        } else {
            let keys = frame.get_keys();
            if keys.len() == 0 {
                // if we have pressed keys before, update the keybinding and the key combination is valid
                if self.last_keycodes.len() != 0 && self.current_keys.is_valid() {
                    bindings.update_binding(self.current_binding.unwrap(), Some(self.current_keys.clone()));
                    self.current_binding = None;
                }
            } else {
                // figure out which keys have been newly pressed
                let new_keys: Vec<u8> = keys.iter().filter_map(|key| {
                    if !self.last_keycodes.contains(key) {
                        Some(*key)
                    } else {
                        None
                    }
                }).collect();
                if new_keys.len() > 0 {
                    // setup new keys, order them by when they were pressed, filter buttons that are no longer pressed
                    self.last_keycodes = self.last_keycodes.iter().chain(new_keys.iter()).filter_map(|key| {
                        if keys.contains(key) {
                            Some(*key)
                        } else {
                            None
                        }
                    }).collect();
                    self.current_keys = KeyCombination::from(&self.last_keycodes);
                }
            }
        }
    }
}

impl KeyCombination {
    pub fn pressed(&self, frame: &imgui::Frame) -> bool {
        let mut pressed = false;

        // only check if a button is pressed if that button is required. Ignore additionally pressed modifier keys.
        if !(self.ctrl && !frame.ctrl_down())
            && !(self.alt && !frame.alt_down())
            && !(self.shift && !frame.shift_down())
        {
            if self.keycodes.len() == 1 {
                pressed = frame.key_pressed(self.keycodes[0] as _);
            } else {
                for (i, key) in self.keycodes.iter().enumerate() {
                    if i == self.keycodes.len()-1 {
                        // check if the final key was just pressed
                        // todo: check last key instead of no repeat?
                        pressed = frame.key_pressed_norepeat(*key as _);
                    } else if !frame.key_down(*key as _) {
                        // and break if any of the other keys aren't.
                        break;
                    }
                }
            }
        }

        pressed
    }

    pub fn is_valid(&self) -> bool {
        self.keycodes.len() > 0
    }
}

impl From<Vec<Button>> for KeyCombination {
    fn from(keys: Vec<Button>) -> Self {
        let ctrl = keys.iter().any(|b| match b {Button::LeftControl | Button::RightControl | Button::Control => true, _ => false});
        let shift = keys.iter().any(|b| match b {Button::LeftShift | Button::RightShift | Button::Shift => true, _ => false});
        let alt = keys.iter().any(|b| match b {Button::LeftAlt | Button::RightAlt | Button::Alt => true, _ => false});

        Self {
            ctrl,
            shift,
            alt,
            keycodes: keys.iter().filter_map(|button| {
                match button {
                    Button::LeftControl | Button::RightControl | Button::Control
                     | Button::LeftShift | Button::RightShift | Button::Shift
                     | Button::LeftAlt | Button::RightAlt | Button::Alt
                    => None,
                    b => Some(*b),
                }
            }).collect(),
        }
    }
}

impl From<&Vec<u8>> for KeyCombination {
    fn from(keys: &Vec<u8>) -> Self {
        Self::from(keys.iter().filter_map(|key| Button::try_from_u8(*key)).collect::<Vec<Button>>())
    }
}

impl Button {
    pub fn try_from_u8(value: u8) -> Option<Button> {
        match value {
            0x01 => Some(Self::MouseLeft),
            0x02 => Some(Self::MouseRight),
            0x04 => Some(Self::MouseMiddle),
            0x05 => Some(Self::MouseX1),
            0x06 => Some(Self::MouseX2),
            0x08 => Some(Self::Backspace),
            0x09 => Some(Self::Tab),
            0x0C => Some(Self::Clear),
            0x0D => Some(Self::Return),
            0x10 => Some(Self::Shift),
            0x11 => Some(Self::Control),
            0x12 => Some(Self::Alt),
            0x13 => Some(Self::Pause),
            0x14 => Some(Self::CapsLock),
            0x15 => Some(Self::ImeKanaOrHangul),
            0x16 => Some(Self::ImeOn),
            0x17 => Some(Self::ImeJunja),
            0x18 => Some(Self::ImeFinal),
            0x19 => Some(Self::ImeHanjaOrKanji),
            0x1A => Some(Self::ImeOff),
            0x1B => Some(Self::Escape),
            0x1C => Some(Self::ImeConvert),
            0x1D => Some(Self::ImeNonConvert),
            0x1E => Some(Self::ImeAccept),
            0x1F => Some(Self::ImeModeChangeRequest),
            0x20 => Some(Self::Space),
            0x21 => Some(Self::PageUp),
            0x22 => Some(Self::PageDown),
            0x23 => Some(Self::End),
            0x24 => Some(Self::Home),
            0x25 => Some(Self::LeftArrow),
            0x26 => Some(Self::UpArrow),
            0x27 => Some(Self::RightArrow),
            0x28 => Some(Self::DownArrow),
            0x29 => Some(Self::Select),
            0x2A => Some(Self::Print),
            0x2B => Some(Self::Execute),
            0x2C => Some(Self::PrintScreen),
            0x2D => Some(Self::Insert),
            0x2E => Some(Self::Delete),
            0x2F => Some(Self::Help),
            0x30 => Some(Self::Alpha0),
            0x31 => Some(Self::Alpha1),
            0x32 => Some(Self::Alpha2),
            0x33 => Some(Self::Alpha3),
            0x34 => Some(Self::Alpha4),
            0x35 => Some(Self::Alpha5),
            0x36 => Some(Self::Alpha6),
            0x37 => Some(Self::Alpha7),
            0x38 => Some(Self::Alpha8),
            0x39 => Some(Self::Alpha9),
            0x41 => Some(Self::A),
            0x42 => Some(Self::B),
            0x43 => Some(Self::C),
            0x44 => Some(Self::D),
            0x45 => Some(Self::E),
            0x46 => Some(Self::F),
            0x47 => Some(Self::G),
            0x48 => Some(Self::H),
            0x49 => Some(Self::I),
            0x4A => Some(Self::J),
            0x4B => Some(Self::K),
            0x4C => Some(Self::L),
            0x4D => Some(Self::M),
            0x4E => Some(Self::N),
            0x4F => Some(Self::O),
            0x50 => Some(Self::P),
            0x51 => Some(Self::Q),
            0x52 => Some(Self::R),
            0x53 => Some(Self::S),
            0x54 => Some(Self::T),
            0x55 => Some(Self::U),
            0x56 => Some(Self::V),
            0x57 => Some(Self::W),
            0x58 => Some(Self::X),
            0x59 => Some(Self::Y),
            0x5A => Some(Self::Z),
            0x5B => Some(Self::LeftWindows),
            0x5C => Some(Self::RightWindows),
            0x5D => Some(Self::Applications),
            0x5F => Some(Self::Sleep),
            0x60 => Some(Self::Keypad0),
            0x61 => Some(Self::Keypad1),
            0x62 => Some(Self::Keypad2),
            0x63 => Some(Self::Keypad3),
            0x64 => Some(Self::Keypad4),
            0x65 => Some(Self::Keypad5),
            0x66 => Some(Self::Keypad6),
            0x67 => Some(Self::Keypad7),
            0x68 => Some(Self::Keypad8),
            0x69 => Some(Self::Keypad9),
            0x6A => Some(Self::KeypadMultiply),
            0x6B => Some(Self::KeypadAdd),
            0x6C => Some(Self::KeypadSeparator),
            0x6D => Some(Self::KeypadSubtract),
            0x6E => Some(Self::KeypadDecimal),
            0x6F => Some(Self::KeypadDivide),
            0x70 => Some(Self::F1),
            0x71 => Some(Self::F2),
            0x72 => Some(Self::F3),
            0x73 => Some(Self::F4),
            0x74 => Some(Self::F5),
            0x75 => Some(Self::F6),
            0x76 => Some(Self::F7),
            0x77 => Some(Self::F8),
            0x78 => Some(Self::F9),
            0x79 => Some(Self::F10),
            0x7A => Some(Self::F11),
            0x7B => Some(Self::F12),
            0x7C => Some(Self::F13),
            0x7D => Some(Self::F14),
            0x7E => Some(Self::F15),
            0x7F => Some(Self::F16),
            0x80 => Some(Self::F17),
            0x81 => Some(Self::F18),
            0x82 => Some(Self::F19),
            0x83 => Some(Self::F20),
            0x84 => Some(Self::F21),
            0x85 => Some(Self::F22),
            0x86 => Some(Self::F23),
            0x87 => Some(Self::F24),
            0x90 => Some(Self::NumLock),
            0x91 => Some(Self::ScrollLock),
            0xA0 => Some(Self::LeftShift),
            0xA1 => Some(Self::RightShift),
            0xA2 => Some(Self::LeftControl),
            0xA3 => Some(Self::RightControl),
            0xA4 => Some(Self::LeftAlt),
            0xA5 => Some(Self::RightAlt),
            0xA6 => Some(Self::BrowserBack),
            0xA7 => Some(Self::BrowserForward),
            0xA8 => Some(Self::BrowserRefresh),
            0xA9 => Some(Self::BrowserStop),
            0xAA => Some(Self::BrowserSearch),
            0xAB => Some(Self::BrowserFavourites),
            0xAC => Some(Self::BrowserHome),
            0xAE => Some(Self::MediaVolumeDown),
            0xAD => Some(Self::MediaVolumeMute),
            0xAF => Some(Self::MediaVolumeUp),
            0xB0 => Some(Self::MediaNextTrack),
            0xB1 => Some(Self::MediaPreviousTrack),
            0xB2 => Some(Self::MediaStop),
            0xB3 => Some(Self::MediaPlayPause),
            0xB4 => Some(Self::LaunchMail),
            0xB5 => Some(Self::LaunchMedia),
            0xB6 => Some(Self::LaunchApp1),
            0xB7 => Some(Self::LaunchApp2),
            0xBA => Some(Self::Oem1),
            0xBB => Some(Self::OemPlus),
            0xBC => Some(Self::OemComma),
            0xBD => Some(Self::OemMinus),
            0xBE => Some(Self::OemPeriod),
            0xBF => Some(Self::Oem2),
            0xC0 => Some(Self::Oem3),
            0xC3 => Some(Self::GamepadA),
            0xC4 => Some(Self::GamepadB),
            0xC5 => Some(Self::GamepadX),
            0xC6 => Some(Self::GamepadY),
            0xC7 => Some(Self::GamepadR1),
            0xC8 => Some(Self::GamepadL1),
            0xC9 => Some(Self::GamepadL2),
            0xCA => Some(Self::GamepadR2),
            0xCB => Some(Self::GamepadDpadUp),
            0xCC => Some(Self::GamepadDpadDown),
            0xCD => Some(Self::GamepadDpadLeft),
            0xCE => Some(Self::GamepadDpadRight),
            0xCF => Some(Self::GamepadMenu),
            0xD0 => Some(Self::GamepadView),
            0xD1 => Some(Self::GamepadL3),
            0xD2 => Some(Self::GamepadR3),
            0xD3 => Some(Self::GamepadLUp),
            0xD4 => Some(Self::GamepadLDown),
            0xD5 => Some(Self::GamepadLRight),
            0xD6 => Some(Self::GamepadLLeft),
            0xD7 => Some(Self::GamepadRUp),
            0xD8 => Some(Self::GamepadRDown),
            0xD9 => Some(Self::GamepadRRight),
            0xDA => Some(Self::GamepadRLeft),
            0xDB => Some(Self::Oem4),
            0xDC => Some(Self::Oem5),
            0xDD => Some(Self::Oem6),
            0xDE => Some(Self::Oem7),
            0xDF => Some(Self::Oem8),
            0xE1 => Some(Self::OemAx),
            0xE2 => Some(Self::Oem102),
            0xE3 => Some(Self::IcoHelp),
            0xE4 => Some(Self::Ico00),
            0xE5 => Some(Self::ImeProcess),
            0xE6 => Some(Self::IcoClear),
            0xE9 => Some(Self::OemReset),
            0xEA => Some(Self::OemJump),
            0xEB => Some(Self::OemPa1),
            0xEC => Some(Self::OemPa2),
            0xED => Some(Self::OemPa3),
            0xEE => Some(Self::OemWsCtrl),
            0xEF => Some(Self::OemCuSel),
            0xF0 => Some(Self::OemAttn),
            0xF1 => Some(Self::OemFinish),
            0xF2 => Some(Self::OemCopy),
            0xF3 => Some(Self::OemAuto),
            0xF4 => Some(Self::OemEnlw),
            0xF5 => Some(Self::OemBackTab),
            0xF6 => Some(Self::Attn),
            0xF7 => Some(Self::CrSel),
            0xF8 => Some(Self::ExSel),
            0xF9 => Some(Self::EraseEof),
            0xFA => Some(Self::MediaPlay),
            0xFB => Some(Self::Zoom),
            0xFD => Some(Self::Pa1),
            0xFE => Some(Self::OemClear),
            _ => None,            
        }
    }
}
