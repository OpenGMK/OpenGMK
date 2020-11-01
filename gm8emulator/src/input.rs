use serde::{Deserialize, Serialize};
use shared::input::{Key, MouseButton};
use std::convert::identity;

const KEY_COUNT: usize = 256;
const MOUSE_BUTTON_COUNT: usize = 3;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InputManager {
    // Keyboard
    kb_held: BoolMap,
    kb_pressed: BoolMap,
    kb_released: BoolMap,
    kb_map: KeyMap,
    kb_key: u32,
    kb_lastkey: u32,
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
    mouse_x_previous: f64,
    mouse_y_previous: f64,
    mouse_held: BoolMap,
    mouse_pressed: BoolMap,
    mouse_released: BoolMap,
    mouse_scroll_up: bool,
    mouse_scroll_down: bool,
    mouse_button: u32,
    mouse_lastbutton: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct BoolMap(Vec<bool>);

impl BoolMap {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn with_capacity(cap: usize) -> Self {
        Self(Vec::with_capacity(cap))
    }

    pub fn get(&self, index: usize) -> bool {
        self.0.get(index).copied().unwrap_or(false)
    }

    pub fn set(&mut self, index: usize, value: bool) {
        match self.0.get_mut(index) {
            Some(b) => *b = value,
            None => {
                while self.0.len() < index {
                    self.0.push(false);
                }
                self.0.push(value);
            },
        }
    }

    pub fn any(&self) -> bool {
        self.0.iter().copied().any(identity)
    }

    pub fn clear(&mut self) {
        self.0.iter_mut().for_each(|b| *b = false)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct KeyMap(Vec<usize>);

impl KeyMap {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn with_capacity(cap: usize) -> Self {
        Self(Vec::with_capacity(cap))
    }

    pub fn get(&self, index: usize) -> usize {
        self.0.get(index).copied().unwrap_or(index)
    }

    pub fn set(&mut self, index: usize, value: usize) {
        match self.0.get_mut(index) {
            Some(k) => *k = value,
            None => {
                for i in self.0.len()..index {
                    self.0.push(i);
                }
                self.0.push(value);
            },
        }
    }

    pub fn unset_all(&mut self) {
        self.0.iter_mut().enumerate().for_each(|(i, k)| *k = i);
    }
}

impl InputManager {
    pub fn new() -> Self {
        Self {
            kb_held: BoolMap::with_capacity(KEY_COUNT),
            kb_pressed: BoolMap::with_capacity(KEY_COUNT),
            kb_released: BoolMap::with_capacity(KEY_COUNT),
            kb_map: KeyMap::with_capacity(KEY_COUNT),
            kb_key: 0,
            kb_lastkey: 0,
            kb_lshift: false,
            kb_rshift: false,
            kb_lctrl: false,
            kb_rctrl: false,
            kb_lalt: false,
            kb_ralt: false,
            numlock: false,
            mouse_x: 0.0,
            mouse_y: 0.0,
            mouse_x_previous: 0.0,
            mouse_y_previous: 0.0,
            mouse_held: BoolMap::with_capacity(MOUSE_BUTTON_COUNT),
            mouse_pressed: BoolMap::with_capacity(MOUSE_BUTTON_COUNT),
            mouse_released: BoolMap::with_capacity(MOUSE_BUTTON_COUNT),
            mouse_scroll_up: false,
            mouse_scroll_down: false,
            mouse_button: 0,
            mouse_lastbutton: 0,
        }
    }

    /// Informs the input manager that a key has been pressed
    pub fn key_press(&mut self, key: Key) {
        // self.kb_handle_direct(key, true);
        let code = self.key_get_map(key as usize);
        if code < KEY_COUNT {
            if !self.kb_held.get(code) {
                self.kb_held.set(code, true);
                self.kb_pressed.set(code, true);
            }
            self.kb_key = code as u32;
            self.kb_lastkey = code as u32;
        }
    }

    /// Informs the input manager that a key has been released
    pub fn key_release(&mut self, key: Key) {
        // self.kb_handle_direct(key, false);
        let code = self.key_get_map(key as usize);
        if code < KEY_COUNT {
            if self.kb_held.get(code) {
                self.kb_held.set(code, false);
                self.kb_released.set(code, true);
            }
            if self.kb_key == code as u32 {
                self.kb_key = 0;
            }
        }
    }

    /// Clears a keypress from the input manager's internal state
    pub fn key_clear(&mut self, code: usize) {
        if code < KEY_COUNT {
            self.kb_held.set(code, false);
            self.kb_pressed.set(code, false);
            self.kb_released.set(code, false);
        }
    }

    /// Clears a mouse press from the input manager's internal state
    pub fn mouse_clear(&mut self, code: usize) {
        if code < MOUSE_BUTTON_COUNT {
            self.mouse_pressed.set(code, false);
            self.mouse_held.set(code, false);
            self.mouse_released.set(code, false);
        }
    }

    /// Checks if a key is currently held, similar to GM8's keyboard_check()
    pub fn key_check(&self, code: usize) -> bool {
        self.kb_held.get(code)
    }

    /// Checks if a key was pressed on this frame, similar to GM8's keyboard_check_pressed()
    pub fn key_check_pressed(&self, code: usize) -> bool {
        self.kb_pressed.get(code)
    }

    /// Checks if a key was released on this frame, similar to GM8's keyboard_check_released()
    pub fn key_check_released(&self, code: usize) -> bool {
        self.kb_released.get(code)
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

    /// Checks if any keyboard key is currently held
    pub fn key_check_any(&self) -> bool {
        self.kb_held.any()
    }

    /// Checks if any keyboard key was pressed on this frame
    pub fn key_check_any_pressed(&self) -> bool {
        self.kb_pressed.any()
    }

    /// Checks if any keyboard key was released on this frame
    pub fn key_check_any_released(&self) -> bool {
        self.kb_released.any()
    }

    /// Get the currently held key
    pub fn key_get_key(&self) -> u32 {
        self.kb_key
    }

    /// Get the last pressed key
    pub fn key_get_lastkey(&self) -> u32 {
        self.kb_lastkey
    }

    /// Sets the variable meant for the currently held key
    pub fn key_set_key(&mut self, key: u32) {
        if key < 0x100 {
            self.kb_key = key;
        }
    }

    /// Sets the variable meant for the last pressed key
    pub fn key_set_lastkey(&mut self, key: u32) {
        if key < 0x100 {
            self.kb_lastkey = key;
        }
    }

    /// Checks if the spoofed numlock is pressed
    pub fn key_get_numlock(&self) -> bool {
        self.numlock
    }

    /// Updates the spoofed numlock state
    pub fn key_set_numlock(&mut self, value: bool) {
        self.numlock = value
    }

    pub fn key_get_map(&self, code: usize) -> usize {
        self.kb_map.get(code)
    }

    pub fn key_set_map(&mut self, code: usize, value: usize) {
        if code < 0x100 {
            self.kb_map.set(code, value);
        }
    }

    pub fn key_unmap_all(&mut self) {
        self.kb_map.unset_all();
    }

    /// Updates the position of the mouse. Coordinates are relative to the top-left of the window
    /// and are measured in absolute screen pixels, i.e. not scaled to window size.
    pub fn set_mouse_pos(&mut self, x: f64, y: f64) {
        self.mouse_x = x;
        self.mouse_y = y;
    }

    /// Informs the input manager that a mouse button has been pressed
    pub fn mouse_press(&mut self, button: MouseButton) {
        let code = button as usize;
        if !self.mouse_held.get(code) {
            self.mouse_pressed.set(code, true);
            self.mouse_held.set(code, true);
        }
        self.mouse_button = code as u32;
        self.mouse_lastbutton = code as u32;
    }

    /// Informs the input manager that a mouse button has been released
    pub fn mouse_release(&mut self, button: MouseButton) {
        let code = button as usize;
        if self.mouse_held.get(code) {
            self.mouse_released.set(code, true);
            self.mouse_held.set(code, false);
        }
        self.mouse_button = 0;
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
    /// and are measured in absolute screen pixels, i.e. not scaled to window size.
    pub fn mouse_get_location(&self) -> (f64, f64) {
        (self.mouse_x, self.mouse_y)
    }

    /// Gets the previous position of the mouse. Coordinates are relative to the top-left of the window
    /// and are measured in absolute screen pixels, i.e. not scaled to window size. Will be (0, 0) on the first frame.
    pub fn mouse_get_previous_location(&self) -> (f64, f64) {
        (self.mouse_x_previous, self.mouse_y_previous)
    }

    /// Checks if a mouse button is currently held
    pub fn mouse_check(&self, button: MouseButton) -> bool {
        self.mouse_held.get(button as usize)
    }

    /// Checks if a mouse button was pressed on this frame
    pub fn mouse_check_pressed(&self, button: MouseButton) -> bool {
        self.mouse_pressed.get(button as usize)
    }

    /// Checks if a mouse button was released on this frame
    pub fn mouse_check_released(&self, button: MouseButton) -> bool {
        self.mouse_released.get(button as usize)
    }

    /// Checks if the mouse wheel was scrolled up on this frame
    pub fn mouse_check_scroll_up(&self) -> bool {
        self.mouse_scroll_up
    }

    /// Checks if the mouse wheel was scrolled down on this frame
    pub fn mouse_check_scroll_down(&self) -> bool {
        self.mouse_scroll_down
    }

    /// Checks if any mouse button is currently held
    pub fn mouse_check_any(&self) -> bool {
        self.mouse_held.any()
    }

    /// Checks if any mouse button was pressed on this frame
    pub fn mouse_check_any_pressed(&self) -> bool {
        self.mouse_pressed.any()
    }

    /// Checks if any mouse button was released on this frame
    pub fn mouse_check_any_released(&self) -> bool {
        self.mouse_released.any()
    }

    /// Gets the currently held mouse button
    pub fn mouse_get_button(&self) -> u32 {
        self.mouse_button
    }

    /// Gets the last pressed mouse button
    pub fn mouse_get_lastbutton(&self) -> u32 {
        self.mouse_lastbutton
    }

    pub fn mouse_set_button(&mut self, button: u32) {
        if button < 4 {
            self.mouse_button = button;
        }
    }

    pub fn mouse_set_lastbutton(&mut self, button: u32) {
        if button < 4 {
            self.mouse_lastbutton = button;
        }
    }

    /// Clears the stored buffers of pressed and released keys and mouse buttons, but not the "currently held" ones.
    /// Should be called in between each frame.
    pub fn clear_presses(&mut self) {
        self.kb_pressed.clear();
        self.kb_released.clear();
        self.mouse_pressed.clear();
        self.mouse_released.clear();
    }

    /// Updates previous mouse position to be the current one
    pub fn mouse_update_previous(&mut self) {
        self.mouse_x_previous = self.mouse_x;
        self.mouse_y_previous = self.mouse_y;
        self.mouse_scroll_up = false;
        self.mouse_scroll_down = false;
    }

    pub fn clear(&mut self) {
        self.kb_lastkey = 0;
        self.kb_key = 0;
        self.kb_held.clear();
        self.kb_pressed.clear();
        self.kb_released.clear();

        self.mouse_button = 0;
        self.mouse_lastbutton = 0;
        self.mouse_held.clear();
        self.mouse_pressed.clear();
        self.mouse_released.clear();
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
