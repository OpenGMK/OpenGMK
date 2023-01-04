mod window;
mod game_window;
mod control_window;
mod savestate_window;
mod input_window;
mod instance_report;
mod keybinds;
mod console;
mod menu_bar;
mod input_edit;
mod macro_window;

use crate::{
    game::{
        savestate::{self, SaveState},
        recording::{
            instance_report::InstanceReport,
            window::{
                Window,
                DisplayInformation,
            },
        },
        replay::{self, Replay},
        Game, SceneChange,
    },
    gml::rand::Random,
    render::{atlas::AtlasRef, PrimitiveType, RendererState},
    types::Colour,
    imgui, input,
};
use ramen::{
    event::Event,
    input::Key,
};
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    path::PathBuf,
    time::Instant,
};

const CLEAR_COLOUR: Colour = Colour::new(0.0196, 0.1059, 0.06275);
const BTN_NEUTRAL_COL: Colour = Colour::new(0.15, 0.15, 0.21);
const BTN_NDOUBLE_COL: Colour = Colour::new(0.21, 0.21, 0.26);
const BTN_NTRIPLE_COL: Colour = Colour::new(0.24, 0.24, 0.315);
const BTN_HELD_COL: Colour = Colour::new(0.486, 1.0, 0.506);
const BTN_HDOUBLE_COL: Colour = Colour::new(0.46, 0.85, 0.48);
const BTN_HTRIPLE_COL: Colour = Colour::new(0.44, 0.7, 0.455);
const BTN_CACTUS_COL: Colour = Colour::new(1.0, 0.788, 0.055);

#[derive(Clone, Copy, PartialEq)]
pub enum KeyState {
    Neutral,
    NeutralWillPress,
    NeutralWillDouble,
    NeutralWillTriple,
    NeutralWillCactus,
    NeutralDoubleEveryFrame,
    Held,
    HeldWillRelease,
    HeldWillDouble,
    HeldWillTriple,
    HeldDoubleEveryFrame,
}

impl KeyState {
    fn is_held(&self) -> bool {
        matches!(
            self,
            Self::Held
                | Self::HeldWillRelease
                | Self::HeldWillDouble
                | Self::HeldWillTriple
                | Self::HeldDoubleEveryFrame
        )
    }

    pub fn ends_in_press(&self) -> bool {
        matches!(
            self,
            Self::Held
                | Self::NeutralWillPress
                | Self::HeldWillDouble
                | Self::HeldDoubleEveryFrame
                | Self::NeutralWillTriple
        )
    }
    pub fn starts_with_press(&self) -> bool {
        matches!(
            self,
            Self::Held
                | Self::HeldWillRelease
                | Self::HeldWillDouble
                | Self::HeldWillTriple
                | Self::HeldDoubleEveryFrame
        )
    }
    
    fn reset_to_state(&mut self, target_state: KeyState) {
        let starts_with_press = self.starts_with_press();
        if target_state.starts_with_press() == starts_with_press {
            *self = target_state;
        } else if starts_with_press {
            // target state expects button released, previous state is held
            *self = match target_state {
                KeyState::Neutral
                    | KeyState::NeutralWillCactus
                    | KeyState::NeutralWillDouble
                    | KeyState::NeutralDoubleEveryFrame
                    => KeyState::HeldWillRelease,
                KeyState::NeutralWillTriple
                    | KeyState::NeutralWillPress
                    => KeyState::HeldWillDouble,
                _ => unreachable!(),
            };
        } else {
            // target state expects button held, previous state is released
            *self = match target_state {
                KeyState::Held
                    | KeyState::HeldWillDouble
                    | KeyState::HeldDoubleEveryFrame
                    => KeyState::NeutralWillPress,
                KeyState::HeldWillTriple
                    | KeyState::HeldWillRelease
                    => KeyState::NeutralWillDouble,
                _ => unreachable!(),
            };
        }
    }

    fn click(&mut self) {
        *self = match self {
            Self::Neutral => Self::NeutralWillPress,
            Self::NeutralWillPress
            | Self::NeutralWillDouble
            | Self::NeutralWillTriple
            | Self::NeutralWillCactus
            | Self::NeutralDoubleEveryFrame => Self::Neutral,
            Self::Held => Self::HeldWillRelease,
            Self::HeldWillRelease | Self::HeldWillDouble | Self::HeldWillTriple | Self::HeldDoubleEveryFrame => {
                Self::Held
            },
        }
    }

    fn reset_to(&mut self, pressed: bool) {
        *self = if pressed {
            if *self == Self::HeldDoubleEveryFrame { Self::HeldDoubleEveryFrame } else { Self::Held }
        } else {
            if *self == Self::NeutralDoubleEveryFrame { Self::NeutralDoubleEveryFrame } else { Self::Neutral }
        };
    }

    fn menu(&mut self, frame: &mut imgui::Frame) -> bool {
        if self.is_held() {
            if frame.menu_item("(Keep Held)") {
                *self = KeyState::Held;
                false
            } else if frame.menu_item("Release") {
                *self = KeyState::HeldWillRelease;
                false
            } else if frame.menu_item("Release, Press") {
                *self = KeyState::HeldWillDouble;
                false
            } else if frame.menu_item("Release, Press, Release") {
                *self = KeyState::HeldWillTriple;
                false
            } else if frame.menu_item("Tap Every Frame") {
                *self = KeyState::HeldDoubleEveryFrame;
                false
            } else {
                true
            }
        } else {
            if frame.menu_item("(Keep Neutral)") {
                *self = KeyState::Neutral;
                false
            } else if frame.menu_item("Press") {
                *self = KeyState::NeutralWillPress;
                false
            } else if frame.menu_item("Press, Release") {
                *self = KeyState::NeutralWillDouble;
                false
            } else if frame.menu_item("Press, Release, Press") {
                *self = KeyState::NeutralWillTriple;
                false
            } else if frame.menu_item("Tap Every Frame") {
                *self = KeyState::NeutralDoubleEveryFrame;
                false
            } else if frame.menu_item("Cactus-Release") {
                *self = KeyState::NeutralWillCactus;
                false
            } else {
                true
            }
        }
    }

    pub fn repr(&self) -> &'static str {
        match self {
            Self::Neutral => "Neutral",
            Self::NeutralWillPress => "Neutral; will press",
            Self::NeutralWillDouble => "Neutral; will press and release",
            Self::NeutralWillTriple => "Neutral; will press, release, press",
            Self::NeutralDoubleEveryFrame => "Neutral; will tap on each frame",
            Self::NeutralWillCactus => "Neutral; will cactus-release",
            Self::Held => "Held",
            Self::HeldWillRelease => "Held; will release",
            Self::HeldWillDouble => "Held; will release and press",
            Self::HeldWillTriple => "Held; will release, press, release",
            Self::HeldDoubleEveryFrame => "Held; will tap on each frame",
        }
    }

    /// Draws the coloured rectangle according to the current state of the button.
    /// Doesn't render any text on it.
    pub fn draw_keystate(&self, frame: &mut imgui::Frame, position: imgui::Vec2<f32>, size: imgui::Vec2<f32>) {
        let wpos = frame.window_position();
        let alpha = if frame.item_hovered() { 255 } else { 190 };
        let r1_min = position + wpos;
        let r1_max = r1_min + imgui::Vec2((size.0 / 2.0).floor(), size.1);
        let r2_min = imgui::Vec2(position.0 + (size.0 / 2.0).floor(), position.1) + wpos;
        let r2_max = position + size + wpos;
        match self {
            KeyState::Neutral => frame.rect(position + wpos, position + size + wpos, BTN_NEUTRAL_COL, alpha),
            KeyState::Held => frame.rect(position + wpos, position + size + wpos, BTN_HELD_COL, alpha),
            KeyState::NeutralWillPress => {
                frame.rect(r1_min, r1_max, BTN_NEUTRAL_COL, alpha);
                frame.rect(r2_min, r2_max, BTN_HELD_COL, alpha);
            },
            KeyState::NeutralWillDouble | KeyState::NeutralDoubleEveryFrame => {
                frame.rect(r1_min, r1_max, BTN_NEUTRAL_COL, alpha);
                frame.rect(r2_min, r2_max, BTN_HDOUBLE_COL, alpha);
            },
            KeyState::NeutralWillTriple => {
                frame.rect(r1_min, r1_max, BTN_NEUTRAL_COL, alpha);
                frame.rect(r2_min, r2_max, BTN_HTRIPLE_COL, alpha);
            },
            KeyState::NeutralWillCactus => {
                frame.rect(r1_min, r1_max, BTN_NEUTRAL_COL, alpha);
                frame.rect(r2_min, r2_max, BTN_CACTUS_COL, alpha);
            },
            KeyState::HeldWillRelease => {
                frame.rect(r1_min, r1_max, BTN_HELD_COL, alpha);
                frame.rect(r2_min, r2_max, BTN_NEUTRAL_COL, alpha);
            },
            KeyState::HeldWillDouble | KeyState::HeldDoubleEveryFrame => {
                frame.rect(r1_min, r1_max, BTN_HELD_COL, alpha);
                frame.rect(r2_min, r2_max, BTN_NDOUBLE_COL, alpha);
            },
            KeyState::HeldWillTriple => {
                frame.rect(r1_min, r1_max, BTN_HELD_COL, alpha);
                frame.rect(r2_min, r2_max, BTN_NTRIPLE_COL, alpha);
            },
        }
        frame.rect_outline(position + wpos, position + size + wpos, Colour::new(0.4, 0.4, 0.65), u8::MAX);
    }

    pub fn push_key_inputs(&self, key: u8, inputs: &mut Vec<replay::Input>) {
        match self {
            Self::NeutralWillPress => {
                inputs.push(replay::Input::KeyPress(key));
            },
            Self::NeutralWillDouble | Self::NeutralDoubleEveryFrame => {
                inputs.push(replay::Input::KeyPress(key));
                inputs.push(replay::Input::KeyRelease(key));
            },
            Self::NeutralWillTriple => {
                inputs.push(replay::Input::KeyPress(key));
                inputs.push(replay::Input::KeyRelease(key));
                inputs.push(replay::Input::KeyPress(key));
            },
            Self::HeldWillRelease | Self::NeutralWillCactus => {
                inputs.push(replay::Input::KeyRelease(key));
            },
            Self::HeldWillDouble | Self::HeldDoubleEveryFrame => {
                inputs.push(replay::Input::KeyRelease(key));
                inputs.push(replay::Input::KeyPress(key));
            },
            Self::HeldWillTriple => {
                inputs.push(replay::Input::KeyRelease(key));
                inputs.push(replay::Input::KeyPress(key));
                inputs.push(replay::Input::KeyRelease(key));
            },
            Self::Neutral | Self::Held => (),
        }
    }
}

#[derive(Deserialize, Serialize)]
pub enum InputMode {
    Mouse,
    Direct,
}

#[derive(Deserialize, Serialize)]
pub struct ProjectConfig {
    ui_width: u16,
    ui_height: u16,
    ui_maximised: bool,
    rerecords: u64,
    watched_ids: Vec<i32>,
    full_keyboard: bool,
    input_mode: InputMode,
    quicksave_slot: usize,
    config_path: PathBuf,
    is_read_only: bool,
    current_frame: usize,
}

impl ProjectConfig {
    fn from_file_or_default(config_path: &PathBuf) -> Self {
        let default_config = Self {
            ui_width: 1280,
            ui_height: 720,
            ui_maximised: false,
            rerecords: 0,
            watched_ids: Vec::new(),
            full_keyboard: false,
            input_mode: InputMode::Mouse,
            quicksave_slot: 0,
            config_path: config_path.clone(),
            is_read_only: false,
            current_frame: 0,
        };
        
        let mut config = if config_path.exists() {
            match bincode::deserialize_from(File::open(&config_path).expect("Couldn't read project.cfg")) {
                Ok(config) => config,
                Err(_) => {
                    println!("Warning: Couldn't parse project.cfg. Using default configuration.");
                    default_config
                },
            }
        } else {
            bincode::serialize_into(File::create(&config_path).expect("Couldn't write project.cfg"), &default_config)
                .expect("Couldn't serialize project.cfg");
            default_config
        };
        // update path in case the project was moved
        config.config_path = config_path.clone();
        config
    }

    pub fn save(&self) {
        let _ = File::create(&self.config_path).map(|f| bincode::serialize_into(f, &self));
    }
}

impl Game {
    pub fn record(&mut self, project_path: PathBuf, pause: bool, start_save_path: Option<&PathBuf>) {
        let mut save_buffer = savestate::Buffer::new();
        let mut startup_successful = true;

        let config_path = {
            let mut p = project_path.clone();
            p.push("project.cfg");
            p
        };
        let mut config = ProjectConfig::from_file_or_default(&config_path);

        let mut replay = Replay::new(self.spoofed_time_nanos.unwrap_or(0), self.rand.seed());

        let mut context = imgui::Context::new();
        context.make_current();
        let io = context.io();
        io.setup_default_keymap();

        let ini_filename = {
            let mut path = project_path.clone();
            path.push("imgui.ini\0");
            path.into_os_string().into_string().expect("Bad project file path")
        };
        unsafe {
            (*cimgui_sys::igGetIO()).IniFilename = ini_filename.as_ptr() as _;
        }
        io.set_display_size(imgui::Vec2(f32::from(config.ui_width), f32::from(config.ui_height)));

        // TODO probably don't store these textures in the same places as the game textures
        let imgui::FontData { data: fdata, size: (fwidth, fheight) } = io.font_data();
        let mut font = self
            .renderer
            .upload_sprite(fdata.into(), fwidth as _, fheight as _, 0, 0)
            .expect("Failed to upload UI font");
        io.set_texture_id((&mut font as *mut AtlasRef).cast());

        let grid = (0i32..(64 * 64 * 4))
            .map(|i| {
                let n = i >> 2;
                let x = n % 64;
                let y = n / 64;
                let a = (y - x).abs() == 32 || (y + x - 63).abs() == 32;
                let b = (y >= 34 && x + y == 97) || ((2..32).contains(&y) && x + y == 33);
                let c = (31..34).contains(&(y - x).abs()) || (31..34).contains(&(y + x - 63).abs());
                match (i & 1 != 0, i & 2 != 0) {
                    (false, false) => u8::from(b) * 64,
                    (true, false) => u8::from(a) * 128 + 64,
                    (false, true) => {
                        if a {
                            99
                        } else {
                            u8::from(b) * 34 + 33
                        }
                    },
                    (true, true) => u8::from(b || c) * 255,
                }
            })
            .collect::<Vec<_>>()
            .into_boxed_slice();
        let grid_ref = self.renderer.upload_sprite(grid, 64, 64, 0, 0).expect("Failed to upload UI images");
        let grid_start = Instant::now();

        let mut keyboard_state = [KeyState::Neutral; 256];
        let mut mouse_state = [KeyState::Neutral; 3];
        let mut new_mouse_pos: Option<(i32, i32)> = None;
        let mut setting_mouse_pos = false;

        let ui_renderer_state = RendererState {
            model_matrix: self.renderer.get_model_matrix(),
            alpha_blending: true,
            blend_mode: self.renderer.get_blend_mode(),
            pixel_interpolation: true,
            texture_repeat: false,
            texture_rects: self.renderer.get_texture_rects(),
            vsync: true,
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

        let save_paths = (0..16)
            .map(|i| {
                let mut path = project_path.clone();
                path.push(&format!("save{}.bin", i + 1));
                path
            })
            .collect::<Vec<_>>();

        let mut game_running = true; // false indicates the game closed or crashed, and so advancing is not allowed
        let mut err_string: Option<String> = None;

        let mut context_menu_window: Option<usize> = None;
        let mut context_menu_pos: imgui::Vec2<f32> = imgui::Vec2(0.0,0.0);
        let mut savestate;
        let mut renderer_state;

        macro_rules! load_backup_recording {
            () => {
                let mut backup_path = project_path.clone();
                backup_path.push("backup.gmtas");

                if backup_path.exists() {
                    match Replay::from_file(&backup_path) {
                        Ok(backup_replay) => {
                            if pause {
                                self.rand.set_seed(backup_replay.start_seed);
                                self.spoofed_time_nanos = Some(backup_replay.start_time);
                                replay.start_seed = backup_replay.start_seed;
                                replay.start_time = backup_replay.start_time;
                            }

                            if backup_replay.contains_part(&replay) {
                                replay = backup_replay;
                            } else if pause {
                                println!("Warning: Game is not part of backup replay");
                            }
                        },
                        Err(e) => err_string = Some(format!("Warning: Failed to load backup replay: {:?}", e)),
                    }
                }
            };
        }

        if pause {
            config.is_read_only = true;
            load_backup_recording!();
        }

        if !save_paths[config.quicksave_slot].exists() || (pause && start_save_path.is_none()) {
            if let Err(e) = match self.init() {
                Ok(()) => match self.scene_change {
                    Some(SceneChange::Room(id)) => self.load_room(id),
                    Some(SceneChange::Restart) => self.restart(),
                    Some(SceneChange::End) => {
                        startup_successful = false;
                        match self.run_game_end_events() {
                            Ok(()) => Err("(Fatal) Game ended during startup".into()),
                            Err(e) => {
                                Err(format!("(Fatal) Game ended during startup, then crashed during Game End: {}", e)
                                    .into())
                            },
                        }
                    },
                    Some(SceneChange::Load(ref mut path)) => {
                        let path = std::mem::take(path);
                        self.load_gm_save(path)
                    },
                    None => Ok(()),
                },
                Err(e) => Err(e),
            } {
                game_running = false;
                startup_successful = false;
                err_string = Some(format!("(Fatal) Game crashed during startup: {}", e));
            }
            for ev in self.stored_events.iter() {
                replay.startup_events.push(ev.clone());
            }
            self.stored_events.clear();

            self.renderer.resize_framebuffer(config.ui_width.into(), config.ui_height.into(), true);
            renderer_state = self.renderer.state();
            self.renderer.set_state(&ui_renderer_state);

            config.current_frame = 0;

            if pause && save_paths[config.quicksave_slot].exists() {
                match SaveState::from_file(&save_paths[config.quicksave_slot], &mut save_buffer) {
                    Ok(save) => savestate = save,
                    Err(e) => {
                        // Just to initialize renderer_state and keep the compiler happy, this won't be used...
                        renderer_state = ui_renderer_state.clone();
                        err_string = Some(format!("(Fatal) Error loading quicksave file: {:?}", e));
                        savestate = SaveState::from(self, replay.clone(), renderer_state.clone());
                        startup_successful = false;
                        game_running = false;
                    },
                }
            } else {
                let mut save_replay = replay.clone();
                save_replay.truncate_frames(config.current_frame);
                savestate = SaveState::from(self, save_replay, renderer_state.clone());

                if let Err(err) = savestate.save_to_file(&save_paths[config.quicksave_slot], &mut save_buffer) {
                    err_string = Some(format!(
                        concat!(
                            "Warning: failed to create {:?} (it has still been saved in memory)\n\n",
                            "Error message: {:?}",
                        ),
                        save_paths[config.quicksave_slot].file_name(),
                        err,
                    ));
                }
                if config.is_read_only {
                    load_backup_recording!();
                }
            }
        } else {
            let save_path = start_save_path.unwrap_or(&save_paths[config.quicksave_slot]);
            match SaveState::from_file(&save_path, &mut save_buffer) {
                Ok(state) => {
                    let (rep, ren) = state.clone().load_into(self);
                    config.current_frame = rep.frame_count();

                    if config.is_read_only {
                        let mut backup_path = project_path.clone();
                        backup_path.push("backup.gmtas");

                        if backup_path.exists() {
                            match Replay::from_file(&backup_path) {
                                Ok(backup_replay) => {
                                    if backup_replay.contains_part(&rep) {
                                        replay = backup_replay;
                                    } else {
                                        replay = rep;
                                    }
                                },
                                Err(e) => {
                                    err_string = Some(format!("Warning: Failed to load backup replay: {:?}", e));
                                    replay = rep
                                },
                            }
                        }
                    } else {
                        replay = rep;
                    }
                    renderer_state = ren;

                    for (i, state) in keyboard_state.iter_mut().enumerate() {
                        *state =
                            if self.input.keyboard_check_direct(i as u8) { KeyState::Held } else { KeyState::Neutral };
                    }

                    for (i, state) in mouse_state.iter_mut().enumerate() {
                        *state =
                            if self.input.mouse_check_button(i as i8 + 1) { KeyState::Held } else { KeyState::Neutral };
                    }

                    self.renderer.resize_framebuffer(config.ui_width.into(), config.ui_height.into(), false);
                    self.renderer.set_state(&ui_renderer_state);
                    savestate = state;
                },
                Err(e) => {
                    // Just to initialize renderer_state and keep the compiler happy, this won't be used...
                    renderer_state = ui_renderer_state.clone();
                    err_string = Some(format!("(Fatal) Error loading quicksave file: {:?}", e));
                    savestate = SaveState::from(self, replay.clone(), renderer_state.clone());
                    startup_successful = false;
                    game_running = false;
                },
            }
        }

        if config.ui_maximised {
            self.window.set_maximised(true);
        } else {
            self.window.set_size((config.ui_width, config.ui_height));
        }

        for (i, state) in keyboard_state.iter_mut().enumerate() {
            if self.input.keyboard_check_direct(i as u8) {
                *state = KeyState::Held;
            }
        }
        for (i, state) in mouse_state.iter_mut().enumerate() {
            if self.input.mouse_check_button(i as i8 + 1) {
                *state = KeyState::Held;
            }
        }

        let mut instance_reports: Vec<(i32, Option<InstanceReport>)> =
            config.watched_ids.iter().map(|id| (*id, InstanceReport::new(&*self, *id))).collect();
        let mut new_rand: Option<Random> = None;

        let mut keybind_path = project_path.clone();
        keybind_path.push("keybindings.cfg");
        let mut keybindings = keybinds::Keybindings::from_file_or_default(&keybind_path);

        let mut windows: Vec<(Box<dyn Window>, bool)> = vec![
            (Box::new(game_window::GameWindow::new()), true),
            (Box::new(control_window::ControlWindow::new()), false),
            (Box::new(savestate_window::SaveStateWindow::new(16)), false),
            (Box::new(input_window::InputWindows::new()), false),
            (Box::new(instance_report::InstanceReportWindow::new()), false),
            // (Box::new(keybinds::KeybindWindow::new()), false),
        ];

        /* ----------------------
        Frame loop begins here
        ---------------------- */

        'gui: loop {
            let time_start = Instant::now();

            // refresh io state
            let io = context.io();
            io.set_mouse_wheel(0.0);

            let mut clear_context_menu = false;

            // poll window events
            self.window.poll_events();
            for event in self.window.events().into_iter().copied() {
                match event {
                    ev @ Event::KeyboardDown(key) | ev @ Event::KeyboardUp(key) => {
                        setting_mouse_pos = false;
                        let state = matches!(ev, Event::KeyboardDown(_));
                        let vk = input::ramen2vk(key);
                        io.set_key(usize::from(vk), state);
                        match key {
                            Key::LeftShift | Key::RightShift => io.set_shift(state),
                            Key::LeftControl | Key::RightControl => io.set_ctrl(state),
                            Key::LeftAlt | Key::RightAlt => io.set_alt(state),
                            _ => (),
                        }
                    },
                    Event::Input(chr) => {
                        io.add_input_character(chr);
                    },
                    Event::MouseMove((x, y)) => {
                        io.set_mouse(imgui::Vec2(x as f32, y as f32));
                    },
                    ev @ Event::MouseDown(btn) | ev @ Event::MouseUp(btn) => usize::try_from(input::ramen2mb(btn))
                        .ok()
                        .and_then(|x| x.checked_sub(1))
                        .into_iter()
                        .for_each(|x| io.set_mouse_button(x, matches!(ev, Event::MouseDown(_)))),
                    Event::ScrollUp => io.set_mouse_wheel(1.0),
                    Event::ScrollDown => io.set_mouse_wheel(-1.0),
                    Event::Resize((width, height)) => {
                        config.ui_width = u16::try_from(width).unwrap_or(u16::MAX);
                        config.ui_height = u16::try_from(height).unwrap_or(u16::MAX);
                        io.set_display_size(imgui::Vec2(width as f32, height as f32));
                        self.renderer.resize_framebuffer(width as _, height as _, false);
                        clear_context_menu = true;
                    },
                    Event::Focus(false) => {
                        io.clear_inputs();
                        clear_context_menu = true;
                    },
                    Event::Maximise(b) => {
                        config.ui_maximised = b;
                        clear_context_menu = true;
                    }
                    Event::CloseRequest => break 'gui,
                    _ => (),
                }
            }

            // present imgui
            let fps_text = format!("FPS: {}", io.framerate().round());
            let win_frame_height = context.frame_height();
            let win_border_size = context.window_border_size();
            let win_padding = context.window_padding();
            let mut frame = context.new_frame();

            // ImGui windows
            // todo: maybe separate control logic from the windows at some point so we can close control/savestate/input windows
            //       and still have the keyboard shortcuts and everything working. Collapsing them is good enough for now.
            {
                let mut close: bool = false;
                menu_bar::show_menu_bar(&mut frame, &mut windows, &mut close);
                if close {
                    break 'gui;
                }

                keybindings.update_disable_bindings();

                let mut display_info = DisplayInformation {
                    game: self,
                    frame: &mut frame,
                    game_running: &mut game_running,
                    setting_mouse_pos: &mut setting_mouse_pos,
                    new_mouse_pos: &mut new_mouse_pos,
                    new_rand: &mut new_rand,
                    err_string: &mut err_string,
                    replay: &mut replay,
                    config: &mut config,

                    keyboard_state: &mut keyboard_state,
                    mouse_state: &mut mouse_state,
                    savestate: &mut savestate,
                    renderer_state: &mut renderer_state,
                    save_buffer: &mut save_buffer,
                    instance_reports: &mut instance_reports,

                    startup_successful: &startup_successful,
                    ui_renderer_state: &ui_renderer_state,
                    fps_text: &fps_text,
                    save_paths: &save_paths,
                    project_path: &project_path,

                    win_frame_height: win_frame_height,
                    win_border_size: win_border_size,
                    win_padding: win_padding,

                    keybindings: &mut keybindings,

                    _clear_context_menu: clear_context_menu,
                    _request_context_menu: false,
                    _context_menu_requested: false,
                };

                let mut new_context_menu_window: Option<usize> = context_menu_window;

                for (index, (win, focus)) in windows.iter_mut().enumerate() {
                    if *focus {
                        display_info.frame.set_next_window_focus();
                        *focus = false;
                    }
                    win.show_window(&mut display_info);

                    if !clear_context_menu {
                        if display_info.context_menu_clear_requested() {
                            clear_context_menu = true;
                        } else if display_info.context_menu_requested() {
                            new_context_menu_window = Some(index);
                            context_menu_pos = display_info.frame.mouse_pos();
                        }
                    }
                    display_info.reset_context_menu_state(clear_context_menu);
                }

                if clear_context_menu {
                    new_context_menu_window = None;
                }

                if context_menu_window != new_context_menu_window {
                    if context_menu_window.is_some() {
                        // Close old context menu
                        match windows.get_mut(context_menu_window.unwrap()) {
                            Some((win, _)) => win.context_menu_close(),
                            None => {},
                        }
                    }

                    context_menu_window = new_context_menu_window;
                }

                if context_menu_window.is_some() {
                    match windows.get_mut(context_menu_window.unwrap()) {
                        Some((win, _)) => {
                            display_info.frame.begin_context_menu(context_menu_pos);
                            if !display_info.frame.window_focused() || !win.show_context_menu(&mut display_info) {
                                win.context_menu_close();
                                context_menu_window = None;
                            }
                            display_info.frame.end();
                        },
                        None => context_menu_window = None,
                    }
                }

                windows.retain(|(win, _)| win.is_open());
            }

            // Show error/info message if there is one
            if let Some(err) = &err_string {
                if !frame.popup(err) {
                    if startup_successful {
                        err_string = None;
                    } else {
                        break 'gui
                    }
                }
            }

            // Done
            frame.render();

            // draw imgui
            let start_xy = f64::from(grid_start.elapsed().as_millis().rem_euclid(2048) as i16) / -32.0;
            self.renderer.draw_sprite_tiled(
                grid_ref,
                start_xy,
                start_xy,
                1.0,
                1.0,
                0xFFFFFF,
                0.5,
                Some(config.ui_width.into()),
                Some(config.ui_height.into()),
            );

            let draw_data = context.draw_data();
            debug_assert!(draw_data.Valid);
            let cmd_list_count = usize::try_from(draw_data.CmdListsCount).unwrap_or(0);
            for list_id in 0..cmd_list_count {
                let draw_list = unsafe { &**draw_data.CmdLists.add(list_id) };
                let cmd_count = usize::try_from(draw_list.CmdBuffer.Size).unwrap_or(0);
                let vertex_buffer = draw_list.VtxBuffer.Data;
                let index_buffer = draw_list.IdxBuffer.Data;
                for cmd_id in 0..cmd_count {
                    let command = unsafe { &*draw_list.CmdBuffer.Data.add(cmd_id) };
                    let vertex_buffer = unsafe { vertex_buffer.add(command.VtxOffset as usize) };
                    let index_buffer = unsafe { index_buffer.add(command.IdxOffset as usize) };
                    if let Some(f) = command.UserCallback {
                        unsafe { f(draw_list, command) };
                    } else {
                        // TODO: don't use the primitive builder for this, it allocates a lot and
                        // also doesn't do instanced drawing I think?
                        self.renderer.reset_primitive_2d(
                            PrimitiveType::TriList,
                            if command.TextureId.is_null() {
                                None
                            } else {
                                Some(unsafe { *(command.TextureId as *mut AtlasRef) })
                            },
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

            self.renderer.finish(config.ui_width.into(), config.ui_height.into(), CLEAR_COLOUR);

            context.io().set_delta_time(time_start.elapsed().as_micros() as f32 / 1000000.0);
        }
        
        config.save();
        let _ = File::create(&keybind_path).map(|f| bincode::serialize_into(f, &keybindings));

        let mut backup_path = project_path.clone();
        backup_path.push("backup.gmtas");
        replay.to_file(&backup_path).expect("backup.gmtas could not be saved.");
    }
}
