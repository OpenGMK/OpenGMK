use crate::{imgui, input, game::{Game, Replay, SaveState, SceneChange}, render::{atlas::AtlasRef, PrimitiveType, Renderer, RendererState}, types::Colour};
use ramen::{event::{Event, Key}, monitor::Size};
use std::{convert::TryFrom, path::PathBuf, time::{Duration, Instant}};

impl Game {
    pub fn record(&mut self, _project_path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let mut ui_width: u16 = 1280;
        let mut ui_height: u16 = 720;
        self.window.set_inner_size(Size::Physical(ui_width.into(), ui_height.into()));

        let mut replay = Replay::new(self.spoofed_time_nanos.unwrap_or(0), self.rand.seed());

        let clear_colour = Colour::new(0.0196, 0.1059, 0.06275);

        let mut context = imgui::Context::new();
        context.make_current();
        let io = context.io();

        io.set_display_size(imgui::Vec2(f32::from(ui_width), f32::from(ui_height)));

        let imgui::FontData { data: fdata, size: (fwidth, fheight) } = io.font_data();
        let mut font = self.renderer.upload_sprite(fdata.into(), fwidth as _, fheight as _, 0, 0)?;
        io.set_texture_id((&mut font as *mut AtlasRef).cast());

        let grid = (0i32..(64 * 64 * 4)).map(|i| {
            let n = i >> 2;
            let x = n % 64;
            let y = n / 64;
            let a = (y - x).abs() == 32 || (y + x - 63).abs() == 32;
            let b = (y >= 34 && x + y == 97) || ((2..32).contains(&y) && x + y == 33);
            let c = (31..34).contains(&(y - x).abs()) || (31..34).contains(&(y + x - 63).abs());
            match (i & 1 != 0, i & 2 != 0) {
                (false, false) => u8::from(b) * 64,
                (true, false) => u8::from(a) * 128 + 64,
                (false, true) => if a { 99 } else { u8::from(b) * 34 + 33 },
                (true, true) => u8::from(b || c) * 255,
            }
        }).collect::<Vec<_>>().into_boxed_slice();
        let grid_ref = self.renderer.upload_sprite(grid, 64, 64, 0, 0)?;
        let grid_start = Instant::now();

        // for imgui callback
        struct GameViewData {
            renderer: *mut Renderer,
            x: i32,
            y: i32,
            w: u32,
            h: u32,
        }

        let mut savestate: Option<SaveState> = None;

        let ui_renderer_state = RendererState {
            model_matrix: self.renderer.get_model_matrix(),
            alpha_blending: true,
            blend_mode: self.renderer.get_blend_mode(),
            pixel_interpolation: true,
            texture_repeat: false,
            sprite_count: self.renderer.get_sprite_count(),
            vsync: false,
            ambient_colour: self.renderer.get_ambient_colour(),
            using_3d: false,
            depth: self.renderer.get_depth(),
            depth_test: false,
            write_depth: false,
            culling: false,
            perspective: false,
            fog: None,
            gouraud: false,
            lighting_enabled: false,
            lights: self.renderer.get_lights(),
            circle_precision: self.renderer.get_circle_precision(),
            primitive_2d: self.renderer.get_primitive_2d(),
            primitive_3d: self.renderer.get_primitive_3d(),
            zbuf_trashed: self.renderer.get_zbuf_trashed(),
        };

        self.init()?;
        match self.scene_change {
            Some(SceneChange::Room(id)) => self.load_room(id)?,
            Some(SceneChange::Restart) => self.restart()?,
            Some(SceneChange::End) => return Ok(self.run_game_end_events()?),
            None => (),
        }
        for ev in self.stored_events.iter() {
            replay.startup_events.push(ev.clone());
        }
        self.stored_events.clear();

        let mut err_string: Option<String> = None;

        self.renderer.resize_framebuffer(ui_width.into(), ui_height.into(), true);
        let mut renderer_state = self.renderer.state();
        self.renderer.set_state(&ui_renderer_state);

        let mut frame_counter = 0; // TODO: this really should be stored in Game and Savestate, not here

        let mut frame_text = String::from("Frame: 0");
        let mut seed_text = format!("Seed: {}", self.rand.seed());

        'gui: loop {
            let time_start = Instant::now();

            // refresh io state
            let io = context.io();
            io.set_mouse_wheel(0.0);

            // poll window events
            let mut space_pressed = false;
            self.window.swap_events();
            for event in self.window.events() {
                if matches!(event, Event::KeyboardDown(Key::Space) | Event::KeyboardRepeat(Key::Space)) {
                    space_pressed = true;
                }

                match event {
                    ev @ Event::KeyboardDown(key) | ev @ Event::KeyboardUp(key) => {
                        let state = matches!(ev, Event::KeyboardDown(_));
                        io.set_key(usize::from(input::ramen2vk(*key)), state);
                        match key {
                            Key::LShift | Key::RShift => io.set_shift(state),
                            Key::LControl | Key::RControl => io.set_ctrl(state),
                            Key::LAlt | Key::RAlt => io.set_alt(state),
                            _ => (),
                        }
                    },
                    Event::MouseMove((point, scale)) => {
                        let (x, y) = point.as_physical(*scale);
                        io.set_mouse(imgui::Vec2(x as f32, y as f32));
                    },
                    ev @ Event::MouseDown(btn) | ev @ Event::MouseUp(btn) => usize::try_from(input::ramen2mb(*btn))
                        .ok().and_then(|x| x.checked_sub(1))
                        .into_iter()
                        .for_each(|x| io.set_mouse_button(x, matches!(ev, Event::MouseDown(_)))),
                    Event::MouseWheel(delta) => io.set_mouse_wheel(delta.get() as f32 / 120.0),
                    Event::Resize((size, scale)) => {
                        let (width, height) = size.as_physical(*scale);
                        ui_width = u16::try_from(width).unwrap_or(u16::MAX);
                        ui_height = u16::try_from(height).unwrap_or(u16::MAX);
                        io.set_display_size(imgui::Vec2(width as f32, height as f32));
                        self.renderer.resize_framebuffer(width, height, false);
                    },
                    Event::CloseRequest(_) => break 'gui,
                    _ => (),
                }
            }

            // present imgui
            let mut is_open = false;
            let fps_text = format!("FPS: {}", io.framerate());
            let win_frame_height = context.frame_height();
            let win_border_size = context.window_border_size();
            let win_padding = context.window_padding();
            let mut frame = context.new_frame();

            frame.begin_window("Control", None, true, false, &mut is_open);
            if (frame.button("Advance", imgui::Vec2(150.0, 20.0), None) || space_pressed) && err_string.is_none() {
                let (w, h) = self.renderer.stored_size();
                let frame = replay.new_frame(self.room.speed);
                // TODO: all of this and also key events
                //frame.mouse_x = mouse_location.0;
                //frame.mouse_y = mouse_location.1;
                //frame.new_seed = None;

                //if let Some(seed) = new_seed {
                //    self.rand.set_seed(seed);
                //}

                // self.input_manager.mouse_update_previous();
                // self.input_manager.set_mouse_pos(mouse_location.0, mouse_location.1);

                self.renderer.set_state(&renderer_state);
                self.renderer.resize_framebuffer(w, h, false);
                self.renderer.set_view(0, 0, self.unscaled_width as _, self.unscaled_height as _,
                    0.0, 0, 0, self.unscaled_width as _, self.unscaled_height as _);
                self.renderer.draw_stored(0, 0, w, h);
                match self.frame() {
                    Ok(()) => {
                        match self.scene_change {
                            Some(SceneChange::Room(id)) => self.load_room(id)?,
                            Some(SceneChange::Restart) => self.restart()?,
                            Some(SceneChange::End) => self.restart()?,
                            None => (),
                        }

                        for ev in self.stored_events.iter() {
                            frame.events.push(ev.clone());
                        }
                        self.stored_events.clear();

                        self.renderer.resize_framebuffer(ui_width.into(), ui_height.into(), true);
                        self.renderer.set_view( 0, 0, ui_width.into(), ui_height.into(),
                            0.0, 0, 0, ui_width.into(), ui_height.into());
                        self.renderer.clear_view(clear_colour, 1.0);
                        renderer_state = self.renderer.state();
                        self.renderer.set_state(&ui_renderer_state);

                        // Fake frame limiter stuff (don't actually frame-limit in record mode)
                        if let Some(t) = self.spoofed_time_nanos.as_mut() {
                            *t += Duration::new(0, 1_000_000_000u32 / self.room.speed).as_nanos();
                        }
                        if frame_counter == self.room.speed {
                            self.fps = self.room.speed;
                            frame_counter = 0;
                        }
                        frame_counter += 1;

                        frame_text = format!("Frame: {}", replay.frame_count());
                        seed_text = format!("Seed: {}", self.rand.seed());
                    },
                    Err(e) => {
                        err_string = Some(format!("{}", e));
                    },
                }
            }

            if frame.button("Save", imgui::Vec2(150.0, 20.0), None) && err_string.is_none() {
                savestate = Some(SaveState::from(self, replay.clone(), renderer_state.clone()));
            }

            if let Some(state) = &savestate {
                if frame.button("Load", imgui::Vec2(150.0, 20.0), None) {
                    err_string = None;
                    let (rep, ren) = state.clone().load_into(self);
                    replay = rep;
                    renderer_state = ren;

                    frame_text = format!("Frame: {}", replay.frame_count());
                    seed_text = format!("Seed: {}", self.rand.seed());
                }
            }

            frame.text(&frame_text);
            frame.text(&seed_text);
            frame.text(&fps_text);
            frame.end();

            unsafe { cimgui_sys::igSetNextWindowSizeConstraints(imgui::Vec2(440.0, 200.0).into(), imgui::Vec2(1800.0, 650.0).into(), None, std::ptr::null_mut()) }
            frame.begin_window("Keyboard", None, true, true, &mut is_open);
            let content_min = win_padding + imgui::Vec2(0.0, win_frame_height * 2.0);
            let content_max = frame.window_size() - win_padding;

            let mut cur_x = content_min.0;
            let mut cur_y = content_min.1;
            let left_part_edge = ((content_max.0 - content_min.0) * (15.0 / 18.5)).floor();
            let button_width = ((left_part_edge - content_min.0 - 14.0) / 15.0).floor();
            let button_height = ((content_max.1 - content_min.1 - 4.0 - (win_padding.1 * 2.0)) / 6.5).floor();
            frame.button("Esc", imgui::Vec2((button_width * 1.5).floor(), button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x = left_part_edge - (button_width * 12.0 + 11.0);
            frame.button("F1", imgui::Vec2(button_width, button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x += button_width + 1.0;
            frame.button("F2", imgui::Vec2(button_width, button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x += button_width + 1.0;
            frame.button("F3", imgui::Vec2(button_width, button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x += button_width + 1.0;
            frame.button("F4", imgui::Vec2(button_width, button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x += button_width + 1.0;
            frame.button("F5", imgui::Vec2(button_width, button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x += button_width + 1.0;
            frame.button("F6", imgui::Vec2(button_width, button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x += button_width + 1.0;
            frame.button("F7", imgui::Vec2(button_width, button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x += button_width + 1.0;
            frame.button("F8", imgui::Vec2(button_width, button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x += button_width + 1.0;
            frame.button("F9", imgui::Vec2(button_width, button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x += button_width + 1.0;
            frame.button("F10", imgui::Vec2(button_width, button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x += button_width + 1.0;
            frame.button("F11", imgui::Vec2(button_width, button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x += button_width + 1.0;
            frame.button("F12", imgui::Vec2(button_width, button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x = content_max.0 - (button_width * 3.0 + 2.0);
            frame.button("PrSc", imgui::Vec2(button_width, button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x += button_width + 1.0;
            frame.button("ScrLk", imgui::Vec2(button_width, button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x += button_width + 1.0;
            frame.button("Pause", imgui::Vec2(button_width, button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x = content_min.0;
            cur_y = (content_max.1 - (win_padding.1 * 2.0)).ceil() - (button_height * 5.0 + 4.0);
            frame.button("`", imgui::Vec2(button_width, button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x += button_width + 1.0;
            frame.button("1", imgui::Vec2(button_width, button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x += button_width + 1.0;
            frame.button("2", imgui::Vec2(button_width, button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x += button_width + 1.0;
            frame.button("3", imgui::Vec2(button_width, button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x += button_width + 1.0;
            frame.button("4", imgui::Vec2(button_width, button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x += button_width + 1.0;
            frame.button("5", imgui::Vec2(button_width, button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x += button_width + 1.0;
            frame.button("6", imgui::Vec2(button_width, button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x += button_width + 1.0;
            frame.button("7", imgui::Vec2(button_width, button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x += button_width + 1.0;
            frame.button("8", imgui::Vec2(button_width, button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x += button_width + 1.0;
            frame.button("9", imgui::Vec2(button_width, button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x += button_width + 1.0;
            frame.button("0", imgui::Vec2(button_width, button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x += button_width + 1.0;
            frame.button("-", imgui::Vec2(button_width, button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x += button_width + 1.0;
            frame.button("=", imgui::Vec2(button_width, button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x += button_width + 1.0;
            frame.button("Back", imgui::Vec2(left_part_edge - cur_x, button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x = content_max.0 - (button_width * 3.0 + 2.0);
            frame.button("Ins", imgui::Vec2(button_width, button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x += button_width + 1.0;
            frame.button("Home", imgui::Vec2(button_width, button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x += button_width + 1.0;
            frame.button("PgUp", imgui::Vec2(button_width, button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x = content_min.0;
            cur_y += button_height + 1.0;
            frame.button("Tab", imgui::Vec2((button_width * 1.5).floor(), button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x += (button_width * 1.5).floor() + 1.0;
            frame.button("Q", imgui::Vec2(button_width, button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x += button_width + 1.0;
            frame.button("W", imgui::Vec2(button_width, button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x += button_width + 1.0;
            frame.button("E", imgui::Vec2(button_width, button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x += button_width + 1.0;
            frame.button("R", imgui::Vec2(button_width, button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x += button_width + 1.0;
            frame.button("T", imgui::Vec2(button_width, button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x += button_width + 1.0;
            frame.button("Y", imgui::Vec2(button_width, button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x += button_width + 1.0;
            frame.button("U", imgui::Vec2(button_width, button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x += button_width + 1.0;
            frame.button("I", imgui::Vec2(button_width, button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x += button_width + 1.0;
            frame.button("O", imgui::Vec2(button_width, button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x += button_width + 1.0;
            frame.button("P", imgui::Vec2(button_width, button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x += button_width + 1.0;
            frame.button("[", imgui::Vec2(button_width, button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x += button_width + 1.0;
            frame.button("]", imgui::Vec2(button_width, button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x += button_width + 1.0;
            frame.button("Enter", imgui::Vec2(left_part_edge - cur_x, button_height * 2.0 + 1.0), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x = content_max.0 - (button_width * 3.0 + 2.0);
            frame.button("Del", imgui::Vec2(button_width, button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x += button_width + 1.0;
            frame.button("End", imgui::Vec2(button_width, button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x += button_width + 1.0;
            frame.button("PgDn", imgui::Vec2(button_width, button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x = content_min.0;
            cur_y += button_height + 1.0;
            frame.button("Caps", imgui::Vec2((button_width * 1.5).floor(), button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x += (button_width * 1.5).floor() + 1.0;
            frame.button("A", imgui::Vec2(button_width, button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x += button_width + 1.0;
            frame.button("S", imgui::Vec2(button_width, button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x += button_width + 1.0;
            frame.button("D", imgui::Vec2(button_width, button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x += button_width + 1.0;
            frame.button("F", imgui::Vec2(button_width, button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x += button_width + 1.0;
            frame.button("G", imgui::Vec2(button_width, button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x += button_width + 1.0;
            frame.button("H", imgui::Vec2(button_width, button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x += button_width + 1.0;
            frame.button("J", imgui::Vec2(button_width, button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x += button_width + 1.0;
            frame.button("K", imgui::Vec2(button_width, button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x += button_width + 1.0;
            frame.button("L", imgui::Vec2(button_width, button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x += button_width + 1.0;
            frame.button(";", imgui::Vec2(button_width, button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x += button_width + 1.0;
            frame.button("'", imgui::Vec2(button_width, button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x += button_width + 1.0;
            frame.button("#", imgui::Vec2(button_width, button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x = content_min.0;
            cur_y += button_height + 1.0;
            frame.button("Shift", imgui::Vec2(button_width * 2.0, button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x += button_width * 2.0 + 1.0;
            frame.button("\\", imgui::Vec2(button_width, button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x += button_width + 1.0;
            frame.button("Z", imgui::Vec2(button_width, button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x += button_width + 1.0;
            frame.button("X", imgui::Vec2(button_width, button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x += button_width + 1.0;
            frame.button("C", imgui::Vec2(button_width, button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x += button_width + 1.0;
            frame.button("V", imgui::Vec2(button_width, button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x += button_width + 1.0;
            frame.button("B", imgui::Vec2(button_width, button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x += button_width + 1.0;
            frame.button("N", imgui::Vec2(button_width, button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x += button_width + 1.0;
            frame.button("M", imgui::Vec2(button_width, button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x += button_width + 1.0;
            frame.button(",", imgui::Vec2(button_width, button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x += button_width + 1.0;
            frame.button(".", imgui::Vec2(button_width, button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x += button_width + 1.0;
            frame.button("/", imgui::Vec2(button_width, button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x += button_width + 1.0;
            frame.button("Shift", imgui::Vec2(left_part_edge - cur_x, button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x = content_min.0;
            cur_y += button_height + 1.0;
            frame.button("Ctrl", imgui::Vec2((button_width * 1.5).floor(), button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x += (button_width * 1.5).floor() + 1.0;
            frame.button("Win", imgui::Vec2(button_width, button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x += button_width + 1.0;
            frame.button("Alt", imgui::Vec2(button_width, button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x += button_width + 1.0;
            frame.button("Space", imgui::Vec2((left_part_edge - cur_x) - (button_width * 3.5 + 3.0).floor(), button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x = left_part_edge - (button_width * 3.5 + 2.0).floor();
            frame.button("Alt", imgui::Vec2(button_width, button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x += button_width + 1.0;
            frame.button("Pg", imgui::Vec2(button_width, button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x += button_width + 1.0;
            frame.button("Ctrl", imgui::Vec2(left_part_edge - cur_x, button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x = content_max.0 - (button_width * 3.0 + 2.0);
            frame.button("←", imgui::Vec2(button_width, button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x += button_width + 1.0;
            frame.button("↓", imgui::Vec2(button_width, button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_y -= button_height + 1.0;
            frame.button("↑", imgui::Vec2(button_width, button_height), Some(imgui::Vec2(cur_x, cur_y)));
            cur_x += button_width + 1.0;
            cur_y += button_height + 1.0;
            frame.button("→", imgui::Vec2(button_width, button_height), Some(imgui::Vec2(cur_x, cur_y)));
            frame.end();

            if err_string.is_none() {
                let (w, h) = self.renderer.stored_size();
                frame.begin_window(
                    &format!("{}###Game", self.get_window_title()),
                    Some(imgui::Vec2(w as f32 + (2.0 * win_border_size), h as f32 + win_border_size + win_frame_height)),
                    false,
                    false,
                    &mut is_open,
                );
                let imgui::Vec2(x, y) = frame.window_position();
                let mut callback_data = GameViewData {
                    renderer: (&mut self.renderer) as *mut _,
                    x: (x + win_border_size) as i32,
                    y: (y + win_frame_height) as i32,
                    w: w,
                    h: h,
                };

                unsafe extern "C" fn callback(_draw_list: *const cimgui_sys::ImDrawList, ptr: *const cimgui_sys::ImDrawCmd) {
                    let data = &*((*ptr).UserCallbackData as *mut GameViewData);
                    (*data.renderer).draw_stored(data.x, data.y, data.w, data.h);
                }

                if !frame.window_collapsed() {
                    frame.callback(callback, &mut callback_data);
                }

                frame.end();
            }

            frame.render();

            // draw imgui
            let start_xy = f64::from(grid_start.elapsed().as_millis().rem_euclid(2048) as i16) / -32.0;
            self.renderer.draw_sprite_tiled(&grid_ref, start_xy, start_xy, 1.0, 1.0, 0xFFFFFF, 0.5,
                Some(ui_width.into()), Some(ui_height.into()));

            let draw_data = context.draw_data();
            debug_assert!(draw_data.Valid);
            let cmd_list_count = usize::try_from(draw_data.CmdListsCount)?;
            for list_id in 0..cmd_list_count {
                let draw_list = unsafe { &**draw_data.CmdLists.add(list_id) };
                let cmd_count = usize::try_from(draw_list.CmdBuffer.Size)?;
                let vertex_buffer = draw_list.VtxBuffer.Data;
                let index_buffer = draw_list.IdxBuffer.Data;
                for cmd_id in 0..cmd_count {
                    let command = unsafe { &*draw_list.CmdBuffer.Data.add(cmd_id) };
                    let vertex_buffer = unsafe { vertex_buffer.add(command.VtxOffset as usize) };
                    let index_buffer = unsafe { index_buffer.add(command.IdxOffset as usize) };
                    if let Some(f) = command.UserCallback {
                        unsafe { f(draw_list, command) };
                    }
                    else {
                        // TODO: don't use the primitive builder for this, it allocates a lot and
                        // also doesn't do instanced drawing I think?
                        self.renderer.reset_primitive_2d(
                            PrimitiveType::TriList,
                            if command.TextureId.is_null() {
                                None
                            } else {
                                Some(unsafe { *(command.TextureId as *mut AtlasRef) })
                            }
                        );

                        for i in 0..(command.ElemCount as usize) {
                            let vert = unsafe { *(vertex_buffer.add(usize::from(*index_buffer.add(i)))) };
                            self.renderer.vertex_2d(
                                f64::from(vert.pos.x) - 0.5,
                                f64::from(vert.pos.y) - 0.5,
                                vert.uv.x.into(),
                                vert.uv.y.into(),
                                (vert.col & 0xFFFFFF) as _,
                                f64::from(vert.col >> 24) / 255.0,
                            );
                        }

                        let clip_x = command.ClipRect.x as i32;
                        let clip_y = command.ClipRect.y as i32;
                        let clip_w = (command.ClipRect.z - command.ClipRect.x) as i32 + 1;
                        let clip_h = (command.ClipRect.w - command.ClipRect.y) as i32 + 1;
                        self.renderer.set_view(clip_x, clip_y, clip_w, clip_h, 0.0, clip_x, clip_y, clip_w, clip_h);
                        self.renderer.draw_primitive_2d();
                    }
                }
            }

            self.renderer.finish(ui_width.into(), ui_height.into(), clear_colour);

            context.io().set_delta_time(time_start.elapsed().as_micros() as f32 / 1000000.0);
        }

        Ok(())
    }
}
