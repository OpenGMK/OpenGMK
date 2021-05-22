const KEY_MAX: usize = 256;
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
    Capital = 0x14,
    ImeKanaOrHangul = 0x15,
    ImeOn = 0x16,
    ImeJunja = 0x17,
    ImeFinal = 0x18,
    ImeHanjaOrKanji = 0x19,
    ImeOff = 0x1A,
    Escape = 0x1B,
    ImeConvert = 0x1C,
    ImeNonconvert = 0x1D,
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
    Scroll = 0x91,
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
    MediaVolumeUp = 0xAF,
    MediaNextTrack = 0xB0,
    MediaPrevTrack = 0xB1,
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

impl Button {
    fn is_key(&self) -> bool {
        !self.is_mouse_button()
    }

    fn is_mouse_button(&self) -> bool {
        matches!(
            self,
            Self::MouseLeft | Self::MouseRight | Self::MouseMiddle,
        )
    }
}

const fn make_is_direct_only() -> [bool; KEY_MAX] {
    let mut table = [false; KEY_MAX];
    let mut i = 0;
    while i < KEY_MAX {
        if
            i == Button::MouseLeft as usize ||
            i == Button::MouseRight as usize ||
            i == Button::MouseMiddle as usize ||
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

#[repr(u8)]
pub enum MouseButton {
    // gamemaker
    Left = 1,
    Right = 2,
    Middle = 3,

    // non-gamemaker
    X1 = 4,
    X2 = 5,
}

fn mouse2button(code: u8) -> Option<Button> {
    match code {
        x if x == MouseButton::Left as u8 => Some(Button::MouseLeft),
        x if x == MouseButton::Right as u8 => Some(Button::MouseRight),
        x if x == MouseButton::Middle as u8 => Some(Button::MouseMiddle),
        _ => None,
    }
}

pub struct Input {
    // basic state
    button_state: [bool; KEY_MAX],
    button_state_press: [bool; KEY_MAX],
    button_state_release: [bool; KEY_MAX],
    mouse_position: (i32, i32),

    // gamemaker weirdness
    key_current: u8,
    key_previous: u8,
    mouse_current: u8,
    mouse_previous: u8,
}

impl Input {
    pub const fn new() -> Self {
        Input {
            button_state: [false; KEY_MAX],
            button_state_press: [false; KEY_MAX],
            button_state_release: [false; KEY_MAX],
            mouse_position: (0, 0),
            key_current: 0,
            key_previous: 0,
            mouse_current: 0,
            mouse_previous: 0,
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
        // TODO: This entirely right?
        if store_cur_prev && self.key_current == code {
            self.key_current = 0;
        }
    }

    pub fn mouse_press(&mut self, code: u8, store_cur_prev: bool) {
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

    pub fn mouse_release(&mut self, code: u8, store_cur_prev: bool) {
        let button = match mouse2button(code) {
            Some(button) => button,
            None => return,
        };
        // TODO: Again, this entirely right?
        if store_cur_prev {
            self.mouse_current = 0;
        }
        self.button_press(button as u8, false);
    }

    // == GameMaker Mappings ==

    fn keyboard_check_any_internal(&self, state: &[bool; KEY_MAX]) -> bool {
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
        if vk == VK_NOKEY {
            !self.keyboard_check_any_internal(state)
        } else if vk == VK_ANYKEY {
            self.keyboard_check_any_internal(state)
        } else if !IS_DIRECT_ONLY[vk as usize] {
            self.keyboard_check_internal(state, vk)
        } else {
            false
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
