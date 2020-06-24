use gmio::{
    atlas::{AtlasBuilder, AtlasRef},
    render::{Renderer, RendererOptions},
    window::{Event, Window, WindowBuilder},
};
use shared::{
    input,
    message::{self, Information, MessageStream},
    types::Colour,
};
use std::{net::TcpStream, path::PathBuf};

const WINDOW_WIDTH: u32 = 350;
const WINDOW_HEIGHT: u32 = 750;

const KEY_BUTTON_SIZE: usize = 48;
const SAVE_BUTTON_SIZE: usize = 32;

pub struct ControlPanel {
    pub window: Window,
    pub renderer: Renderer,
    pub clear_colour: Colour,
    pub key_buttons: Vec<KeyButton>,
    pub save_buttons: Vec<SaveButton>,
    pub stream: TcpStream,
    mouse_x: i32,
    mouse_y: i32,

    key_button_l_neutral: AtlasRef,
    key_button_l_held: AtlasRef,
    key_button_r_neutral: AtlasRef,
    key_button_r_neutral2: AtlasRef,
    key_button_r_neutral3: AtlasRef,
    key_button_r_held: AtlasRef,
    key_button_r_held2: AtlasRef,
    key_button_r_held3: AtlasRef,
    save_button_active: AtlasRef,
    save_button_inactive: AtlasRef,

    context_menu_key: Option<input::Key>,

    pub read_buffer: Vec<u8>,
    pub project_dir: PathBuf,
}

#[derive(Clone, Copy)]
pub struct KeyButton {
    pub x: i32,
    pub y: i32,
    pub key: input::Key,
    pub state: KeyButtonState,
}

#[derive(Clone)]
pub struct SaveButton {
    pub x: i32,
    pub y: i32,
    pub filename: String,
    pub exists: bool,
}

#[derive(Clone, Copy)]
pub enum KeyButtonState {
    Neutral,
    NeutralWillPress,
    NeutralWillPR,
    NeutralWillPRP,
    Held,
    HeldWillRelease,
    HeldWillRP,
    HeldWillRPR,
}

impl KeyButton {
    pub fn contains_point(&self, x: i32, y: i32) -> bool {
        x >= self.x && x < (self.x + KEY_BUTTON_SIZE as i32) && y >= self.y && y < (self.y + KEY_BUTTON_SIZE as i32)
    }
}

impl SaveButton {
    pub fn contains_point(&self, x: i32, y: i32) -> bool {
        x >= self.x && x < (self.x + SAVE_BUTTON_SIZE as i32) && y >= self.y && y < (self.y + SAVE_BUTTON_SIZE as i32)
    }
}

impl ControlPanel {
    pub fn new(stream: TcpStream, project_name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut project_dir = std::env::current_dir()?;
        project_dir.push("projects");
        project_dir.push(project_name);
        let wb = WindowBuilder::new().with_size(WINDOW_WIDTH, WINDOW_HEIGHT);
        let mut window = wb.build()?;
        let clear_colour = Colour::new(220.0 / 255.0, 220.0 / 255.0, 220.0 / 255.0);
        let mut renderer = Renderer::new(
            (),
            &RendererOptions { size: (WINDOW_WIDTH, WINDOW_HEIGHT), vsync: false },
            &window,
            clear_colour,
        )?;
        window.set_visible(true);

        let mut atlases = AtlasBuilder::new(1024);
        let key_button_l_neutral = Self::upload_bmp(&mut atlases, include_bytes!("images/KeyBtnLNeutral.bmp"));
        let key_button_l_held = Self::upload_bmp(&mut atlases, include_bytes!("images/KeyBtnLHeld.bmp"));
        let key_button_r_neutral = Self::upload_bmp(&mut atlases, include_bytes!("images/KeyBtnRNeutral.bmp"));
        let key_button_r_neutral2 = Self::upload_bmp(&mut atlases, include_bytes!("images/KeyBtnRNeutral2.bmp"));
        let key_button_r_neutral3 = Self::upload_bmp(&mut atlases, include_bytes!("images/KeyBtnRNeutral3.bmp"));
        let key_button_r_held = Self::upload_bmp(&mut atlases, include_bytes!("images/KeyBtnRHeld.bmp"));
        let key_button_r_held2 = Self::upload_bmp(&mut atlases, include_bytes!("images/KeyBtnRHeld2.bmp"));
        let key_button_r_held3 = Self::upload_bmp(&mut atlases, include_bytes!("images/KeyBtnRHeld3.bmp"));
        let save_button_active = Self::upload_bmp(&mut atlases, include_bytes!("images/save_active.bmp"));
        let save_button_inactive = Self::upload_bmp(&mut atlases, include_bytes!("images/save_inactive.bmp"));
        renderer.push_atlases(atlases)?;

        let mut save_buttons = Vec::with_capacity(2 * 8);
        for y in 0..2 {
            for x in 0..8 {
                let filename = format!("save{}.bin", (y * 8) + x);
                project_dir.push(&filename);
                let exists = project_dir.exists();
                project_dir.pop();
                save_buttons.push(SaveButton {
                    x: 47 + (SAVE_BUTTON_SIZE * x) as i32,
                    y: 200 + (SAVE_BUTTON_SIZE * y) as i32,
                    filename,
                    exists,
                });
            }
        }

        renderer.finish(WINDOW_WIDTH, WINDOW_HEIGHT, clear_colour);
        Ok(Self {
            window,
            renderer,
            clear_colour,
            key_buttons: vec![
                KeyButton { x: 103, y: 100, key: input::Key::Left, state: KeyButtonState::Neutral },
                KeyButton { x: 151, y: 100, key: input::Key::Down, state: KeyButtonState::Neutral },
                KeyButton { x: 199, y: 100, key: input::Key::Right, state: KeyButtonState::Neutral },
                KeyButton { x: 151, y: 52, key: input::Key::Up, state: KeyButtonState::Neutral },
                KeyButton { x: 32, y: 50, key: input::Key::R, state: KeyButtonState::Neutral },
                KeyButton { x: 32, y: 100, key: input::Key::Shift, state: KeyButtonState::Neutral },
                KeyButton { x: 270, y: 50, key: input::Key::F2, state: KeyButtonState::Neutral },
                KeyButton { x: 270, y: 100, key: input::Key::Z, state: KeyButtonState::Neutral },
            ],
            save_buttons,
            stream,
            mouse_x: 0,
            mouse_y: 0,

            key_button_l_neutral,
            key_button_l_held,
            key_button_r_neutral,
            key_button_r_neutral2,
            key_button_r_neutral3,
            key_button_r_held,
            key_button_r_held2,
            key_button_r_held3,
            save_button_active,
            save_button_inactive,

            context_menu_key: None,
            read_buffer: Vec::new(),
            project_dir,
        })
    }

    pub fn update(&mut self) -> Result<bool, Box<dyn std::error::Error>> {
        match self.stream.receive_message::<Information>(&mut self.read_buffer)? {
            None => return Ok(false),
            Some(Some(Information::KeyPressed { key })) => self.handle_key(key)?,
            Some(Some(s)) => println!("Got TCP message: '{:?}'", s),
            Some(None) => (),
        }

        'evloop: for event in self.window.process_events() {
            match event {
                Event::MouseMove(x, y) => {
                    self.mouse_x = *x;
                    self.mouse_y = *y;
                },

                Event::MouseButtonUp(input::MouseButton::Left) => {
                    for button in self.key_buttons.iter_mut() {
                        if button.contains_point(self.mouse_x, self.mouse_y) {
                            button.state = match button.state {
                                KeyButtonState::Neutral => KeyButtonState::NeutralWillPress,
                                KeyButtonState::NeutralWillPress
                                | KeyButtonState::NeutralWillPR
                                | KeyButtonState::NeutralWillPRP => KeyButtonState::Neutral,
                                KeyButtonState::Held => KeyButtonState::HeldWillRelease,
                                KeyButtonState::HeldWillRelease
                                | KeyButtonState::HeldWillRP
                                | KeyButtonState::HeldWillRPR => KeyButtonState::Held,
                            };
                        }
                    }

                    for button in self.save_buttons.iter_mut() {
                        if button.contains_point(self.mouse_x, self.mouse_y) {
                            self.stream.send_message(&message::Message::Save { filename: button.filename.clone() })?;
                            println!("Probably saved to {}", &button.filename);
                            button.exists = true;
                        }
                    }
                },

                Event::MouseButtonUp(input::MouseButton::Right) => {
                    for button in self.key_buttons.iter_mut() {
                        if button.contains_point(self.mouse_x, self.mouse_y) {
                            let options = match button.state {
                                KeyButtonState::Neutral
                                | KeyButtonState::NeutralWillPress
                                | KeyButtonState::NeutralWillPR
                                | KeyButtonState::NeutralWillPRP => [
                                    ("Press-Release-Press\0".into(), 3),
                                    ("Press-Release\0".into(), 2),
                                    ("Press\0".into(), 1),
                                    ("Reset\0".into(), 0),
                                ],
                                KeyButtonState::Held
                                | KeyButtonState::HeldWillRelease
                                | KeyButtonState::HeldWillRP
                                | KeyButtonState::HeldWillRPR => [
                                    ("Release-Press-Release\0".into(), 7),
                                    ("Release-Press\0".into(), 6),
                                    ("Release\0".into(), 5),
                                    ("Reset\0".into(), 4),
                                ],
                            };
                            self.window.show_context_menu(&options);
                            self.context_menu_key = Some(button.key);
                            break 'evloop
                        }
                    }
                },

                Event::MenuOption(option) => {
                    if let Some(target_key) = self.context_menu_key {
                        let new_state = match option {
                            0 => KeyButtonState::Neutral,
                            1 => KeyButtonState::NeutralWillPress,
                            2 => KeyButtonState::NeutralWillPR,
                            3 => KeyButtonState::NeutralWillPRP,
                            4 => KeyButtonState::Held,
                            5 => KeyButtonState::HeldWillRelease,
                            6 => KeyButtonState::HeldWillRP,
                            7 => KeyButtonState::HeldWillRPR,
                            _ => continue,
                        };

                        for button in self.key_buttons.iter_mut() {
                            if button.key == target_key {
                                button.state = new_state;
                            }
                        }
                    }
                },

                Event::KeyboardDown(key) => {
                    let key = *key;
                    self.handle_key(key)?;
                    break
                },

                _ => (),
            }
        }

        Ok(true)
    }

    pub fn handle_key(&mut self, key: input::Key) -> Result<(), Box<dyn std::error::Error>> {
        match key {
            input::Key::Space => {
                self.send_advance()?;
            },

            input::Key::Q => {
                self.stream.send_message(&message::Message::Save { filename: "save.bin".into() })?;
                println!("Probably saved");
            },

            input::Key::W => {
                self.stream.send_message(&message::Message::Load {
                    keys_requested: self.key_buttons.iter().map(|x| x.key).collect(),
                    mouse_buttons_requested: Vec::new(),
                    filename: "save.bin".into(),
                })?;
                self.await_update()?;
                println!("Loaded");
            },

            _ => (),
        }

        Ok(())
    }

    fn send_advance(&mut self) -> Result<bool, Box<dyn std::error::Error>> {
        let mut key_inputs = Vec::new();
        let mut keys_requested = Vec::new();

        for key in self.key_buttons.iter() {
            keys_requested.push(key.key);
            match key.state {
                KeyButtonState::Neutral | KeyButtonState::Held => (),
                KeyButtonState::NeutralWillPress => key_inputs.push((key.key, true)),
                KeyButtonState::HeldWillRelease => key_inputs.push((key.key, false)),
                KeyButtonState::NeutralWillPR => {
                    key_inputs.push((key.key, true));
                    key_inputs.push((key.key, false));
                },
                KeyButtonState::NeutralWillPRP => {
                    key_inputs.push((key.key, true));
                    key_inputs.push((key.key, false));
                    key_inputs.push((key.key, true));
                },
                KeyButtonState::HeldWillRP => {
                    key_inputs.push((key.key, false));
                    key_inputs.push((key.key, true));
                },
                KeyButtonState::HeldWillRPR => {
                    key_inputs.push((key.key, false));
                    key_inputs.push((key.key, true));
                    key_inputs.push((key.key, false));
                },
            }
        }

        self.stream.send_message(message::Message::Advance {
            key_inputs,
            mouse_inputs: Vec::new(),
            mouse_location: (0.0, 0.0),
            keys_requested,
            mouse_buttons_requested: Vec::new(),
        })?;

        self.await_update()
    }

    pub fn await_update(&mut self) -> Result<bool, Box<dyn std::error::Error>> {
        loop {
            match self.stream.receive_message::<message::Information>(&mut self.read_buffer) {
                Ok(Some(Some(message::Information::Update {
                    keys_held,
                    mouse_buttons_held: _,
                    mouse_location: _,
                    frame_count: _,
                    seed: _,
                    instance: _,
                }))) => {
                    for button in self.key_buttons.iter_mut() {
                        if keys_held.contains(&button.key) {
                            button.state = KeyButtonState::Held;
                        } else {
                            button.state = KeyButtonState::Neutral;
                        }
                    }
                    break Ok(true)
                },
                Err(e) => break Err(e.into()),
                _ => (),
            }
        }
    }

    pub fn draw(&mut self) {
        self.renderer.set_view(
            WINDOW_WIDTH,
            WINDOW_HEIGHT,
            WINDOW_WIDTH,
            WINDOW_HEIGHT,
            0,
            0,
            WINDOW_WIDTH as _,
            WINDOW_HEIGHT as _,
            0.0,
            0,
            0,
            WINDOW_WIDTH as _,
            WINDOW_HEIGHT as _,
        );
        for button in self.key_buttons.iter() {
            let alpha = if button.contains_point(self.mouse_x, self.mouse_y) { 1.0 } else { 0.6 };
            let atlas_ref_l = match button.state {
                KeyButtonState::Neutral
                | KeyButtonState::NeutralWillPress
                | KeyButtonState::NeutralWillPR
                | KeyButtonState::NeutralWillPRP => &self.key_button_l_neutral,
                KeyButtonState::Held
                | KeyButtonState::HeldWillRelease
                | KeyButtonState::HeldWillRP
                | KeyButtonState::HeldWillRPR => &self.key_button_l_held,
            };
            let atlas_ref_r = match button.state {
                KeyButtonState::Neutral | KeyButtonState::HeldWillRelease => &self.key_button_r_neutral,
                KeyButtonState::Held | KeyButtonState::NeutralWillPress => &self.key_button_r_held,
                KeyButtonState::NeutralWillPR => &self.key_button_r_held2,
                KeyButtonState::NeutralWillPRP => &self.key_button_r_held3,
                KeyButtonState::HeldWillRP => &self.key_button_r_neutral2,
                KeyButtonState::HeldWillRPR => &self.key_button_r_neutral3,
            };
            self.renderer.draw_sprite(atlas_ref_l, button.x as _, button.y as _, 1.0, 1.0, 0.0, 0xFFFFFF, alpha);
            self.renderer.draw_sprite(
                atlas_ref_r,
                (button.x + atlas_ref_l.w) as _,
                button.y as _,
                1.0,
                1.0,
                0.0,
                0xFFFFFF,
                alpha,
            );
        }

        for button in self.save_buttons.iter() {
            let alpha = if button.contains_point(self.mouse_x, self.mouse_y) { 1.0 } else { 0.75 };
            let atlas_ref = if button.exists { &self.save_button_active } else { &self.save_button_inactive };
            self.renderer.draw_sprite(atlas_ref, button.x as _, button.y as _, 1.0, 1.0, 0.0, 0xFFFFFF, alpha);
        }

        self.renderer.finish(WINDOW_WIDTH, WINDOW_HEIGHT, self.clear_colour)
    }

    // Little helper function, input MUST be a BMP file in 32-bit RGBA format. Best used with include_bytes!()
    fn upload_bmp(atlases: &mut AtlasBuilder, bmp: &[u8]) -> AtlasRef {
        fn read_u32(data: &[u8], pos: usize) -> u32 {
            let bytes = &data[pos..pos + 4];
            u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
        }

        let offset = read_u32(bmp, 10);
        let w = read_u32(bmp, 18);
        let h = read_u32(bmp, 22);
        let rgba = &bmp[(offset as usize)..(offset + (w * h * 4)) as usize];
        let mut corrected_rgba: Vec<u8> = Vec::with_capacity((w * h * 4) as usize);
        for bytes in rgba.rchunks_exact((w * 4) as usize) {
            corrected_rgba.extend_from_slice(bytes);
        }
        atlases
            .texture(w as _, h as _, 0, 0, corrected_rgba.into_boxed_slice())
            .expect("Failed to pack a texture for control panel")
    }
}
