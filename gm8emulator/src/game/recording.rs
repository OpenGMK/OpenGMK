use crate::{
    game::{
        replay::{self, Replay},
        savestate::{self, SaveState},
        Game, SceneChange,
    },
    gml::rand::Random,
    imgui, input,
    instance::Field,
    render::{atlas::AtlasRef, PrimitiveType, Renderer, RendererState},
    types::Colour,
};
use ramen::{
    event::{Event, Key},
    monitor::Size,
};
use serde::{Deserialize, Serialize};
use std::{
    convert::TryFrom,
    fs::File,
    path::PathBuf,
    time::{Duration, Instant},
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
enum KeyState {
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

    fn menu(&mut self, frame: &mut imgui::Frame, pos: imgui::Vec2<f32>) -> bool {
        frame.begin_context_menu(pos);
        let open = if !frame.window_focused() {
            false
        } else if self.is_held() {
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
        };
        frame.end();
        open
    }

    fn repr(&self) -> &'static str {
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
}

enum ContextMenu {
    Button { pos: imgui::Vec2<f32>, key: Key },
    MouseButton { pos: imgui::Vec2<f32>, button: i8 },
    Instances { pos: imgui::Vec2<f32>, options: Vec<(String, i32)> },
    Seed { pos: imgui::Vec2<f32> },
}

#[derive(Deserialize, Serialize)]
enum InputMode {
    Mouse,
    Direct,
}

#[derive(Deserialize, Serialize)]
struct ProjectConfig {
    ui_width: u16,
    ui_height: u16,
    rerecords: u64,
    watched_ids: Vec<i32>,
    full_keyboard: bool,
    input_mode: InputMode,
    quicksave_slot: usize,
}

impl Game {
    pub fn record(&mut self, project_path: PathBuf) {
        let mut save_buffer = savestate::Buffer::new();
        let mut startup_successful = true;

        let config_path = {
            let mut p = project_path.clone();
            p.push("project.cfg");
            p
        };
        let default_config = ProjectConfig {
            ui_width: 1280,
            ui_height: 720,
            rerecords: 0,
            watched_ids: Vec::new(),
            full_keyboard: false,
            input_mode: InputMode::Mouse,
            quicksave_slot: 0,
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

        let mut replay = Replay::new(self.spoofed_time_nanos.unwrap_or(0), self.rand.seed());

        let mut context = imgui::Context::new();
        context.make_current();
        let io = context.io();

        let ini_filename = {
            let mut path = project_path.clone();
            path.push("imgui.ini\0");
            path.into_os_string().into_string().expect("Bad project file path")
        };
        unsafe {
            (*cimgui_sys::igGetIO()).IniFilename = ini_filename.as_ptr() as _;
        }
        io.set_display_size(imgui::Vec2(f32::from(config.ui_width), f32::from(config.ui_height)));

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

        // for imgui callback
        struct GameViewData {
            renderer: *mut Renderer,
            x: i32,
            y: i32,
            w: u32,
            h: u32,
        }

        let mut keyboard_state = [KeyState::Neutral; 256];
        let mut mouse_state = [KeyState::Neutral; 3];
        //let mut will_scroll_up = false;
        //let mut will_scroll_down = false;
        let mut new_mouse_pos: Option<(i32, i32)> = None;
        let mut setting_mouse_pos = false;

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

        let save_paths = (0..16)
            .map(|i| {
                let mut path = project_path.clone();
                path.push(&format!("save{}.bin", i + 1));
                path
            })
            .collect::<Vec<_>>();

        let mut game_running = true; // false indicates the game closed or crashed, and so advancing is not allowed
        let mut err_string: Option<String> = None;

        let mut frame_text = String::from("Frame: 0");
        let mut seed_text = format!("Seed: {}", self.rand.seed());
        let mut rerecord_text = format!("Re-record count: {}", config.rerecords);
        let save_text = (0..16).map(|i| format!("Save {}", i + 1)).collect::<Vec<_>>();
        let load_text = (0..16).map(|i| format!("Load {}", i + 1)).collect::<Vec<_>>();
        let select_text = (0..16).map(|i| format!("Select###Select{}", i + 1)).collect::<Vec<_>>();
        let mut context_menu: Option<ContextMenu> = None;
        let mut savestate;
        let mut renderer_state;

        if !save_paths[config.quicksave_slot].exists() {
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
            savestate = SaveState::from(self, replay.clone(), renderer_state.clone());

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
        } else {
            match SaveState::from_file(&save_paths[config.quicksave_slot], &mut save_buffer) {
                Ok(state) => {
                    let (rep, ren) = state.clone().load_into(self);
                    replay = rep;
                    renderer_state = ren;

                    for (i, state) in keyboard_state.iter_mut().enumerate() {
                        *state =
                            if self.input.keyboard_check_direct(i as u8) { KeyState::Held } else { KeyState::Neutral };
                    }

                    for (i, state) in mouse_state.iter_mut().enumerate() {
                        *state =
                            if self.input.mouse_check_button(i as i8 + 1) { KeyState::Held } else { KeyState::Neutral };
                    }

                    frame_text = format!("Frame: {}", replay.frame_count());
                    seed_text = format!("Seed: {}", self.rand.seed());
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

        self.window.set_inner_size(Size::Physical(config.ui_width.into(), config.ui_height.into()));

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
        let mut instance_images: Vec<AtlasRef> = Vec::new();
        let mut new_rand: Option<Random> = None;
        let mut callback_data; // Putting this outside the loop makes sure it never goes out of scope

        /* ----------------------
        Frame loop begins here
        ---------------------- */

        'gui: loop {
            let time_start = Instant::now();

            // refresh io state
            let io = context.io();
            io.set_mouse_wheel(0.0);

            // poll window events
            self.window.swap_events();
            for event in self.window.events() {
                match event {
                    ev @ Event::KeyboardDown(key) | ev @ Event::KeyboardUp(key) => {
                        setting_mouse_pos = false;
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
                        .ok()
                        .and_then(|x| x.checked_sub(1))
                        .into_iter()
                        .for_each(|x| io.set_mouse_button(x, matches!(ev, Event::MouseDown(_)))),
                    Event::MouseWheel(delta) => io.set_mouse_wheel(delta.get() as f32 / 120.0),
                    Event::Resize((size, scale)) => {
                        let (width, height) = size.as_physical(*scale);
                        config.ui_width = u16::try_from(width).unwrap_or(u16::MAX);
                        config.ui_height = u16::try_from(height).unwrap_or(u16::MAX);
                        io.set_display_size(imgui::Vec2(width as f32, height as f32));
                        self.renderer.resize_framebuffer(width, height, false);
                        context_menu = None;
                    },
                    Event::Focus(false) => {
                        io.clear_inputs();
                        context_menu = None;
                    },
                    Event::CloseRequest(_) => break 'gui,
                    _ => (),
                }
            }

            // present imgui
            let fps_text = format!("FPS: {}", io.framerate().round());
            let win_frame_height = context.frame_height();
            let win_border_size = context.window_border_size();
            let win_padding = context.window_padding();
            let mut frame = context.new_frame();

            // Game window
            if game_running {
                if setting_mouse_pos {
                    frame.begin_screen_cover();
                    frame.end();
                    unsafe {
                        cimgui_sys::igSetNextWindowCollapsed(false, 0);
                        cimgui_sys::igSetNextWindowFocus();
                    }
                }

                let (w, h) = self.renderer.stored_size();
                frame.setup_next_window(imgui::Vec2(f32::from(config.ui_width) - w as f32 - 8.0, 8.0), None, None);
                frame.begin_window(
                    &format!("{}###Game", self.get_window_title()),
                    Some(imgui::Vec2(
                        w as f32 + (2.0 * win_border_size),
                        h as f32 + win_border_size + win_frame_height,
                    )),
                    false,
                    false,
                    None,
                );
                let imgui::Vec2(x, y) = frame.window_position();
                callback_data = GameViewData {
                    renderer: (&mut self.renderer) as *mut _,
                    x: (x + win_border_size) as i32,
                    y: (y + win_frame_height) as i32,
                    w: w,
                    h: h,
                };

                unsafe extern "C" fn callback(
                    _draw_list: *const cimgui_sys::ImDrawList,
                    ptr: *const cimgui_sys::ImDrawCmd,
                ) {
                    let data = &*((*ptr).UserCallbackData as *mut GameViewData);
                    (*data.renderer).draw_stored(data.x, data.y, data.w, data.h);
                }

                if !frame.window_collapsed() {
                    frame.callback(callback, &mut callback_data);

                    if setting_mouse_pos && frame.left_clicked() {
                        setting_mouse_pos = false;
                        let imgui::Vec2(mouse_x, mouse_y) = frame.mouse_pos();
                        new_mouse_pos =
                            Some((-(x + win_border_size - mouse_x) as i32, -(y + win_frame_height - mouse_y) as i32));
                    }

                    if frame.window_hovered() && frame.right_clicked() {
                        unsafe {
                            cimgui_sys::igSetWindowFocusNil();
                        }
                        let offset = frame.window_position() + imgui::Vec2(win_border_size, win_frame_height);
                        let imgui::Vec2(x, y) = frame.mouse_pos() - offset;
                        let (x, y) = self.translate_screen_to_room(x as _, y as _);

                        let mut options: Vec<(String, i32)> = Vec::new();
                        let mut iter = self.room.instance_list.iter_by_drawing();
                        while let Some(handle) = iter.next(&self.room.instance_list) {
                            let instance = self.room.instance_list.get(handle);
                            instance.update_bbox(self.get_instance_mask_sprite(handle));
                            if x >= instance.bbox_left.get()
                                && x <= instance.bbox_right.get()
                                && y >= instance.bbox_top.get()
                                && y <= instance.bbox_bottom.get()
                            {
                                use crate::game::GetAsset;
                                let id = instance.id.get();
                                let description = match self.assets.objects.get_asset(instance.object_index.get()) {
                                    Some(obj) => format!("{} ({})", obj.name, id.to_string()),
                                    None => format!("<deleted object> ({})", id.to_string()),
                                };
                                options.push((description, id));
                            }
                        }

                        if options.len() > 0 {
                            context_menu = Some(ContextMenu::Instances { pos: frame.mouse_pos(), options });
                        }
                    }
                }

                frame.end();
            } else {
                setting_mouse_pos = false;
            }

            // Control window
            frame.setup_next_window(imgui::Vec2(8.0, 8.0), None, None);
            frame.begin_window("Control", None, true, false, None);
            if (frame.button("Advance (Space)", imgui::Vec2(165.0, 20.0), None)
                || frame.key_pressed(input::ramen2vk(Key::Space)))
                && game_running
                && err_string.is_none()
            {
                let (w, h) = self.renderer.stored_size();
                let frame = replay.new_frame();

                self.input.mouse_step();
                for (i, state) in keyboard_state.iter().enumerate() {
                    let i = i as u8;
                    match state {
                        KeyState::NeutralWillPress => {
                            self.input.button_press(i, true);
                            frame.inputs.push(replay::Input::KeyPress(i));
                        },
                        KeyState::NeutralWillDouble | KeyState::NeutralDoubleEveryFrame => {
                            self.input.button_press(i, true);
                            self.input.button_release(i, true);
                            frame.inputs.push(replay::Input::KeyPress(i));
                            frame.inputs.push(replay::Input::KeyRelease(i));
                        },
                        KeyState::NeutralWillTriple => {
                            self.input.button_press(i, true);
                            self.input.button_release(i, true);
                            self.input.button_press(i, true);
                            frame.inputs.push(replay::Input::KeyPress(i));
                            frame.inputs.push(replay::Input::KeyRelease(i));
                            frame.inputs.push(replay::Input::KeyPress(i));
                        },
                        KeyState::HeldWillRelease | KeyState::NeutralWillCactus => {
                            self.input.button_release(i, true);
                            frame.inputs.push(replay::Input::KeyRelease(i));
                        },
                        KeyState::HeldWillDouble | KeyState::HeldDoubleEveryFrame => {
                            self.input.button_release(i, true);
                            self.input.button_press(i, true);
                            frame.inputs.push(replay::Input::KeyRelease(i));
                            frame.inputs.push(replay::Input::KeyPress(i));
                        },
                        KeyState::HeldWillTriple => {
                            self.input.button_release(i, true);
                            self.input.button_press(i, true);
                            self.input.button_release(i, true);
                            frame.inputs.push(replay::Input::KeyRelease(i));
                            frame.inputs.push(replay::Input::KeyPress(i));
                            frame.inputs.push(replay::Input::KeyRelease(i));
                        },
                        KeyState::Neutral | KeyState::Held => (),
                    }
                }

                for (i, state) in mouse_state.iter().enumerate() {
                    let i = i as i8 + 1;
                    match state {
                        KeyState::NeutralWillPress => {
                            self.input.mouse_press(i, true);
                            frame.inputs.push(replay::Input::MousePress(i));
                            println!("Pressed {}", i);
                        },
                        KeyState::NeutralWillDouble | KeyState::NeutralDoubleEveryFrame => {
                            self.input.mouse_press(i, true);
                            self.input.mouse_release(i, true);
                            frame.inputs.push(replay::Input::MousePress(i));
                            frame.inputs.push(replay::Input::MouseRelease(i));
                        },
                        KeyState::NeutralWillTriple => {
                            self.input.mouse_press(i, true);
                            self.input.mouse_release(i, true);
                            self.input.mouse_press(i, true);
                            frame.inputs.push(replay::Input::MousePress(i));
                            frame.inputs.push(replay::Input::MouseRelease(i));
                            frame.inputs.push(replay::Input::MousePress(i));
                        },
                        KeyState::HeldWillRelease | KeyState::NeutralWillCactus => {
                            self.input.mouse_release(i, true);
                            frame.inputs.push(replay::Input::MouseRelease(i));
                            println!("Released {}", i);
                        },
                        KeyState::HeldWillDouble | KeyState::HeldDoubleEveryFrame => {
                            self.input.mouse_release(i, true);
                            self.input.mouse_press(i, true);
                            frame.inputs.push(replay::Input::MouseRelease(i));
                            frame.inputs.push(replay::Input::MousePress(i));
                        },
                        KeyState::HeldWillTriple => {
                            self.input.mouse_release(i, true);
                            self.input.mouse_press(i, true);
                            self.input.mouse_release(i, true);
                            frame.inputs.push(replay::Input::MouseRelease(i));
                            frame.inputs.push(replay::Input::MousePress(i));
                            frame.inputs.push(replay::Input::MouseRelease(i));
                        },
                        KeyState::Neutral | KeyState::Held => (),
                    }
                }

                if let Some((x, y)) = new_mouse_pos {
                    frame.mouse_x = x;
                    frame.mouse_y = y;
                    self.input.mouse_move_to((x, y));
                }

                if let Some(rand) = new_rand {
                    frame.new_seed = Some(rand.seed());
                    self.rand.set_seed(rand.seed());
                }

                self.renderer.set_state(&renderer_state);
                self.renderer.resize_framebuffer(w, h, false);
                self.renderer.set_view(
                    0,
                    0,
                    self.unscaled_width as _,
                    self.unscaled_height as _,
                    0.0,
                    0,
                    0,
                    self.unscaled_width as _,
                    self.unscaled_height as _,
                );
                self.renderer.draw_stored(0, 0, w, h);
                if let Err(e) = match self.frame() {
                    Ok(()) => match self.scene_change {
                        Some(SceneChange::Room(id)) => self.load_room(id),
                        Some(SceneChange::Restart) => self.restart(),
                        Some(SceneChange::End) => self.restart(),
                        None => Ok(()),
                    },
                    Err(e) => Err(e.into()),
                } {
                    err_string = Some(format!("Game crashed: {}\n\nPlease load a savestate.", e));
                    game_running = false;
                }

                for ev in self.stored_events.iter() {
                    frame.events.push(ev.clone());
                }
                self.stored_events.clear();
                for (i, state) in keyboard_state.iter_mut().enumerate() {
                    state.reset_to(self.input.keyboard_check_direct(i as u8));
                }
                for (i, state) in mouse_state.iter_mut().enumerate() {
                    state.reset_to(self.input.mouse_check_button(i as i8 + 1));
                }

                // Fake frame limiter stuff (don't actually frame-limit in record mode)
                if let Some(t) = self.spoofed_time_nanos.as_mut() {
                    *t += Duration::new(0, 1_000_000_000u32 / self.room.speed).as_nanos();
                }
                if self.frame_counter == self.room.speed {
                    self.fps = self.room.speed;
                    self.frame_counter = 0;
                }
                self.frame_counter += 1;

                frame_text = format!("Frame: {}", replay.frame_count());
                seed_text = format!("Seed: {}", self.rand.seed());

                self.renderer.resize_framebuffer(config.ui_width.into(), config.ui_height.into(), true);
                self.renderer.set_view(
                    0,
                    0,
                    config.ui_width.into(),
                    config.ui_height.into(),
                    0.0,
                    0,
                    0,
                    config.ui_width.into(),
                    config.ui_height.into(),
                );
                self.renderer.clear_view(CLEAR_COLOUR, 1.0);
                renderer_state = self.renderer.state();
                self.renderer.set_state(&ui_renderer_state);
                context_menu = None;
                new_rand = None;
                new_mouse_pos = None;

                instance_reports =
                    config.watched_ids.iter().map(|id| (*id, InstanceReport::new(&*self, *id))).collect();
            }

            if (frame.button("Quick Save (Q)", imgui::Vec2(165.0, 20.0), None)
                || frame.key_pressed(input::ramen2vk(Key::Q)))
                && game_running
                && err_string.is_none()
            {
                savestate = SaveState::from(self, replay.clone(), renderer_state.clone());
                if let Err(err) = savestate.save_to_file(&save_paths[config.quicksave_slot], &mut save_buffer) {
                    err_string = Some(format!(
                        concat!(
                            "Warning: failed to save quicksave.bin (it has still been saved in memory)\n\n",
                            "Error message: {:?}",
                        ),
                        err,
                    ));
                }
                context_menu = None;
            }

            if frame.button("Load Quicksave (W)", imgui::Vec2(165.0, 20.0), None)
                || frame.key_pressed(input::ramen2vk(Key::W))
            {
                if startup_successful {
                    err_string = None;
                    game_running = true;
                    let (rep, ren) = savestate.clone().load_into(self);
                    replay = rep;
                    renderer_state = ren;

                    for (i, state) in keyboard_state.iter_mut().enumerate() {
                        *state =
                            if self.input.keyboard_check_direct(i as u8) { KeyState::Held } else { KeyState::Neutral };
                    }

                    for (i, state) in mouse_state.iter_mut().enumerate() {
                        *state =
                            if self.input.mouse_check_button(i as i8 + 1) { KeyState::Held } else { KeyState::Neutral };
                    }

                    frame_text = format!("Frame: {}", replay.frame_count());
                    seed_text = format!("Seed: {}", self.rand.seed());
                    context_menu = None;
                    new_rand = None;
                    new_mouse_pos = None;
                    instance_reports =
                        config.watched_ids.iter().map(|id| (*id, InstanceReport::new(&*self, *id))).collect();
                    config.rerecords += 1;
                    rerecord_text = format!("Re-record count: {}", config.rerecords);
                    let _ = File::create(&config_path).map(|f| bincode::serialize_into(f, &config));
                }
            }

            if frame.button("Export to .gmtas", imgui::Vec2(165.0, 20.0), None) {
                let mut filepath = project_path.clone();
                filepath.push("save.gmtas");
                match replay.to_file(&filepath) {
                    Ok(()) => (),
                    Err(replay::WriteError::IOErr(err)) => {
                        err_string = Some(format!("Failed to write save.gmtas: {}", err))
                    },
                    Err(replay::WriteError::CompressErr(err)) => {
                        err_string = Some(format!("Failed to compress save.gmtas: {}", err))
                    },
                    Err(replay::WriteError::SerializeErr(err)) => {
                        err_string = Some(format!("Failed to serialize save.gmtas: {}", err))
                    },
                }
            }

            frame.text(&frame_text);
            if new_rand.is_some() {
                frame.coloured_text(&seed_text, Colour::new(1.0, 0.5, 0.5));
            } else {
                frame.text(&seed_text);
            }
            frame.text(&rerecord_text);
            frame.text(&fps_text);

            let keyboard_label = if config.full_keyboard {
                "Simple Keyboard###KeyboardLayout"
            } else {
                "Full Keyboard###KeyboardLayout"
            };
            if frame.button(keyboard_label, imgui::Vec2(165.0, 20.0), None) {
                config.full_keyboard = !config.full_keyboard;
                let _ = File::create(&config_path).map(|f| bincode::serialize_into(f, &config));
            }

            let input_label = match config.input_mode {
                InputMode::Direct => "Switch to mouse input###InputMethod",
                InputMode::Mouse => "Switch to direct input###InputMethod",
            };
            if frame.button(input_label, imgui::Vec2(165.0, 20.0), None) {
                config.input_mode = match config.input_mode {
                    InputMode::Mouse => InputMode::Direct,
                    InputMode::Direct => InputMode::Mouse,
                }
            }

            if frame.button(">", imgui::Vec2(18.0, 18.0), Some(imgui::Vec2(160.0, 138.0))) {
                if let Some(rand) = &mut new_rand {
                    rand.cycle();
                    seed_text = format!("Seed: {}*", rand.seed());
                } else {
                    let mut rand = self.rand.clone();
                    rand.cycle();
                    seed_text = format!("Seed: {}*", rand.seed());
                    new_rand = Some(rand);
                }
            }
            if frame.item_hovered() && frame.right_clicked() {
                context_menu = Some(ContextMenu::Seed { pos: frame.mouse_pos() });
            }
            frame.end();

            // Savestates window
            frame.setup_next_window(imgui::Vec2(306.0, 8.0), Some(imgui::Vec2(225.0, 330.0)), None);
            frame.begin_window("Savestates", None, true, false, None);
            let rect_size = imgui::Vec2(frame.window_size().0, 24.0);
            let pos = frame.window_position() + frame.content_position() - imgui::Vec2(8.0, 8.0);
            for i in 0..8 {
                let min = imgui::Vec2(0.0, ((i * 2 + 1) * 24) as f32);
                frame.rect(min + pos, min + rect_size + pos, Colour::new(1.0, 1.0, 1.0), 15);
            }
            for i in 0..16 {
                unsafe {
                    cimgui_sys::igPushStyleColorVec4(cimgui_sys::ImGuiCol__ImGuiCol_Button as _, cimgui_sys::ImVec4 {
                        x: 0.98,
                        y: 0.59,
                        z: 0.26,
                        w: 0.4,
                    });
                    cimgui_sys::igPushStyleColorVec4(
                        cimgui_sys::ImGuiCol__ImGuiCol_ButtonHovered as _,
                        cimgui_sys::ImVec4 { x: 0.98, y: 0.59, z: 0.26, w: 1.0 },
                    );
                    cimgui_sys::igPushStyleColorVec4(
                        cimgui_sys::ImGuiCol__ImGuiCol_ButtonActive as _,
                        cimgui_sys::ImVec4 { x: 0.98, y: 0.53, z: 0.06, w: 1.0 },
                    );
                }
                let y = (24 * i + 21) as f32;
                if i == config.quicksave_slot {
                    let min = imgui::Vec2(0.0, (i * 24) as f32);
                    frame.rect(min + pos, min + rect_size + pos, Colour::new(0.1, 0.4, 0.2), 255);
                }
                if frame.button(&save_text[i], imgui::Vec2(60.0, 20.0), Some(imgui::Vec2(4.0, y))) && game_running {
                    match SaveState::from(self, replay.clone(), renderer_state.clone())
                        .save_to_file(&save_paths[i], &mut save_buffer)
                    {
                        Ok(()) => (),
                        Err(savestate::WriteError::IOErr(err)) => {
                            err_string = Some(format!("Failed to write savestate #{}: {}", i, err))
                        },
                        Err(savestate::WriteError::CompressErr(err)) => {
                            err_string = Some(format!("Failed to compress savestate #{}: {}", i, err))
                        },
                        Err(savestate::WriteError::SerializeErr(err)) => {
                            err_string = Some(format!("Failed to serialize savestate #{}: {}", i, err))
                        },
                    }
                }
                unsafe {
                    cimgui_sys::igPopStyleColor(3);
                }

                if save_paths[i].exists() {
                    if frame.button(&load_text[i], imgui::Vec2(60.0, 20.0), Some(imgui::Vec2(75.0, y)))
                        && startup_successful
                    {
                        match SaveState::from_file(&save_paths[i], &mut save_buffer) {
                            Ok(state) => {
                                let (new_replay, new_renderer_state) = state.load_into(self);
                                replay = new_replay;
                                renderer_state = new_renderer_state;

                                for (i, state) in keyboard_state.iter_mut().enumerate() {
                                    *state = if self.input.keyboard_check_direct(i as u8) {
                                        KeyState::Held
                                    } else {
                                        KeyState::Neutral
                                    };
                                }
                                for (i, state) in mouse_state.iter_mut().enumerate() {
                                    *state = if self.input.mouse_check_button(i as i8 + 1) {
                                        KeyState::Held
                                    } else {
                                        KeyState::Neutral
                                    };
                                }

                                frame_text = format!("Frame: {}", replay.frame_count());
                                seed_text = format!("Seed: {}", self.rand.seed());
                                context_menu = None;
                                new_rand = None;
                                new_mouse_pos = None;
                                err_string = None;
                                game_running = true;
                                config.rerecords += 1;
                                rerecord_text = format!("Re-record count: {}", config.rerecords);
                                let _ = File::create(&config_path).map(|f| bincode::serialize_into(f, &config));
                            },
                            Err(err) => {
                                let filename = save_paths[i].to_string_lossy();
                                err_string = Some(match err {
                                    savestate::ReadError::IOErr(err) => {
                                        format!("Error reading {}:\n\n{}", filename, err)
                                    },
                                    savestate::ReadError::DecompressErr(err) => {
                                        format!("Error decompressing {}:\n\n{}", filename, err)
                                    },
                                    savestate::ReadError::DeserializeErr(err) => {
                                        format!("Error deserializing {}:\n\n{}", filename, err)
                                    },
                                });
                            },
                        }
                        instance_reports =
                            config.watched_ids.iter().map(|id| (*id, InstanceReport::new(&*self, *id))).collect();
                    }

                    if frame.button(&select_text[i], imgui::Vec2(60.0, 20.0), Some(imgui::Vec2(146.0, y)))
                        && config.quicksave_slot != i
                    {
                        //                        config.quicksave_slot = i;
                        match SaveState::from_file(&save_paths[i], &mut save_buffer) {
                            Ok(state) => {
                                savestate = state;
                                config.quicksave_slot = i;
                                let _ = File::create(&config_path).map(|f| bincode::serialize_into(f, &config));
                            },
                            Err(e) => {
                                println!(
                                    "Error: Failed to select quicksave slot {:?}. {:?}",
                                    save_paths[i].file_name(),
                                    e
                                );
                            },
                        }
                    }
                }
            }
            frame.end();

            // Macro for keyboard keys and mouse buttons...
            macro_rules! kb_btn {
                ($name: expr, $size: expr, $x: expr, $y: expr, key $code: expr) => {
                    let vk = input::ramen2vk($code);
                    let state = &mut keyboard_state[usize::from(vk)];
                    let clicked = frame.invisible_button($name, $size, Some(imgui::Vec2($x, $y)));
                    let hovered = frame.item_hovered();
                    match config.input_mode {
                        InputMode::Mouse => {
                            if clicked {
                                state.click();
                            }
                            if frame.right_clicked() && hovered {
                                unsafe {
                                    cimgui_sys::igSetWindowFocusNil();
                                }
                                context_menu = Some(ContextMenu::Button { pos: frame.mouse_pos(), key: $code });
                            }
                            if frame.middle_clicked() && hovered {
                                unsafe {
                                    cimgui_sys::igSetWindowFocusNil();
                                }
                                *state = if state.is_held() {
                                    KeyState::HeldWillDouble
                                } else {
                                    KeyState::NeutralWillDouble
                                };
                            }
                        },
                        InputMode::Direct => {
                            if frame.key_pressed(vk) {
                                *state = match state {
                                    // if neutral and setting would stay neutral => will press
                                    KeyState::Neutral | KeyState::NeutralWillDouble | KeyState::NeutralWillCactus => {
                                        KeyState::NeutralWillPress
                                    },
                                    // if held but would release => keep held
                                    KeyState::HeldWillRelease | KeyState::HeldWillTriple => KeyState::Held,
                                    // otherwise just keep the state
                                    _ => *state,
                                };
                            } else if frame.key_released(vk) {
                                *state = match state {
                                    // if held and setting would stay held => will release
                                    KeyState::Held | KeyState::HeldWillDouble | KeyState::HeldDoubleEveryFrame => {
                                        KeyState::HeldWillRelease
                                    },
                                    // if neutral but would press => keep neutral
                                    KeyState::NeutralWillPress | KeyState::NeutralWillTriple => KeyState::Neutral,
                                    // otherwise just keep the state
                                    _ => *state,
                                };
                            }
                        },
                    }
                    draw_keystate(&mut frame, state, imgui::Vec2($x, $y), $size);
                    frame.text_centered($name, imgui::Vec2($x, $y) + imgui::Vec2($size.0 / 2.0, $size.1 / 2.0));
                    if hovered {
                        unsafe {
                            cimgui_sys::igSetCursorPos(cimgui_sys::ImVec2 { x: 8.0, y: 22.0 });
                        }
                        frame.text(state.repr());
                    }
                };

                ($name: expr, $size: expr, $x: expr, $y: expr, mouse $code: expr) => {
                    let state = &mut mouse_state[$code as usize];
                    if frame.invisible_button($name, $size, Some(imgui::Vec2($x, $y))) {
                        state.click();
                    }
                    let hovered = frame.item_hovered();
                    if frame.right_clicked() && hovered {
                        unsafe {
                            cimgui_sys::igSetWindowFocusNil();
                        }
                        context_menu = Some(ContextMenu::MouseButton { pos: frame.mouse_pos(), button: $code });
                    }
                    if frame.middle_clicked() && hovered {
                        unsafe {
                            cimgui_sys::igSetWindowFocusNil();
                        }
                        *state = if state.is_held() { KeyState::HeldWillDouble } else { KeyState::NeutralWillDouble };
                    }
                    draw_keystate(&mut frame, state, imgui::Vec2($x, $y), $size);
                    frame.text_centered($name, imgui::Vec2($x, $y) + imgui::Vec2($size.0 / 2.0, $size.1 / 2.0));
                    if hovered {
                        unsafe {
                            cimgui_sys::igSetCursorPos(cimgui_sys::ImVec2 { x: 8.0, y: 22.0 });
                        }
                        frame.text(state.repr());
                    }
                };

                ($name: expr, $size: expr, $x: expr, $y: expr) => {
                    let pos = frame.window_position();
                    frame.invisible_button($name, $size, Some(imgui::Vec2($x, $y)));
                    frame.rect(imgui::Vec2($x, $y) + pos, imgui::Vec2($x, $y) + $size + pos, BTN_NEUTRAL_COL, 190);
                    frame.rect_outline(
                        imgui::Vec2($x, $y) + pos,
                        imgui::Vec2($x, $y) + $size + pos,
                        Colour::new(0.4, 0.4, 0.65),
                        u8::MAX,
                    );
                    frame.text_centered($name, imgui::Vec2($x, $y) + imgui::Vec2($size.0 / 2.0, $size.1 / 2.0));
                };
            }

            // Keyboard window
            if config.full_keyboard {
                frame.setup_next_window(
                    imgui::Vec2(8.0, 350.0),
                    Some(imgui::Vec2(917.0, 362.0)),
                    Some(imgui::Vec2(440.0, 200.0)),
                );
                frame.begin_window("Keyboard###FullKeyboard", None, true, false, None);
                if !frame.window_collapsed() {
                    frame.rect(
                        imgui::Vec2(0.0, win_frame_height) + frame.window_position(),
                        imgui::Vec2(frame.window_size().0, win_frame_height + 20.0) + frame.window_position(),
                        Colour::new(0.14, 0.14, 0.14),
                        255,
                    );
                    let content_min = win_padding + imgui::Vec2(0.0, win_frame_height * 2.0);
                    let content_max = frame.window_size() - win_padding;

                    let mut cur_x = content_min.0;
                    let mut cur_y = content_min.1;
                    let left_part_edge = ((content_max.0 - content_min.0) * (15.0 / 18.5)).floor();
                    let button_width = ((left_part_edge - content_min.0 - 14.0) / 15.0).floor();
                    let button_height = ((content_max.1 - content_min.1 - 4.0 - (win_padding.1 * 2.0)) / 6.5).floor();
                    let button_size = imgui::Vec2(button_width, button_height);
                    kb_btn!(
                        "Esc",
                        imgui::Vec2((button_width * 1.5).floor(), button_height),
                        cur_x,
                        cur_y,
                        key Key::Escape
                    );
                    cur_x = left_part_edge - (button_width * 12.0 + 11.0);
                    kb_btn!("F1", button_size, cur_x, cur_y, key Key::F1);
                    cur_x += button_width + 1.0;
                    kb_btn!("F2", button_size, cur_x, cur_y, key Key::F2);
                    cur_x += button_width + 1.0;
                    kb_btn!("F3", button_size, cur_x, cur_y, key Key::F3);
                    cur_x += button_width + 1.0;
                    kb_btn!("F4", button_size, cur_x, cur_y, key Key::F4);
                    cur_x += button_width + 1.0;
                    kb_btn!("F5", button_size, cur_x, cur_y, key Key::F5);
                    cur_x += button_width + 1.0;
                    kb_btn!("F6", button_size, cur_x, cur_y, key Key::F6);
                    cur_x += button_width + 1.0;
                    kb_btn!("F7", button_size, cur_x, cur_y, key Key::F7);
                    cur_x += button_width + 1.0;
                    kb_btn!("F8", button_size, cur_x, cur_y, key Key::F8);
                    cur_x += button_width + 1.0;
                    kb_btn!("F9", button_size, cur_x, cur_y, key Key::F9);
                    cur_x += button_width + 1.0;
                    kb_btn!("F10", button_size, cur_x, cur_y, key Key::F10);
                    cur_x += button_width + 1.0;
                    kb_btn!("F11", button_size, cur_x, cur_y, key Key::F11);
                    cur_x += button_width + 1.0;
                    kb_btn!("F12", button_size, cur_x, cur_y, key Key::F12);
                    cur_x = content_max.0 - (button_width * 3.0 + 2.0);
                    kb_btn!("PrSc", button_size, cur_x, cur_y, key Key::PrintScreen);
                    cur_x += button_width + 1.0;
                    kb_btn!("ScrLk", button_size, cur_x, cur_y, key Key::ScrollLock);
                    cur_x += button_width + 1.0;
                    kb_btn!("Pause", button_size, cur_x, cur_y, key Key::Pause);
                    cur_x = content_min.0;
                    cur_y = (content_max.1 - (win_padding.1 * 2.0)).ceil() - (button_height * 5.0 + 4.0);
                    kb_btn!("`", button_size, cur_x, cur_y);
                    cur_x += button_width + 1.0;
                    kb_btn!("1", button_size, cur_x, cur_y, key Key::Num1);
                    cur_x += button_width + 1.0;
                    kb_btn!("2", button_size, cur_x, cur_y, key Key::Num2);
                    cur_x += button_width + 1.0;
                    kb_btn!("3", button_size, cur_x, cur_y, key Key::Num3);
                    cur_x += button_width + 1.0;
                    kb_btn!("4", button_size, cur_x, cur_y, key Key::Num4);
                    cur_x += button_width + 1.0;
                    kb_btn!("5", button_size, cur_x, cur_y, key Key::Num5);
                    cur_x += button_width + 1.0;
                    kb_btn!("6", button_size, cur_x, cur_y, key Key::Num6);
                    cur_x += button_width + 1.0;
                    kb_btn!("7", button_size, cur_x, cur_y, key Key::Num7);
                    cur_x += button_width + 1.0;
                    kb_btn!("8", button_size, cur_x, cur_y, key Key::Num8);
                    cur_x += button_width + 1.0;
                    kb_btn!("9", button_size, cur_x, cur_y, key Key::Num9);
                    cur_x += button_width + 1.0;
                    kb_btn!("0", button_size, cur_x, cur_y, key Key::Num0);
                    cur_x += button_width + 1.0;
                    kb_btn!("-", button_size, cur_x, cur_y, key Key::Subtract);
                    cur_x += button_width + 1.0;
                    kb_btn!("=", button_size, cur_x, cur_y);
                    cur_x += button_width + 1.0;
                    kb_btn!(
                        "Back",
                        imgui::Vec2(left_part_edge - cur_x, button_height),
                        cur_x,
                        cur_y,
                        key Key::Backspace
                    );
                    cur_x = content_max.0 - (button_width * 3.0 + 2.0);
                    kb_btn!("Ins", button_size, cur_x, cur_y, key Key::Insert);
                    cur_x += button_width + 1.0;
                    kb_btn!("Home", button_size, cur_x, cur_y, key Key::Home);
                    cur_x += button_width + 1.0;
                    kb_btn!("PgUp", button_size, cur_x, cur_y, key Key::PageUp);
                    cur_x = content_min.0;
                    cur_y += button_height + 1.0;
                    kb_btn!(
                        "Tab",
                        imgui::Vec2((button_width * 1.5).floor(), button_height),
                        cur_x,
                        cur_y,
                        key Key::Tab
                    );
                    cur_x += (button_width * 1.5).floor() + 1.0;
                    kb_btn!("Q", button_size, cur_x, cur_y, key Key::Q);
                    cur_x += button_width + 1.0;
                    kb_btn!("W", button_size, cur_x, cur_y, key Key::W);
                    cur_x += button_width + 1.0;
                    kb_btn!("E", button_size, cur_x, cur_y, key Key::E);
                    cur_x += button_width + 1.0;
                    kb_btn!("R", button_size, cur_x, cur_y, key Key::R);
                    cur_x += button_width + 1.0;
                    kb_btn!("T", button_size, cur_x, cur_y, key Key::T);
                    cur_x += button_width + 1.0;
                    kb_btn!("Y", button_size, cur_x, cur_y, key Key::Y);
                    cur_x += button_width + 1.0;
                    kb_btn!("U", button_size, cur_x, cur_y, key Key::U);
                    cur_x += button_width + 1.0;
                    kb_btn!("I", button_size, cur_x, cur_y, key Key::I);
                    cur_x += button_width + 1.0;
                    kb_btn!("O", button_size, cur_x, cur_y, key Key::O);
                    cur_x += button_width + 1.0;
                    kb_btn!("P", button_size, cur_x, cur_y, key Key::P);
                    cur_x += button_width + 1.0;
                    kb_btn!("[", button_size, cur_x, cur_y);
                    cur_x += button_width + 1.0;
                    kb_btn!("]", button_size, cur_x, cur_y);
                    cur_x += button_width + 1.0;
                    kb_btn!(
                        "Enter",
                        imgui::Vec2(left_part_edge - cur_x, button_height * 2.0 + 1.0),
                        cur_x,
                        cur_y,
                        key Key::Enter
                    );
                    cur_x = content_max.0 - (button_width * 3.0 + 2.0);
                    kb_btn!("Del", button_size, cur_x, cur_y, key Key::Delete);
                    cur_x += button_width + 1.0;
                    kb_btn!("End", button_size, cur_x, cur_y, key Key::End);
                    cur_x += button_width + 1.0;
                    kb_btn!("PgDn", button_size, cur_x, cur_y, key Key::PageDown);
                    cur_x = content_min.0;
                    cur_y += button_height + 1.0;
                    kb_btn!(
                        "Caps",
                        imgui::Vec2((button_width * 1.5).floor(), button_height),
                        cur_x,
                        cur_y,
                        key Key::CapsLock
                    );
                    cur_x += (button_width * 1.5).floor() + 1.0;
                    kb_btn!("A", button_size, cur_x, cur_y, key Key::A);
                    cur_x += button_width + 1.0;
                    kb_btn!("S", button_size, cur_x, cur_y, key Key::S);
                    cur_x += button_width + 1.0;
                    kb_btn!("D", button_size, cur_x, cur_y, key Key::D);
                    cur_x += button_width + 1.0;
                    kb_btn!("F", button_size, cur_x, cur_y, key Key::F);
                    cur_x += button_width + 1.0;
                    kb_btn!("G", button_size, cur_x, cur_y, key Key::G);
                    cur_x += button_width + 1.0;
                    kb_btn!("H", button_size, cur_x, cur_y, key Key::H);
                    cur_x += button_width + 1.0;
                    kb_btn!("J", button_size, cur_x, cur_y, key Key::J);
                    cur_x += button_width + 1.0;
                    kb_btn!("K", button_size, cur_x, cur_y, key Key::K);
                    cur_x += button_width + 1.0;
                    kb_btn!("L", button_size, cur_x, cur_y, key Key::L);
                    cur_x += button_width + 1.0;
                    kb_btn!(";", button_size, cur_x, cur_y);
                    cur_x += button_width + 1.0;
                    kb_btn!("'", button_size, cur_x, cur_y);
                    cur_x += button_width + 1.0;
                    kb_btn!("#", button_size, cur_x, cur_y);
                    cur_x = content_min.0;
                    cur_y += button_height + 1.0;
                    kb_btn!("Shift", imgui::Vec2(button_width * 2.0, button_height), cur_x, cur_y, key Key::LShift);
                    cur_x += button_width * 2.0 + 1.0;
                    kb_btn!("\\", button_size, cur_x, cur_y);
                    cur_x += button_width + 1.0;
                    kb_btn!("Z", button_size, cur_x, cur_y, key Key::Z);
                    cur_x += button_width + 1.0;
                    kb_btn!("X", button_size, cur_x, cur_y, key Key::X);
                    cur_x += button_width + 1.0;
                    kb_btn!("C", button_size, cur_x, cur_y, key Key::C);
                    cur_x += button_width + 1.0;
                    kb_btn!("V", button_size, cur_x, cur_y, key Key::V);
                    cur_x += button_width + 1.0;
                    kb_btn!("B", button_size, cur_x, cur_y, key Key::B);
                    cur_x += button_width + 1.0;
                    kb_btn!("N", button_size, cur_x, cur_y, key Key::N);
                    cur_x += button_width + 1.0;
                    kb_btn!("M", button_size, cur_x, cur_y, key Key::M);
                    cur_x += button_width + 1.0;
                    kb_btn!(",", button_size, cur_x, cur_y, key Key::Comma);
                    cur_x += button_width + 1.0;
                    kb_btn!(".", button_size, cur_x, cur_y, key Key::Period);
                    cur_x += button_width + 1.0;
                    kb_btn!("/", button_size, cur_x, cur_y);
                    cur_x += button_width + 1.0;
                    kb_btn!(
                        "RShift",
                        imgui::Vec2(left_part_edge - cur_x, button_height),
                        cur_x,
                        cur_y,
                        key Key::RShift
                    );
                    cur_x = content_min.0;
                    cur_y += button_height + 1.0;
                    kb_btn!(
                        "Ctrl",
                        imgui::Vec2((button_width * 1.5).floor(), button_height),
                        cur_x,
                        cur_y,
                        key Key::LControl
                    );
                    cur_x += (button_width * 1.5).floor() + 1.0;
                    kb_btn!("Win", button_size, cur_x, cur_y, key Key::LSuper);
                    cur_x += button_width + 1.0;
                    kb_btn!("Alt", button_size, cur_x, cur_y, key Key::LAlt);
                    cur_x += button_width + 1.0;
                    kb_btn!(
                        "Space",
                        imgui::Vec2((left_part_edge - cur_x) - (button_width * 3.5 + 3.0).floor(), button_height),
                        cur_x,
                        cur_y,
                        key Key::Space
                    );
                    cur_x = left_part_edge - (button_width * 3.5 + 2.0).floor();
                    kb_btn!("RAlt", button_size, cur_x, cur_y, key Key::RAlt);
                    cur_x += button_width + 1.0;
                    kb_btn!("Pg", button_size, cur_x, cur_y, key Key::Applications);
                    cur_x += button_width + 1.0;
                    kb_btn!(
                        "RCtrl",
                        imgui::Vec2(left_part_edge - cur_x, button_height),
                        cur_x,
                        cur_y,
                        key Key::RControl
                    );
                    cur_x = content_max.0 - (button_width * 3.0 + 2.0);
                    kb_btn!("<", button_size, cur_x, cur_y, key Key::Left);
                    cur_x += button_width + 1.0;
                    kb_btn!("v", button_size, cur_x, cur_y, key Key::Down);
                    cur_y -= button_height + 1.0;
                    kb_btn!("^", button_size, cur_x, cur_y, key Key::Up);
                    cur_x += button_width + 1.0;
                    cur_y += button_height + 1.0;
                    kb_btn!(">", button_size, cur_x, cur_y, key Key::Right);
                }
                frame.end();
            } else {
                frame.setup_next_window(
                    imgui::Vec2(50.0, 354.0),
                    Some(imgui::Vec2(365.0, 192.0)),
                    Some(imgui::Vec2(201.0, 122.0)),
                );
                frame.begin_window("Keyboard###SimpleKeyboard", None, true, false, None);
                if !frame.window_collapsed() {
                    frame.rect(
                        imgui::Vec2(0.0, win_frame_height) + frame.window_position(),
                        imgui::Vec2(frame.window_size().0, win_frame_height + 20.0) + frame.window_position(),
                        Colour::new(0.14, 0.14, 0.14),
                        255,
                    );
                    let content_min = win_padding + imgui::Vec2(0.0, win_frame_height * 2.0);
                    let content_max = frame.window_size() - win_padding;

                    let button_width = (((content_max.0 - content_min.0) - 2.0) / 6.0).floor();
                    let button_height = ((content_max.1 - content_min.1) / 2.5).floor();
                    let button_size = imgui::Vec2(button_width, button_height);
                    let arrows_left_bound =
                        content_min.0 + ((content_max.0 - content_min.0) / 2.0 - (button_width * 1.5)).floor();
                    kb_btn!("<", button_size, arrows_left_bound, content_max.1 - button_height - 8.0, key Key::Left);
                    kb_btn!(
                        "v",
                        button_size,
                        arrows_left_bound + button_width + 1.0,
                        content_max.1 - button_height - 8.0,
                        key Key::Down
                    );
                    kb_btn!(
                        ">",
                        button_size,
                        arrows_left_bound + (button_width * 2.0 + 2.0),
                        content_max.1 - button_height - 8.0,
                        key Key::Right
                    );
                    kb_btn!(
                        "^",
                        button_size,
                        arrows_left_bound + button_width + 1.0,
                        content_max.1 - (button_height * 2.0) - 9.0,
                        key Key::Up
                    );
                    kb_btn!("R", button_size, content_min.0, content_min.1, key Key::R);
                    kb_btn!("Shift", button_size, content_min.0, content_max.1 - button_height - 8.0, key Key::LShift);
                    kb_btn!("F2", button_size, content_max.0 - button_width, content_min.1, key Key::F2);
                    kb_btn!(
                        "Z",
                        button_size,
                        content_max.0 - button_width,
                        content_max.1 - button_height - 8.0,
                        key Key::Z
                    );
                }
                frame.end();
            }

            // Mouse input window
            frame.setup_next_window(imgui::Vec2(2.0, 210.0), None, None);
            frame.begin_window("Mouse", Some(imgui::Vec2(300.0, 138.0)), false, false, None);
            if !frame.window_collapsed() {
                frame.rect(
                    imgui::Vec2(0.0, win_frame_height) + frame.window_position(),
                    imgui::Vec2(frame.window_size().0, win_frame_height + 20.0) + frame.window_position(),
                    Colour::new(0.14, 0.14, 0.14),
                    255,
                );

                let button_size = imgui::Vec2(40.0, 40.0);
                kb_btn!("Left", button_size, 4.0, 65.0, mouse 0);
                kb_btn!("Middle", button_size, 48.0, 65.0, mouse 2);
                kb_btn!("Right", button_size, 92.0, 65.0, mouse 1);
                if frame.button("Set Mouse", imgui::Vec2(150.0, 20.0), Some(imgui::Vec2(150.0, 50.0))) {
                    if game_running {
                        setting_mouse_pos = true;
                    } else {
                        err_string = Some("The game is not running. Please load a savestate.".into());
                    }
                }

                if let Some((x, y)) = new_mouse_pos {
                    unsafe {
                        cimgui_sys::igPushStyleColorVec4(
                            cimgui_sys::ImGuiCol__ImGuiCol_Text as _,
                            cimgui_sys::ImVec4 { x: 1.0, y: 0.5, z: 0.5, w: 1.0 },
                        );
                    }
                    frame.text_centered(&format!("x: {}*", x), imgui::Vec2(225.0, 80.0));
                    frame.text_centered(&format!("y: {}*", y), imgui::Vec2(225.0, 96.0));
                    unsafe {
                        cimgui_sys::igPopStyleColor(1);
                    }
                } else {
                    frame.text_centered(&format!("x: {}", self.input.mouse_x()), imgui::Vec2(225.0, 80.0));
                    frame.text_centered(&format!("y: {}", self.input.mouse_y()), imgui::Vec2(225.0, 96.0));
                }
            }
            frame.end();

            // Instance-watcher windows
            let previous_len = config.watched_ids.len();
            instance_images.clear();
            instance_images.reserve(config.watched_ids.len());
            config.watched_ids.retain(|id| {
                let mut open = true;
                frame.begin_window(&format!("Instance {}", id), None, true, false, Some(&mut open));
                if let Some((_, Some(report))) = instance_reports.iter().find(|(i, _)| i == id) {
                    frame.text(&report.object_name);
                    frame.text(&report.id);
                    frame.text("");
                    if frame.begin_tree_node("General Variables") {
                        report.general_vars.iter().for_each(|s| frame.text(s));
                        frame.pop_tree_node();
                    }
                    if frame.begin_tree_node("Physics Variables") {
                        report.physics_vars.iter().for_each(|s| frame.text(s));
                        frame.pop_tree_node();
                    }
                    if frame.begin_tree_node("Image Variables") {
                        report.image_vars.iter().for_each(|s| frame.text(s));
                        frame.pop_tree_node();
                    }
                    if frame.begin_tree_node("Timeline Variables") {
                        report.timeline_vars.iter().for_each(|s| frame.text(s));
                        frame.pop_tree_node();
                    }
                    if frame.begin_tree_node("Alarms") {
                        report.alarms.iter().for_each(|s| frame.text(s));
                        frame.pop_tree_node();
                    }
                    if frame.begin_tree_node("Fields") {
                        report.fields.iter().for_each(|f| match f {
                            ReportField::Single(s) => frame.text(s),
                            ReportField::Array(label, array) => {
                                if frame.begin_tree_node(label) {
                                    array.iter().for_each(|s| frame.text(s));
                                    frame.pop_tree_node();
                                }
                            },
                        });
                        frame.pop_tree_node();
                    }
                    if let Some(handle) = self.room.instance_list.get_by_instid(*id) {
                        use crate::game::GetAsset;
                        let instance = self.room.instance_list.get(handle);
                        if let Some(atlas_ref) = self
                            .assets
                            .sprites
                            .get_asset(instance.sprite_index.get())
                            .and_then(|x| x.get_atlas_ref(instance.image_index.get().floor().to_i32()))
                        {
                            if atlas_ref.w <= 48 && atlas_ref.h <= 48 {
                                let i = instance_images.len();
                                instance_images.push(*atlas_ref);
                                let imgui::Vec2(win_x, win_y) = frame.window_position();
                                let win_w = frame.window_size().0;
                                let center_x = win_x + win_w - 28.0;
                                let center_y = win_y + 46.0;
                                let min_x = center_x - (atlas_ref.w / 2) as f32;
                                let min_y = center_y - (atlas_ref.h / 2) as f32;
                                unsafe {
                                    cimgui_sys::ImDrawList_AddImage(
                                        cimgui_sys::igGetWindowDrawList(),
                                        instance_images.as_mut_ptr().add(i) as _,
                                        cimgui_sys::ImVec2 { x: min_x, y: min_y },
                                        cimgui_sys::ImVec2 {
                                            x: min_x + atlas_ref.w as f32,
                                            y: min_y + atlas_ref.h as f32,
                                        },
                                        cimgui_sys::ImVec2 { x: 0.0, y: 0.0 },
                                        cimgui_sys::ImVec2 { x: 1.0, y: 1.0 },
                                        instance.image_blend.get() as u32 | 0xFF000000,
                                    );
                                }
                            }
                        }
                    }
                } else {
                    frame.text_centered("<deleted instance>", imgui::Vec2(160.0, 35.0));
                }
                frame.end();
                open
            });

            if config.watched_ids.len() != previous_len {
                instance_reports =
                    config.watched_ids.iter().map(|id| (*id, InstanceReport::new(&*self, *id))).collect();
                let _ = File::create(&config_path).map(|f| bincode::serialize_into(f, &config));
            }

            // Context menu windows (aka right-click menus)
            match &context_menu {
                Some(ContextMenu::Button { pos, key }) => {
                    let key_state = &mut keyboard_state[usize::from(input::ramen2vk(*key))];
                    if !key_state.menu(&mut frame, *pos) {
                        context_menu = None;
                    }
                },
                Some(ContextMenu::MouseButton { pos, button }) => {
                    let key_state = &mut mouse_state[*button as usize];
                    if !key_state.menu(&mut frame, *pos) {
                        context_menu = None;
                    }
                },
                Some(ContextMenu::Instances { pos, options }) => {
                    frame.begin_context_menu(*pos);
                    if !frame.window_focused() {
                        context_menu = None;
                    } else {
                        for (label, id) in options {
                            if frame.menu_item(label) {
                                if !config.watched_ids.contains(id) {
                                    config.watched_ids.push(*id);
                                    instance_reports.push((*id, InstanceReport::new(&*self, *id)));
                                    let _ = File::create(&config_path).map(|f| bincode::serialize_into(f, &config));
                                }
                                context_menu = None;
                                break
                            }
                        }
                    }
                    frame.end();
                },
                Some(ContextMenu::Seed { pos }) => {
                    frame.begin_context_menu(*pos);
                    if !frame.window_focused() {
                        context_menu = None;
                    } else {
                        let count;
                        if new_rand.is_some() && frame.menu_item("Reset") {
                            count = None;
                            context_menu = None;
                            new_rand = None;
                            seed_text = format!("Seed: {}", self.rand.seed());
                        } else if frame.menu_item("+1 RNG call") {
                            count = Some(1);
                            context_menu = None;
                        } else if frame.menu_item("+5 RNG calls") {
                            count = Some(5);
                            context_menu = None;
                        } else if frame.menu_item("+10 RNG calls") {
                            count = Some(10);
                            context_menu = None;
                        } else if frame.menu_item("+50 RNG calls") {
                            count = Some(50);
                            context_menu = None;
                        } else {
                            count = None;
                        }
                        if let Some(count) = count {
                            if let Some(rand) = &mut new_rand {
                                for _ in 0..count {
                                    rand.cycle();
                                }
                                seed_text = format!("Seed: {}*", rand.seed());
                            } else {
                                let mut rand = self.rand.clone();
                                for _ in 0..count {
                                    rand.cycle();
                                }
                                seed_text = format!("Seed: {}*", rand.seed());
                                new_rand = Some(rand);
                            }
                        }
                    }
                    frame.end();
                },
                None => (),
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
                &grid_ref,
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

        let _ = File::create(&config_path).map(|f| bincode::serialize_into(f, &config));
    }
}

struct InstanceReport {
    object_name: String,
    id: String,
    general_vars: [String; 7],
    physics_vars: [String; 13],
    image_vars: [String; 11],
    timeline_vars: [String; 5],
    alarms: Vec<String>,
    fields: Vec<ReportField>,
}

enum ReportField {
    Single(String),
    Array(String, Vec<String>),
}

impl InstanceReport {
    fn new(game: &Game, id: i32) -> Option<Self> {
        use crate::game::GetAsset;
        if let Some((handle, instance)) =
            game.room.instance_list.get_by_instid(id).map(|x| (x, game.room.instance_list.get(x)))
        {
            instance.update_bbox(game.get_instance_mask_sprite(handle));
            let object_name = game
                .assets
                .objects
                .get_asset(instance.object_index.get())
                .map(|x| x.name.decode(game.encoding))
                .unwrap_or("<deleted object>".into());

            Some(Self {
                object_name: object_name.clone().into(),
                id: id.to_string(),
                general_vars: [
                    format!("object_index: {} ({})", instance.object_index.get(), object_name),
                    format!("x: {:.4}", instance.x.get()),
                    format!("y: {:.4}", instance.y.get()),
                    format!("xprevious: {:.4}", instance.xprevious.get()),
                    format!("yprevious: {:.4}", instance.yprevious.get()),
                    format!("xstart: {:.4}", instance.xstart.get()),
                    format!("ystart: {:.4}", instance.ystart.get()),
                ],
                physics_vars: [
                    format!("speed: {:.4}", instance.speed.get()),
                    format!("direction: {:.4}", instance.direction.get()),
                    format!("hspeed: {:.4}", instance.hspeed.get()),
                    format!("vspeed: {:.4}", instance.vspeed.get()),
                    format!("gravity: {:.4}", instance.gravity.get()),
                    format!("gravity_direction: {:.4}", instance.gravity_direction.get()),
                    format!("friction: {:.4}", instance.friction.get()),
                    format!("solid: {}", instance.solid.get()),
                    format!("persistent: {}", instance.persistent.get()),
                    format!("bbox_left: {}", instance.bbox_left.get()),
                    format!("bbox_right: {}", instance.bbox_right.get()),
                    format!("bbox_top: {}", instance.bbox_top.get()),
                    format!("bbox_bottom: {}", instance.bbox_bottom.get()),
                ],
                image_vars: [
                    format!(
                        "sprite_index: {} ({})",
                        instance.sprite_index.get(),
                        game.assets
                            .sprites
                            .get_asset(instance.sprite_index.get())
                            .map(|x| x.name.decode(game.encoding))
                            .unwrap_or("<deleted sprite>".into()),
                    ),
                    format!(
                        "mask_index: {} ({})",
                        instance.mask_index.get(),
                        game.assets
                            .sprites
                            .get_asset(instance.mask_index.get())
                            .map(|x| x.name.decode(game.encoding))
                            .unwrap_or("<same as sprite>".into()),
                    ),
                    format!("image_index: {:.4}", instance.image_index.get()),
                    format!("image_speed: {:.4}", instance.image_speed.get()),
                    format!("visible: {}", instance.visible.get()),
                    format!("depth: {:.4}", instance.depth.get()),
                    format!("image_xscale: {:.4}", instance.image_xscale.get()),
                    format!("image_yscale: {:.4}", instance.image_yscale.get()),
                    format!("image_angle: {:.4}", instance.image_angle.get()),
                    format!("image_blend: {}", instance.image_blend.get()),
                    format!("image_alpha: {:.4}", instance.image_alpha.get()),
                ],
                timeline_vars: [
                    format!(
                        "timeline_index: {} ({})",
                        instance.timeline_index.get(),
                        game.assets
                            .timelines
                            .get_asset(instance.timeline_index.get())
                            .map(|x| x.name.decode(game.encoding))
                            .unwrap_or("<deleted timeline>".into()),
                    ),
                    format!("timeline_running: {}", instance.timeline_running.get()),
                    format!("timeline_speed: {:.4}", instance.timeline_speed.get()),
                    format!("timeline_position: {:.4}", instance.timeline_position.get()),
                    format!("timeline_loop: {}", instance.timeline_loop.get()),
                ],
                alarms: instance.alarms.borrow().iter().map(|(id, time)| format!("alarm[{}]: {}", id, time)).collect(),
                fields: instance
                    .fields
                    .borrow()
                    .iter()
                    .map(|(id, field)| {
                        let field_name = game.compiler.get_field_name(*id).unwrap_or("<???>".into());
                        match field {
                            Field::Single(value) => ReportField::Single(format!("{}: {}", field_name, value)),
                            Field::Array(map) => ReportField::Array(
                                field_name,
                                map.iter().map(|(index, value)| format!("[{}]: {}", index, value)).collect(),
                            ),
                        }
                    })
                    .collect(),
            })
        } else {
            None
        }
    }
}

// Draws the coloured rectangle according to the current state of the button.
// Doesn't render any text on it.
fn draw_keystate(frame: &mut imgui::Frame, state: &KeyState, position: imgui::Vec2<f32>, size: imgui::Vec2<f32>) {
    let wpos = frame.window_position();
    let alpha = if frame.item_hovered() { 255 } else { 190 };
    let r1_min = position + wpos;
    let r1_max = r1_min + imgui::Vec2((size.0 / 2.0).floor(), size.1);
    let r2_min = imgui::Vec2(position.0 + (size.0 / 2.0).floor(), position.1) + wpos;
    let r2_max = position + size + wpos;
    match *state {
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
