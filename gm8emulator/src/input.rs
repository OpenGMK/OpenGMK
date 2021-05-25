use std::convert::TryFrom;

const KEY_MAX: usize = u8::max_value() as usize;
const MB_ANY: i8 = -1;
const MB_NONE: i8 = 0;
const VK_NOKEY: u8 = 0; // TODO: dont redefine
const VK_ANYKEY: u8 = 1; // TODO: dont redefine

#[repr(u8)]
pub enum Button {
    // 0x00: Unmapped
    MouseLeft = 0x01,
    MouseRight = 0x02,
    // 0x03: Ctrl+Break? (TODO)
    MouseMiddle = 0x04,
    MouseX1 = 0x05,
    MouseX2 = 0x06,
    // 0x07: Unmapped
    Backspace = 0x08,
    Tab = 0x09,
    // 0x0A-B: Reserved
    Clear = 0x0C,
    Return = 0x0D,
    // 0x0E-F: Unmapped
    Shift = 0x10,
    Control = 0x11,
    Alt = 0x12,
    Pause = 0x13,
    CapsLock = 0x14,
    ImeKanaOrHangul = 0x15,
    ImeOn = 0x16,
    ImeJunja = 0x17,
    ImeFinal = 0x18,
    ImeHanjaOrKanji = 0x19,
    ImeOff = 0x1A,
    Escape = 0x1B,
    ImeConvert = 0x1C,
    ImeNonConvert = 0x1D,
    ImeAccept = 0x1E,
    ImeModeChangeRequest = 0x1F,
    Space = 0x20,
    PageUp = 0x21,
    PageDown = 0x22,
    End = 0x23,
    Home = 0x24,
    LeftArrow = 0x25,
    UpArrow = 0x26,
    RightArrow = 0x27,
    DownArrow = 0x28,
    Select = 0x29,
    Print = 0x2A,
    Execute = 0x2B,
    PrintScreen = 0x2C,
    Insert = 0x2D,
    Delete = 0x2E,
    Help = 0x2F,
    Alpha0 = 0x30,
    Alpha1 = 0x31,
    Alpha2 = 0x32,
    Alpha3 = 0x33,
    Alpha4 = 0x34,
    Alpha5 = 0x35,
    Alpha6 = 0x36,
    Alpha7 = 0x37,
    Alpha8 = 0x38,
    Alpha9 = 0x39,
    // 0x3A-40: Unmapped
    A = 0x41,
    B = 0x42,
    C = 0x43,
    D = 0x44,
    E = 0x45,
    F = 0x46,
    G = 0x47,
    H = 0x48,
    I = 0x49,
    J = 0x4A,
    K = 0x4B,
    L = 0x4C,
    M = 0x4D,
    N = 0x4E,
    O = 0x4F,
    P = 0x50,
    Q = 0x51,
    R = 0x52,
    S = 0x53,
    T = 0x54,
    U = 0x55,
    V = 0x56,
    W = 0x57,
    X = 0x58,
    Y = 0x59,
    Z = 0x5A,
    LeftWindows = 0x5B,
    RightWindows = 0x5C,
    Applications = 0x5D,
    // 0x5E: Reserved
    Sleep = 0x5F,
    Keypad0 = 0x60,
    Keypad1 = 0x61,
    Keypad2 = 0x62,
    Keypad3 = 0x63,
    Keypad4 = 0x64,
    Keypad5 = 0x65,
    Keypad6 = 0x66,
    Keypad7 = 0x67,
    Keypad8 = 0x68,
    Keypad9 = 0x69,
    KeypadMultiply = 0x6A,
    KeypadAdd = 0x6B,
    KeypadSeparator = 0x6C,
    KeypadSubtract = 0x6D,
    KeypadDecimal = 0x6E,
    KeypadDivide = 0x6F,
    F1 = 0x70,
    F2 = 0x71,
    F3 = 0x72,
    F4 = 0x73,
    F5 = 0x74,
    F6 = 0x75,
    F7 = 0x76,
    F8 = 0x77,
    F9 = 0x78,
    F10 = 0x79,
    F11 = 0x7A,
    F12 = 0x7B,
    F13 = 0x7C,
    F14 = 0x7D,
    F15 = 0x7E,
    F16 = 0x7F,
    F17 = 0x80,
    F18 = 0x81,
    F19 = 0x82,
    F20 = 0x83,
    F21 = 0x84,
    F22 = 0x85,
    F23 = 0x86,
    F24 = 0x87,
    // 0x88-0x8F: Unmapped
    NumLock = 0x90,
    ScrollLock = 0x91,
    // 0x92-96: OEM Specific (TODO)
    // 0x97-9F: Unmapped
    LeftShift = 0xA0,
    RightShift = 0xA1,
    LeftControl = 0xA2,
    RightControl = 0xA3,
    LeftAlt = 0xA4,
    RightAlt = 0xA5,
    BrowserBack = 0xA6,
    BrowserForward = 0xA7,
    BrowserRefresh = 0xA8,
    BrowserStop = 0xA9,
    BrowserSearch = 0xAA,
    BrowserFavourites = 0xAB,
    BrowserHome = 0xAC,
    MediaVolumeDown = 0xAE,
    MediaVolumeMute = 0xAD,
    MediaVolumeUp = 0xAF,
    MediaNextTrack = 0xB0,
    MediaPreviousTrack = 0xB1,
    MediaStop = 0xB2,
    MediaPlayPause = 0xB3,
    LaunchMail = 0xB4,
    LaunchMedia = 0xB5,
    LaunchApp1 = 0xB6,
    LaunchApp2 = 0xB7,
    // 0xB8-B9: Reserved
    Oem1 = 0xBA,
    OemPlus = 0xBB,
    OemComma = 0xBC,
    OemMinus = 0xBD,
    OemPeriod = 0xBE,
    Oem2 = 0xBF,
    Oem3 = 0xC0,
    // 0xC1-C2: Reserved
    GamepadA = 0xC3,
    GamepadB = 0xC4,
    GamepadX = 0xC5,
    GamepadY = 0xC6,
    GamepadR1 = 0xC7,
    GamepadL1 = 0xC8,
    GamepadL2 = 0xC9,
    GamepadR2 = 0xCA,
    GamepadDpadUp = 0xCB,
    GamepadDpadDown = 0xCC,
    GamepadDpadLeft = 0xCD,
    GamepadDpadRight = 0xCE,
    GamepadMenu = 0xCF,
    GamepadView = 0xD0,
    GamepadL3 = 0xD1,
    GamepadR3 = 0xD2,
    GamepadLUp = 0xD3,
    GamepadLDown = 0xD4,
    GamepadLRight = 0xD5,
    GamepadLLeft = 0xD6,
    GamepadRUp = 0xD7,
    GamepadRDown = 0xD8,
    GamepadRRight = 0xD9,
    GamepadRLeft = 0xDA,
    Oem4 = 0xDB,
    Oem5 = 0xDC,
    Oem6 = 0xDD,
    Oem7 = 0xDE,
    Oem8 = 0xDF,
    // 0xE0: Reserved
    OemAx = 0xE1,
    Oem102 = 0xE2,
    IcoHelp = 0xE3,
    Ico00 = 0xE4,
    ImeProcess = 0xE5,
    IcoClear = 0xE6,
    // 0xE7: Packet (can't map to GM)
    // 0xE8: Unmapped
    OemReset = 0xE9,
    OemJump = 0xEA,
    OemPa1 = 0xEB,
    OemPa2 = 0xEC,
    OemPa3 = 0xED,
    OemWsCtrl = 0xEE,
    OemCuSel = 0xEF,
    OemAttn = 0xF0,
    OemFinish = 0xF1,
    OemCopy = 0xF2,
    OemAuto = 0xF3,
    OemEnlw = 0xF4,
    OemBackTab = 0xF5,
    Attn = 0xF6,
    CrSel = 0xF7,
    ExSel = 0xF8,
    EraseEof = 0xF9,
    MediaPlay = 0xFA,
    Zoom = 0xFB,
    // 0xFC: Reserved
    Pa1 = 0xFD,
    OemClear = 0xFE,
    // 0xFF: Unmapped
}

impl TryFrom<ramen::event::Key> for Button {
    type Error = ();
    fn try_from(key: ramen::event::Key) -> Result<Self, Self::Error> {
        match key {
            ramen::event::Key::Attn => Ok(Self::Attn),
            ramen::event::Key::LAlt => Ok(Self::LeftAlt),
            ramen::event::Key::RAlt => Ok(Self::RightAlt),
            ramen::event::Key::Applications => Ok(Self::Applications),
            ramen::event::Key::Backspace => Ok(Self::Backspace),
            ramen::event::Key::CapsLock => Ok(Self::CapsLock),
            ramen::event::Key::Clear => Ok(Self::Clear),
            ramen::event::Key::LControl => Ok(Self::LeftControl),
            ramen::event::Key::RControl => Ok(Self::RightControl),
            ramen::event::Key::CrSel => Ok(Self::CrSel),
            ramen::event::Key::Delete => Ok(Self::Delete),
            ramen::event::Key::End => Ok(Self::End),
            ramen::event::Key::Enter => Ok(Self::Return),
            ramen::event::Key::EraseEof => Ok(Self::EraseEof),
            ramen::event::Key::Escape => Ok(Self::Escape),
            ramen::event::Key::Execute => Ok(Self::Execute),
            ramen::event::Key::ExSel => Ok(Self::ExSel),
            ramen::event::Key::Help => Ok(Self::Help),
            ramen::event::Key::Home => Ok(Self::Home),
            ramen::event::Key::Insert => Ok(Self::Insert),
            ramen::event::Key::NumLock => Ok(Self::NumLock),
            ramen::event::Key::Pa1 => Ok(Self::Pa1),
            ramen::event::Key::PageUp => Ok(Self::PageUp),
            ramen::event::Key::PageDown => Ok(Self::PageDown),
            ramen::event::Key::Pause => Ok(Self::Pause),
            ramen::event::Key::Play => Ok(Self::MediaPlay),
            ramen::event::Key::Print => Ok(Self::Print),
            ramen::event::Key::PrintScreen => Ok(Self::PrintScreen),
            ramen::event::Key::LShift => Ok(Self::LeftShift),
            ramen::event::Key::RShift => Ok(Self::RightShift),
            ramen::event::Key::ScrollLock => Ok(Self::ScrollLock),
            ramen::event::Key::Select => Ok(Self::Select),
            ramen::event::Key::Sleep => Ok(Self::Sleep),
            ramen::event::Key::Space => Ok(Self::Space),
            ramen::event::Key::LSuper => Ok(Self::LeftWindows),
            ramen::event::Key::RSuper => Ok(Self::RightWindows),
            ramen::event::Key::Tab => Ok(Self::Tab),
            ramen::event::Key::Zoom => Ok(Self::Zoom),
            ramen::event::Key::Left => Ok(Self::LeftArrow),
            ramen::event::Key::Up => Ok(Self::UpArrow),
            ramen::event::Key::Right => Ok(Self::RightArrow),
            ramen::event::Key::Down => Ok(Self::DownArrow),
            ramen::event::Key::Num0 => Ok(Self::Alpha0),
            ramen::event::Key::Num1 => Ok(Self::Alpha1),
            ramen::event::Key::Num2 => Ok(Self::Alpha2),
            ramen::event::Key::Num3 => Ok(Self::Alpha3),
            ramen::event::Key::Num4 => Ok(Self::Alpha4),
            ramen::event::Key::Num5 => Ok(Self::Alpha5),
            ramen::event::Key::Num6 => Ok(Self::Alpha6),
            ramen::event::Key::Num7 => Ok(Self::Alpha7),
            ramen::event::Key::Num8 => Ok(Self::Alpha8),
            ramen::event::Key::Num9 => Ok(Self::Alpha9),
            ramen::event::Key::A => Ok(Self::A),
            ramen::event::Key::B => Ok(Self::B),
            ramen::event::Key::C => Ok(Self::C),
            ramen::event::Key::D => Ok(Self::D),
            ramen::event::Key::E => Ok(Self::E),
            ramen::event::Key::F => Ok(Self::F),
            ramen::event::Key::G => Ok(Self::G),
            ramen::event::Key::H => Ok(Self::H),
            ramen::event::Key::I => Ok(Self::I),
            ramen::event::Key::J => Ok(Self::J),
            ramen::event::Key::K => Ok(Self::K),
            ramen::event::Key::L => Ok(Self::L),
            ramen::event::Key::M => Ok(Self::M),
            ramen::event::Key::N => Ok(Self::N),
            ramen::event::Key::O => Ok(Self::O),
            ramen::event::Key::P => Ok(Self::P),
            ramen::event::Key::Q => Ok(Self::Q),
            ramen::event::Key::R => Ok(Self::R),
            ramen::event::Key::S => Ok(Self::S),
            ramen::event::Key::T => Ok(Self::T),
            ramen::event::Key::U => Ok(Self::U),
            ramen::event::Key::V => Ok(Self::V),
            ramen::event::Key::W => Ok(Self::W),
            ramen::event::Key::X => Ok(Self::X),
            ramen::event::Key::Y => Ok(Self::Y),
            ramen::event::Key::Z => Ok(Self::Z),
            ramen::event::Key::Comma => Ok(Self::OemComma),
            ramen::event::Key::Minus => Ok(Self::OemMinus),
            ramen::event::Key::Period => Ok(Self::OemPeriod),
            ramen::event::Key::Plus => Ok(Self::OemPlus),
            ramen::event::Key::Oem1 => Ok(Self::Oem1),
            ramen::event::Key::Oem2 => Ok(Self::Oem2),
            ramen::event::Key::Oem3 => Ok(Self::Oem3),
            ramen::event::Key::Oem4 => Ok(Self::Oem4),
            ramen::event::Key::Oem5 => Ok(Self::Oem5),
            ramen::event::Key::Oem6 => Ok(Self::Oem6),
            ramen::event::Key::Oem7 => Ok(Self::Oem7),
            ramen::event::Key::Oem8 => Ok(Self::Oem8),
            ramen::event::Key::Oem102 => Ok(Self::Oem102),
            ramen::event::Key::OemClear => Ok(Self::OemClear),
            ramen::event::Key::Add => Ok(Self::KeypadAdd),
            ramen::event::Key::Subtract => Ok(Self::KeypadSubtract),
            ramen::event::Key::Multiply => Ok(Self::KeypadMultiply),
            ramen::event::Key::Divide => Ok(Self::KeypadDivide),
            ramen::event::Key::Decimal => Ok(Self::KeypadDecimal),
            ramen::event::Key::Separator => Ok(Self::KeypadSeparator),
            ramen::event::Key::Numpad0 => Ok(Self::Keypad0),
            ramen::event::Key::Numpad1 => Ok(Self::Keypad1),
            ramen::event::Key::Numpad2 => Ok(Self::Keypad2),
            ramen::event::Key::Numpad3 => Ok(Self::Keypad3),
            ramen::event::Key::Numpad4 => Ok(Self::Keypad4),
            ramen::event::Key::Numpad5 => Ok(Self::Keypad5),
            ramen::event::Key::Numpad6 => Ok(Self::Keypad6),
            ramen::event::Key::Numpad7 => Ok(Self::Keypad7),
            ramen::event::Key::Numpad8 => Ok(Self::Keypad8),
            ramen::event::Key::Numpad9 => Ok(Self::Keypad9),
            ramen::event::Key::F1 => Ok(Self::F1),
            ramen::event::Key::F2 => Ok(Self::F2),
            ramen::event::Key::F3 => Ok(Self::F3),
            ramen::event::Key::F4 => Ok(Self::F4),
            ramen::event::Key::F5 => Ok(Self::F5),
            ramen::event::Key::F6 => Ok(Self::F6),
            ramen::event::Key::F7 => Ok(Self::F7),
            ramen::event::Key::F8 => Ok(Self::F8),
            ramen::event::Key::F9 => Ok(Self::F9),
            ramen::event::Key::F10 => Ok(Self::F10),
            ramen::event::Key::F11 => Ok(Self::F11),
            ramen::event::Key::F12 => Ok(Self::F12),
            ramen::event::Key::F13 => Ok(Self::F13),
            ramen::event::Key::F14 => Ok(Self::F14),
            ramen::event::Key::F15 => Ok(Self::F15),
            ramen::event::Key::F16 => Ok(Self::F16),
            ramen::event::Key::F17 => Ok(Self::F17),
            ramen::event::Key::F18 => Ok(Self::F18),
            ramen::event::Key::F19 => Ok(Self::F19),
            ramen::event::Key::F20 => Ok(Self::F20),
            ramen::event::Key::F21 => Ok(Self::F21),
            ramen::event::Key::F22 => Ok(Self::F22),
            ramen::event::Key::F23 => Ok(Self::F23),
            ramen::event::Key::F24 => Ok(Self::F24),
            ramen::event::Key::BrowserBack => Ok(Self::BrowserBack),
            ramen::event::Key::BrowserFavourites => Ok(Self::BrowserFavourites),
            ramen::event::Key::BrowserForward => Ok(Self::BrowserForward),
            ramen::event::Key::BrowserHome => Ok(Self::BrowserHome),
            ramen::event::Key::BrowserRefresh => Ok(Self::BrowserRefresh),
            ramen::event::Key::BrowserSearch => Ok(Self::BrowserSearch),
            ramen::event::Key::BrowserStop => Ok(Self::BrowserStop),
            ramen::event::Key::ImeAccept => Ok(Self::ImeAccept),
            ramen::event::Key::ImeConvert => Ok(Self::ImeConvert),
            ramen::event::Key::ImeNonConvert => Ok(Self::ImeNonConvert),
            ramen::event::Key::ImeFinal => Ok(Self::ImeFinal),
            ramen::event::Key::ImeModeChange => Ok(Self::ImeModeChangeRequest),
            ramen::event::Key::ImeProcess => Ok(Self::ImeProcess),
            ramen::event::Key::ImeOn => Ok(Self::ImeOn),
            ramen::event::Key::ImeOff => Ok(Self::ImeOff),
            ramen::event::Key::ImeKana => Ok(Self::ImeKanaOrHangul),
            ramen::event::Key::ImeKanji => Ok(Self::ImeHanjaOrKanji),
            ramen::event::Key::ImeJunja => Ok(Self::ImeJunja),
            ramen::event::Key::MediaNextTrack => Ok(Self::MediaNextTrack),
            ramen::event::Key::MediaPreviousTrack => Ok(Self::MediaPreviousTrack),
            ramen::event::Key::MediaPlayPause => Ok(Self::MediaPlayPause),
            ramen::event::Key::MediaStop => Ok(Self::MediaStop),
            ramen::event::Key::VolumeDown => Ok(Self::MediaVolumeDown),
            ramen::event::Key::VolumeUp => Ok(Self::MediaVolumeUp),
            ramen::event::Key::VolumeMute => Ok(Self::MediaVolumeMute),
            ramen::event::Key::LaunchApplication1 => Ok(Self::LaunchApp1),
            ramen::event::Key::LaunchApplication2 => Ok(Self::LaunchApp2),
            ramen::event::Key::LaunchMail => Ok(Self::LaunchMail),
            ramen::event::Key::LaunchMediaSelect => Ok(Self::LaunchMedia),
        }
    }
}

pub fn ramen2vk(x: ramen::event::Key) -> u8 {
    Button::try_from(x).map(|e| e as u8).unwrap_or(0)
}

const fn make_is_direct_only() -> [bool; KEY_MAX] {
    let mut table = [false; KEY_MAX];
    let mut i = 0;
    while i < KEY_MAX {
        if
            i == Button::MouseLeft as usize ||
            i == Button::MouseRight as usize ||
            i == Button::MouseMiddle as usize ||
            i == Button::MouseX1 as usize ||
            i == Button::MouseX2 as usize ||
            i == Button::LeftShift as usize ||
            i == Button::RightShift as usize ||
            i == Button::LeftControl as usize ||
            i == Button::RightControl as usize ||
            i == Button::LeftAlt as usize ||
            i == Button::RightAlt as usize
            // TODO: Maybe the gamepad ones too? Ask Adam/Floogle/renex about this one...
        {
            table[i] = true;
        }
        i += 1;
    }
    table
}
const IS_DIRECT_ONLY: [bool; KEY_MAX] = make_is_direct_only();

const fn make_vk_fn_input_remap() -> [u8; KEY_MAX] {
    let mut table = [0u8; KEY_MAX];
    let mut i = 0;
    while i < KEY_MAX {
        table[i] = if i == Button::Shift as usize {
            Button::LeftShift as u8
        } else if i == Button::Control as usize {
            Button::LeftControl as u8
        } else if i == Button::Alt as usize {
            Button::LeftAlt as u8
        } else {
            i as u8
        };
        i += 1;
    }
    table
}
const VK_FN_INPUT_REMAP: [u8; KEY_MAX] = make_vk_fn_input_remap();

#[repr(i8)]
pub enum MouseButton {
    // gamemaker
    Left = 1,
    Right = 2,
    Middle = 3,

    // non-gamemaker
    X1 = 4,
    X2 = 5,
}

impl TryFrom<ramen::event::MouseButton> for MouseButton {
    type Error = ();
    fn try_from(key: ramen::event::MouseButton) -> Result<Self, Self::Error> {
        match key {
            ramen::event::MouseButton::Left => Ok(Self::Left),
            ramen::event::MouseButton::Right => Ok(Self::Right),
            ramen::event::MouseButton::Middle => Ok(Self::Middle),
            ramen::event::MouseButton::Mouse4 => Ok(Self::X1),
            ramen::event::MouseButton::Mouse5 => Ok(Self::X2),
        }
    }
}

pub fn ramen2mb(x: ramen::event::MouseButton) -> i8 {
    MouseButton::try_from(x).map(|e| e as i8).unwrap_or(0)
}

fn mouse2button(code: i8) -> Option<Button> {
    match code {
        x if x == MouseButton::Left as i8 => Some(Button::MouseLeft),
        x if x == MouseButton::Right as i8 => Some(Button::MouseRight),
        x if x == MouseButton::Middle as i8 => Some(Button::MouseMiddle),
        x if x == MouseButton::X1 as i8 => Some(Button::MouseX1),
        x if x == MouseButton::X2 as i8 => Some(Button::MouseX2),
        _ => None,
    }
}

const fn gen_default_keymap() -> [u8; KEY_MAX] {
    let mut map = [0u8; KEY_MAX];
    let mut i = 0;
    while i < KEY_MAX {
        map[i] = i as u8;
    }
    map
}
const DEFAULT_KEYMAP: [u8; KEY_MAX] = gen_default_keymap();

pub struct Input {
    // basic state
    button_remap: [u8; KEY_MAX],
    button_state: [bool; KEY_MAX],
    button_state_press: [bool; KEY_MAX],
    button_state_release: [bool; KEY_MAX],
    mouse_position: (i32, i32),
    mouse_wheel: (bool, bool),

    // gamemaker weirdness
    key_current: u8,
    key_previous: u8,
    mouse_current: i8,
    mouse_previous: i8,
    mouse_position_previous: (i32, i32),
}

impl Input {
    pub const fn new() -> Self {
        Input {
            button_remap: DEFAULT_KEYMAP,
            button_state: [false; KEY_MAX],
            button_state_press: [false; KEY_MAX],
            button_state_release: [false; KEY_MAX],
            mouse_position: (0, 0),
            mouse_wheel: (false, false),
            key_current: 0,
            key_previous: 0,
            mouse_current: 0,
            mouse_previous: 0,
            mouse_position_previous: (0, 0),
        }
    }

    pub fn button_press(&mut self, code: u8, store_cur_prev: bool) {
        let code = VK_FN_INPUT_REMAP[code as usize];
        self.button_state[code as usize] = true;
        self.button_state_press[code as usize] = true;
        if store_cur_prev {
            self.key_current = code;
            self.key_previous = code;
        }
    }

    pub fn button_release(&mut self, code: u8, store_cur_prev: bool) {
        let code = VK_FN_INPUT_REMAP[code as usize];
        self.button_state[code as usize] = false;
        self.button_state_release[code as usize] = true;
        if store_cur_prev && self.key_current == code {
            self.key_current = 0;
        }
    }

    pub fn mouse_press(&mut self, code: i8, store_cur_prev: bool) {
        let button = match mouse2button(code) {
            Some(button) => button,
            None => return,
        };
        if store_cur_prev {
            self.mouse_current = code;
            self.mouse_previous = code;
        }
        self.button_press(button as u8, false);
    }

    pub fn mouse_release(&mut self, code: i8, store_cur_prev: bool) {
        let button = match mouse2button(code) {
            Some(button) => button,
            None => return,
        };
        if store_cur_prev {
            self.mouse_current = 0;
        }
        self.button_press(button as u8, false);
    }

    pub fn mouse_scroll(&mut self, delta: i32) {
        if delta > 0 {
            self.mouse_wheel.0 = true;
        } else if delta < 0 {
            self.mouse_wheel.1 = true;
        }
    }

    // == GameMaker Mappings ==

    fn keyboard_check_any_internal_indirect(&self, state: &[bool; KEY_MAX]) -> bool {
        state
            .iter()
            .enumerate()
            .filter(|(i, _)| !IS_DIRECT_ONLY[*i])
            .any(|(_, x)| *x)
    }

    fn keyboard_check_internal(&self, state: &[bool; KEY_MAX], vk: u8) -> bool {
        if vk == Button::Shift as u8 {
            state[Button::LeftShift as usize] || state[Button::RightShift as usize]
        } else if vk == Button::Control as u8 {
            state[Button::LeftControl as usize] || state[Button::RightControl as usize]
        } else if vk == Button::Alt as u8 {
            state[Button::LeftAlt as usize] || state[Button::RightAlt as usize]
        } else {
            state[vk as usize]
        }
    }

    fn keyboard_check_internal_indirect(&self, state: &[bool; KEY_MAX], vk: u8) -> bool {
        match vk {
            VK_NOKEY => !self.keyboard_check_any_internal_indirect(state),
            VK_ANYKEY => self.keyboard_check_any_internal_indirect(state),
            _ => !IS_DIRECT_ONLY[vk as usize] && self.keyboard_check_internal(state, vk),
        }
    }

    #[inline]
    pub fn keyboard_check(&self, vk: u8) -> bool {
        self.keyboard_check_internal_indirect(&self.button_state, vk)
    }

    #[inline]
    pub fn keyboard_check_pressed(&self, vk: u8) -> bool {
        self.keyboard_check_internal_indirect(&self.button_state_press, vk)
    }

    #[inline]
    pub fn keyboard_check_released(&self, vk: u8) -> bool {
        self.keyboard_check_internal_indirect(&self.button_state_release, vk)
    }

    #[inline]
    pub fn keyboard_check_direct(&self, vk: u8) -> bool {
        self.keyboard_check_internal(&self.button_state, vk)
    }

    #[inline]
    pub fn keyboard_get_map(&mut self, vk: u8) -> u8 {
        self.button_remap[vk as usize]
    }

    #[inline]
    pub fn keyboard_set_map(&mut self, vk_from: u8, vk_to: u8) {
        self.button_remap[vk_from as usize] = vk_to;
    }

    pub fn keyboard_unset_map(&mut self) {
        self.button_remap = DEFAULT_KEYMAP;
    }

    #[inline]
    pub fn keyboard_key(&self) -> u8 {
        self.key_current
    }

    #[inline]
    pub fn keyboard_lastkey(&self) -> u8 {
        self.key_previous
    }

    #[inline]
    pub fn set_keyboard_key(&mut self, vk: u8) {
        self.key_current = vk;
    }

    #[inline]
    pub fn set_keyboard_lastkey(&mut self, vk: u8) {
        self.key_previous = vk;
    }

    fn mouse_check_button_internal_indirect(&self, state: &[bool; KEY_MAX], mb: i8) -> bool {
        match mb {
            MB_ANY =>
                state[Button::MouseLeft as usize] ||
                state[Button::MouseRight as usize] ||
                state[Button::MouseMiddle as usize],
            MB_NONE => 
                !state[Button::MouseLeft as usize] &&
                !state[Button::MouseRight as usize] &&
                !state[Button::MouseMiddle as usize],
            x if x == MouseButton::Left as i8 => state[Button::MouseLeft as usize],
            x if x == MouseButton::Right as i8 => state[Button::MouseRight as usize],
            x if x == MouseButton::Middle as i8 => state[Button::MouseMiddle as usize],
            _ => false,
        }
    }

    #[inline]
    pub fn mouse_button(&self) -> i8 {
        self.mouse_current
    }

    #[inline]
    pub fn mouse_lastbutton(&self) -> i8 {
        self.mouse_previous
    }

    #[inline]
    pub fn set_mouse_button(&mut self, mb: i8) {
        self.mouse_current = mb;
    }

    #[inline]
    pub fn set_mouse_lastbutton(&mut self, mb: i8) {
        self.mouse_previous = mb;
    }

    #[inline]
    pub fn mouse_check_button(&self, mb: i8) -> bool {
        self.mouse_check_button_internal_indirect(&self.button_state, mb)
    }

    #[inline]
    pub fn mouse_check_button_pressed(&self, mb: i8) -> bool {
        self.mouse_check_button_internal_indirect(&self.button_state_press, mb)
    }

    #[inline]
    pub fn mouse_check_button_released(&self, mb: i8) -> bool {
        self.mouse_check_button_internal_indirect(&self.button_state_release, mb)
    }

    #[inline]
    pub fn mouse_wheel_up(&self) -> bool {
        self.mouse_wheel.0
    }

    #[inline]
    pub fn mouse_wheel_down(&self) -> bool {
        self.mouse_wheel.1
    }

    #[inline]
    pub fn mouse_x(&self) -> i32 {
        self.mouse_position.0
    }

    #[inline]
    pub fn mouse_y(&self) -> i32 {
        self.mouse_position.1
    }

    /// Clears the button press and release buffers.
    /// Should be called after each frame.
    pub fn step(&mut self) {
        self.button_state_press.iter_mut().for_each(|x| *x = false);
        self.button_state_release.iter_mut().for_each(|x| *x = false);
    }

    /// Hard reset, clearing all state.
    pub fn reset(&mut self) {
        *self = Self::new();
    }
}
