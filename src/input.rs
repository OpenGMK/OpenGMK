// We put the WINAPI constants here because gamemaker GML works with them and needs them :l
pub const VK_ADD: u8 = 0x6B;
pub const VK_ALT: u8 = 0x12; // real name: VK_MENU
pub const VK_BACKSPACE: u8 = 0x08;
pub const VK_CONTROL: u8 = 0x11;
pub const VK_DECIMAL: u8 = 0x6E;
pub const VK_DELETE: u8 = 0x2E;
pub const VK_DIVIDE: u8 = 0x6F;
pub const VK_DOWN: u8 = 0x28;
pub const VK_END: u8 = 0x23;
pub const VK_ENTER: u8 = 0x0D; // alias for VK_RETURN
pub const VK_ESCAPE: u8 = 0x1B;
pub const VK_F1: u8 = 0x70;
pub const VK_F2: u8 = 0x71;
pub const VK_F3: u8 = 0x72;
pub const VK_F4: u8 = 0x73;
pub const VK_F5: u8 = 0x74;
pub const VK_F6: u8 = 0x75;
pub const VK_F7: u8 = 0x76;
pub const VK_F8: u8 = 0x77;
pub const VK_F9: u8 = 0x78;
pub const VK_F10: u8 = 0x79;
pub const VK_F11: u8 = 0x7A;
pub const VK_F12: u8 = 0x7B;
pub const VK_HOME: u8 = 0x24;
pub const VK_INSERT: u8 = 0x2D;
pub const VK_MULTIPLY: u8 = 0x6A;
pub const VK_LCONTROL: u8 = 0xA2;
pub const VK_LEFT: u8 = 0x25;
pub const VK_LSHIFT: u8 = 0xA0;
pub const VK_NUMPAD0: u8 = 0x60;
pub const VK_NUMPAD1: u8 = 0x61;
pub const VK_NUMPAD2: u8 = 0x62;
pub const VK_NUMPAD3: u8 = 0x63;
pub const VK_NUMPAD4: u8 = 0x64;
pub const VK_NUMPAD5: u8 = 0x65;
pub const VK_NUMPAD6: u8 = 0x66;
pub const VK_NUMPAD7: u8 = 0x67;
pub const VK_NUMPAD8: u8 = 0x68;
pub const VK_NUMPAD9: u8 = 0x69;
pub const VK_PAGEDOWN: u8 = 0x22; // real name: VK_NEXT
pub const VK_PAGEUP: u8 = 0x21; // real name: VK_PRIOR
pub const VK_PAUSE: u8 = 0x13;
pub const VK_PRINTSCREEN: u8 = 0x2C; // real name: VK_SNAPSHOT
pub const VK_RCONTROL: u8 = 0xA3;
pub const VK_RETURN: u8 = 0x0D;
pub const VK_RIGHT: u8 = 0x27;
pub const VK_RSHIFT: u8 = 0xA1;
pub const VK_SHIFT: u8 = 0x10;
pub const VK_SPACE: u8 = 0x20;
pub const VK_SUBTRACT: u8 = 0x6D;
pub const VK_TAB: u8 = 0x09;
pub const VK_UP: u8 = 0x26;

pub const MB_LEFT: u8 = 0x00;
pub const MB_RIGHT: u8 = 0x01;
pub const MB_MIDDLE: u8 = 0x02;

const KEY_COUNT: usize = 124;
const MOUSE_BUTTON_COUNT: usize = 3;

use std::convert::identity;

pub struct InputManager {
    // Keyboard
    kb_held: [bool; KEY_COUNT],
    kb_pressed: [bool; KEY_COUNT],
    kb_released: [bool; KEY_COUNT],
    kb_lshift: bool,
    kb_rshift: bool,
    kb_lctrl: bool,
    kb_rctrl: bool,
    kb_lalt: bool,
    kb_ralt: bool,
    numlock: bool,

    // Mouse
    mouse_x: f64,
    mouse_y: f64,
    mouse_held: [bool; MOUSE_BUTTON_COUNT],
    mouse_pressed: [bool; MOUSE_BUTTON_COUNT],
    mouse_released: [bool; MOUSE_BUTTON_COUNT],
    mouse_scroll_up: bool,
    mouse_scroll_down: bool,
}

impl InputManager {
    pub const fn new() -> Self {
        Self {
            kb_held: [false; KEY_COUNT],
            kb_pressed: [false; KEY_COUNT],
            kb_released: [false; KEY_COUNT],
            kb_lshift: false,
            kb_rshift: false,
            kb_lctrl: false,
            kb_rctrl: false,
            kb_lalt: false,
            kb_ralt: false,
            numlock: false,
            mouse_x: 0.0,
            mouse_y: 0.0,
            mouse_held: [false; MOUSE_BUTTON_COUNT],
            mouse_pressed: [false; MOUSE_BUTTON_COUNT],
            mouse_released: [false; MOUSE_BUTTON_COUNT],
            mouse_scroll_up: false,
            mouse_scroll_down: false,
        }
    }

    /// Informs the input manager that a key has been pressed
    pub fn key_press(&mut self, key: Key) {
        // self.kb_handle_direct(key, true);
        let code = key as usize;
        if code < KEY_COUNT {
            if !self.kb_held[code] {
                self.kb_held[code] = true;
                self.kb_pressed[code] = true;
            }
        }
    }

    /// Informs the input manager that a key has been released
    pub fn key_release(&mut self, key: Key) {
        // self.kb_handle_direct(key, false);
        let code = key as usize;
        if code < KEY_COUNT {
            if self.kb_held[code] {
                self.kb_held[code] = false;
                self.kb_released[code] = true;
            }
        }
    }

    /// Checks if a key was pressed on this frame, similar to GM8's keyboard_check_pressed()
    pub fn key_check(&self, code: usize) -> bool {
        self.kb_held.get(code).copied().unwrap_or(false)
    }

    /// Checks if a key was pressed on this frame, similar to GM8's keyboard_check_pressed()
    pub fn key_check_pressed(&self, code: usize) -> bool {
        self.kb_pressed.get(code).copied().unwrap_or(false)
    }

    /// Checks if a key was pressed on this frame, similar to GM8's keyboard_check_pressed()
    pub fn key_check_released(&self, code: usize) -> bool {
        self.kb_released.get(code).copied().unwrap_or(false)
    }

    /// Checks if left shift is pressed (for compat with keyboard_check_direct)
    pub fn key_check_lshift(&self) -> bool {
        self.kb_lshift
    }

    /// Checks if right shift is pressed (for compat with keyboard_check_direct)
    pub fn key_check_rshift(&self) -> bool {
        self.kb_rshift
    }

    /// Checks if left control is pressed (for compat with keyboard_check_direct)
    pub fn key_check_lctrl(&self) -> bool {
        self.kb_lctrl
    }

    /// Checks if right control is pressed (for compat with keyboard_check_direct)
    pub fn key_check_rctrl(&self) -> bool {
        self.kb_rctrl
    }

    /// Checks if left alt is pressed (for compat with keyboard_check_direct)
    pub fn key_check_lalt(&self) -> bool {
        self.kb_lalt
    }

    /// Checks if right alt is pressed (for compat with keyboard_check_direct)
    pub fn key_check_ralt(&self) -> bool {
        self.kb_ralt
    }

    /// Checks if any keyboard key is held
    pub fn key_check_any(&self) -> bool {
        self.kb_held.iter().copied().any(identity)
    }

    /// Checks if any keyboard key was pressed
    pub fn key_check_any_pressed(&self) -> bool {
        self.kb_pressed.iter().copied().any(identity)
    }

    /// Checks if any keyboard key was released
    pub fn key_check_any_released(&self) -> bool {
        self.kb_released.iter().copied().any(identity)
    }

    /// Checks if the spoofed numlock is pressed
    pub fn key_get_numlock(&self) -> bool {
        self.numlock
    }

    /// Checks if the spoofed numlock is pressed
    pub fn key_set_numlock(&mut self, value: bool) {
        self.numlock = value
    }

    /// Updates the position of the mouse. Coordinates are relative to the top-left of the window
    /// and are measured in absolute screen pixels, ie. not scaled to window size.
    pub fn set_mouse_pos(&mut self, x: f64, y: f64) {
        self.mouse_x = x;
        self.mouse_y = y;
    }

    /// Informs the input manager that a mouse button has been pressed
    pub fn mouse_press(&mut self, button: MouseButton) {
        let code = button as usize;
        if code < MOUSE_BUTTON_COUNT {
            self.mouse_pressed[code] = true;
            self.mouse_held[code] = true;
        }
    }

    /// Informs the input manager that a mouse button has been released
    pub fn mouse_release(&mut self, button: MouseButton) {
        let code = button as usize;
        if code < MOUSE_BUTTON_COUNT {
            self.mouse_released[code] = true;
            self.mouse_held[code] = false;
        }
    }

    /// Informs the input manager that the mouse wheel was scrolled up
    pub fn mouse_scroll_up(&mut self) {
        self.mouse_scroll_up = true;
    }

    /// Informs the input manager that the mouse wheel was scrolled down
    pub fn mouse_scroll_down(&mut self) {
        self.mouse_scroll_down = true;
    }

    /// Gets the position of the mouse. Coordinates are relative to the top-left of the window
    /// and are measured in absolute screen pixels, ie. not scaled to window size.
    pub fn mouse_get_location(&self) -> (f64, f64) {
        (self.mouse_x, self.mouse_y)
    }

    /// Checks if a mouse button is currently held
    pub fn mouse_check(&self, button: MouseButton) -> bool {
        self.mouse_held.get(button as usize).copied().unwrap_or(false)
    }

    /// Checks if a mouse button is currently held
    pub fn mouse_check_pressed(&self, button: MouseButton) -> bool {
        self.mouse_pressed.get(button as usize).copied().unwrap_or(false)
    }

    /// Checks if a mouse button is currently held
    pub fn mouse_check_released(&self, button: MouseButton) -> bool {
        self.mouse_released.get(button as usize).copied().unwrap_or(false)
    }

    /// Checks if the mouse wheel was scrolled up on this frame
    pub fn mouse_check_scroll_up(&self) -> bool {
        self.mouse_scroll_up
    }

    /// Checks if the mouse wheel was scrolled down on this frame
    pub fn mouse_check_scroll_down(&self) -> bool {
        self.mouse_scroll_down
    }

    /// Checks if any mouse button is held
    pub fn mouse_check_any(&self) -> bool {
        self.mouse_held.iter().copied().any(identity)
    }

    /// Checks if any mouse button was pressed
    pub fn mouse_check_any_pressed(&self) -> bool {
        self.mouse_pressed.iter().copied().any(identity)
    }

    /// Checks if any mouse button is held
    pub fn mouse_check_any_released(&self) -> bool {
        self.mouse_released.iter().copied().any(identity)
    }

    /// Clears the stored buffers of pressed and released keys and mouse buttons, but not the "currently held" ones.
    /// Should be called in between each frame.
    pub fn clear_presses(&mut self) {
        self.kb_pressed.iter_mut().for_each(|x| *x = false);
        self.kb_released.iter_mut().for_each(|x| *x = false);
        self.mouse_pressed.iter_mut().for_each(|x| *x = false);
        self.mouse_released.iter_mut().for_each(|x| *x = false);
        self.mouse_scroll_up = false;
        self.mouse_scroll_down = false;
    }

    // fn kb_handle_direct(&mut self, key: VirtualKeyCode, held: bool) {
    //     match key {
    //         VirtualKeyCode::LShift => self.kb_lshift = held,
    //         VirtualKeyCode::RShift => self.kb_rshift = held,
    //         VirtualKeyCode::LControl => self.kb_lctrl = held,
    //         VirtualKeyCode::RControl => self.kb_rctrl = held,
    //         VirtualKeyCode::LAlt => self.kb_lalt = held,
    //         VirtualKeyCode::RAlt => self.kb_ralt = held,
    //         _ => (),
    //     }
    // }
}

// TODO: VK_ANYKEY, VK_NOKEY, VK_LALT, VK_RALT...

#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
#[rustfmt::skip]
pub enum Key {
    Add = VK_ADD,
    Alt = VK_ALT,
    Backspace = VK_BACKSPACE,
    Control = VK_CONTROL,
    Decimal = VK_DECIMAL,
    Delete = VK_DELETE,
    Divide = VK_DIVIDE,
    Down = VK_DOWN,
    End = VK_END,
    Enter = VK_ENTER,
    Escape = VK_ESCAPE,
    F1 = VK_F1,
    F2 = VK_F2,
    F3 = VK_F3,
    F4 = VK_F4,
    F5 = VK_F5,
    F6 = VK_F6,
    F7 = VK_F7,
    F8 = VK_F8,
    F9 = VK_F9,
    F10 = VK_F10,
    F11 = VK_F11,
    F12 = VK_F12,
    Home = VK_HOME,
    Insert = VK_INSERT,
    LeftControl = VK_LCONTROL,
    Left = VK_LEFT,
    LeftShift = VK_LSHIFT,
    Multiply = VK_MULTIPLY,
    Numpad0 = VK_NUMPAD0,
    Numpad1 = VK_NUMPAD1,
    Numpad2 = VK_NUMPAD2,
    Numpad3 = VK_NUMPAD3,
    Numpad4 = VK_NUMPAD4,
    Numpad5 = VK_NUMPAD5,
    Numpad6 = VK_NUMPAD6,
    Numpad7 = VK_NUMPAD7,
    Numpad8 = VK_NUMPAD8,
    Numpad9 = VK_NUMPAD9,
    PageDown = VK_PAGEDOWN,
    PageUp = VK_PAGEUP,
    Pause = VK_PAUSE,
    PrintScreen = VK_PRINTSCREEN,
    RightControl = VK_RCONTROL,
    Right = VK_RIGHT,
    RightShift = VK_RSHIFT,
    Shift = VK_SHIFT,
    Space = VK_SPACE,
    Subtract = VK_SUBTRACT,
    Tab = VK_TAB,
    Up = VK_UP,

    A = b'A', B = b'B', C = b'C', D = b'D', E = b'E', F = b'F', G = b'G', H = b'H', I = b'I',
    J = b'J', K = b'K', L = b'L', M = b'M', N = b'N', O = b'O', P = b'P', Q = b'Q', R = b'R',
    S = b'S', T = b'T', U = b'U', V = b'V', W = b'W', X = b'X', Y = b'Y', Z = b'Z',
}

impl Key {
    #[rustfmt::skip]
    pub fn from_winapi(vk: u8) -> Option<Self> {
        Some(match vk {
            VK_ADD => Self::Add,
            VK_ALT => Self::Alt,
            VK_BACKSPACE => Self::Backspace,
            VK_CONTROL => Self::Control,
            VK_DECIMAL => Self::Decimal,
            VK_DELETE => Self::Delete,
            VK_DIVIDE => Self::Divide,
            VK_DOWN => Self::Down,
            VK_END => Self::End,
            VK_ENTER => Self::Enter,
            VK_ESCAPE => Self::Escape,
            VK_F1 => Self::F1,
            VK_F2 => Self::F2,
            VK_F3 => Self::F3,
            VK_F4 => Self::F4,
            VK_F5 => Self::F5,
            VK_F6 => Self::F6,
            VK_F7 => Self::F7,
            VK_F8 => Self::F8,
            VK_F9 => Self::F9,
            VK_F10 => Self::F10,
            VK_F11 => Self::F11,
            VK_F12 => Self::F12,
            VK_HOME => Self::Home,
            VK_INSERT => Self::Insert,
            VK_LCONTROL => Self::LeftControl,
            VK_LEFT => Self::Left,
            VK_LSHIFT => Self::LeftShift,
            VK_MULTIPLY => Self::Multiply,
            VK_NUMPAD0 => Self::Numpad0,
            VK_NUMPAD1 => Self::Numpad1,
            VK_NUMPAD2 => Self::Numpad2,
            VK_NUMPAD3 => Self::Numpad3,
            VK_NUMPAD4 => Self::Numpad4,
            VK_NUMPAD5 => Self::Numpad5,
            VK_NUMPAD6 => Self::Numpad6,
            VK_NUMPAD7 => Self::Numpad7,
            VK_NUMPAD8 => Self::Numpad8,
            VK_NUMPAD9 => Self::Numpad9,
            VK_PAGEDOWN => Self::PageDown,
            VK_PAGEUP => Self::PageUp,
            VK_PAUSE => Self::Pause,
            VK_PRINTSCREEN => Self::PrintScreen,
            VK_RCONTROL => Self::RightControl,
            VK_RIGHT => Self::Right,
            VK_RSHIFT => Self::RightShift,
            VK_SHIFT => Self::Shift,
            VK_SPACE => Self::Space,
            VK_SUBTRACT => Self::Subtract,
            VK_TAB => Self::Tab,
            VK_UP => Self::Up,

            b'A' => Key::A, b'B' => Key::B, b'C' => Key::C, b'D' => Key::D, b'E' => Key::E,
            b'F' => Key::F, b'G' => Key::G, b'H' => Key::H, b'I' => Key::I, b'J' => Key::J,
            b'K' => Key::K, b'L' => Key::L, b'M' => Key::M, b'N' => Key::N, b'O' => Key::O,
            b'P' => Key::P, b'Q' => Key::Q, b'R' => Key::R, b'S' => Key::S, b'T' => Key::T,
            b'U' => Key::U, b'V' => Key::V, b'W' => Key::W, b'X' => Key::X, b'Y' => Key::Y,
            b'Z' => Key::Z,

            _ => return None,
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
#[rustfmt::skip]
pub enum MouseButton {
    Left = 1,
    Right = 2,
    Middle = 3,
}

impl MouseButton {
    pub fn from_winapi(mb: u8) -> Option<Self> {
        Some(match mb {
            MB_LEFT => Self::Left,
            MB_RIGHT => Self::Right,
            MB_MIDDLE => Self::Middle,

            _ => return None,
        })
    }
}
