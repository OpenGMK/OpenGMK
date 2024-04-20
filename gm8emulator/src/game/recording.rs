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
mod set_mouse_dialog;
mod popup_dialog;

use crate::{
    game::{
        savestate::{self, SaveState},
        recording::{
            instance_report::InstanceReport,
            window::{
                Window,
                DisplayInformation,
                Openable,
            },
        },
        replay::{self, Replay},
        Game, SceneChange,
    },
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

use super::replay::FrameRng;
const GRID_COLOUR_GOOD: Colour = Colour::new(0.25, 0.625, 0.38671875);
const GRID_COLOUR_BAD: Colour = Colour::new(0.75, 0.359375, 0.0);
const CLEAR_COLOUR_GOOD: Colour = Colour::new(0.0196, 0.1059, 0.06275);
const CLEAR_COLOUR_BAD: Colour = Colour::new(0.078125, 0.046875, 0.03515625);
const BTN_NEUTRAL_COL: Colour = Colour::new(0.15, 0.15, 0.21);
const BTN_NDOUBLE_COL: Colour = Colour::new(0.21, 0.21, 0.26);
const BTN_NTRIPLE_COL: Colour = Colour::new(0.24, 0.24, 0.315);
const BTN_HELD_COL: Colour = Colour::new(0.486, 1.0, 0.506);
const BTN_HDOUBLE_COL: Colour = Colour::new(0.46, 0.85, 0.48);
const BTN_HTRIPLE_COL: Colour = Colour::new(0.44, 0.7, 0.455);
const BTN_CACTUS_COL: Colour = Colour::new(1.0, 0.788, 0.055);

// How long the grid takes to switch between colors, in miliseconds.
const GRID_CHANGE_TIME: u32 = 500;

struct UIState<'g> {
    /// The Game struct, constructed outside this file, to be controlled by the TAS UI
    game: &'g mut Game,

    /// Path to the project folder for this project
    project_path: PathBuf,

    /// Project configuration file, contains metadata such as UI size and re-record count, and is
    /// serialised to and from a file, to be retained when the program is not running
    config: ProjectConfig,

    /// Replay struct for what's currently on screen
    /// Gets updated with new frames when the user advances, or gets overwritten from a file when loading a savestate
    replay: Replay,

    /// Buffer for lz4 stuff
    lz4_buffer: savestate::Buffer,

    /// Atlas ref for grid background
    grid_ref: AtlasRef,

    /// Instant for calculating grid background delta time
    grid_start: Instant,

    /// True if the game is running, false if the game has crashed or exited in any way
    game_running: bool,

    /// Whether the game started up successfully without crashing
    /// If false, the UI will still show, but no features will be usable, there is no game state to work with
    startup_successful: bool,

    /// Important informational string to be displayed to the user, if any
    /// Will be displayed above all windows and prevent doing anything else until the message is closed
    err_string: Option<String>,

    /// What the game thinks is the current state of the keyboard keys we care about
    keyboard_state: [KeyState; 256],

    /// What the game thinks is the current state of the mouse buttons we care about
    mouse_state: [KeyState; 3],

    /// Mouse position set by the user to be taken into use next time they advance a frame
    new_mouse_pos: Option<(i32, i32)>,

    /// Whether the user is currently in the process of setting a mouse position
    /// If so, mouse inputs should be "eaten" by this process and not sent to imgui windows
    setting_mouse_pos: bool,

    /// OpenGL state for the UI
    ui_renderer_state: RendererState,
    
    /// A SaveState cached in memory to prevent having to load it from a file
    /// Usually used for whichever savestate is "selected" for quick access
    cached_savestate: SaveState,

    /// What the game thinks the current OpenGL state is, and will be briefly taken into use during frame advance
    game_renderer_state: RendererState,

    /// Whether or not context menus should close. Reset at the start of the frameloop then updated accordingly by io and windows
    clear_context_menu: bool,

    /// Index of the window that currently has control over the context menu
    context_menu_window: Option<usize>,

    /// Index of the window that currently has control over the modal dialog
    modal_window_handler: Option<usize>,

    /// Position of the context menu
    context_menu_pos: imgui::Vec2<f32>,

    /// Cached reports on the current state of any instances the user is "watching"
    instance_reports: Vec<(i32, Option<InstanceReport>)>,

    /// Until which frame the game should advance
    run_until_frame: Option<usize>,

    /// Whether or not the current state of the game is clean or has potentially been modified
    clean_state: bool,

    /// Previous value of clean_state, used to switch between grid colors
    clean_state_previous: bool,

    /// Time instant of when the last clean_state switch occured, used to fade between grid colors
    clean_state_instant: Option<Instant>,

    /// Current blend color of the grid
    grid_colour: Colour,

    /// Current clear color
    grid_colour_background: Colour,

    /// New RNG seed selected by the user, if they have changed it, to be taken into use on next frame advance
    new_rand: Option<FrameRng>,

    /// The currently open windows
    windows: Vec<(Box<dyn Window>, bool)>,

    /// List of PathBufs to savestate files
    save_paths: Vec<PathBuf>,

    /// Path of the keybind file
    keybind_path: PathBuf,

    /// Active keybindings
    keybindings: keybinds::Keybindings,

    win_frame_height: f32,
    win_border_size: f32,
    win_padding: imgui::Vec2<f32>,
}

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
pub enum WindowKind {
    Control,
    Game,
    InstanceReports,
    Input,
    Savestates,
    InputEditor,
    Keybindings,
    Macro(usize),
    Console(usize),
}

#[derive(Deserialize, Serialize)]
pub struct ProjectConfig {
    ui_width: u16,
    ui_height: u16,
    ui_maximised: bool,
    rerecords: u64,
    watched_ids: Vec<i32>,
    open_windows: Vec<WindowKind>,
    full_keyboard: bool,
    input_mode: InputMode,
    quicksave_slot: usize,
    config_path: PathBuf,
    is_read_only: bool,
    current_frame: usize,
    set_mouse_using_textbox: bool
}

impl ProjectConfig {
    fn from_file_or_default(config_path: &PathBuf) -> Self {
        let default_config = Self {
            ui_width: 1280,
            ui_height: 720,
            ui_maximised: false,
            rerecords: 0,
            watched_ids: Vec::new(),
            open_windows: Vec::new(),
            full_keyboard: false,
            input_mode: InputMode::Mouse,
            quicksave_slot: 0,
            config_path: config_path.clone(),
            is_read_only: false,
            current_frame: 0,
            set_mouse_using_textbox: false,
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

    /// Saves the configuration file. If that failed it will return a description of the error, otherwise None
    pub fn save(&self) -> Option<String> {
        File::create(&self.config_path)
            .map(|f| bincode::serialize_into(f, &self))
            .err()
            .map(|e| format!("Config file was not saved to disk because of an error: {}", e))
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

        let mut clean_state = true;

        // Generate white grid sprite. Color is blended in when drawn.
        // It's not entirely accurate to the K3 one anymore, someone's probably going to be upset at that.
        let grid = (0i32..(64 * 64 * 4))
            .map(|i| {
                let n = i >> 2; // pixel index (floor(index/4))
                let x = n % 64;
                let y = n / 64;
                let c = (31..34).contains(&(y - x).abs()) || (31..34).contains(&(y + x - 63).abs());
                match (i & 1 != 0, i & 2 != 0) {
                    (false, false) => u8::from(c) * 255, // r
                    (true, false) => u8::from(c) * 255, // g
                    (false, true) => u8::from(c) * 255, // b
                    (true, true) => u8::from(c) * 255, // a
                }
            })
            .collect::<Vec<_>>()
            .into_boxed_slice();
        let grid_ref = self.renderer.upload_sprite(grid, 64, 64, 0, 0).expect("Failed to upload UI images");
        let grid_start = Instant::now();

        let mut keyboard_state = [KeyState::Neutral; 256];
        let mut mouse_state = [KeyState::Neutral; 3];

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

        let savestate;
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
                        savestate = SaveState::from(self, replay.clone(), renderer_state.clone(), false);
                        startup_successful = false;
                        game_running = false;
                    },
                }
            } else {
                let mut save_replay = replay.clone();
                save_replay.truncate_frames(config.current_frame);
                savestate = SaveState::from(self, save_replay, renderer_state.clone(), true);

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
                    clean_state = state.clean_state;
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
                    savestate = SaveState::from(self, replay.clone(), renderer_state.clone(), false);
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

        let mut keybind_path = project_path.clone();
        keybind_path.push("keybindings.cfg");

        let instance_reports = config.watched_ids.iter().map(|id| (*id, InstanceReport::new(&*self, *id))).collect();
        let keybindings = keybinds::Keybindings::from_file_or_default(&keybind_path);

        let mut windows: Vec<(Box<dyn Window>, bool)> = vec![
            (Box::new(game_window::GameWindow::new()), true),
            (Box::new(control_window::ControlWindow::new()), false),
            (Box::new(savestate_window::SaveStateWindow::new(16)), false),
            (Box::new(input_window::InputWindows::new()), false),
            (Box::new(instance_report::InstanceReportWindow::new()), false),
        ];

        for window in &config.open_windows {
            match window {
                WindowKind::InputEditor => windows.push((Box::new(input_edit::InputEditWindow::open(0)), false)),
                WindowKind::Keybindings => windows.push((Box::new(keybinds::KeybindWindow::open(0)), false)),
                WindowKind::Macro(id) => windows.push((Box::new(macro_window::MacroWindow::open(*id)), false)),
                WindowKind::Console(id) => windows.push((Box::new(console::ConsoleWindow::open(*id)), false)),
                WindowKind::Control 
                 | WindowKind::Game
                 | WindowKind::InstanceReports
                 | WindowKind::Input
                 | WindowKind::Savestates => panic!("Control windows can not be stored in project config"),
            }
        }

        /* ----------------------
        Frame loop begins here
        ---------------------- */
        UIState {
            game: self,
            project_path,
            config,
            replay,
            lz4_buffer: save_buffer,
            grid_ref,
            grid_start,
            game_running,
            startup_successful,
            err_string,
            keyboard_state,
            mouse_state,
            new_mouse_pos: None,
            setting_mouse_pos: false,
            ui_renderer_state,
            cached_savestate: savestate,
            game_renderer_state: renderer_state,
            clear_context_menu: false,
            context_menu_window: None,
            context_menu_pos: imgui::Vec2(0.0, 0.0),
            run_until_frame: None,
            clean_state,
            clean_state_previous: clean_state,
            clean_state_instant: None,
            grid_colour: GRID_COLOUR_GOOD,
            grid_colour_background: CLEAR_COLOUR_GOOD,
            instance_reports,
            new_rand: None,
            save_paths,
            keybind_path,
            keybindings,
            windows,
            win_border_size: 0.0,
            win_frame_height: 0.0,
            win_padding: imgui::Vec2(0.0, 0.0),
            modal_window_handler: None,
        }.run(&mut context);
    }
}

impl UIState<'_> {
    fn run(mut self, context: &mut imgui::Context) {
        'gui: loop {
            let time_start = Instant::now();

            // refresh io state
            let io = context.io();
            io.set_mouse_wheel(0.0);
            self.clear_context_menu = false;

            // poll window events
            if !self.poll_window_events(io) {
                break 'gui;
            }

            // present imgui
            let fps_text = format!("FPS: {}", io.framerate().round());
            self.win_frame_height = context.frame_height();
            self.win_border_size = context.window_border_size();
            self.win_padding = context.window_padding();
            let mut frame = context.new_frame();

            // ImGui windows
            // todo: maybe separate control logic from the windows at some point so we can close control/savestate/input windows
            //       and still have the keyboard shortcuts and everything working. Collapsing them is good enough for now.
            if !self.update_windows(&mut frame, &fps_text) {
                break 'gui;
            }

            // Show error/info message if there is one
            if let Some(err) = &self.err_string {
                if !frame.popup(err) {
                    if self.startup_successful {
                        self.err_string = None;
                    } else {
                        break 'gui
                    }
                }
            }

            // Done
            frame.render();
            self.update_grid_colour();
            self.render_ui_frame(context);

            context.io().set_delta_time(time_start.elapsed().as_micros() as f32 / 1000000.0);
        }

        self.save_config();

        if let Some(e) = self.err_string {
            println!("Warning: recording.rs exited with an error present: {}", e);
        }
    }

    fn update_grid_colour(&mut self) {
        if self.clean_state != self.clean_state_previous {
            self.clean_state_previous = self.clean_state;
            self.clean_state_instant = Some(Instant::now());
        }

        if self.clean_state_instant.is_some() {
            let target_grid_colour = if self.clean_state { GRID_COLOUR_GOOD } else { GRID_COLOUR_BAD };
            let target_grid_background = if self.clean_state { CLEAR_COLOUR_GOOD } else { CLEAR_COLOUR_BAD };

            let time = self.clean_state_instant.unwrap().elapsed().as_millis();
            if time < GRID_CHANGE_TIME as _ {
                self.grid_colour = self.grid_colour.lerp(target_grid_colour, time as f64 / GRID_CHANGE_TIME as f64);
                self.grid_colour_background = self.grid_colour_background.lerp(target_grid_background, time as f64 / GRID_CHANGE_TIME as f64);
            } else {
                self.clean_state_instant = None;
                self.grid_colour = target_grid_colour;
                self.grid_colour_background = target_grid_background;
            }
        }
    }

    fn render_ui_frame(&mut self, context: &mut imgui::Context) {
        // draw imgui
        let start_xy = f64::from(self.grid_start.elapsed().as_millis().rem_euclid(2048) as i16) / -32.0;
        self.game.renderer.draw_sprite_tiled(
            self.grid_ref,
            start_xy,
            start_xy,
            1.0,
            1.0,
            self.grid_colour.as_decimal() as _,
            0.5,
            Some(self.config.ui_width.into()),
            Some(self.config.ui_height.into()),
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
                    self.game.renderer.reset_primitive_2d(
                        PrimitiveType::TriList,
                        if command.TextureId.is_null() {
                            None
                        } else {
                            Some(unsafe { *(command.TextureId as *mut AtlasRef) })
                        },
                    );

                    for i in 0..(command.ElemCount as usize) {
                        let vert = unsafe { *(vertex_buffer.add(usize::from(*index_buffer.add(i)))) };
                        self.game.renderer.vertex_2d(
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
                    self.game.renderer.set_view(clip_x, clip_y, clip_w, clip_h, 0.0, clip_x, clip_y, clip_w, clip_h);
                    self.game.renderer.draw_primitive_2d();
                }
            }
        }

        self.game.renderer.finish(self.config.ui_width.into(), self.config.ui_height.into(), self.grid_colour_background);
    }

    /// Pulls new window events from operating system and updates config, imgui and renderer accordingly.
    /// Returns false if the program should exit (eg. the 'X' button was pressed), otherwise true.
    fn poll_window_events(&mut self, io: &mut imgui::IO) -> bool {
        self.game.window.poll_events();
        for event in self.game.window.events().into_iter().copied() {
            match event {
                ev @ Event::KeyboardDown(key) | ev @ Event::KeyboardUp(key) => {
                    let state = matches!(ev, Event::KeyboardDown(_));
                    if state { // Only cancel mouse selection when pressing down a key
                        if key == Key::Escape {
                            if self.setting_mouse_pos {
                                // Unset new mouse position if we pressed escape
                                self.new_mouse_pos = None;
                            }
                            // Cancel running until a frame when Escape was pressed
                            self.run_until_frame = None;
                        }

                        if !self.config.set_mouse_using_textbox {
                            // If we are not setting the mouse using a textbox, let any keypress cancel out of it
                            self.setting_mouse_pos = false;
                        }
                    }
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
                    self.config.ui_width = u16::try_from(width).unwrap_or(u16::MAX);
                    self.config.ui_height = u16::try_from(height).unwrap_or(u16::MAX);
                    io.set_display_size(imgui::Vec2(width as f32, height as f32));
                    self.game.renderer.resize_framebuffer(width as _, height as _, false);
                    self.clear_context_menu = true;
                },
                Event::Focus(false) => {
                    io.clear_inputs();
                    self.clear_context_menu = true;
                },
                Event::Maximise(b) => {
                    self.config.ui_maximised = b;
                    self.clear_context_menu = true;
                }
                Event::CloseRequest => return false,
                _ => (),
            }
        }
        true
    }

    /// Updates all imgui windows (including the context menus and menu bar)
    /// Returns false if the application should exit
    fn update_windows(&mut self, frame: &mut imgui::Frame, fps_text: &String) -> bool {

        // Update menu bar
        if !self.show_menu_bar(frame) {
            return false;
        }
        
        self.keybindings.update_disable_bindings();

        let mut display_info = DisplayInformation {
            game: self.game,
            frame,
            game_running: &mut self.game_running,
            setting_mouse_pos: &mut self.setting_mouse_pos,
            new_mouse_pos: &mut self.new_mouse_pos,
            new_rand: &mut self.new_rand,
            err_string: &mut self.err_string,
            replay: &mut self.replay,
            config: &mut self.config,

            keyboard_state: &mut self.keyboard_state,
            mouse_state: &mut self.mouse_state,
            savestate: &mut self.cached_savestate,
            renderer_state: &mut self.game_renderer_state,
            save_buffer: &mut self.lz4_buffer,
            instance_reports: &mut self.instance_reports,

            clean_state: &mut self.clean_state,
            run_until_frame: &mut self.run_until_frame,

            startup_successful: &self.startup_successful,
            ui_renderer_state: &self.ui_renderer_state,
            fps_text: &fps_text,
            save_paths: &self.save_paths,
            project_path: &self.project_path,

            win_frame_height: self.win_frame_height,
            win_border_size: self.win_border_size,
            win_padding: self.win_padding,

            keybindings: &mut self.keybindings,

            _clear_context_menu: self.clear_context_menu,
            _request_context_menu: false,
            _context_menu_requested: false,
            _modal_dialog: None,
        };

        let mut new_context_menu_window: Option<usize> = self.context_menu_window;
        let mut new_modal_window: Option<(usize, &'static str)> = None;

        for (index, (win, focus)) in self.windows.iter_mut().enumerate() {
            if *focus {
                display_info.frame.set_next_window_focus();
                *focus = false;
            }
            win.show_window(&mut display_info);

            if !self.clear_context_menu {
                if display_info.context_menu_clear_requested() {
                    self.clear_context_menu = true;
                } else if display_info.context_menu_requested() {
                    new_context_menu_window = Some(index);
                    self.context_menu_pos = display_info.frame.mouse_pos();
                }
            }
            display_info.reset_context_menu_state(self.clear_context_menu);

            if let Some(modal) = display_info._modal_dialog {
                new_modal_window = Some((index, modal));
            }
            display_info._modal_dialog = None;
        }

        if self.clear_context_menu {
            new_context_menu_window = None;
        }

        if self.context_menu_window != new_context_menu_window {
            if self.context_menu_window.is_some() {
                // Close old context menu
                match self.windows.get_mut(self.context_menu_window.unwrap()) {
                    Some((win, _)) => win.context_menu_close(),
                    None => {},
                }
            }

            self.context_menu_window = new_context_menu_window;
        }

        if self.context_menu_window.is_some() {
            let index = self.context_menu_window.unwrap();
            match self.windows.get_mut(index) {
                Some((win, _)) => {
                    display_info.frame.begin_context_menu(self.context_menu_pos);
                    if !display_info.frame.window_focused() || !win.show_context_menu(&mut display_info) {
                        win.context_menu_close();
                        self.context_menu_window = None;
                    }
                    display_info.frame.end();

                    if let Some(modal) = display_info._modal_dialog {
                        new_modal_window = Some((index, modal));
                    }
                },
                None => self.context_menu_window = None,
            }
        }

        if let Some((modal_index, name)) = new_modal_window {
            // If we have requested a new modal window, set handling window index and open it
            self.modal_window_handler = Some(modal_index);
            display_info.frame.open_popup(name);
        }

        if let Some(index) = self.modal_window_handler {
            match self.windows.get_mut(index) {
                Some((win, _)) => {
                    if !win.handle_modal(&mut display_info) {
                        self.modal_window_handler = None;
                    }
                },
                None => self.modal_window_handler = None,
            }
        }

        self.windows.retain(|(win, _)| win.is_open());

        true
    }

    fn save_config(&mut self) {
        self.config.open_windows = self.windows.iter().filter_map(|(win, _)| win.stored_kind()).collect();
        self.err_string = self.config.save();

        let _ = File::create(&self.keybind_path).map(|f| bincode::serialize_into(f, &self.keybindings));

        let mut backup_path = self.project_path.clone();
        backup_path.push("backup.gmtas");
        self.replay.to_file(&backup_path).expect("backup.gmtas could not be saved.");
    }
}

impl Colour {
    fn lerp(&self, target: Colour, amount: f64) -> Colour {
        Colour {
            r: (target.r-self.r)*amount+self.r,
            g: (target.g-self.g)*amount+self.g,
            b: (target.b-self.b)*amount+self.b,
        }
    }
}
