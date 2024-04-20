use crate::{
    imgui,
    game::{
        Game,
        recording::{WindowKind, KeyState, ProjectConfig, instance_report::InstanceReport, keybinds::{Keybindings, Binding}, popup_dialog::Dialog},
        replay::{Replay, FrameRng},
        savestate::{self, SaveState},
    },
    render::RendererState,
};
use std::path::PathBuf;

pub struct DisplayInformation<'a, 'f> {
    pub game: &'a mut Game,
    pub frame: &'a mut imgui::Frame<'f>,
    pub game_running: &'a mut bool,
    pub setting_mouse_pos: &'a mut bool,
    pub new_mouse_pos: &'a mut Option<(i32, i32)>,
    pub new_rand: &'a mut Option<FrameRng>,
    pub config: &'a mut ProjectConfig,
    pub err_string: &'a mut Option<String>,
    pub replay: &'a mut Replay,

    pub keyboard_state: &'a mut [KeyState; 256],
    pub mouse_state: &'a mut [KeyState; 3],
    pub savestate: &'a mut SaveState,
    pub renderer_state: &'a mut RendererState,
    pub save_buffer: &'a mut savestate::Buffer,
    pub instance_reports: &'a mut Vec<(i32, Option<InstanceReport>)>,

    pub clean_state: &'a mut bool,
    pub run_until_frame: &'a mut Option<usize>,
    
    pub save_paths: &'a Vec<PathBuf>,
    pub fps_text: &'a String,
    pub ui_renderer_state: &'a RendererState,
    pub startup_successful: &'a bool,
    pub project_path: &'a PathBuf,

    pub win_padding: imgui::Vec2<f32>,
    pub win_frame_height: f32,
    pub win_border_size: f32,

    pub keybindings: &'a mut Keybindings,

    // These probably shouldn't be pub. I just don't know how to initialize the struct otherwise.
    pub _clear_context_menu: bool,
    pub _request_context_menu: bool,
    pub _context_menu_requested: bool,
    pub _modal_dialog: Option<&'static str>,
}

pub trait WindowType {
    fn window_type_self(&self) -> std::any::TypeId;
}
pub trait Window: WindowType {
    fn show_window(&mut self, info: &mut DisplayInformation);

    fn is_open(&self) -> bool;

    fn name(&self) -> String;

    fn window_id(&self) -> usize { 0 }
    
    /// Returns the WindowType that is stored in the config. If it returns None it will not be stored in the config and won't automatically open on startup.
    fn stored_kind(&self) -> Option<WindowKind> { None }

    /// Displays the context menu. Returns false if the context menu is not open anymore.
    fn show_context_menu(&mut self, _info: &mut DisplayInformation) -> bool { false }

    /// Runs when an open context menu on this window is closed.
    fn context_menu_close(&mut self) { }

    /// Handles potential modal windows that can be opened from this window. Returns true if any of the modal windows are currently open, false otherwise
    fn handle_modal(&mut self, _info: &mut DisplayInformation) -> bool { false }
}
impl<T> WindowType for T
    where T: Window + 'static
{
    fn window_type_self(&self) -> std::any::TypeId {
        std::any::TypeId::of::<T>()
    }
}

pub trait Openable<T>: Window
    where T: 'static
{
    fn window_type() -> std::any::TypeId {
        std::any::TypeId::of::<T>()
    }

    fn window_name() -> &'static str;

    fn open(id: usize) -> Self;
}

impl DisplayInformation<'_, '_> {
    pub fn update_instance_reports(&mut self) {
        *self.instance_reports = self.config.watched_ids.iter().map(|id| (*id, InstanceReport::new(self.game, *id))).collect();
    }

    pub fn keybind_pressed(&self, binding: Binding) -> bool {
        self.keybindings.keybind_pressed(binding, self.frame)
    }

    pub fn savestate_exists(&mut self, slot: usize) -> bool {
        slot < self.save_paths.len() && self.save_paths[slot].exists()
    }

    pub fn savestate_save(&mut self, slot: usize) -> bool {
        if slot >= self.save_paths.len() {
            false
        } else {
            let mut savestate_replay = self.replay.clone();

            // make sure the saved replay is only up to the savestate.
            savestate_replay.truncate_frames(self.config.current_frame);

            let state = SaveState::from(self.game, savestate_replay, self.renderer_state.clone(), *self.clean_state);
            let result = self.savestate_save_to_file(slot, &state);
            if slot == self.config.quicksave_slot {
                *self.savestate = state;
            }
            result
        }
    }

    pub fn savestate_load(&mut self, slot: usize) -> bool {
        *self.run_until_frame = None;
        if slot == self.config.quicksave_slot {
            self.savestate_load_from_state(self.savestate.clone());
            true
        } else {
            self.savestate_load_from_slot(slot)
        }
    }

    pub fn savestate_set_quicksave_slot(&mut self, slot: usize) -> bool {
        if let Some(state) = self.savestate_from_slot(slot) {
            *self.savestate = state;
            self.config.quicksave_slot = slot;
            self.config.save();
            true
        } else {
            false
        }
    }

    fn savestate_save_to_file(&mut self, slot: usize, state: &SaveState) -> bool {
        let path = &self.save_paths[slot];
        match state.save_to_file(&path, self.save_buffer)
        {
            Ok(()) => true,
            Err(err) => {
                *self.err_string = Some(match err {
                    savestate::WriteError::IOErr(err) =>
                        format!("Failed to write savestate #{}: {}", slot, err),
                    savestate::WriteError::CompressErr(err) =>
                        format!("Failed to compress savestate #{}: {}", slot, err),
                    savestate::WriteError::SerializeErr(err) =>
                        format!("Failed to serialize savestate #{}: {}", slot, err),
                });
                false
            }
        }
    }

    fn savestate_from_slot(&mut self, slot: usize) -> Option<SaveState> {
        if slot < self.save_paths.len() {
            let path = &self.save_paths[slot];
            if !path.exists() {
                return None;
            }

            match SaveState::from_file(&path, self.save_buffer) {
                Ok(state) => {
                    Some(state)
                },
                Err(err) => {
                    let filename = path.to_string_lossy();
                    *self.err_string = Some(match err {
                        savestate::ReadError::IOErr(err) =>
                            format!("Error reading {}:\n\n{}", filename, err),
                        savestate::ReadError::DecompressErr(err) =>
                            format!("Error decompressing {}:\n\n{}", filename, err),
                        savestate::ReadError::DeserializeErr(err) =>
                            format!("Error deserializing {}:\n\n{}", filename, err),
                    });
                    None
                },
            }
        } else {
            None
        }
    }
    
    fn savestate_load_from_slot(&mut self, slot: usize) -> bool {
        if let Some(state) = self.savestate_from_slot(slot) {
            self.savestate_load_from_state(state);
            true
        } else {
            false
        }
    }

    fn savestate_load_from_state(&mut self, state: SaveState) {
        *self.clean_state = state.clean_state;
        let (new_replay, new_renderer_state) = state.load_into(self.game);
        *self.renderer_state = new_renderer_state;

        for (i, state) in self.keyboard_state.iter_mut().enumerate() {
            *state =
                if self.game.input.keyboard_check_direct(i as u8) { KeyState::Held } else { KeyState::Neutral };
        }
        for (i, state) in self.mouse_state.iter_mut().enumerate() {
            *state =
                if self.game.input.mouse_check_button(i as i8 + 1) { KeyState::Held } else { KeyState::Neutral };
        }

        self.clear_context_menu();
        *self.new_rand = None;
        *self.new_mouse_pos = None;
        *self.err_string = None;
        *self.game_running = true;

        self.config.current_frame = new_replay.frame_count();

        if self.config.is_read_only {
            if !self.replay.contains_part(&new_replay) {
                *self.err_string = Some("Savestate is not part of recording.\nPlease load a savestate that's part of the current recording or try again in Read/Write mode.".into());
                *self.game_running = false;
            }
        } else {
            *self.replay = new_replay;
        }
        self.config.rerecords += 1;
        self.config.save();

        self.update_instance_reports();
    }

    pub fn clear_context_menu(&mut self) {
        self._clear_context_menu = true;
    }

    pub fn context_menu_clear_requested(&self) -> bool {
        self._clear_context_menu
    }

    pub fn request_context_menu(&mut self) -> bool {
        if !self._clear_context_menu && !self._context_menu_requested {
            self._request_context_menu = true;
            // don't allow another context menu to be requested in the same frame
            self._context_menu_requested = true;
            true
        } else {
            false
        }
    }

    pub fn context_menu_requested(&self) -> bool {
        self._request_context_menu
    }

    pub fn reset_context_menu_state(&mut self, clear_context_menu: bool) {
        self._clear_context_menu = clear_context_menu;
        self._request_context_menu = false;
    }

    pub fn request_modal(&mut self, modal: &mut dyn Dialog)  {
        modal.reset();
        self._modal_dialog = Some(modal.get_name());
    }
}
