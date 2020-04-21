const KEY_COUNT: usize = 124;
const MOUSE_BUTTON_COUNT: usize = 3;

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
    pub fn key_press(&mut self, scancode: u32) {
        self.kb_held[scancode as usize] = true;
        self.kb_pressed[scancode as usize] = true;
    }

    /// Informs the input manager that a key has been released
    pub fn key_release(&mut self, scancode: u32) {
        self.kb_held[scancode as usize] = false;
        self.kb_released[scancode as usize] = true;
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

    /// Checks if a key was pressed on this frame, similar to GM8's keyboard_check_pressed()
    pub fn key_check(&self, scancode: u32) -> bool {
        self.kb_held.get(scancode as usize).copied().unwrap_or(false)
    }

    /// Checks if a key was pressed on this frame, similar to GM8's keyboard_check_pressed()
    pub fn key_check_pressed(&self, scancode: u32) -> bool {
        self.kb_pressed.get(scancode as usize).copied().unwrap_or(false)
    }

    /// Checks if a key was pressed on this frame, similar to GM8's keyboard_check_pressed()
    pub fn key_check_released(&self, scancode: u32) -> bool {
        self.kb_released.get(scancode as usize).copied().unwrap_or(false)
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
    pub fn mouse_location(&self) -> (f64, f64) {
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
}
