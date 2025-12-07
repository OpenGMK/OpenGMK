use crate::{
    game::{
        recording::{
            keybinds::Binding,
            window::{EmulatorContext, Window},
            InputMode, KeyState,
        },
        replay::{self, Frame, FrameRng},
        Game, GameClock, SceneChange,
    },
    imgui_utils::{UiCustomFunction, Vec2},
};

use super::popup_dialog::{string_input::RNGSelect, Dialog, DialogState};

pub struct ControlWindow {
    seed_text: String,
    rerecord_text: String,
    rng_select: RNGSelect,
    seed_base: (i32, i32, i32),
}

impl Window for ControlWindow {
    fn name(&self) -> String {
        "Control".to_owned()
    }

    fn show_window(&mut self, info: &mut EmulatorContext) {
        info.frame.window(self.name()).position([8.0, 8.0], imgui::Condition::FirstUseEver).build(|| {
            self.update_texts(info);

            let run_until_frame = if let Some(frame) = *info.run_until_frame {
                if frame > info.config.current_frame {
                    true
                } else {
                    *info.run_until_frame = None;
                    false
                }
            } else {
                false
            };

            let content_width = info.frame.content_region_avail()[0];

            if (info.frame.button_with_size("Advance", [content_width, 20.0])
                || info.keybind_pressed(Binding::Advance)
                || run_until_frame
              ) && *info.game_running
                && info.err_string.is_none()
            {
                self.advance_frame(info);
            }

            if (info.frame.button_with_size("Quick Save", [content_width, 20.0])
                || info.keybind_pressed(Binding::Quicksave)
              ) && *info.game_running
                && info.err_string.is_none()
            {
                info.savestate_save(info.config.quicksave_slot);
            }

            if info.frame.button_with_size("Load Quicksave", [content_width, 20.0])
                || info.keybind_pressed(Binding::Quickload)
            {
                if *info.startup_successful {
                    info.savestate_load(info.config.quicksave_slot);
                }
            }

            if info.frame.button_with_size("Export to .gmtas", [content_width, 20.0])
                || info.keybind_pressed(Binding::ExportGmtas)
            {
                let mut filepath = info.project_path.clone();
                filepath.push("save.gmtas");
                match info.replay.to_file(&filepath) {
                    Ok(()) => (),
                    Err(replay::WriteError::IOErr(err)) => {
                        *info.err_string = Some(format!("Failed to write save.gmtas: {}", err))
                    },
                    Err(replay::WriteError::CompressErr(err)) => {
                        *info.err_string = Some(format!("Failed to compress save.gmtas: {}", err))
                    },
                    Err(replay::WriteError::SerializeErr(err)) => {
                        *info.err_string = Some(format!("Failed to serialize save.gmtas: {}", err))
                    },
                }
            }

            let frame_text = match info.config.is_read_only {
                true => format!("Frame: {}/{}", info.config.current_frame, info.replay.frame_count()),
                false => format!("Frame: {}", info.config.current_frame),
            };
            info.frame.text(&frame_text);

            if info.new_rand.is_some() {
                info.frame.text_colored([1.0, 0.5, 0.5, 1.0], &self.seed_text);
            } else {
                info.frame.text(&self.seed_text);
            }
            info.frame.text(&self.rerecord_text);
            info.frame.text(&info.fps_text);

            let keyboard_label = match info.config.full_keyboard {
                true => "Simple Keyboard###KeyboardLayout",
                false => "Full Keyboard###KeyboardLayout",
            };
            if info.frame.button_with_size(keyboard_label, [content_width, 20.0])
                || info.keybind_pressed(Binding::ToggleKeyboard)
            {
                info.config.full_keyboard = !info.config.full_keyboard;
                info.config.save();
            }

            let input_label = match info.config.input_mode {
                InputMode::Direct => "Switch to mouse input###InputMethod",
                InputMode::Mouse => "Switch to direct input###InputMethod",
            };
            if info.frame.button_with_size(input_label, [content_width, 20.0])
                || info.keybind_pressed(Binding::ToggleDirect)
            {
                info.config.input_mode = match info.config.input_mode {
                    InputMode::Mouse => InputMode::Direct,
                    InputMode::Direct => InputMode::Mouse,
                }
            }

            let read_only_label = match info.config.is_read_only {
                true => "Switch to Read/Write###IsReadOnly",
                false => "Switch to Read-only###IsReadOnly",
            };
            if info.frame.button_with_size(read_only_label, [content_width, 20.0])
                || info.keybind_pressed(Binding::ToggleReadOnly)
            {
                info.config.is_read_only = !info.config.is_read_only;
                info.config.save();
            }

            let mouse_set_label = match info.config.set_mouse_using_textbox {
                true => "Set Mouse: textbox###mouse_set_label",
                false => "Set Mouse: clicking###mouse_set_label",
            };
            if info.frame.button_with_size(mouse_set_label, [content_width, 20.0]) {
                info.config.set_mouse_using_textbox = !info.config.set_mouse_using_textbox;
                info.config.save();
            }

            if info.frame.button_with_size_and_pos(">", Vec2(18.0, 18.0), Vec2(content_width - 18.0, 138.0))
                || info.keybind_pressed(Binding::NextRand)
            {
                if let Some(rand) = &mut info.new_rand {
                    rand.increase();
                } else {
                    *info.new_rand = Some(FrameRng::Increment(1));
                }
            }

            if info.frame.is_item_hovered() && info.frame.is_mouse_clicked(imgui::MouseButton::Right) {
                info.request_context_menu();
            }
        });
    }

    fn is_open(&self) -> bool {
        true
    }

    fn show_context_menu(&mut self, info: &mut EmulatorContext) -> bool {
        let current_increment = if let Some(FrameRng::Increment(amount)) = info.new_rand { *amount } else { 0 };
        *info.new_rand = if info.new_rand.is_some() && info.frame.menu_item("Reset") {
            None
        } else if info.frame.menu_item("+1 RNG call") {
            Some(FrameRng::Increment(current_increment + 1))
        } else if info.frame.menu_item("+5 RNG calls") {
            Some(FrameRng::Increment(current_increment + 5))
        } else if info.frame.menu_item("+10 RNG calls") {
            Some(FrameRng::Increment(current_increment + 10))
        } else if info.frame.menu_item("+50 RNG calls") {
            Some(FrameRng::Increment(current_increment + 50))
        } else if info.frame.menu_item("Pick RNG") {
            info.request_modal(&mut self.rng_select);
            return false;
        } else {
            return true; // Keep context menu open if nothing was clicked
        };

        return false;
    }

    fn handle_modal(&mut self, info: &mut EmulatorContext) -> bool {
        match self.rng_select.show(info) {
            DialogState::Submit => {
                *info.new_rand = self.rng_select.get_result();
                false
            },
            DialogState::Open => true,
            _ => false,
        }
    }
}

impl ControlWindow {
    pub fn new() -> Self {
        ControlWindow {
            rerecord_text: format!("Re-Records: {}", 0),
            seed_text: format!("Seed: {}", 0),
            rng_select: RNGSelect::new("Pick RNG"),
            seed_base: (0, 0, 0), // Stores (base_seed, cycles, result_seed) to not have to re-calculate that every frame
        }
    }

    fn update_texts(&mut self, info: &mut EmulatorContext) {
        self.rerecord_text = format!("Re-Records: {}", info.config.rerecords);
        if let Some(rand) = info.new_rand {
            self.seed_text = match *rand {
                FrameRng::Increment(amount) => {
                    if self.seed_base.0 != info.game.rand.seed() || self.seed_base.1 != amount {
                        let mut rng = info.game.rand.clone();
                        for _ in 0..amount {
                            rng.cycle();
                        }
                        self.seed_base = (info.game.rand.seed(), amount, rng.seed());
                    }
                    format!("Seed: +{} ({})", self.seed_base.1, self.seed_base.2)
                },
                FrameRng::Override(new_seed) => {
                    if new_seed == info.game.rand.seed() {
                        *info.new_rand = None; // Unset new seed if the game's seed is already set to it
                    }
                    format!("Seed: {}", new_seed)
                },
            }
        } else {
            self.seed_text = format!("Seed: {}", info.game.rand.seed());
        }
    }

    fn advance_frame(&mut self, info: &mut EmulatorContext) {
        info.game.input.mouse_step();

        let frame: &mut Frame;
        let mut current_frame: Frame;

        if info.config.is_read_only && matches!(info.replay.get_frame(info.config.current_frame), Some(_)) {
            current_frame = info.replay.get_frame(info.config.current_frame).unwrap().clone();
            frame = &mut current_frame;
        } else {
            if info.config.is_read_only {
                // We're advancing at the end of the current replay while in read-only mode, possible options:
                // don't advance
                // switch to read/write
                // => add onto it but stay in read-only
                // TODO: make that a setting -- Probably the best option but for now I'll go with adding onto it
                // TODO: pause the playback once it reached the end in read-only mode whenever a real-time mode has been implemented.

                // Assert to make sure we're actually at the end of the replay.
                // Should always be true unless replay.get_frame() returned None in the middle of the replay
                //  or we're somehow in a state where the current frame is outside the bounds of the replay.
                // In both cases something is very much wrong.
                assert_eq!(info.config.current_frame, info.replay.frame_count());
                // info.config.is_read_only = false;
                // info.config.save();
            }

            // if we write a new frame in the middle of the recording, truncate all following frames
            if info.replay.frame_count() > info.config.current_frame {
                info.replay.truncate_frames(info.config.current_frame);
            }

            let new_frame = info.replay.new_frame();
            self.update_keyboard_state(info.keyboard_state, new_frame);
            self.update_mouse_state(info.mouse_state, new_frame);

            if let Some((x, y)) = *info.new_mouse_pos {
                new_frame.mouse_x = x;
                new_frame.mouse_y = y;
            }

            if let Some(rand) = &*info.new_rand {
                new_frame.new_seed = Some(rand.clone());
            }

            frame = new_frame;
        }

        info.game.set_input_from_frame(frame);

        if let Some(error) = self.run_frame(info.game, info.renderer_state) {
            *info.err_string = Some(error);
            *info.game_running = false;
        }

        info.config.current_frame += 1;

        if !info.config.is_read_only {
            for ev in info.game.stored_events.iter() {
                frame.events.push(ev.clone());
            }
        }
        info.game.stored_events.clear();
        for (i, state) in info.keyboard_state.iter_mut().enumerate() {
            state.reset_to(info.game.input.keyboard_check_direct(i as u8));
        }
        for (i, state) in info.mouse_state.iter_mut().enumerate() {
            state.reset_to(info.game.input.mouse_check_button(i as i8 + 1));
        }

        // Fake frame limiter stuff (don't actually frame-limit in record mode)
        if let GameClock::SpoofedNanos(t) = &mut info.game.clock {
            *t += 1_000_000_000 / info.game.room.speed as u128;
        }
        if info.game.frame_counter == info.game.room.speed {
            info.game.fps = info.game.room.speed;
            info.game.frame_counter = 0;
        }
        info.game.frame_counter += 1;

        info.game.renderer.resize_framebuffer(info.config.ui_width.into(), info.config.ui_height.into(), true);
        info.game.renderer.set_view(
            0, 0,
            info.config.ui_width.into(),
            info.config.ui_height.into(),
            0.0,
            0, 0,
            info.config.ui_width.into(),
            info.config.ui_height.into(),
        );
        info.game.renderer.clear_view(
            if *info.clean_state {
                crate::game::recording::CLEAR_COLOUR_GOOD
            } else {
                crate::game::recording::CLEAR_COLOUR_BAD
            },
            1.0,
        );
        *info.renderer_state = info.game.renderer.state();
        info.game.renderer.set_state(info.ui_renderer_state);
        info.clear_context_menu();
        *info.new_rand = None;
        *info.new_mouse_pos = None;

        info.update_instance_reports();
    }

    fn update_keyboard_state(&self, keyboard_state: &mut [KeyState; 256], frame: &mut Frame) {
        for (i, state) in keyboard_state.iter().enumerate() {
            let i = i as u8;
            state.push_key_inputs(i, &mut frame.inputs);
        }
    }

    fn update_mouse_state(&self, mouse_state: &mut [KeyState; 3], frame: &mut Frame) {
        for (i, state) in mouse_state.iter().enumerate() {
            let i = i as i8 + 1;
            match state {
                KeyState::NeutralWillPress => {
                    frame.inputs.push(replay::Input::MousePress(i));
                },
                KeyState::NeutralWillDouble | KeyState::NeutralDoubleEveryFrame => {
                    frame.inputs.push(replay::Input::MousePress(i));
                    frame.inputs.push(replay::Input::MouseRelease(i));
                },
                KeyState::NeutralWillTriple => {
                    frame.inputs.push(replay::Input::MousePress(i));
                    frame.inputs.push(replay::Input::MouseRelease(i));
                    frame.inputs.push(replay::Input::MousePress(i));
                },
                KeyState::HeldWillRelease | KeyState::NeutralWillCactus => {
                    frame.inputs.push(replay::Input::MouseRelease(i));
                },
                KeyState::HeldWillDouble | KeyState::HeldDoubleEveryFrame => {
                    frame.inputs.push(replay::Input::MouseRelease(i));
                    frame.inputs.push(replay::Input::MousePress(i));
                },
                KeyState::HeldWillTriple => {
                    frame.inputs.push(replay::Input::MouseRelease(i));
                    frame.inputs.push(replay::Input::MousePress(i));
                    frame.inputs.push(replay::Input::MouseRelease(i));
                },
                KeyState::Neutral | KeyState::Held => (),
            }
        }
    }

    /// runs a frame of the game
    /// if an error occured it will return a message, otherwise None
    fn run_frame(&self, game: &mut Game, renderer_state: &crate::render::RendererState) -> Option<String> {
        let (w, h) = game.renderer.stored_size();

        game.renderer.set_state(&renderer_state);
        game.renderer.resize_framebuffer(w, h, false);
        game.renderer.set_view(
            0,
            0,
            game.unscaled_width as _,
            game.unscaled_height as _,
            0.0,
            0,
            0,
            game.unscaled_width as _,
            game.unscaled_height as _,
        );
        game.renderer.draw_stored(0, 0, w, h);
        if let Err(e) = match game.frame() {
            Ok(()) => match game.scene_change {
                Some(SceneChange::Room(id)) => game.load_room(id),
                Some(SceneChange::Restart) => game.restart(),
                Some(SceneChange::End) => game.restart(),
                Some(SceneChange::Load(ref mut path)) => {
                    let path = std::mem::take(path);
                    game.load_gm_save(path)
                },
                None => Ok(()),
            },
            Err(e) => Err(e.into()),
        } {
            Some(format!("Game crashed: {}\n\nPlease load a savestate.", e))
        } else {
            None
        }
    }
}
