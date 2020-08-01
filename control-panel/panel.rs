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
use chrono::{DateTime, offset::Utc};
use std::{fs::File, fs, time::SystemTime, io::Read, net::TcpStream, path::PathBuf};
// use std::collections::HashMap;

// section: consts

const WINDOW_WIDTH: u32 = 350;
const WINDOW_HEIGHT: u32 = 750;

const KEY_BUTTON_SIZE: usize = 48;
const SAVE_BUTTON_SIZE: usize = 32;

const PRIMARY_SAVE_NAME: &str = "save.bin";

// section: enums

#[derive(Debug, Clone)]
pub enum Action {
    Advance,
    Update,
    Save(String),
    Nothing,
}

#[derive(Debug, Clone)]
pub enum MenuContext {
    KeyButton(input::Key),
    MouseButton(input::MouseButton),
    ButtonInfo(String),
}

#[derive(Clone, Copy)]
pub enum ButtonState {
    Neutral,
    NeutralWillPress,
    NeutralWillPR,
    NeutralWillPRP,
    Held,
    HeldWillRelease,
    HeldWillRP,
    HeldWillRPR,
}

// section: ButtonInfo

#[derive(Debug, Clone)]
pub struct ButtonInfo {
    name: String,
    filename: String,
    description: String,
    action_left_click: Action,
    action_right_click: Action,
    show_context_menu: bool,
    sprite: Option<AtlasRef>,
    xstart: i32,
    ystart: i32,
    width: i32,
    height: i32,
    xscale: f32,
    yscale: f32,
    angle: f32,
    colour: i32,
    alpha_normal: f32,
    alpha_hover: f32,
    exists: bool,
}

impl Default for ButtonInfo {
    fn default() -> Self {
        ButtonInfo {
            name: "".to_string(),
            filename: "".to_string(),
            description: "".to_string(),
            action_left_click: Action::Nothing,
            action_right_click: Action::Nothing,
            show_context_menu: false,
            sprite: None,
            xstart: 0,
            ystart: 0,
            width: KEY_BUTTON_SIZE as i32,
            height: KEY_BUTTON_SIZE as i32,
            xscale: 1.0,
            yscale: 1.0,
            angle: 0.0,
            colour: 0xFFFFFF,
            alpha_normal: 1.0,
            alpha_hover: 0.8,
            exists: true,
        }
    }
}

impl ButtonInfo {
    fn new(
        name: &str,
        filename: &str,
        description: &str,
        action_left_click: Action,
        action_right_click: Action,
        show_context_menu: bool,
        sprite: Option<AtlasRef>,
        xstart: i32,
        ystart: i32,
        width: i32,
        height: i32,
        xscale: f32,
        yscale: f32,
        angle: f32,
        colour: i32,
        alpha_normal: f32,
        alpha_hover: f32,
        exists: bool,
    ) -> ButtonInfo {
        ButtonInfo {
            name: name.to_string(),
            filename: filename.to_string(),
            description: description.to_string(),
            action_left_click: action_left_click,
            action_right_click: action_right_click,
            show_context_menu: show_context_menu,
            sprite: sprite,
            xstart: xstart,
            ystart: ystart,
            width: width,
            height: height,
            xscale: xscale,
            yscale: yscale,
            angle: angle,
            colour: colour,
            alpha_normal: alpha_normal,
            alpha_hover: alpha_hover,
            exists: exists,
        }
    }

    pub fn contains_point(&self, mouse_x: i32, mouse_y: i32) -> bool {
        mouse_x >= self.xstart
            && mouse_x < (self.xstart + self.width)
            && mouse_y >= self.ystart
            && mouse_y < (self.ystart + self.height)
    }

    pub fn draw_this(&self, renderer: &mut Renderer, mouse_x: i32, mouse_y: i32) {
        if let Some(sprite) = self.sprite {
            renderer.draw_sprite(
                &sprite,
                self.xstart.into(),
                self.ystart.into(),
                self.xscale.into(),
                self.yscale.into(),
                self.angle.into(),
                self.colour.into(),
                if self.contains_point(mouse_x, mouse_y) { self.alpha_hover.into() } else { self.alpha_normal.into() },
            )
        }
    }
}

// section: structs (to be consolidated into ButtonInfo)

// #[derive(Clone, Copy)]
pub struct KeyButton {
    pub x: i32,
    pub y: i32,
    pub key: input::Key,
    pub state: ButtonState,
    pub label: AtlasRef,
}

#[derive(Clone, Copy)]
pub struct MouseButton {
    pub x: i32,
    pub y: i32,
    pub button: input::MouseButton,
    pub state: ButtonState,
}

#[derive(Clone, Copy)]
pub struct MousePositionButton {
    pub x: i32,
    pub y: i32,
    pub active: bool,
}

#[derive(Clone, Copy)]
pub struct SeedChanger {
    pub x: i32,
    pub y: i32,
}

// #[derive(Debug, Clone)]
// pub struct SaveButton {
//     pub x: i32,
//     pub y: i32,
//     pub name: String,
//     pub filename: String,
//     pub exists: bool,
// }

// section: impls (to be consolidated into ButtonInfo)

impl SeedChanger {
    pub fn contains_point(&self, x: i32, y: i32) -> bool {
        x >= self.x && x < (self.x + 180) && y >= (self.y - 14) && y < (self.y + 3)
    }
}

impl KeyButton {
    pub fn contains_point(&self, x: i32, y: i32) -> bool {
        x >= self.x && x < (self.x + KEY_BUTTON_SIZE as i32) && y >= self.y && y < (self.y + KEY_BUTTON_SIZE as i32)
    }
}

impl MouseButton {
    pub fn contains_point(&self, x: i32, y: i32) -> bool {
        x >= self.x && x < (self.x + KEY_BUTTON_SIZE as i32) && y >= self.y && y < (self.y + KEY_BUTTON_SIZE as i32)
    }
}

impl MousePositionButton {
    pub fn contains_point(&self, x: i32, y: i32) -> bool {
        x >= self.x && x < (self.x + 25 as i32) && y >= self.y && y < (self.y + 25 as i32)
    }
}

// impl SaveButton {
//     pub fn contains_point(&self, x: i32, y: i32) -> bool {
//         x >= self.x && x < (self.x + SAVE_BUTTON_SIZE as i32) && y >= self.y && y < (self.y + SAVE_BUTTON_SIZE as i32)
//     }
// }

// section: ControlPanel

pub struct ControlPanel {
    pub window: Window,
    pub renderer: Renderer,
    pub clear_colour: Colour,
    pub font: Font,
    pub font_small: Font,

    pub buttons: Vec<ButtonInfo>,
    // pub save_buttons: Vec<ButtonInfo>,

    pub key_buttons: Vec<KeyButton>,
    pub mouse_buttons: Vec<MouseButton>,
    pub mouse_position_button: MousePositionButton,
    pub seed_changer: SeedChanger,
    pub stream: TcpStream,
    mouse_x: i32,
    mouse_y: i32,
    watched_id: Option<ID>,
    watched_instance: Option<InstanceDetails>,
    pub seed: i32,
    pub new_seed: Option<i32>,

    pub frame_count: usize,
    pub game_mouse_pos: (f64, f64),
    pub client_mouse_pos: (i32, i32),

    key_button_l_neutral: AtlasRef,
    key_button_l_held: AtlasRef,
    key_button_r_neutral: AtlasRef,
    key_button_r_neutral2: AtlasRef,
    key_button_r_neutral3: AtlasRef,
    key_button_r_held: AtlasRef,
    key_button_r_held2: AtlasRef,
    key_button_r_held3: AtlasRef,
    mouse_pos_normal: AtlasRef,
    save_button_active: AtlasRef,
    save_button_inactive: AtlasRef,
    button_outline: AtlasRef,

    menu_context: Option<MenuContext>,

    pub read_buffer: Vec<u8>,
    pub project_dir: PathBuf,
}

impl ControlPanel {
    pub fn perform_action(&mut self, action: Action) -> Result<bool, Box<dyn std::error::Error>> {
        return match action {
            Action::Advance => self.send_advance(),
            Action::Save(mut s) => self.save(&mut s),
            // todo: maybe rethink name since save buttons can either save or load; possibly Action::Memory
            Action::Update => self.update(),
            Action::Nothing => Ok(true),
        }
        // &[(String, usize)]
    }

    pub fn new(stream: TcpStream, project_name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut project_dir = std::env::current_dir()?;
        project_dir.push("projects");
        project_dir.push(project_name);
        let wb = WindowBuilder::new().with_size(WINDOW_WIDTH, WINDOW_HEIGHT);
        let mut window = wb.build()?;
        let clear_colour = Colour::new(220.0 / 255.0, 220.0 / 255.0, 220.0 / 255.0);
        let mut renderer = Renderer::new(
            (),
            &RendererOptions { size: (WINDOW_WIDTH, WINDOW_HEIGHT), vsync: true, interpolate_pixels: false },
            &window,
            clear_colour,
        )?;

        let mut buttons = vec![
            ButtonInfo {
                name: "advance_button_normal".to_string(),
                action_left_click: Action::Advance,
                xstart: 280,
                ystart: 8,
                ..ButtonInfo::default()
            },
            ButtonInfo {
                name: "big_save_button_normal".to_string(),
                filename: PRIMARY_SAVE_NAME.to_string(),
                action_left_click: Action::Save(PRIMARY_SAVE_NAME.to_string()),
                action_right_click: Action::Save(PRIMARY_SAVE_NAME.to_string()),
                xstart: 125,
                ystart: 400,
                ..ButtonInfo::default()
            },
        ];

        // let mut save_buttons = Vec::with_capacity(2 * 8);
        for y in 0..2 {
            for x in 0..8 {
                let id = (y * 8) + x + 1;
                let filename = format!("save{}.bin", id);
                project_dir.push(&filename);
                let exists = project_dir.exists();
                let name = format!("save_button_{}", if project_dir.exists() { "active" } else { "inactive"});
                project_dir.pop();
                buttons.push(ButtonInfo {
                    xstart: 47 + (SAVE_BUTTON_SIZE * x) as i32,
                    ystart: 438 + (SAVE_BUTTON_SIZE * y) as i32,
                    name: name.to_string(),
                    action_left_click: Action::Save(filename.to_string()),
                    action_right_click: Action::Save(filename.to_string()),
                    filename: filename.to_string(),
                    exists: exists,
                    ..ButtonInfo::default()
                });
            }
        }

        let mut atlases = AtlasBuilder::new(1024);

        for mut button in buttons.iter_mut() {
            let path = format!("../../control-panel/images/{}.bmp", button.name.to_string());
            let mut file = File::open(&path)?;
            let mut file_content = Vec::new();
            file.read_to_end(&mut file_content)?;
            let sprite = Self::upload_bmp(&mut atlases, &file_content);
            button.sprite = Some(AtlasRef::from(sprite));
             // future: if width/height are not 0, aka have a pre-set bound box, ignore this
            button.width = AtlasRef::from(sprite).w;
            button.height = AtlasRef::from(sprite).h;
            // eprintln!("{:?}", button.sprite);
        }

        let key_button_l_neutral = Self::upload_bmp(&mut atlases, include_bytes!("images/key_button_l_neutral.bmp"));
        let key_button_l_held = Self::upload_bmp(&mut atlases, include_bytes!("images/key_button_l_held.bmp"));
        let key_button_r_neutral = Self::upload_bmp(&mut atlases, include_bytes!("images/key_button_r_neutral.bmp"));
        let key_button_r_neutral2 = Self::upload_bmp(&mut atlases, include_bytes!("images/key_button_r_neutral2.bmp"));
        let key_button_r_neutral3 = Self::upload_bmp(&mut atlases, include_bytes!("images/key_button_r_neutral3.bmp"));
        let key_button_r_held = Self::upload_bmp(&mut atlases, include_bytes!("images/key_button_r_held.bmp"));
        let key_button_r_held2 = Self::upload_bmp(&mut atlases, include_bytes!("images/key_button_r_held2.bmp"));
        let key_button_r_held3 = Self::upload_bmp(&mut atlases, include_bytes!("images/key_button_r_held3.bmp"));
        let mouse_pos_normal = Self::upload_bmp(&mut atlases, include_bytes!("images/mouse_pos_normal.bmp"));
        let save_button_active = Self::upload_bmp(&mut atlases, include_bytes!("images/save_button_active.bmp"));
        let save_button_inactive = Self::upload_bmp(&mut atlases, include_bytes!("images/save_button_inactive.bmp"));
        let button_outline = Self::upload_bmp(&mut atlases, include_bytes!("images/button_outline.bmp"));

        let label_up = Self::upload_bmp(&mut atlases, include_bytes!("images/label_up.bmp"));
        let label_down = Self::upload_bmp(&mut atlases, include_bytes!("images/label_down.bmp"));
        let label_left = Self::upload_bmp(&mut atlases, include_bytes!("images/label_left.bmp"));
        let label_right = Self::upload_bmp(&mut atlases, include_bytes!("images/label_right.bmp"));
        let label_r = Self::upload_bmp(&mut atlases, include_bytes!("images/label_r.bmp"));
        let label_z = Self::upload_bmp(&mut atlases, include_bytes!("images/label_z.bmp"));
        let label_f2 = Self::upload_bmp(&mut atlases, include_bytes!("images/label_f2.bmp"));
        let label_shift = Self::upload_bmp(&mut atlases, include_bytes!("images/label_shift.bmp"));

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

        window.set_visible(true);
        renderer.finish(WINDOW_WIDTH, WINDOW_HEIGHT, clear_colour);
        Ok(Self {
            window,
            renderer,
            clear_colour,
            font,
            font_small,
            buttons,
            key_buttons: vec![
                KeyButton { x: 103, y: 150, key: input::Key::Left, state: ButtonState::Neutral, label: label_left },
                KeyButton { x: 151, y: 150, key: input::Key::Down, state: ButtonState::Neutral, label: label_down },
                KeyButton { x: 199, y: 150, key: input::Key::Right, state: ButtonState::Neutral, label: label_right },
                KeyButton { x: 151, y: 102, key: input::Key::Up, state: ButtonState::Neutral, label: label_up },
                KeyButton { x: 32, y: 90, key: input::Key::R, state: ButtonState::Neutral, label: label_r },
                KeyButton { x: 32, y: 150, key: input::Key::Shift, state: ButtonState::Neutral, label: label_shift },
                KeyButton { x: 270, y: 90, key: input::Key::F2, state: ButtonState::Neutral, label: label_f2 },
                KeyButton { x: 270, y: 150, key: input::Key::Z, state: ButtonState::Neutral, label: label_z },
            ],
            mouse_buttons: vec![
                MouseButton { x: 4, y: 248, button: input::MouseButton::Left, state: ButtonState::Neutral },
                MouseButton { x: 56, y: 248, button: input::MouseButton::Middle, state: ButtonState::Neutral },
                MouseButton { x: 108, y: 248, button: input::MouseButton::Right, state: ButtonState::Neutral },
            ],
            mouse_position_button: MousePositionButton { x: 310, y: 250, active: false },
            // save_buttons,
            seed_changer: SeedChanger { x: 8, y: 540 },
            stream,
            mouse_x: 0,
            mouse_y: 0,
            watched_id: None,
            watched_instance: None,
            seed: 0,
            new_seed: None,

            frame_count: 0,
            game_mouse_pos: (0.0, 0.0),
            client_mouse_pos: (0, 0),

            key_button_l_neutral,
            key_button_l_held,
            key_button_r_neutral,
            key_button_r_neutral2,
            key_button_r_neutral3,
            key_button_r_held,
            key_button_r_held2,
            key_button_r_held3,
            mouse_pos_normal,
            save_button_active,
            save_button_inactive,
            button_outline,

            menu_context: None,
            read_buffer: Vec::new(),
            project_dir,
        })
    }

    pub fn update(&mut self) -> Result<bool, Box<dyn std::error::Error>> {
        loop {
            match self.stream.receive_message::<Information>(&mut self.read_buffer)? {
                None => return Ok(false),
                Some(Some(Information::KeyPressed { key })) => self.handle_key(key)?,
                Some(Some(Information::LeftClick { x, y })) => {
                    if self.mouse_position_button.active {
                        self.game_mouse_pos = (f64::from(x), f64::from(y));
                        self.mouse_position_button.active = false;
                    }
                },
                Some(Some(Information::MousePosition { x, y })) => self.client_mouse_pos = (x, y),
                Some(Some(Information::InstanceClicked { details })) => {
                    self.watched_id = Some(details.id);
                    self.watched_instance = Some(details);
                },
                Some(Some(s)) => println!("Got TCP message: '{:?}'", s),
                Some(None) => break,
            }
        }

        'evloop: for event in self.window.process_events() {
            match event {
                Event::MouseMove(x, y) => {
                    self.mouse_x = *x;
                    self.mouse_y = *y;
                },

                Event::MouseButtonUp(input::MouseButton::Left) => {
                    let x = self.mouse_x;
                    let y = self.mouse_y;

                    // todo: extract this match as MouseButton::Right does it as well
                    if let Some(index) = self.buttons.iter().position(|b| b.contains_point(x, y)) {
                        match &self.buttons[index].action_left_click {
                            Action::Advance => {
                                self.perform_action(Action::Advance)?;
                            },
                            Action::Save(filename) => {
                                let file = filename.to_string();
                                self.perform_action(Action::Save(file))?;
                                self.buttons[index].exists = true;
                            },
                            _ => (),
                        };
                        break 'evloop
                    };

                    for button in self.key_buttons.iter_mut() {
                        if button.contains_point(self.mouse_x, self.mouse_y) {
                            button.state = match button.state {
                                ButtonState::Neutral => ButtonState::NeutralWillPress,
                                ButtonState::NeutralWillPress
                                | ButtonState::NeutralWillPR
                                | ButtonState::NeutralWillPRP => ButtonState::Neutral,
                                ButtonState::Held => ButtonState::HeldWillRelease,
                                ButtonState::HeldWillRelease | ButtonState::HeldWillRP | ButtonState::HeldWillRPR => {
                                    ButtonState::Held
                                },
                            };
                        }
                    }

                    for button in self.mouse_buttons.iter_mut() {
                        if button.contains_point(self.mouse_x, self.mouse_y) {
                            button.state = match button.state {
                                ButtonState::Neutral => ButtonState::NeutralWillPress,
                                ButtonState::NeutralWillPress
                                | ButtonState::NeutralWillPR
                                | ButtonState::NeutralWillPRP => ButtonState::Neutral,
                                ButtonState::Held => ButtonState::HeldWillRelease,
                                ButtonState::HeldWillRelease | ButtonState::HeldWillRP | ButtonState::HeldWillRPR => {
                                    ButtonState::Held
                                },
                            };
                        }
                    }

                    if self.mouse_position_button.contains_point(self.mouse_x, self.mouse_y) {
                        self.mouse_position_button.active = !self.mouse_position_button.active;
                        self.stream.send_message(&message::Message::SetUpdateMouse {
                            update: self.mouse_position_button.active,
                        })?;
                    }

                    if self.seed_changer.contains_point(self.mouse_x, self.mouse_y) {
                        if let Some(seed) = self.new_seed {
                            self.new_seed = Some(seed + 1);
                        } else {
                            self.new_seed = Some(self.seed + 1);
                        }
                    }
                },

                Event::MouseButtonUp(input::MouseButton::Right) => {
                    let x = self.mouse_x;
                    let y = self.mouse_y;

                    if let Some(index) = self.buttons.iter().position(|b| b.contains_point(x, y)) {
                        match &self.buttons[index].action_right_click {
                            Action::Advance => {
                                self.perform_action(Action::Advance)?;
                            },
                            Action::Save(filename) => {
                                if self.buttons[index].exists {
                                    if self.buttons[index].filename == PRIMARY_SAVE_NAME {
                                        self.window.show_context_menu(&[("Load [W]\0".into(), 1), ("Save [Q]\0".into(), 0)]);
                                    } else {
                                        self.window.show_context_menu(&[("Load\0".into(), 1), ("Save\0".into(), 0)]);
                                    }
                                    self.menu_context = Some(MenuContext::ButtonInfo(filename.into()));
                                }
                            },
                            _ => (),
                        };
                        break 'evloop
                    };

                    for button in self.key_buttons.iter_mut() {
                        if button.contains_point(self.mouse_x, self.mouse_y) {
                            // button.x = 123;
                            // self.perform_action(Action::Advance)?;
                            let options = match button.state {
                                ButtonState::Neutral
                                | ButtonState::NeutralWillPress
                                | ButtonState::NeutralWillPR
                                | ButtonState::NeutralWillPRP => [
                                    ("Press-Release-Press\0".into(), 3),
                                    ("Press-Release\0".into(), 2),
                                    ("Press\0".into(), 1),
                                    ("Reset\0".into(), 0),
                                ],
                                ButtonState::Held
                                | ButtonState::HeldWillRelease
                                | ButtonState::HeldWillRP
                                | ButtonState::HeldWillRPR => [
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

                    for button in self.mouse_buttons.iter_mut() {
                        if button.contains_point(self.mouse_x, self.mouse_y) {
                            let options = match button.state {
                                ButtonState::Neutral
                                | ButtonState::NeutralWillPress
                                | ButtonState::NeutralWillPR
                                | ButtonState::NeutralWillPRP => [
                                    ("Press-Release-Press\0".into(), 3),
                                    ("Press-Release\0".into(), 2),
                                    ("Press\0".into(), 1),
                                    ("Reset\0".into(), 0),
                                ],
                                ButtonState::Held
                                | ButtonState::HeldWillRelease
                                | ButtonState::HeldWillRP
                                | ButtonState::HeldWillRPR => [
                                    ("Release-Press-Release\0".into(), 7),
                                    ("Release-Press\0".into(), 6),
                                    ("Release\0".into(), 5),
                                    ("Reset\0".into(), 4),
                                ],
                            };
                            self.window.show_context_menu(&options);
                            self.menu_context = Some(MenuContext::MouseButton(button.button));
                            break 'evloop
                        }
                    }

                    // for button in self.save_buttons.iter() {
                    //     if button.contains_point(self.mouse_x, self.mouse_y) && button.exists {
                    //         self.window.show_context_menu(&[("Load\0".into(), 1), ("Save\0".into(), 0)]);
                    //         self.menu_context = Some(MenuContext::ButtonInfo(button.filename.clone()));
                    //         break 'evloop
                    //     }
                    // }
                },

                Event::MenuOption(option) => {
                    match &self.menu_context {
                        Some(MenuContext::KeyButton(target_key)) => {
                            let new_state = match option {
                                0 => ButtonState::Neutral,
                                1 => ButtonState::NeutralWillPress,
                                2 => ButtonState::NeutralWillPR,
                                3 => ButtonState::NeutralWillPRP,
                                4 => ButtonState::Held,
                                5 => ButtonState::HeldWillRelease,
                                6 => ButtonState::HeldWillRP,
                                7 => ButtonState::HeldWillRPR,
                                _ => continue,
                            };

                            for button in self.key_buttons.iter_mut() {
                                if button.key == *target_key {
                                    button.state = new_state;
                                }
                            }
                        },

                        Some(MenuContext::MouseButton(target_button)) => {
                            let new_state = match option {
                                0 => ButtonState::Neutral,
                                1 => ButtonState::NeutralWillPress,
                                2 => ButtonState::NeutralWillPR,
                                3 => ButtonState::NeutralWillPRP,
                                4 => ButtonState::Held,
                                5 => ButtonState::HeldWillRelease,
                                6 => ButtonState::HeldWillRP,
                                7 => ButtonState::HeldWillRPR,
                                _ => continue,
                            };

                            for button in self.mouse_buttons.iter_mut() {
                                if button.button == *target_button {
                                    button.state = new_state;
                                }
                            }
                        },

                        Some(MenuContext::ButtonInfo(filename)) => {
                            match option {
                                0 => {
                                    // Save
                                    let file = filename.to_string();
                                    self.perform_action(Action::Save(file))?;
                                    break
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
                self.perform_action(Action::Advance)?;
            },

            input::Key::Q => {
                self.perform_action(Action::Save(PRIMARY_SAVE_NAME.to_string()))?;
            },

            input::Key::W => {
                self.stream.send_message(&message::Message::Load {
                    keys_requested: self.key_buttons.iter().map(|x| x.key).collect(),
                    mouse_buttons_requested: Vec::new(),
                    filename: PRIMARY_SAVE_NAME.into(),
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
                ButtonState::Neutral | ButtonState::Held => (),
                ButtonState::NeutralWillPress => key_inputs.push((key.key, true)),
                ButtonState::HeldWillRelease => key_inputs.push((key.key, false)),
                ButtonState::NeutralWillPR => {
                    key_inputs.push((key.key, true));
                    key_inputs.push((key.key, false));
                },
                ButtonState::NeutralWillPRP => {
                    key_inputs.push((key.key, true));
                    key_inputs.push((key.key, false));
                    key_inputs.push((key.key, true));
                },
                ButtonState::HeldWillRP => {
                    key_inputs.push((key.key, false));
                    key_inputs.push((key.key, true));
                },
                ButtonState::HeldWillRPR => {
                    key_inputs.push((key.key, false));
                    key_inputs.push((key.key, true));
                    key_inputs.push((key.key, false));
                },
            }
        }

        let mut mouse_inputs = Vec::new();
        let mut mouse_buttons_requested = Vec::new();

        for button in self.mouse_buttons.iter() {
            mouse_buttons_requested.push(button.button);
            match button.state {
                ButtonState::Neutral | ButtonState::Held => (),
                ButtonState::NeutralWillPress => mouse_inputs.push((button.button, true)),
                ButtonState::HeldWillRelease => mouse_inputs.push((button.button, false)),
                ButtonState::NeutralWillPR => {
                    mouse_inputs.push((button.button, true));
                    mouse_inputs.push((button.button, false));
                },
                ButtonState::NeutralWillPRP => {
                    mouse_inputs.push((button.button, true));
                    mouse_inputs.push((button.button, false));
                    mouse_inputs.push((button.button, true));
                },
                ButtonState::HeldWillRP => {
                    mouse_inputs.push((button.button, false));
                    mouse_inputs.push((button.button, true));
                },
                ButtonState::HeldWillRPR => {
                    mouse_inputs.push((button.button, false));
                    mouse_inputs.push((button.button, true));
                    mouse_inputs.push((button.button, false));
                },
            }
        }

        self.stream.send_message(message::Message::Advance {
            key_inputs,
            mouse_inputs,
            mouse_location: self.game_mouse_pos,
            keys_requested,
            mouse_buttons_requested,
            instance_requested: self.watched_id,
            new_seed: self.new_seed,
        })?;

        self.await_update()
    }

    fn save(&mut self, filename: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let path = &mut self.project_dir.clone();
        path.push(filename);
        let mut msg = "created";

        if path.exists() {
            msg = "saved";
            let metadata = fs::metadata(path)?;
            if let Ok(systemtime) = metadata.modified() {
                let datetime: DateTime<Utc> = systemtime.into();
                eprintln!("{} UTC - prior updated time", datetime.format("%b %d, %Y @ %T"));
                // todo: instead simply get back this timestamp after the message stream
            }
        }

        let current_time: DateTime<Utc> = SystemTime::now().into();
        self.stream.send_message(&message::Message::Save { filename: filename.into() })?;
        eprintln!("{} UTC - {} {} - please check the file to verify", current_time.format("%b %d, %Y @ %T"), filename, msg);
        eprintln!("-----");

        Ok(true)
    }

    pub fn await_update(&mut self) -> Result<bool, Box<dyn std::error::Error>> {
        loop {
            match self.stream.receive_message::<message::Information>(&mut self.read_buffer) {
                Ok(Some(Some(message::Information::Update {
                    keys_held,
                    mouse_buttons_held,
                    mouse_location,
                    frame_count,
                    seed,
                    instance,
                }))) => {
                    self.frame_count = frame_count;
                    self.game_mouse_pos = mouse_location;
                    self.watched_instance = instance;
                    self.seed = seed;
                    self.new_seed = None;
                    for button in self.key_buttons.iter_mut() {
                        if keys_held.contains(&button.key) {
                            button.state = ButtonState::Held;
                        } else {
                            button.state = ButtonState::Neutral;
                        }
                    }
                    for button in self.mouse_buttons.iter_mut() {
                        if mouse_buttons_held.contains(&button.button) {
                            button.state = ButtonState::Held;
                        } else {
                            button.state = ButtonState::Neutral;
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

        for button in self.buttons.iter() {
            // eprintln!("{:?}", button);
            button.draw_this(&mut self.renderer, self.mouse_x, self.mouse_y);
        }

        for button in self.key_buttons.iter() {
            let alpha = if button.contains_point(self.mouse_x, self.mouse_y) { 1.0 } else { 0.6 };
            let atlas_ref_l = match button.state {
                ButtonState::Neutral
                | ButtonState::NeutralWillPress
                | ButtonState::NeutralWillPR
                | ButtonState::NeutralWillPRP => &self.key_button_l_neutral,
                ButtonState::Held
                | ButtonState::HeldWillRelease
                | ButtonState::HeldWillRP
                | ButtonState::HeldWillRPR => &self.key_button_l_held,
            };
            let atlas_ref_r = match button.state {
                ButtonState::Neutral | ButtonState::HeldWillRelease => &self.key_button_r_neutral,
                ButtonState::Held | ButtonState::NeutralWillPress => &self.key_button_r_held,
                ButtonState::NeutralWillPR => &self.key_button_r_held2,
                ButtonState::NeutralWillPRP => &self.key_button_r_held3,
                ButtonState::HeldWillRP => &self.key_button_r_neutral2,
                ButtonState::HeldWillRPR => &self.key_button_r_neutral3,
            };
            self.renderer.draw_sprite(atlas_ref_l, button.x as _, button.y as _, 1.0, 1.0, 0.0, 0xFFFFFF, alpha);
            self.renderer.draw_sprite(
                atlas_ref_r,
                f64::from(button.x + atlas_ref_l.w),
                f64::from(button.y),
                1.0,
                1.0,
                0.0,
                0xFFFFFF,
                alpha,
            );
            self.renderer.draw_sprite(
                &button.label,
                f64::from(button.x),
                f64::from(button.y),
                1.0,
                1.0,
                0.0,
                0xFFFFFF,
                alpha,
            );
            self.renderer.draw_sprite(
                &self.button_outline,
                f64::from(button.x),
                f64::from(button.y),
                1.0,
                1.0,
                0.0,
                0xFFFFFF,
                alpha,
            );
        }

        for button in self.mouse_buttons.iter() {
            let alpha = if button.contains_point(self.mouse_x, self.mouse_y) { 1.0 } else { 0.6 };
            let atlas_ref_l = match button.state {
                ButtonState::Neutral
                | ButtonState::NeutralWillPress
                | ButtonState::NeutralWillPR
                | ButtonState::NeutralWillPRP => &self.key_button_l_neutral,
                ButtonState::Held
                | ButtonState::HeldWillRelease
                | ButtonState::HeldWillRP
                | ButtonState::HeldWillRPR => &self.key_button_l_held,
            };
            let atlas_ref_r = match button.state {
                ButtonState::Neutral | ButtonState::HeldWillRelease => &self.key_button_r_neutral,
                ButtonState::Held | ButtonState::NeutralWillPress => &self.key_button_r_held,
                ButtonState::NeutralWillPR => &self.key_button_r_held2,
                ButtonState::NeutralWillPRP => &self.key_button_r_held3,
                ButtonState::HeldWillRP => &self.key_button_r_neutral2,
                ButtonState::HeldWillRPR => &self.key_button_r_neutral3,
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
            self.renderer.draw_sprite(
                &self.button_outline,
                f64::from(button.x),
                f64::from(button.y),
                1.0,
                1.0,
                0.0,
                0xFFFFFF,
                alpha,
            );
        }

        self.renderer.draw_sprite(
            &self.mouse_pos_normal,
            self.mouse_position_button.x as _,
            self.mouse_position_button.y as _,
            1.0,
            1.0,
            0.0,
            if self.mouse_position_button.active { 0x4CB122 } else { 0xFFFFFF },
            if self.mouse_position_button.contains_point(self.mouse_x, self.mouse_y) { 1.0 } else { 0.6 },
        );

        let (x, y) = self.game_mouse_pos;
        draw_text(&mut self.renderer, &format!("x: {}", x), 180.0, 266.0, &self.font_small, 0, 1.0);
        draw_text(&mut self.renderer, &format!("y: {}", y), 180.0, 286.0, &self.font_small, 0, 1.0);
        if self.mouse_position_button.active {
            let (x, y) = self.client_mouse_pos;
            draw_text(&mut self.renderer, &x.to_string(), 250.0, 266.0, &self.font_small, 0xA0A0A0, 1.0);
            draw_text(&mut self.renderer, &y.to_string(), 250.0, 286.0, &self.font_small, 0xA0A0A0, 1.0);
        }

        // for button in self.save_buttons.iter() {
        //     let alpha = if button.contains_point(self.mouse_x, self.mouse_y) { 1.0 } else { 0.75 };
        //     let atlas_ref = if button.exists { &self.save_button_active } else { &self.save_button_inactive };
        //     self.renderer.draw_sprite(atlas_ref, button.xstart as _, button.ystart as _, 1.0, 1.0, 0.0, 0xFFFFFF, alpha);
        //     draw_text(
        //         &mut self.renderer,
        //         &button.name,
        //         f64::from(button.xstart) + 8.0,
        //         f64::from(button.ystart) + 20.0,
        //         &self.font,
        //         0,
        //         alpha,
        //     );
        // } // todo: uncomment

        draw_text(&mut self.renderer, "Keyboard", 123.0, 82.0, &self.font, 0, 1.0);
        draw_text(&mut self.renderer, "Mouse", 143.0, 236.0, &self.font, 0, 1.0);
        draw_text(&mut self.renderer, "Saves", 143.0, 390.0, &self.font, 0, 1.0);

        let (seed, seed_col) = if let Some(s) = self.new_seed { (s, 0xFF) } else { (self.seed, 0) };
        draw_text(
            &mut self.renderer,
            &format!("Seed: {}", seed),
            self.seed_changer.x.into(),
            self.seed_changer.y.into(),
            &self.font_small,
            seed_col,
            if self.seed_changer.contains_point(self.mouse_x, self.mouse_y) { 1.0 } else { 0.75 },
        );

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

// section: helpers (to be consolidated into scope)

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

// pub fn draw_string(
//     &mut self,
//     x: Real,
//     y: Real,
//     string: &str,
//     line_height: Option<i32>,
//     max_width: Option<i32>,
//     xscale: Real,
//     yscale: Real,
//     angle: Real,
// ) {
