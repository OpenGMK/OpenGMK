use crate::{
    imgui,
    input::Button,
};
use std::{
    convert::From,
    default::Default,
    collections::HashMap,
};

#[derive(Eq, PartialEq, Hash)]
pub enum Binding {
    Advance,
    Quicksave,
    Quickload,
}

pub struct KeyCombination {
    ctrl: bool,
    alt: bool,
    shift: bool,
    keycodes: Vec<u8>,
}
pub struct Keybindings {
    bindings: HashMap<Binding, KeyCombination>,
}

impl Keybindings {
    pub fn keybind_pressed(&self, bind: Binding, frame: &imgui::Frame) -> bool {
        match self.bindings.get(&bind) {
            Some(keys) => keys.pressed(frame),
            None => false,
        }
    }

    pub fn update_binding(&mut self, bind: Binding, keys: KeyCombination) {
        self.bindings.insert(bind, keys);
    }
}

impl Default for Keybindings {
    fn default() -> Self {
        let mut bindings = HashMap::new();
        bindings.insert(Binding::Advance, KeyCombination::from(vec![Button::Space]));
        bindings.insert(Binding::Quickload, KeyCombination::from(vec![Button::W]));
        bindings.insert(Binding::Quicksave, KeyCombination::from(vec![Button::Q]));

        Self {
            bindings
        }
    }
}

impl KeyCombination {
    pub fn pressed(&self, frame: &imgui::Frame) -> bool {
        let mut pressed = false;

        if frame.ctrl_down() == self.ctrl
            && frame.alt_down() == self.alt
            && frame.shift_down() == self.shift
        {
            if self.keycodes.len() == 1 {
                pressed = frame.key_pressed(self.keycodes[0]);
            } else {
                for (i, key) in self.keycodes.iter().enumerate() {
                    if i == self.keycodes.len()-1 {
                        // check if the final key was just pressed
                        // todo: check last key instead of no repeat?
                        pressed = frame.key_pressed_norepeat(*key);
                    } else if !frame.key_down(*key) {
                        // and break if any of the other keys aren't.
                        break;
                    }
                }
            }
        }

        pressed
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
                    b => Some(*b as u8),
                }
            }).collect(),
        }
    }
}
