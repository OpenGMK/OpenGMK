use crate::{ 
    imgui,  game::{
        Game,
        recording::{KeyState, ContextMenu, ProjectConfig, instance_report::InstanceReport, keybinds::{Keybindings, Binding}},
        replay::Replay,
        savestate::{self, SaveState},
    },
    gml::rand::Random,
    render::RendererState,
};
use std::path::PathBuf;

 pub struct DisplayInformation<'a, 'f> {
    pub game: &'a mut Game,
    pub frame: &'a mut imgui::Frame<'f>,
    pub context_menu: &'a mut Option<ContextMenu>,
    pub game_running: &'a mut bool,
    pub setting_mouse_pos: &'a mut bool,
    pub new_mouse_pos: &'a mut Option<(i32, i32)>,
    pub new_rand: &'a mut Option<Random>,
    pub config: &'a mut ProjectConfig,
    pub err_string: &'a mut Option<String>,
    pub replay: &'a mut Replay,

    pub keyboard_state: &'a mut [KeyState; 256],
    pub mouse_state: &'a mut [KeyState; 3],
    pub savestate: &'a mut SaveState,
    pub renderer_state: &'a mut RendererState,
    pub save_buffer: &'a mut savestate::Buffer,
    pub instance_reports: &'a mut Vec<(i32, Option<InstanceReport>)>,
    
    pub save_paths: &'a Vec<PathBuf>,
    pub fps_text: &'a String,
    pub ui_renderer_state: &'a RendererState,
    pub startup_successful: &'a bool,
    pub project_path: &'a PathBuf,

    pub win_padding: imgui::Vec2<f32>,
    pub win_frame_height: f32,
    pub win_border_size: f32,

    pub keybindings: &'a mut Keybindings,
}

pub trait Window {
    fn show_window(&mut self, info: &mut DisplayInformation);

    fn is_open(&self) -> bool;
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

            let state = SaveState::from(self.game, savestate_replay, self.renderer_state.clone());
            let result = self.savestate_save_to_file(slot, &state);
            if slot == self.config.quicksave_slot {
                *self.savestate = state;
            }
            result
        }
    }

    pub fn savestate_load(&mut self, slot: usize) -> bool {
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

        *self.context_menu = None;
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
}
