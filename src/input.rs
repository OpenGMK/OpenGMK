use winit::event::VirtualKeyCode;

const KEY_COUNT: usize = 124;
const MOUSE_BUTTON_COUNT: usize = 3;

pub const VK_ADD: u8 = 0x6B;
pub const VK_ALT: u8 = 0x12; // VK_MENU
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
pub const VK_PAGEDOWN: u8 = 0x22; // VK_NEXT
pub const VK_PAGEUP: u8 = 0x21; // VK_PRIOR
pub const VK_PAUSE: u8 = 0x13;
pub const VK_PRINTSCREEN: u8 = 0x2C; // VK_SNAPSHOT
pub const VK_RCONTROL: u8 = 0xA3;
pub const VK_RETURN: u8 = 0x0D;
pub const VK_RIGHT: u8 = 0x27;
pub const VK_RSHIFT: u8 = 0xA1;
pub const VK_SHIFT: u8 = 0x10;
pub const VK_SPACE: u8 = 0x20;
pub const VK_SUBTRACT: u8 = 0x6D;
pub const VK_TAB: u8 = 0x09;
pub const VK_UP: u8 = 0x26;

pub const MB_LEFT: usize = 0;
pub const MB_RIGHT: usize = 1;
pub const MB_MIDDLE: usize = 2;

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
    pub fn new() -> Self {
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
    pub fn key_press(&mut self, key: VirtualKeyCode) {
        self.kb_handle_direct(key, true);
        if let Some(code) = Self::kb_map_code(key) {
            if !self.kb_held[code] {
                self.kb_held[code] = true;
                self.kb_pressed[code] = true;
            }
        }
    }

    /// Informs the input manager that a key has been released
    pub fn key_release(&mut self, key: VirtualKeyCode) {
        self.kb_handle_direct(key, false);
        if let Some(code) = Self::kb_map_code(key) {
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
        self.kb_held.iter().copied().any(|x| x)
    }

    /// Checks if any keyboard key was pressed
    pub fn key_check_any_pressed(&self) -> bool {
        self.kb_pressed.iter().copied().any(|x| x)
    }

    /// Checks if any keyboard key was released
    pub fn key_check_any_released(&self) -> bool {
        self.kb_released.iter().copied().any(|x| x)
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
    pub fn mouse_press(&mut self, button: usize) {
        self.mouse_pressed[button] = true;
        self.mouse_held[button] = true;
    }

    /// Informs the input manager that a mouse button has been released
    pub fn mouse_release(&mut self, button: usize) {
        self.mouse_released[button] = true;
        self.mouse_held[button] = false;
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
    pub fn mouse_check(&self, button: usize) -> bool {
        self.mouse_held.get(button).copied().unwrap_or(false)
    }

    /// Checks if a mouse button is currently held
    pub fn mouse_check_pressed(&self, button: usize) -> bool {
        self.mouse_pressed.get(button).copied().unwrap_or(false)
    }

    /// Checks if a mouse button is currently held
    pub fn mouse_check_released(&self, button: usize) -> bool {
        self.mouse_released.get(button).copied().unwrap_or(false)
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
        self.mouse_held.iter().copied().any(|x| x)
    }

    /// Checks if any mouse button was pressed
    pub fn mouse_check_any_pressed(&self) -> bool {
        self.mouse_pressed.iter().copied().any(|x| x)
    }

    /// Checks if any mouse button is held
    pub fn mouse_check_any_released(&self) -> bool {
        self.mouse_released.iter().copied().any(|x| x)
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

    fn kb_handle_direct(&mut self, key: VirtualKeyCode, held: bool) {
        match key {
            VirtualKeyCode::LShift => self.kb_lshift = held,
            VirtualKeyCode::RShift => self.kb_rshift = held,
            VirtualKeyCode::LControl => self.kb_lctrl = held,
            VirtualKeyCode::RControl => self.kb_rctrl = held,
            VirtualKeyCode::LAlt => self.kb_lalt = held,
            VirtualKeyCode::RAlt => self.kb_ralt = held,
            _ => (),
        }
    }

    fn kb_map_code(key: VirtualKeyCode) -> Option<usize> {
        match key {
            VirtualKeyCode::Back => Some(8), // backspace
            VirtualKeyCode::Tab => Some(9),
            VirtualKeyCode::Return => Some(13),
            VirtualKeyCode::LShift | VirtualKeyCode::RShift => Some(16),
            VirtualKeyCode::LControl | VirtualKeyCode::RControl => Some(17),
            VirtualKeyCode::LAlt | VirtualKeyCode::RAlt => Some(18),
            VirtualKeyCode::Pause => Some(19),
            VirtualKeyCode::Escape => Some(27),
            VirtualKeyCode::Space => Some(32),
            VirtualKeyCode::PageUp => Some(33),
            VirtualKeyCode::PageDown => Some(34),
            VirtualKeyCode::End => Some(35),
            VirtualKeyCode::Home => Some(36),
            VirtualKeyCode::Left => Some(37),
            VirtualKeyCode::Up => Some(38),
            VirtualKeyCode::Right => Some(39),
            VirtualKeyCode::Down => Some(40),
            VirtualKeyCode::Snapshot => Some(44), // printscreen key
            VirtualKeyCode::Insert => Some(45),
            VirtualKeyCode::Delete => Some(46),
            VirtualKeyCode::A => Some(65),
            VirtualKeyCode::B => Some(66),
            VirtualKeyCode::C => Some(67),
            VirtualKeyCode::D => Some(68),
            VirtualKeyCode::E => Some(69),
            VirtualKeyCode::F => Some(70),
            VirtualKeyCode::G => Some(71),
            VirtualKeyCode::H => Some(72),
            VirtualKeyCode::I => Some(73),
            VirtualKeyCode::J => Some(74),
            VirtualKeyCode::K => Some(75),
            VirtualKeyCode::L => Some(76),
            VirtualKeyCode::M => Some(77),
            VirtualKeyCode::N => Some(78),
            VirtualKeyCode::O => Some(79),
            VirtualKeyCode::P => Some(80),
            VirtualKeyCode::Q => Some(81),
            VirtualKeyCode::R => Some(82),
            VirtualKeyCode::S => Some(83),
            VirtualKeyCode::T => Some(84),
            VirtualKeyCode::U => Some(85),
            VirtualKeyCode::V => Some(86),
            VirtualKeyCode::W => Some(87),
            VirtualKeyCode::X => Some(88),
            VirtualKeyCode::Y => Some(89),
            VirtualKeyCode::Z => Some(90),
            VirtualKeyCode::Numpad0 => Some(96),
            VirtualKeyCode::Numpad1 => Some(97),
            VirtualKeyCode::Numpad2 => Some(98),
            VirtualKeyCode::Numpad3 => Some(99),
            VirtualKeyCode::Numpad4 => Some(100),
            VirtualKeyCode::Numpad5 => Some(101),
            VirtualKeyCode::Numpad6 => Some(102),
            VirtualKeyCode::Numpad7 => Some(103),
            VirtualKeyCode::Numpad8 => Some(104),
            VirtualKeyCode::Numpad9 => Some(105),
            VirtualKeyCode::Multiply => Some(106),
            VirtualKeyCode::Add => Some(107),
            VirtualKeyCode::Subtract => Some(109),
            VirtualKeyCode::Decimal => Some(110),
            VirtualKeyCode::Divide => Some(111),
            VirtualKeyCode::F1 => Some(112),
            VirtualKeyCode::F2 => Some(113),
            VirtualKeyCode::F3 => Some(114),
            VirtualKeyCode::F4 => Some(115),
            VirtualKeyCode::F5 => Some(116),
            VirtualKeyCode::F6 => Some(117),
            VirtualKeyCode::F7 => Some(118),
            VirtualKeyCode::F8 => Some(119),
            VirtualKeyCode::F9 => Some(120),
            VirtualKeyCode::F10 => Some(121),
            VirtualKeyCode::F11 => Some(122),
            VirtualKeyCode::F12 => Some(123),
            _ => None,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum Key {
    Add = VK_ADD,
    Alt = VK_ALT,
    // anykey
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
}

impl Key {
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
            _ => return None,
        })
    }
}

// TODO: VK_ANYKEY, VK_NOKEY, VK_LALT, VK_RALT...
