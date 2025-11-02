use crate::{
    game::recording::window::{
        EmulatorContext,
        Openable,
        Window}, imgui_key_utils::{
        ImGuiKeyDef,
        ImguiKeyCustomFunctions
    }, imgui_utils::{
        TableColumnSetupCustomFunction,
        UiCustomFunction,
        Vec2
    }
};
use imgui::{TableColumnFlags, TableColumnSetup, TableFlags};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
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
            Self::Advance        => Some(KeyCombination::from(&vec![imgui::Key::Space])),
            Self::Quickload      => Some(KeyCombination::from(&vec![imgui::Key::W])),
            Self::Quicksave      => Some(KeyCombination::from(&vec![imgui::Key::Q])),
            Self::SelectNext     => Some(KeyCombination::from(&vec![imgui::Key::ModShift, imgui::Key::Apostrophe])),
            Self::SelectPrevious => Some(KeyCombination::from(&vec![imgui::Key::ModShift, imgui::Key::Minus])),
            Self::ToggleReadOnly => Some(KeyCombination::from(&vec![imgui::Key::ModShift, imgui::Key::Alpha8])),
            Self::ToggleDirect   => Some(KeyCombination::from(&vec![imgui::Key::ModCtrl,  imgui::Key::D])),
            Self::ToggleKeyboard => Some(KeyCombination::from(&vec![imgui::Key::ModCtrl,  imgui::Key::K])),
            Self::NextRand       => Some(KeyCombination::from(&vec![imgui::Key::ModCtrl,  imgui::Key::R])),
            Self::ExportGmtas    => Some(KeyCombination::from(&vec![imgui::Key::ModCtrl,  imgui::Key::ModShift, imgui::Key::E])),
            Self::ToggleMacros   => Some(KeyCombination::from(&vec![imgui::Key::ModCtrl,  imgui::Key::Alpha1])),
            Self::SetMouse       => Some(KeyCombination::from(&vec![imgui::Key::ModCtrl,  imgui::Key::M])),
            //_ => None,
        }
    }
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyCombination {
    ctrl: bool,
    alt: bool,
    shift: bool,
    #[serde_as(as = "Vec<ImGuiKeyDef>")]
    keycodes: Vec<imgui::Key>,
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

        for (index, key) in self.keycodes.iter().enumerate() {
            key.fmt(f)?;
            if index != self.keycodes.len()-1 {
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
    last_keycodes: Vec<imgui::Key>,
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

    pub fn keybind_pressed(&self, bind: Binding, frame: &imgui::Ui) -> bool {
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

    fn show_window(&mut self, info: &mut EmulatorContext) {
        unsafe { imgui::sys::igPushStyleVar_Vec2(imgui::sys::ImGuiStyleVar_WindowPadding.try_into().unwrap(), Vec2(0.0, 4.0).into()); }
        let mut is_open = self.is_open;
        info.frame
            .window("Keybindings")
            .opened(&mut is_open)
            .build(|| {
                if let Some(table) = info.frame.begin_table_header_with_flags(
                    "Keybindings",
                    [
                        TableColumnSetup::new("Action"),
                        TableColumnSetup::new("Keybind"),
                        TableColumnSetup::with_flags_and_init_width_or_weight("Set", TableColumnFlags::WIDTH_FIXED, Self::CONTROL_BUTTON_WIDTH),
                        TableColumnSetup::with_flags_and_init_width_or_weight("Default", TableColumnFlags::WIDTH_FIXED, Self::CONTROL_BUTTON_WIDTH)
                    ],
                    TableFlags::ROW_BG | TableFlags::BORDERS
                ) {
                    for (binding, keys) in &mut info.keybindings.bindings {
                        self.binding_entry(binding, keys, info.frame);
                    }

                    table.end();
                }
            });
        self.is_open = is_open;
        unsafe { imgui::sys::igPopStyleVar(1); }

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
    const CANCEL_COLOR: [f32; 4] = [0.5, 0.6, 0.7, 1.0];
    const CONTROL_BUTTON_WIDTH: f32 = 60.0;
    const CONTROL_BUTTON_HEIGHT: f32 = 20.0;

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

    fn binding_entry(&mut self, binding: &Binding, keys: &mut Option<KeyCombination>, frame: &imgui::Ui) {
        frame.table_next_column();
        frame.text(format!("{}", binding));

        frame.table_next_column();
        let is_setting_binding = matches!(self.current_binding, Some(_));
        let text = if is_setting_binding && *binding == self.current_binding.unwrap() {
            frame.text_colored(Self::CANCEL_COLOR, self.current_keys.to_string());
            "Cancel"
        } else {
            let name = if let Some(keycombination) = keys { keycombination.to_string() } else { String::from("Not set") };
            frame.text(name);
            "Set"
        };

        frame.table_next_column();
        if frame.button_with_size(format!("{}###{}", text, binding), [Self::CONTROL_BUTTON_WIDTH, Self::CONTROL_BUTTON_HEIGHT]) {
            if !is_setting_binding {
                self.current_binding = Some(*binding);
                self.last_keycodes.clear();
                self.current_keys = KeyCombination::from(&self.last_keycodes);
            } else {
                self.current_binding = None;
            }
        }

        frame.table_next_column();
        if frame.button_with_size(format!("Default###default{}", binding), [Self::CONTROL_BUTTON_WIDTH, Self::CONTROL_BUTTON_HEIGHT]) {
            *keys = binding.default_binding();
            self.current_binding = None;
        }
    }

    fn update_current_keys(&mut self, frame: &imgui::Ui, bindings: &mut Keybindings) {
        if frame.is_key_pressed(imgui::Key::Escape) {
            bindings.update_binding(self.current_binding.unwrap(), None);
            self.current_binding = None;
        } else {
            let keys = frame.get_held_keys(false, true);
            if keys.len() == 0 {
                // if we have pressed keys before, update the keybinding and the key combination is valid
                if self.last_keycodes.len() != 0 && self.current_keys.is_valid() {
                    bindings.update_binding(self.current_binding.unwrap(), Some(self.current_keys.clone()));
                    self.current_binding = None;
                }
            } else {
                // figure out which keys have been newly pressed
                let new_keys: Vec<imgui::Key> = keys.iter().filter_map(|key| {
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
    pub fn pressed(&self, frame: &imgui::Ui) -> bool {
        let mut pressed = false;

        // only check if a button is pressed if that button is required. Ignore additionally pressed modifier keys.
        if !(self.ctrl && !frame.io().key_ctrl)
            && !(self.alt && !frame.io().key_alt)
            && !(self.shift && !frame.io().key_shift)
        {
            if self.keycodes.len() == 1 {
                pressed = frame.is_key_index_pressed(self.keycodes[0] as _);
            } else {
                for (i, key) in self.keycodes.iter().enumerate() {
                    if i == self.keycodes.len()-1 {
                        // check if the final key was just pressed
                        // todo: check last key instead of no repeat?
                        pressed = frame.is_key_index_pressed_no_repeat(*key as _);
                    } else if !frame.is_key_index_down(*key as _) {
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

impl From<&Vec<imgui::Key>> for KeyCombination {
    fn from(keys: &Vec<imgui::Key>) -> Self {
        let ctrl  = keys.iter().any(|b| match b {imgui::Key::LeftCtrl  | imgui::Key::RightCtrl  | imgui::Key::ModCtrl  | imgui::Key::ReservedForModAlt    => true, _ => false});
        let shift = keys.iter().any(|b| match b {imgui::Key::LeftShift | imgui::Key::RightShift | imgui::Key::ModShift | imgui::Key::ReservedForModShift  => true, _ => false});
        let alt   = keys.iter().any(|b| match b {imgui::Key::LeftAlt   | imgui::Key::RightAlt   | imgui::Key::ModAlt   | imgui::Key::ReservedForModAlt    => true, _ => false});

        Self {
            ctrl,
            shift,
            alt,
            keycodes: keys.iter().filter_map(|button| {
                match button {
                       imgui::Key::LeftCtrl  | imgui::Key::RightCtrl  | imgui::Key::ModCtrl  | imgui::Key::ReservedForModCtrl
                     | imgui::Key::LeftShift | imgui::Key::RightShift | imgui::Key::ModShift | imgui::Key::ReservedForModShift
                     | imgui::Key::LeftAlt   | imgui::Key::RightAlt   | imgui::Key::ModAlt   | imgui::Key::ReservedForModAlt
                    => None,
                    b => Some(*b),
                }
            }).collect(),
        }
    }
}
