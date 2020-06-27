use crate::font::{self, Font};
use gmio::{
    atlas::{AtlasBuilder, AtlasRef},
    render::{Renderer, RendererOptions},
    window::{Event, Window, WindowBuilder},
};
use shared::{
    input,
    message::{self, Information, InstanceDetails, MessageStream},
    types::{Colour, ID},
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
    pub font: Font,
    pub font_small: Font,
    pub advance_button: AdvanceButton,
    pub key_buttons: Vec<KeyButton>,
    pub big_save_button: BigSaveButton,
    pub save_buttons: Vec<SaveButton>,
    pub stream: TcpStream,
    mouse_x: i32,
    mouse_y: i32,
    watched_id: Option<ID>,
    watched_instance: Option<InstanceDetails>,

    pub frame_count: usize,
    pub game_mouse_pos: (f64, f64),

    advance_button_normal: AtlasRef,
    big_save_button_normal: AtlasRef,
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

    menu_context: Option<MenuContext>,

    pub read_buffer: Vec<u8>,
    pub project_dir: PathBuf,
}

#[derive(Clone)]
pub enum MenuContext {
    KeyButton(input::Key),
    SaveButton(String),
    BigSaveButton,
}

#[derive(Clone, Copy)]
pub struct AdvanceButton {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Copy)]
pub struct KeyButton {
    pub x: i32,
    pub y: i32,
    pub key: input::Key,
    pub state: KeyButtonState,
}

#[derive(Clone, Copy)]
pub struct BigSaveButton {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone)]
pub struct SaveButton {
    pub x: i32,
    pub y: i32,
    pub name: String,
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

impl AdvanceButton {
    pub fn contains_point(&self, x: i32, y: i32) -> bool {
        x >= self.x && x < (self.x + 100) && y >= self.y && y < (self.y + 40)
    }
}

impl BigSaveButton {
    pub fn contains_point(&self, x: i32, y: i32) -> bool {
        x >= self.x && x < (self.x + 100) && y >= self.y && y < (self.y + 30)
    }
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
            &RendererOptions { size: (WINDOW_WIDTH, WINDOW_HEIGHT), vsync: true },
            &window,
            clear_colour,
        )?;

        let mut atlases = AtlasBuilder::new(1024);
        let advance_button_normal = Self::upload_bmp(&mut atlases, include_bytes!("images/advance.bmp"));
        let big_save_button_normal = Self::upload_bmp(&mut atlases, include_bytes!("images/save_main.bmp"));
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

        // Helper fn: create a Font
        fn make_font(
            font: &rusttype::Font,
            scale: f32,
            atlases: &mut AtlasBuilder,
        ) -> Result<Font, Box<dyn std::error::Error>> {
            (0..=127)
                .map(|i| {
                    let scale = rusttype::Scale::uniform(scale);
                    let glyph = font.glyph(char::from(i)).scaled(scale).positioned(rusttype::Point { x: 0.0, y: 0.0 });
                    let (x, y, w, h) = match glyph.pixel_bounding_box() {
                        Some(bbox) => (-bbox.min.x, -bbox.min.y, bbox.max.x - bbox.min.x, bbox.max.y - bbox.min.y),
                        None => (0, 0, 0, 0),
                    };
                    let mut data: Vec<u8> = Vec::with_capacity((w * h * 4) as usize);
                    glyph.draw(|_, _, a| {
                        data.push(0xFF);
                        data.push(0xFF);
                        data.push(0xFF);
                        data.push((a * 255.0) as u8);
                    });
                    let atlas_ref = atlases.texture(w, h, x, y, data.into_boxed_slice()).ok_or("Couldn't pack font")?;
                    let hmetrics = glyph.unpositioned().h_metrics();
                    Ok(font::Character {
                        atlas_ref,
                        advance_width: hmetrics.advance_width.into(),
                        left_side_bearing: hmetrics.left_side_bearing.into(),
                    })
                })
                .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()
                .map(|x| x.into_boxed_slice().into())
        }

        let rt_font = rusttype::Font::try_from_bytes(include_bytes!("misc/visitor.ttf")).ok_or("Couldn't load font")?;
        let font = make_font(&rt_font, 20.0, &mut atlases)?;
        let font_small = make_font(&rt_font, 15.0, &mut atlases)?;

        renderer.push_atlases(atlases)?;

        let mut save_buttons = Vec::with_capacity(2 * 8);
        for y in 0..2 {
            for x in 0..8 {
                let id = (y * 8) + x + 1;
                let filename = format!("save{}.bin", id);
                project_dir.push(&filename);
                let exists = project_dir.exists();
                project_dir.pop();
                save_buttons.push(SaveButton {
                    x: 47 + (SAVE_BUTTON_SIZE * x) as i32,
                    y: 278 + (SAVE_BUTTON_SIZE * y) as i32,
                    name: id.to_string(),
                    filename,
                    exists,
                });
            }
        }

        window.set_visible(true);
        renderer.finish(WINDOW_WIDTH, WINDOW_HEIGHT, clear_colour);
        Ok(Self {
            window,
            renderer,
            clear_colour,
            font,
            font_small,
            advance_button: AdvanceButton { x: 240, y: 8 },
            key_buttons: vec![
                KeyButton { x: 103, y: 150, key: input::Key::Left, state: KeyButtonState::Neutral },
                KeyButton { x: 151, y: 150, key: input::Key::Down, state: KeyButtonState::Neutral },
                KeyButton { x: 199, y: 150, key: input::Key::Right, state: KeyButtonState::Neutral },
                KeyButton { x: 151, y: 102, key: input::Key::Up, state: KeyButtonState::Neutral },
                KeyButton { x: 32, y: 90, key: input::Key::R, state: KeyButtonState::Neutral },
                KeyButton { x: 32, y: 150, key: input::Key::Shift, state: KeyButtonState::Neutral },
                KeyButton { x: 270, y: 90, key: input::Key::F2, state: KeyButtonState::Neutral },
                KeyButton { x: 270, y: 150, key: input::Key::Z, state: KeyButtonState::Neutral },
            ],
            big_save_button: BigSaveButton { x: 125, y: 240 },
            save_buttons,
            stream,
            mouse_x: 0,
            mouse_y: 0,
            watched_id: None,
            watched_instance: None,

            frame_count: 0,
            game_mouse_pos: (0.0, 0.0),

            advance_button_normal,
            big_save_button_normal,
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

            menu_context: None,
            read_buffer: Vec::new(),
            project_dir,
        })
    }

    pub fn update(&mut self) -> Result<bool, Box<dyn std::error::Error>> {
        match self.stream.receive_message::<Information>(&mut self.read_buffer)? {
            None => return Ok(false),
            Some(Some(Information::KeyPressed { key })) => self.handle_key(key)?,
            Some(Some(Information::InstanceClicked { details })) => {
                self.watched_id = Some(details.id);
                self.watched_instance = Some(details);
            },
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
                    if self.advance_button.contains_point(self.mouse_x, self.mouse_y) {
                        self.send_advance()?;
                        break
                    }

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

                    if self.big_save_button.contains_point(self.mouse_x, self.mouse_y) {
                        self.window.show_context_menu(&[("Load [W]\0".into(), 1), ("Save [Q]\0".into(), 0)]);
                        self.menu_context = Some(MenuContext::SaveButton("save.bin".into()));
                        break
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
                            self.menu_context = Some(MenuContext::KeyButton(button.key));
                            break 'evloop
                        }
                    }

                    for button in self.save_buttons.iter() {
                        if button.contains_point(self.mouse_x, self.mouse_y) && button.exists {
                            self.window.show_context_menu(&[("Load\0".into(), 1), ("Save\0".into(), 0)]);
                            self.menu_context = Some(MenuContext::SaveButton(button.filename.clone()));
                            break 'evloop
                        }
                    }

                    if self.big_save_button.contains_point(self.mouse_x, self.mouse_y) {
                        self.window.show_context_menu(&[("Load [W]\0".into(), 1), ("Save [Q]\0".into(), 0)]);
                        self.menu_context = Some(MenuContext::SaveButton("save.bin".into()));
                        break
                    }
                },

                Event::MenuOption(option) => {
                    match &self.menu_context {
                        Some(MenuContext::KeyButton(target_key)) => {
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
                                if button.key == *target_key {
                                    button.state = new_state;
                                }
                            }
                        },

                        Some(MenuContext::SaveButton(filename)) => {
                            match option {
                                0 => {
                                    // Save
                                    self.stream.send_message(&message::Message::Save { filename: filename.clone() })?;
                                    println!("Probably saved to {}", filename);
                                },

                                1 => {
                                    // Load
                                    self.stream.send_message(&message::Message::Load {
                                        keys_requested: self.key_buttons.iter().map(|x| x.key).collect(),
                                        mouse_buttons_requested: Vec::new(),
                                        filename: filename.clone(),
                                        instance_requested: self.watched_id,
                                    })?;
                                    self.await_update()?;
                                    println!("Loaded");
                                    break
                                },

                                _ => continue,
                            }
                        },

                        _ => (),
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
                    instance_requested: self.watched_id,
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
            mouse_location: self.game_mouse_pos,
            keys_requested,
            mouse_buttons_requested: Vec::new(),
            instance_requested: self.watched_id,
        })?;

        self.await_update()
    }

    pub fn await_update(&mut self) -> Result<bool, Box<dyn std::error::Error>> {
        loop {
            match self.stream.receive_message::<message::Information>(&mut self.read_buffer) {
                Ok(Some(Some(message::Information::Update {
                    keys_held,
                    mouse_buttons_held: _,
                    mouse_location,
                    frame_count,
                    seed: _,
                    instance,
                }))) => {
                    self.frame_count = frame_count;
                    self.game_mouse_pos = mouse_location;
                    self.watched_instance = instance;
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

        draw_text(&mut self.renderer, "Frame:", 4.0, 19.0, &self.font, 0, 1.0);
        draw_text(&mut self.renderer, &self.frame_count.to_string(), 4.0, 32.0, &self.font, 0, 1.0);

        self.renderer.draw_sprite(
            &self.advance_button_normal,
            self.advance_button.x.into(),
            self.advance_button.y.into(),
            1.0,
            1.0,
            0.0,
            0xFFFFFF,
            if self.advance_button.contains_point(self.mouse_x, self.mouse_y) { 1.0 } else { 0.8 },
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

        self.renderer.draw_sprite(
            &self.big_save_button_normal,
            self.big_save_button.x.into(),
            self.big_save_button.y.into(),
            1.0,
            1.0,
            0.0,
            0xFFFFFF,
            if self.big_save_button.contains_point(self.mouse_x, self.mouse_y) { 1.0 } else { 0.8 },
        );

        for button in self.save_buttons.iter() {
            let alpha = if button.contains_point(self.mouse_x, self.mouse_y) { 1.0 } else { 0.75 };
            let atlas_ref = if button.exists { &self.save_button_active } else { &self.save_button_inactive };
            self.renderer.draw_sprite(atlas_ref, button.x as _, button.y as _, 1.0, 1.0, 0.0, 0xFFFFFF, alpha);
            draw_text(
                &mut self.renderer,
                &button.name,
                f64::from(button.x) + 8.0,
                f64::from(button.y) + 20.0,
                &self.font,
                0,
                alpha,
            );
        }

        draw_text(&mut self.renderer, "Keyboard", 123.0, 82.0, &self.font, 0, 1.0);
        draw_text(&mut self.renderer, "Saves", 143.0, 230.0, &self.font, 0, 1.0);

        if let Some(id) = self.watched_id.as_ref() {
            draw_text(&mut self.renderer, "Watching:", 8.0, 605.0, &self.font, 0, 1.0);
            if let Some(details) = self.watched_instance.as_ref() {
                draw_text(
                    &mut self.renderer,
                    &format!("{} ({})", details.object_name, details.id),
                    8.0,
                    618.0,
                    &self.font_small,
                    0,
                    1.0,
                );
                draw_text(
                    &mut self.renderer,
                    &format!("x: {}", details.x),
                    8.0,
                    638.0,
                    &self.font_small,
                    0x303030,
                    1.0,
                );
                draw_text(
                    &mut self.renderer,
                    &format!("y: {}", details.y),
                    8.0,
                    651.0,
                    &self.font_small,
                    0x303030,
                    1.0,
                );
                draw_text(
                    &mut self.renderer,
                    &format!("speed: {}", details.speed),
                    8.0,
                    664.0,
                    &self.font_small,
                    0x303030,
                    1.0,
                );
                draw_text(
                    &mut self.renderer,
                    &format!("direction: {}", details.direction),
                    8.0,
                    677.0,
                    &self.font_small,
                    0x303030,
                    1.0,
                );
                draw_text(
                    &mut self.renderer,
                    &format!("bbox_left: {}", details.bbox_left),
                    8.0,
                    690.0,
                    &self.font_small,
                    0x303030,
                    1.0,
                );
                draw_text(
                    &mut self.renderer,
                    &format!("bbox_right: {}", details.bbox_right),
                    8.0,
                    703.0,
                    &self.font_small,
                    0x303030,
                    1.0,
                );
                draw_text(
                    &mut self.renderer,
                    &format!("bbox_top: {}", details.bbox_top),
                    8.0,
                    716.0,
                    &self.font_small,
                    0x303030,
                    1.0,
                );
                draw_text(
                    &mut self.renderer,
                    &format!("bbox_bottom: {}", details.bbox_bottom),
                    8.0,
                    729.0,
                    &self.font_small,
                    0x303030,
                    1.0,
                );
                let mut alarms = details
                    .alarms
                    .iter()
                    .filter(|(_, x)| **x > 0)
                    .map(|(index, timer)| format!("[{}]={}", index, timer))
                    .collect::<Vec<_>>();
                if alarms.len() > 0 {
                    alarms.sort();
                    draw_text(
                        &mut self.renderer,
                        &format!("alarms: {}", alarms.join(", ")),
                        8.0,
                        742.0,
                        &self.font_small,
                        0x303030,
                        1.0,
                    );
                }
            } else {
                draw_text(&mut self.renderer, &format!("<deleted> ({})", id), 8.0, 618.0, &self.font_small, 0, 1.0);
            }
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

fn draw_text(renderer: &mut Renderer, text: &str, mut x: f64, y: f64, font: &Font, colour: i32, alpha: f64) {
    for c in text.chars() {
        if let Some(character) = font.get(c as u8) {
            renderer.draw_sprite(
                &character.atlas_ref,
                x + character.left_side_bearing,
                y,
                1.0,
                1.0,
                0.0,
                colour,
                alpha,
            );
            x += character.advance_width;
        }
    }
}
