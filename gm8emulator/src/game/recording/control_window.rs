use crate::{
    imgui, input,
    game::{
        Game,
        SceneChange,
        recording::{
            KeyState,
            InputMode,
            ContextMenu,
            window::{ Window, DisplayInformation, },
        },
        replay::{self, Frame},
    },
    types::Colour
};
use ramen::event::Key;
use std::time::Duration;

pub struct ControlWindow {
    frame_text: String,
    seed_text: String,
    rerecord_text: String,
}

impl Window for ControlWindow {
    fn show_window(&mut self, info: &mut DisplayInformation) {
        info.frame.setup_next_window(imgui::Vec2(8.0, 8.0), None, None);
        info.frame.begin_window("Control", None, true, false, None);

        self.update_texts(info);

        if (
            info.frame.button("Advance (Space)", imgui::Vec2(165.0, 20.0), None) ||
                info.frame.key_pressed(input::ramen2vk(Key::Space))
        ) && *info.game_running && info.err_string.is_none() {
            self.advance_frame(info);
        }

        if (info.frame.button("Quick Save (Q)", imgui::Vec2(165.0, 20.0), None) || info.frame.key_pressed(input::ramen2vk(Key::Q))) && *info.game_running && info.err_string.is_none() {
            info.savestate_save(info.config.quicksave_slot);
        }

        if info.frame.button("Load Quicksave (W)", imgui::Vec2(165.0, 20.0), None) || info.frame.key_pressed(input::ramen2vk(Key::W)) {
            if *info.startup_successful {
                info.savestate_load(info.config.quicksave_slot);
            }
        }

        if info.frame.button("Export to .gmtas", imgui::Vec2(165.0, 20.0), None) {
            let mut filepath = info.project_path.clone();
            filepath.push("save.gmtas");
            match info.replay.to_file(&filepath) {
                Ok(()) => (),
                Err(replay::WriteError::IOErr(err)) =>
                    *info.err_string = Some(format!("Failed to write save.gmtas: {}", err)),
                Err(replay::WriteError::CompressErr(err)) =>
                    *info.err_string = Some(format!("Failed to compress save.gmtas: {}", err)),
                Err(replay::WriteError::SerializeErr(err)) =>
                    *info.err_string = Some(format!("Failed to serialize save.gmtas: {}", err)),
            }
        }
        
        info.frame.text(&self.frame_text);
        if info.new_rand.is_some() {
            info.frame.coloured_text(&self.seed_text, Colour::new(1.0, 0.5, 0.5));
        } else {
            info.frame.text(&self.seed_text);
        }
        info.frame.text(&self.rerecord_text);
        info.frame.text(&info.fps_text);

        let keyboard_label = if info.config.full_keyboard {
            "Simple Keyboard###KeyboardLayout"
        } else {
            "Full Keyboard###KeyboardLayout"
        };
        if info.frame.button(keyboard_label, imgui::Vec2(165.0, 20.0), None) {
            info.config.full_keyboard = !info.config.full_keyboard;
            info.config.save();
        }

        let input_label = match info.config.input_mode {
            InputMode::Direct => "Switch to mouse input###InputMethod",
            InputMode::Mouse => "Switch to direct input###InputMethod",
        };
        if info.frame.button(input_label, imgui::Vec2(165.0, 20.0), None) {
            info.config.input_mode = match info.config.input_mode {
                InputMode::Mouse => InputMode::Direct,
                InputMode::Direct => InputMode::Mouse,
            }
        }

        if info.frame.button(">", imgui::Vec2(18.0, 18.0), Some(imgui::Vec2(160.0, 138.0))) {
            if let Some(rand) = &mut info.new_rand {
                rand.cycle();
            } else {
                let mut rand = info.game.rand.clone();
                rand.cycle();
                *info.new_rand = Some(rand);
            }
        }
        if info.frame.item_hovered() && info.frame.right_clicked() {
            *info.context_menu = Some(ContextMenu::Seed { pos: info.frame.mouse_pos() });
        }
        info.frame.end();
    }
}

impl ControlWindow {
    pub fn new() -> Self {
        ControlWindow {
            frame_text: format!("Frame: {}", 0),
            rerecord_text: format!("Re-Records: {}", 0),
            seed_text: format!("Seed: {}", 0),
        }
    }

    fn update_texts(&mut self, info: &mut DisplayInformation) {
        self.frame_text = format!("Frame: {}", info.replay.frame_count());
        self.rerecord_text = format!("Re-Records: {}", info.config.rerecords);
        if let Some(rand) = info.new_rand {
            self.seed_text = format!("Seed: {}", rand.seed())
        } else {
            self.seed_text = format!("Seed: {}", info.game.rand.seed());
        }
    }

    fn advance_frame(&mut self, info: &mut DisplayInformation) {
        let frame = info.replay.new_frame();

        info.game.input.mouse_step();
        self.update_keyboard_state(info.keyboard_state, info.game, frame);
        self.update_mouse_state(info.mouse_state, info.game, frame);

        if let Some((x, y)) = *info.new_mouse_pos {
            frame.mouse_x = x;
            frame.mouse_y = y;
            info.game.input.mouse_move_to((x, y));
        }

        if let Some(rand) = &*info.new_rand {
            frame.new_seed = Some(rand.seed());
            info.game.rand.set_seed(rand.seed());
        }

        if let Some(error) = self.run_frame(info.game, info.renderer_state) {
            *info.err_string = Some(error);
            *info.game_running = false;
        }

        for ev in info.game.stored_events.iter() {
            frame.events.push(ev.clone());
        }
        info.game.stored_events.clear();
        for (i, state) in info.keyboard_state.iter_mut().enumerate() {
            state.reset_to(info.game.input.keyboard_check_direct(i as u8));
        }
        for (i, state) in info.mouse_state.iter_mut().enumerate() {
            state.reset_to(info.game.input.mouse_check_button(i as i8 + 1));
        }

        // Fake frame limiter stuff (don't actually frame-limit in record mode)
        if let Some(t) = info.game.spoofed_time_nanos.as_mut() {
            *t += Duration::new(0, 1_000_000_000u32 / info.game.room.speed).as_nanos();
        }
        if info.game.frame_counter == info.game.room.speed {
            info.game.fps = info.game.room.speed;
            info.game.frame_counter = 0;
        }
        info.game.frame_counter += 1;

        info.game.renderer.resize_framebuffer(info.config.ui_width.into(), info.config.ui_height.into(), true);
        info.game.renderer.set_view( 0, 0, info.config.ui_width.into(), info.config.ui_height.into(),
            0.0, 0, 0, info.config.ui_width.into(), info.config.ui_height.into());
        info.game.renderer.clear_view(crate::game::recording::CLEAR_COLOUR, 1.0);
        *info.renderer_state = info.game.renderer.state();
        info.game.renderer.set_state(info.ui_renderer_state);
        *info.context_menu = None;
        *info.new_rand = None;
        *info.new_mouse_pos = None;

        info.update_instance_reports();
    }

    fn update_keyboard_state(&self, keyboard_state: &mut [KeyState; 256], game: &mut Game, frame: &mut Frame) {
        for (i, state) in keyboard_state.iter().enumerate() {
            let i = i as u8;
            match state {
                KeyState::NeutralWillPress => {
                    game.input.button_press(i, true);
                    frame.inputs.push(replay::Input::KeyPress(i));
                },
                KeyState::NeutralWillDouble | KeyState::NeutralDoubleEveryFrame => {
                    game.input.button_press(i, true);
                    game.input.button_release(i, true);
                    frame.inputs.push(replay::Input::KeyPress(i));
                    frame.inputs.push(replay::Input::KeyRelease(i));
                },
                KeyState::NeutralWillTriple => {
                    game.input.button_press(i, true);
                    game.input.button_release(i, true);
                    game.input.button_press(i, true);
                    frame.inputs.push(replay::Input::KeyPress(i));
                    frame.inputs.push(replay::Input::KeyRelease(i));
                    frame.inputs.push(replay::Input::KeyPress(i));
                },
                KeyState::HeldWillRelease | KeyState::NeutralWillCactus => {
                    game.input.button_release(i, true);
                    frame.inputs.push(replay::Input::KeyRelease(i));
                },
                KeyState::HeldWillDouble | KeyState::HeldDoubleEveryFrame => {
                    game.input.button_release(i, true);
                    game.input.button_press(i, true);
                    frame.inputs.push(replay::Input::KeyRelease(i));
                    frame.inputs.push(replay::Input::KeyPress(i));
                },
                KeyState::HeldWillTriple => {
                    game.input.button_release(i, true);
                    game.input.button_press(i, true);
                    game.input.button_release(i, true);
                    frame.inputs.push(replay::Input::KeyRelease(i));
                    frame.inputs.push(replay::Input::KeyPress(i));
                    frame.inputs.push(replay::Input::KeyRelease(i));
                },
                KeyState::Neutral | KeyState::Held => (),
            }
        }
    }

    fn update_mouse_state(&self, mouse_state: &mut [KeyState; 3], game: &mut Game, frame: &mut Frame) {
        for (i, state) in mouse_state.iter().enumerate() {
            let i = i as i8 + 1;
            match state {
                KeyState::NeutralWillPress => {
                    game.input.mouse_press(i, true);
                    frame.inputs.push(replay::Input::MousePress(i));
                    println!("Pressed {}", i);
                },
                KeyState::NeutralWillDouble | KeyState::NeutralDoubleEveryFrame => {
                    game.input.mouse_press(i, true);
                    game.input.mouse_release(i, true);
                    frame.inputs.push(replay::Input::MousePress(i));
                    frame.inputs.push(replay::Input::MouseRelease(i));
                },
                KeyState::NeutralWillTriple => {
                    game.input.mouse_press(i, true);
                    game.input.mouse_release(i, true);
                    game.input.mouse_press(i, true);
                    frame.inputs.push(replay::Input::MousePress(i));
                    frame.inputs.push(replay::Input::MouseRelease(i));
                    frame.inputs.push(replay::Input::MousePress(i));
                },
                KeyState::HeldWillRelease | KeyState::NeutralWillCactus => {
                    game.input.mouse_release(i, true);
                    frame.inputs.push(replay::Input::MouseRelease(i));
                    println!("Released {}", i);
                },
                KeyState::HeldWillDouble | KeyState::HeldDoubleEveryFrame => {
                    game.input.mouse_release(i, true);
                    game.input.mouse_press(i, true);
                    frame.inputs.push(replay::Input::MouseRelease(i));
                    frame.inputs.push(replay::Input::MousePress(i));
                },
                KeyState::HeldWillTriple => {
                    game.input.mouse_release(i, true);
                    game.input.mouse_press(i, true);
                    game.input.mouse_release(i, true);
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
        game.renderer.set_view(0, 0, game.unscaled_width as _, game.unscaled_height as _,
            0.0, 0, 0, game.unscaled_width as _, game.unscaled_height as _);
        game.renderer.draw_stored(0, 0, w, h);
        if let Err(e) = match game.frame() {
            Ok(()) => {
                match game.scene_change {
                    Some(SceneChange::Room(id)) => game.load_room(id),
                    Some(SceneChange::Restart) => game.restart(),
                    Some(SceneChange::End) => game.restart(),
                    None => Ok(()),
                }
            },
            Err(e) => Err(e.into()),
        } {
            Some(format!("Game crashed: {}\n\nPlease load a savestate.", e))
        } else {
            None
        }
    }
}
