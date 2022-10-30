use crate::types::ArraySerde;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{
        Display,
        Error,
        Formatter
    },
    convert::TryFrom,
    num::NonZeroI32
};

const KEY_MAX: usize = u8::max_value() as usize + 1;
const MB_ANY: i8 = -1;
const MB_NONE: i8 = 0;
const VK_NOKEY: u8 = 0; // TODO: dont redefine
const VK_ANYKEY: u8 = 1; // TODO: dont redefine

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
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

impl Display for Button {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            Self::MouseLeft => write!(f, "Mouse Left"),
            Self::MouseRight => write!(f, "Mouse Right"),
            Self::MouseMiddle => write!(f, "Mouse Middle"),
            Self::MouseX1 => write!(f, "Mouse X1"),
            Self::MouseX2 => write!(f, "Mouse X2"),
            Self::Backspace => write!(f, "Backspace"),
            Self::Tab => write!(f, "Tab"),
            Self::Clear => write!(f, "Clear"),
            Self::Return => write!(f, "Return"),
            Self::Shift => write!(f, "Shift"),
            Self::Control => write!(f, "Control"),
            Self::Alt => write!(f, "Alt"),
            Self::Pause => write!(f, "Pause"),
            Self::CapsLock => write!(f, "CapsLock"),
            Self::ImeKanaOrHangul => write!(f, "Ime Kana Or Hangul"),
            Self::ImeOn => write!(f, "Ime On"),
            Self::ImeJunja => write!(f, "Ime Junja"),
            Self::ImeFinal => write!(f, "Ime Final"),
            Self::ImeHanjaOrKanji => write!(f, "Ime Hanja Or Kanji"),
            Self::ImeOff => write!(f, "Ime Off"),
            Self::Escape => write!(f, "Escape"),
            Self::ImeConvert => write!(f, "Ime Convert"),
            Self::ImeNonConvert => write!(f, "Ime Non Convert"),
            Self::ImeAccept => write!(f, "Ime Accept"),
            Self::ImeModeChangeRequest => write!(f, "Ime Mode Change Request"),
            Self::Space => write!(f, "Space"),
            Self::PageUp => write!(f, "Page Up"),
            Self::PageDown => write!(f, "Page Down"),
            Self::End => write!(f, "End"),
            Self::Home => write!(f, "Home"),
            Self::LeftArrow => write!(f, "Left Arrow"),
            Self::UpArrow => write!(f, "Up Arrow"),
            Self::RightArrow => write!(f, "Right Arrow"),
            Self::DownArrow => write!(f, "Down Arrow"),
            Self::Select => write!(f, "Select"),
            Self::Print => write!(f, "Print"),
            Self::Execute => write!(f, "Execute"),
            Self::PrintScreen => write!(f, "Print Screen"),
            Self::Insert => write!(f, "Insert"),
            Self::Delete => write!(f, "Delete"),
            Self::Help => write!(f, "Help"),
            Self::Alpha0 => write!(f, "0"),
            Self::Alpha1 => write!(f, "1"),
            Self::Alpha2 => write!(f, "2"),
            Self::Alpha3 => write!(f, "3"),
            Self::Alpha4 => write!(f, "4"),
            Self::Alpha5 => write!(f, "5"),
            Self::Alpha6 => write!(f, "6"),
            Self::Alpha7 => write!(f, "7"),
            Self::Alpha8 => write!(f, "8"),
            Self::Alpha9 => write!(f, "9"),
            Self::A => write!(f, "A"),
            Self::B => write!(f, "B"),
            Self::C => write!(f, "C"),
            Self::D => write!(f, "D"),
            Self::E => write!(f, "E"),
            Self::F => write!(f, "F"),
            Self::G => write!(f, "G"),
            Self::H => write!(f, "H"),
            Self::I => write!(f, "I"),
            Self::J => write!(f, "J"),
            Self::K => write!(f, "K"),
            Self::L => write!(f, "L"),
            Self::M => write!(f, "M"),
            Self::N => write!(f, "N"),
            Self::O => write!(f, "O"),
            Self::P => write!(f, "P"),
            Self::Q => write!(f, "Q"),
            Self::R => write!(f, "R"),
            Self::S => write!(f, "S"),
            Self::T => write!(f, "T"),
            Self::U => write!(f, "U"),
            Self::V => write!(f, "V"),
            Self::W => write!(f, "W"),
            Self::X => write!(f, "X"),
            Self::Y => write!(f, "Y"),
            Self::Z => write!(f, "Z"),
            Self::LeftWindows => write!(f, "Left Windows"),
            Self::RightWindows => write!(f, "Right Windows"),
            Self::Applications => write!(f, "Applications"),
            Self::Sleep => write!(f, "Sleep"),
            Self::Keypad0 => write!(f, "Keypad 0"),
            Self::Keypad1 => write!(f, "Keypad 1"),
            Self::Keypad2 => write!(f, "Keypad 2"),
            Self::Keypad3 => write!(f, "Keypad 3"),
            Self::Keypad4 => write!(f, "Keypad 4"),
            Self::Keypad5 => write!(f, "Keypad 5"),
            Self::Keypad6 => write!(f, "Keypad 6"),
            Self::Keypad7 => write!(f, "Keypad 7"),
            Self::Keypad8 => write!(f, "Keypad 8"),
            Self::Keypad9 => write!(f, "Keypad 9"),
            Self::KeypadMultiply => write!(f, "Keypad Multiply"),
            Self::KeypadAdd => write!(f, "Keypad Add"),
            Self::KeypadSeparator => write!(f, "Keypad Separator"),
            Self::KeypadSubtract => write!(f, "Keypad Subtract"),
            Self::KeypadDecimal => write!(f, "Keypad Decimal"),
            Self::KeypadDivide => write!(f, "Keypad Divide"),
            Self::F1 => write!(f, "F1"),
            Self::F2 => write!(f, "F2"),
            Self::F3 => write!(f, "F3"),
            Self::F4 => write!(f, "F4"),
            Self::F5 => write!(f, "F5"),
            Self::F6 => write!(f, "F6"),
            Self::F7 => write!(f, "F7"),
            Self::F8 => write!(f, "F8"),
            Self::F9 => write!(f, "F9"),
            Self::F10 => write!(f, "F10"),
            Self::F11 => write!(f, "F11"),
            Self::F12 => write!(f, "F12"),
            Self::F13 => write!(f, "F13"),
            Self::F14 => write!(f, "F14"),
            Self::F15 => write!(f, "F15"),
            Self::F16 => write!(f, "F16"),
            Self::F17 => write!(f, "F17"),
            Self::F18 => write!(f, "F18"),
            Self::F19 => write!(f, "F19"),
            Self::F20 => write!(f, "F20"),
            Self::F21 => write!(f, "F21"),
            Self::F22 => write!(f, "F22"),
            Self::F23 => write!(f, "F23"),
            Self::F24 => write!(f, "F24"),
            Self::NumLock => write!(f, "NumLock"),
            Self::ScrollLock => write!(f, "ScrollLock"),
            Self::LeftShift => write!(f, "Left Shift"),
            Self::RightShift => write!(f, "Right Shift"),
            Self::LeftControl => write!(f, "Left Control"),
            Self::RightControl => write!(f, "Right Control"),
            Self::LeftAlt => write!(f, "Left Alt"),
            Self::RightAlt => write!(f, "Right Alt"),
            Self::BrowserBack => write!(f, "Browser Back"),
            Self::BrowserForward => write!(f, "Browser Forward"),
            Self::BrowserRefresh => write!(f, "Browser Refresh"),
            Self::BrowserStop => write!(f, "Browser Stop"),
            Self::BrowserSearch => write!(f, "Browser Search"),
            Self::BrowserFavourites => write!(f, "Browser Favourites"),
            Self::BrowserHome => write!(f, "Browser Home"),
            Self::MediaVolumeDown => write!(f, "Media Volume Down"),
            Self::MediaVolumeMute => write!(f, "Media Volume Mute"),
            Self::MediaVolumeUp => write!(f, "Media Volume Up"),
            Self::MediaNextTrack => write!(f, "Media Next Track"),
            Self::MediaPreviousTrack => write!(f, "Media Previous Track"),
            Self::MediaStop => write!(f, "Media Stop"),
            Self::MediaPlayPause => write!(f, "Media Play Pause"),
            Self::LaunchMail => write!(f, "Launch Mail"),
            Self::LaunchMedia => write!(f, "Launch Media"),
            Self::LaunchApp1 => write!(f, "Launch App1"),
            Self::LaunchApp2 => write!(f, "Launch App2"),
            Self::Oem1 => write!(f, "Oem1"),
            Self::OemPlus => write!(f, "Plus"),
            Self::OemComma => write!(f, "Comma"),
            Self::OemMinus => write!(f, "Minus"),
            Self::OemPeriod => write!(f, "Period"),
            Self::Oem2 => write!(f, "Oem2"),
            Self::Oem3 => write!(f, "Oem3"),
            Self::GamepadA => write!(f, "Gamepad A"),
            Self::GamepadB => write!(f, "Gamepad B"),
            Self::GamepadX => write!(f, "Gamepad X"),
            Self::GamepadY => write!(f, "Gamepad Y"),
            Self::GamepadR1 => write!(f, "Gamepad R1"),
            Self::GamepadL1 => write!(f, "Gamepad L1"),
            Self::GamepadL2 => write!(f, "Gamepad L2"),
            Self::GamepadR2 => write!(f, "Gamepad R2"),
            Self::GamepadDpadUp => write!(f, "Gamepad Dpad Up"),
            Self::GamepadDpadDown => write!(f, "Gamepad Dpad Down"),
            Self::GamepadDpadLeft => write!(f, "Gamepad Dpad Left"),
            Self::GamepadDpadRight => write!(f, "Gamepad Dpad Right"),
            Self::GamepadMenu => write!(f, "Gamepad Menu"),
            Self::GamepadView => write!(f, "Gamepad View"),
            Self::GamepadL3 => write!(f, "Gamepad L3"),
            Self::GamepadR3 => write!(f, "Gamepad R3"),
            Self::GamepadLUp => write!(f, "Gamepad L Up"),
            Self::GamepadLDown => write!(f, "Gamepad L Down"),
            Self::GamepadLRight => write!(f, "Gamepad L Right"),
            Self::GamepadLLeft => write!(f, "Gamepad L Left"),
            Self::GamepadRUp => write!(f, "Gamepad R Up"),
            Self::GamepadRDown => write!(f, "Gamepad R Down"),
            Self::GamepadRRight => write!(f, "Gamepad R Right"),
            Self::GamepadRLeft => write!(f, "Gamepad R Left"),
            Self::Oem4 => write!(f, "Oem4"),
            Self::Oem5 => write!(f, "Oem5"),
            Self::Oem6 => write!(f, "Oem6"),
            Self::Oem7 => write!(f, "Oem7"),
            Self::Oem8 => write!(f, "Oem8"),
            Self::OemAx => write!(f, "Oem Ax"),
            Self::Oem102 => write!(f, "Oem102"),
            Self::IcoHelp => write!(f, "IcoHelp"),
            Self::Ico00 => write!(f, "Ico00"),
            Self::ImeProcess => write!(f, "Ime Process"),
            Self::IcoClear => write!(f, "Ico Clear"),
            Self::OemReset => write!(f, "Oem Reset"),
            Self::OemJump => write!(f, "Oem Jump"),
            Self::OemPa1 => write!(f, "Oem Pa1"),
            Self::OemPa2 => write!(f, "Oem Pa2"),
            Self::OemPa3 => write!(f, "Oem Pa3"),
            Self::OemWsCtrl => write!(f, "OemWsCtrl"),
            Self::OemCuSel => write!(f, "OemCuSel"),
            Self::OemAttn => write!(f, "OemAttn"),
            Self::OemFinish => write!(f, "OemFinish"),
            Self::OemCopy => write!(f, "OemCopy"),
            Self::OemAuto => write!(f, "OemAuto"),
            Self::OemEnlw => write!(f, "OemEnlw"),
            Self::OemBackTab => write!(f, "OemBackTab"),
            Self::Attn => write!(f, "Attn"),
            Self::CrSel => write!(f, "CrSel"),
            Self::ExSel => write!(f, "ExSel"),
            Self::EraseEof => write!(f, "EraseEof"),
            Self::MediaPlay => write!(f, "Media Play"),
            Self::Zoom => write!(f, "Zoom"),
            Self::Pa1 => write!(f, "Pa1"),
            Self::OemClear => write!(f, "OemClear"),
        }
    }
}

impl TryFrom<ramen::input::Key> for Button {
    type Error = ();

    fn try_from(key: ramen::input::Key) -> Result<Self, Self::Error> {
        match key {
            ramen::input::Key::Attn => Ok(Self::Attn),
            ramen::input::Key::LeftAlt => Ok(Self::LeftAlt),
            ramen::input::Key::RightAlt => Ok(Self::RightAlt),
            ramen::input::Key::Applications => Ok(Self::Applications),
            ramen::input::Key::Backspace => Ok(Self::Backspace),
            ramen::input::Key::CapsLock => Ok(Self::CapsLock),
            ramen::input::Key::Clear => Ok(Self::Clear),
            ramen::input::Key::LeftControl => Ok(Self::LeftControl),
            ramen::input::Key::RightControl => Ok(Self::RightControl),
            ramen::input::Key::CrSel => Ok(Self::CrSel),
            ramen::input::Key::Delete => Ok(Self::Delete),
            ramen::input::Key::End => Ok(Self::End),
            ramen::input::Key::Return => Ok(Self::Return),
            ramen::input::Key::EraseEof => Ok(Self::EraseEof),
            ramen::input::Key::Escape => Ok(Self::Escape),
            ramen::input::Key::Execute => Ok(Self::Execute),
            ramen::input::Key::ExSel => Ok(Self::ExSel),
            ramen::input::Key::Help => Ok(Self::Help),
            ramen::input::Key::Home => Ok(Self::Home),
            ramen::input::Key::Insert => Ok(Self::Insert),
            ramen::input::Key::NumLock => Ok(Self::NumLock),
            ramen::input::Key::Pa1 => Ok(Self::Pa1),
            ramen::input::Key::PageUp => Ok(Self::PageUp),
            ramen::input::Key::PageDown => Ok(Self::PageDown),
            ramen::input::Key::Pause => Ok(Self::Pause),
            ramen::input::Key::Play => Ok(Self::MediaPlay),
            ramen::input::Key::Print => Ok(Self::Print),
            ramen::input::Key::PrintScreen => Ok(Self::PrintScreen),
            ramen::input::Key::LeftShift => Ok(Self::LeftShift),
            ramen::input::Key::RightShift => Ok(Self::RightShift),
            ramen::input::Key::ScrollLock => Ok(Self::ScrollLock),
            ramen::input::Key::Select => Ok(Self::Select),
            ramen::input::Key::Sleep => Ok(Self::Sleep),
            ramen::input::Key::Space => Ok(Self::Space),
            ramen::input::Key::LeftSuper => Ok(Self::LeftWindows),
            ramen::input::Key::RightSuper => Ok(Self::RightWindows),
            ramen::input::Key::Tab => Ok(Self::Tab),
            ramen::input::Key::Zoom => Ok(Self::Zoom),
            ramen::input::Key::LeftArrow => Ok(Self::LeftArrow),
            ramen::input::Key::UpArrow => Ok(Self::UpArrow),
            ramen::input::Key::RightArrow => Ok(Self::RightArrow),
            ramen::input::Key::DownArrow => Ok(Self::DownArrow),
            ramen::input::Key::Alpha0 => Ok(Self::Alpha0),
            ramen::input::Key::Alpha1 => Ok(Self::Alpha1),
            ramen::input::Key::Alpha2 => Ok(Self::Alpha2),
            ramen::input::Key::Alpha3 => Ok(Self::Alpha3),
            ramen::input::Key::Alpha4 => Ok(Self::Alpha4),
            ramen::input::Key::Alpha5 => Ok(Self::Alpha5),
            ramen::input::Key::Alpha6 => Ok(Self::Alpha6),
            ramen::input::Key::Alpha7 => Ok(Self::Alpha7),
            ramen::input::Key::Alpha8 => Ok(Self::Alpha8),
            ramen::input::Key::Alpha9 => Ok(Self::Alpha9),
            ramen::input::Key::A => Ok(Self::A),
            ramen::input::Key::B => Ok(Self::B),
            ramen::input::Key::C => Ok(Self::C),
            ramen::input::Key::D => Ok(Self::D),
            ramen::input::Key::E => Ok(Self::E),
            ramen::input::Key::F => Ok(Self::F),
            ramen::input::Key::G => Ok(Self::G),
            ramen::input::Key::H => Ok(Self::H),
            ramen::input::Key::I => Ok(Self::I),
            ramen::input::Key::J => Ok(Self::J),
            ramen::input::Key::K => Ok(Self::K),
            ramen::input::Key::L => Ok(Self::L),
            ramen::input::Key::M => Ok(Self::M),
            ramen::input::Key::N => Ok(Self::N),
            ramen::input::Key::O => Ok(Self::O),
            ramen::input::Key::P => Ok(Self::P),
            ramen::input::Key::Q => Ok(Self::Q),
            ramen::input::Key::R => Ok(Self::R),
            ramen::input::Key::S => Ok(Self::S),
            ramen::input::Key::T => Ok(Self::T),
            ramen::input::Key::U => Ok(Self::U),
            ramen::input::Key::V => Ok(Self::V),
            ramen::input::Key::W => Ok(Self::W),
            ramen::input::Key::X => Ok(Self::X),
            ramen::input::Key::Y => Ok(Self::Y),
            ramen::input::Key::Z => Ok(Self::Z),
            ramen::input::Key::Comma => Ok(Self::OemComma),
            ramen::input::Key::Minus => Ok(Self::OemMinus),
            ramen::input::Key::Period => Ok(Self::OemPeriod),
            ramen::input::Key::Plus => Ok(Self::OemPlus),
            ramen::input::Key::Oem102 => Ok(Self::Oem102),
            ramen::input::Key::OemClear => Ok(Self::OemClear),
            ramen::input::Key::KeypadAdd => Ok(Self::KeypadAdd),
            ramen::input::Key::KeypadSubtract => Ok(Self::KeypadSubtract),
            ramen::input::Key::KeypadMultiply => Ok(Self::KeypadMultiply),
            ramen::input::Key::KeypadDivide => Ok(Self::KeypadDivide),
            ramen::input::Key::KeypadDecimal => Ok(Self::KeypadDecimal),
            ramen::input::Key::KeypadSeparator => Ok(Self::KeypadSeparator),
            ramen::input::Key::Keypad0 => Ok(Self::Keypad0),
            ramen::input::Key::Keypad1 => Ok(Self::Keypad1),
            ramen::input::Key::Keypad2 => Ok(Self::Keypad2),
            ramen::input::Key::Keypad3 => Ok(Self::Keypad3),
            ramen::input::Key::Keypad4 => Ok(Self::Keypad4),
            ramen::input::Key::Keypad5 => Ok(Self::Keypad5),
            ramen::input::Key::Keypad6 => Ok(Self::Keypad6),
            ramen::input::Key::Keypad7 => Ok(Self::Keypad7),
            ramen::input::Key::Keypad8 => Ok(Self::Keypad8),
            ramen::input::Key::Keypad9 => Ok(Self::Keypad9),
            ramen::input::Key::F1 => Ok(Self::F1),
            ramen::input::Key::F2 => Ok(Self::F2),
            ramen::input::Key::F3 => Ok(Self::F3),
            ramen::input::Key::F4 => Ok(Self::F4),
            ramen::input::Key::F5 => Ok(Self::F5),
            ramen::input::Key::F6 => Ok(Self::F6),
            ramen::input::Key::F7 => Ok(Self::F7),
            ramen::input::Key::F8 => Ok(Self::F8),
            ramen::input::Key::F9 => Ok(Self::F9),
            ramen::input::Key::F10 => Ok(Self::F10),
            ramen::input::Key::F11 => Ok(Self::F11),
            ramen::input::Key::F12 => Ok(Self::F12),
            ramen::input::Key::F13 => Ok(Self::F13),
            ramen::input::Key::F14 => Ok(Self::F14),
            ramen::input::Key::F15 => Ok(Self::F15),
            ramen::input::Key::F16 => Ok(Self::F16),
            ramen::input::Key::F17 => Ok(Self::F17),
            ramen::input::Key::F18 => Ok(Self::F18),
            ramen::input::Key::F19 => Ok(Self::F19),
            ramen::input::Key::F20 => Ok(Self::F20),
            ramen::input::Key::F21 => Ok(Self::F21),
            ramen::input::Key::F22 => Ok(Self::F22),
            ramen::input::Key::F23 => Ok(Self::F23),
            ramen::input::Key::F24 => Ok(Self::F24),
            ramen::input::Key::BrowserBack => Ok(Self::BrowserBack),
            ramen::input::Key::BrowserFavourites => Ok(Self::BrowserFavourites),
            ramen::input::Key::BrowserForward => Ok(Self::BrowserForward),
            ramen::input::Key::BrowserHome => Ok(Self::BrowserHome),
            ramen::input::Key::BrowserRefresh => Ok(Self::BrowserRefresh),
            ramen::input::Key::BrowserSearch => Ok(Self::BrowserSearch),
            ramen::input::Key::BrowserStop => Ok(Self::BrowserStop),
            ramen::input::Key::ImeAccept => Ok(Self::ImeAccept),
            ramen::input::Key::ImeConvert => Ok(Self::ImeConvert),
            ramen::input::Key::ImeNonConvert => Ok(Self::ImeNonConvert),
            ramen::input::Key::ImeFinal => Ok(Self::ImeFinal),
            ramen::input::Key::ImeModeChangeRequest => Ok(Self::ImeModeChangeRequest),
            ramen::input::Key::ImeProcess => Ok(Self::ImeProcess),
            ramen::input::Key::ImeOn => Ok(Self::ImeOn),
            ramen::input::Key::ImeOff => Ok(Self::ImeOff),
            ramen::input::Key::ImeKanaOrHangul => Ok(Self::ImeKanaOrHangul),
            ramen::input::Key::ImeHanjaOrKanji => Ok(Self::ImeHanjaOrKanji),
            ramen::input::Key::ImeJunja => Ok(Self::ImeJunja),
            ramen::input::Key::MediaNextTrack => Ok(Self::MediaNextTrack),
            ramen::input::Key::MediaPreviousTrack => Ok(Self::MediaPreviousTrack),
            ramen::input::Key::MediaPlayPause => Ok(Self::MediaPlayPause),
            ramen::input::Key::MediaStop => Ok(Self::MediaStop),
            ramen::input::Key::MediaVolumeDown => Ok(Self::MediaVolumeDown),
            ramen::input::Key::MediaVolumeUp => Ok(Self::MediaVolumeUp),
            ramen::input::Key::MediaVolumeMute => Ok(Self::MediaVolumeMute),
            ramen::input::Key::LaunchApplication1 => Ok(Self::LaunchApp1),
            ramen::input::Key::LaunchApplication2 => Ok(Self::LaunchApp2),
            ramen::input::Key::LaunchMail => Ok(Self::LaunchMail),
            ramen::input::Key::LaunchMediaSelect => Ok(Self::LaunchMedia),
            _ => Err(()),
        }
    }
}

pub fn ramen2vk(x: ramen::input::Key) -> u8 {
    Button::try_from(x).map(|e| e as u8).unwrap_or(0)
}

const fn make_is_direct_only() -> [bool; KEY_MAX] {
    let mut table = [false; KEY_MAX];
    let mut i = 0;
    while i < KEY_MAX {
        if i == Button::MouseLeft as usize
            || i == Button::MouseRight as usize
            || i == Button::MouseMiddle as usize
            || i == Button::MouseX1 as usize
            || i == Button::MouseX2 as usize
            || i == Button::LeftShift as usize
            || i == Button::RightShift as usize
            || i == Button::LeftControl as usize
            || i == Button::RightControl as usize
            || i == Button::LeftAlt as usize
            || i == Button::RightAlt as usize
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

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
#[repr(i8)]
pub enum MouseButton {
    // gamemaker
    Left = 1,
    Right = 2,
    Middle = 3,
}

impl TryFrom<ramen::input::MouseButton> for MouseButton {
    type Error = ();

    fn try_from(key: ramen::input::MouseButton) -> Result<Self, Self::Error> {
        match key {
            ramen::input::MouseButton::Left => Ok(Self::Left),
            ramen::input::MouseButton::Right => Ok(Self::Right),
            ramen::input::MouseButton::Middle => Ok(Self::Middle),
        }
    }
}

pub fn ramen2mb(x: ramen::input::MouseButton) -> i8 {
    MouseButton::try_from(x).map(|e| e as i8).unwrap_or(0)
}

fn mouse2button(code: i8) -> Option<Button> {
    match code {
        x if x == MouseButton::Left as i8 => Some(Button::MouseLeft),
        x if x == MouseButton::Right as i8 => Some(Button::MouseRight),
        x if x == MouseButton::Middle as i8 => Some(Button::MouseMiddle),
        _ => None,
    }
}

const fn gen_default_keymap() -> [u8; KEY_MAX] {
    let mut map = [0u8; KEY_MAX];
    let mut i = 0;
    while i < KEY_MAX {
        map[i] = i as u8;
        i += 1;
    }
    map
}
const DEFAULT_KEYMAP: [u8; KEY_MAX] = gen_default_keymap();

#[derive(Clone, Deserialize, Serialize)]
pub struct Input {
    // basic state
    button_remap: ArraySerde<u8, KEY_MAX>,
    button_state: ArraySerde<bool, KEY_MAX>,
    button_state_press: ArraySerde<bool, KEY_MAX>,
    button_state_release: ArraySerde<bool, KEY_MAX>,
    mouse_position: (i32, i32),
    mouse_wheel: (bool, bool),

    // gamemaker weirdness
    key_current: u8,
    key_previous: u8,
    mouse_current: i8,
    mouse_previous: i8,
    mouse_position_previous: (i32, i32),
    numlock_state: bool, // spoofed!
}

impl Input {
    pub const fn new() -> Self {
        Input {
            button_remap: ArraySerde(DEFAULT_KEYMAP),
            button_state: ArraySerde([false; KEY_MAX]),
            button_state_press: ArraySerde([false; KEY_MAX]),
            button_state_release: ArraySerde([false; KEY_MAX]),
            mouse_position: (0, 0),
            mouse_wheel: (false, false),
            key_current: 0,
            key_previous: 0,
            mouse_current: 0,
            mouse_previous: 0,
            mouse_position_previous: (0, 0),
            numlock_state: false,
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

    #[inline]
    pub fn mouse_move_to(&mut self, pos: (i32, i32)) {
        self.mouse_position = pos;
    }

    pub fn mouse_release(&mut self, code: i8, store_cur_prev: bool) {
        let button = match mouse2button(code) {
            Some(button) => button,
            None => return,
        };
        if store_cur_prev {
            self.mouse_current = 0;
        }
        self.button_release(button as u8, false);
    }

    pub fn mouse_scroll_up(&mut self) {
        self.mouse_wheel.0 = true;
    }

    pub fn mouse_scroll_down(&mut self) {
        self.mouse_wheel.1 = true;
    }

    // == GameMaker Mappings ==

    fn keyboard_check_any_internal_indirect(&self, state: &[bool; KEY_MAX]) -> bool {
        state.iter().enumerate().any(|(vk, flag)| match vk {
            vk if vk == Button::Shift as usize => {
                state[Button::LeftShift as usize] || state[Button::RightShift as usize]
            },
            vk if vk == Button::Control as usize => {
                state[Button::LeftControl as usize] || state[Button::RightControl as usize]
            },
            vk if vk == Button::Alt as usize => state[Button::LeftAlt as usize] || state[Button::RightAlt as usize],
            vk if !IS_DIRECT_ONLY[vk] => *flag,
            _ => false,
        })
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
    pub fn keyboard_check_any(&self) -> bool {
        self.keyboard_check_any_internal_indirect(&self.button_state)
    }

    #[inline]
    pub fn keyboard_check_pressed(&self, vk: u8) -> bool {
        self.keyboard_check_internal_indirect(&self.button_state_press, vk)
    }

    #[inline]
    pub fn keyboard_check_pressed_any(&self) -> bool {
        self.keyboard_check_any_internal_indirect(&self.button_state_press)
    }

    #[inline]
    pub fn keyboard_check_released(&self, vk: u8) -> bool {
        self.keyboard_check_internal_indirect(&self.button_state_release, vk)
    }

    #[inline]
    pub fn keyboard_check_released_any(&self) -> bool {
        self.keyboard_check_any_internal_indirect(&self.button_state_release)
    }

    #[inline]
    pub fn keyboard_check_direct(&self, vk: u8) -> bool {
        self.keyboard_check_internal(&self.button_state, vk)
    }

    pub fn keyboard_clear(&mut self, vk: u8) {
        // TODO this sucks

        let (vk, vk2) = match vk {
            x if x == Button::Control as u8 => (Button::LeftControl as u8, Button::RightControl as u8),
            x if x == Button::Shift as u8 => (Button::LeftShift as u8, Button::RightShift as u8),
            x if x == Button::Alt as u8 => (Button::LeftAlt as u8, Button::RightAlt as u8),
            _ => (vk, vk),
        };

        self.button_state[vk as usize] = false;
        self.button_state_press[vk as usize] = false;
        self.button_state_release[vk as usize] = false;
        self.button_state[vk2 as usize] = false;
        self.button_state_press[vk2 as usize] = false;
        self.button_state_release[vk2 as usize] = false;
    }

    pub fn keyboard_clear_all(&mut self) {
        self.key_current = 0;
        self.key_previous = 0;
        // TODO: self.key_lastchar = 0;
        self.button_state.iter_mut().for_each(|x| *x = false);
        self.button_state_press.iter_mut().for_each(|x| *x = false);
        self.button_state_release.iter_mut().for_each(|x| *x = false);
    }

    #[inline]
    pub fn keyboard_get_map(&mut self, vk: u8) -> u8 {
        self.button_remap[vk as usize]
    }

    #[inline]
    pub fn keyboard_set_map(&mut self, vk_from: u8, vk_to: u8) {
        self.button_remap[vk_from as usize] = vk_to;
    }

    #[inline]
    pub fn keyboard_get_numlock(&self) -> bool {
        self.numlock_state
    }

    #[inline]
    pub fn keyboard_set_numlock(&mut self, state: bool) {
        self.numlock_state = state;
    }

    pub fn keyboard_unset_map(&mut self) {
        self.button_remap.0 = DEFAULT_KEYMAP;
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
            MB_ANY => {
                state[Button::MouseLeft as usize]
                    || state[Button::MouseRight as usize]
                    || state[Button::MouseMiddle as usize]
            },
            MB_NONE => {
                !state[Button::MouseLeft as usize]
                    && !state[Button::MouseRight as usize]
                    && !state[Button::MouseMiddle as usize]
            },

            // unlike `mouse2button`, gm constants only
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

    pub fn mouse_clear(&mut self, mb: i8) {
        let button = match mb {
            // unlike `mouse2button`, gm constants only
            x if x == MouseButton::Left as i8 => Button::MouseLeft,
            x if x == MouseButton::Right as i8 => Button::MouseRight,
            x if x == MouseButton::Middle as i8 => Button::MouseMiddle,
            _ => return,
        };
        self.keyboard_clear(button as u8);
    }

    pub fn mouse_clear_all(&mut self) {
        self.mouse_current = 0;
        self.mouse_previous = 0;
        for button in &[Button::MouseLeft, Button::MouseRight, Button::MouseMiddle] {
            self.keyboard_clear(*button as u8);
        }
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
    pub fn mouse_check_button_any(&self) -> bool {
        self.mouse_check_button(MB_ANY)
    }

    #[inline]
    pub fn mouse_check_button_pressed(&self, mb: i8) -> bool {
        self.mouse_check_button_internal_indirect(&self.button_state_press, mb)
    }

    #[inline]
    pub fn mouse_check_button_pressed_any(&self) -> bool {
        self.mouse_check_button_pressed(MB_ANY)
    }

    #[inline]
    pub fn mouse_check_button_released(&self, mb: i8) -> bool {
        self.mouse_check_button_internal_indirect(&self.button_state_release, mb)
    }

    #[inline]
    pub fn mouse_check_button_released_any(&self) -> bool {
        self.mouse_check_button_released(MB_ANY)
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

    #[inline]
    pub fn mouse_x_previous(&self) -> i32 {
        self.mouse_position_previous.0
    }

    #[inline]
    pub fn mouse_y_previous(&self) -> i32 {
        self.mouse_position_previous.1
    }

    /// Clears the button press and release buffers.
    /// Should be called after each frame.
    pub fn step(&mut self) {
        self.button_state_press.iter_mut().for_each(|x| *x = false);
        self.button_state_release.iter_mut().for_each(|x| *x = false);
    }

    pub fn mouse_step(&mut self) {
        self.mouse_position_previous = self.mouse_position;
        self.mouse_wheel = (false, false);
    }

    /// Hard reset, clearing all state.
    pub fn reset(&mut self) {
        *self = Self::new();
    }
}
