use crate::{
    imgui,
    game::{
        Game,
        replay::Replay,
        savestate::{self, SaveState},
        recording::{
            KeyState,
            window::{Window, DisplayInformation},
        },
    },
    render::RendererState,
    types::Colour,
};
use std::path::PathBuf;

// Savestates window
pub struct SaveStateWindow {
    capacity: usize,
    save_text: Vec<String>,
    load_text: Vec<String>,
    select_text: Vec<String>,
}

impl Window for SaveStateWindow {
    fn show_window(&mut self, info: &mut DisplayInformation) {
        info.frame.setup_next_window(imgui::Vec2(306.0, 8.0), Some(imgui::Vec2(225.0, 330.0)), None);
        info.frame.begin_window("Savestates", None, true, false, None);
        let rect_size = imgui::Vec2(info.frame.window_size().0, 24.0);
        let pos = info.frame.window_position() + info.frame.content_position() - imgui::Vec2(8.0, 8.0);
        for i in 0..(self.capacity/2) {
            let min = imgui::Vec2(0.0, ((i * 2 + 1) * 24) as f32);
            info.frame.rect(min + pos, min + rect_size + pos, Colour::new(1.0, 1.0, 1.0), 15);
        }
        for i in 0..self.capacity {
            unsafe {
                cimgui_sys::igPushStyleColorVec4(cimgui_sys::ImGuiCol__ImGuiCol_Button as _, cimgui_sys::ImVec4 { x: 0.98, y: 0.59, z: 0.26, w: 0.4 });
                cimgui_sys::igPushStyleColorVec4(cimgui_sys::ImGuiCol__ImGuiCol_ButtonHovered as _, cimgui_sys::ImVec4 { x: 0.98, y: 0.59, z: 0.26, w: 1.0 });
                cimgui_sys::igPushStyleColorVec4(cimgui_sys::ImGuiCol__ImGuiCol_ButtonActive as _, cimgui_sys::ImVec4 { x: 0.98, y: 0.53, z: 0.06, w: 1.0 });
            }
            let y = (24 * i + 21) as f32;
            if i == info.config.quicksave_slot {
                let min = imgui::Vec2(0.0, (i * 24) as f32);
                info.frame.rect(min + pos, min + rect_size + pos, Colour::new(0.1, 0.4, 0.2), 255);
            }
            if info.frame.button(&self.save_text[i], imgui::Vec2(60.0, 20.0), Some(imgui::Vec2(4.0, y))) && *info.game_running {
                if let Some(err) = self.save_savestate(&info.save_paths[i], i, info.game, info.replay, info.renderer_state, info.save_buffer) {
                    *info.err_string = Some(err);
                }
            }
            unsafe {
                cimgui_sys::igPopStyleColor(3);
            }

            if info.save_paths[i].exists() {
                if info.frame.button(&self.load_text[i], imgui::Vec2(60.0, 20.0), Some(imgui::Vec2(75.0, y))) && *info.startup_successful {
                    match self.load_savestate(info.game, &info.save_paths[i], info.save_buffer) {
                        Ok((new_replay, new_renderer_state)) => {
                            *info.replay = new_replay;
                            *info.renderer_state = new_renderer_state;

                            for (i, state) in info.keyboard_state.iter_mut().enumerate() {
                                *state = if info.game.input.keyboard_check_direct(i as u8) { KeyState::Held } else { KeyState::Neutral };
                            }
                            for (i, state) in info.mouse_state.iter_mut().enumerate() {
                                *state = if info.game.input.mouse_check_button(i as i8 + 1) { KeyState::Held } else { KeyState::Neutral };
                            }
                
                            // todo: find a better way to share these
                            //frame_text = format!("Frame: {}", replay.frame_count());
                            //seed_text = format!("Seed: {}", game.rand.seed());
                            *info.context_menu = None;
                            *info.new_rand = None;
                            *info.new_mouse_pos = None;
                            *info.err_string = None;
                            *info.game_running = true;
                            info.config.rerecords += 1;
                            //rerecord_text = format!("Re-record count: {}", config.rerecords);
                            info.config.save();

                            info.update_instance_reports();
                        },
                        Err(err) => {
                            *info.err_string = Some(err);
                        }
                    }
                }

                if info.frame.button(&self.select_text[i], imgui::Vec2(60.0, 20.0), Some(imgui::Vec2(146.0, y))) && info.config.quicksave_slot != i {
                    match SaveState::from_file(&info.save_paths[i], info.save_buffer) {
                        Ok(state) => {
                            *info.savestate = state;
                            info.config.quicksave_slot = i;
                            info.config.save();
                        }
                        Err(e) => {
                            println!(
                                "Error: Failed to select quicksave slot {:?}. {:?}",
                                info.save_paths[i].file_name(),
                                e
                            );
                        }
                    }
                }
            }
        }
        info.frame.end();
    }
}

impl SaveStateWindow {
    pub fn new(capacity: usize) -> Self {
        SaveStateWindow {
            capacity: capacity,
            save_text: (0..capacity).map(|i| format!("Save {}", i + 1)).collect::<Vec<_>>(),
            load_text: (0..capacity).map(|i| format!("Load {}", i + 1)).collect::<Vec<_>>(),
            select_text: (0..capacity).map(|i| format!("Select###Select{}", i + 1)).collect::<Vec<_>>(),
        }
    }

    // todo: this probably shouldn't be done in the window, or at least have a way that other windows can access.
    fn save_savestate(&self, path: &PathBuf, index: usize, game: &mut Game, replay: &Replay, renderer_state: &RendererState, save_buffer: &mut savestate::Buffer)
    -> Option<String> {
        match SaveState::from(game, replay.clone(), renderer_state.clone())
        .save_to_file(&path, save_buffer)
        {
            Ok(()) => None,
            Err(savestate::WriteError::IOErr(err)) =>
                Some(format!("Failed to write savestate #{}: {}", index, err)),
            Err(savestate::WriteError::CompressErr(err)) =>
                Some(format!("Failed to compress savestate #{}: {}", index, err)),
            Err(savestate::WriteError::SerializeErr(err)) =>
                Some(format!("Failed to serialize savestate #{}: {}", index, err)),
        }
    }
    
    // todo: this probably shouldn't be done in the window, or at least have a way that other windows can access.
    fn load_savestate(&self, game: &mut Game, path: &PathBuf, save_buffer: &mut savestate::Buffer) -> Result<(Replay, RendererState), String> {
        match SaveState::from_file(path, save_buffer) {
            Ok(state) => {
                let (new_replay, new_renderer_state) = state.load_into(game);
                Ok((new_replay, new_renderer_state))
            },
            Err(err) => {
                let filename = path.to_string_lossy();
                Err(match err {
                    savestate::ReadError::IOErr(err) =>
                        format!("Error reading {}:\n\n{}", filename, err),
                    savestate::ReadError::DecompressErr(err) =>
                        format!("Error decompressing {}:\n\n{}", filename, err),
                    savestate::ReadError::DeserializeErr(err) =>
                        format!("Error deserializing {}:\n\n{}", filename, err),
                })
            },
        }
    }
}
