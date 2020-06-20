use crate::{
    atlas::{AtlasBuilder, AtlasRef},
    window::{Event, Window, WindowBuilder},
    input,
    render::{Renderer, RendererOptions},
    types::Colour,
};

const WINDOW_WIDTH: u32 = 350;
const WINDOW_HEIGHT: u32 = 750;

const KEY_BUTTON_SIZE: usize = 48;

pub struct ControlPanel {
    pub window: Window,
    pub renderer: Renderer,
    pub key_buttons: Vec<KeyButton>,
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
}

pub struct KeyButton {
    pub x: i32,
    pub y: i32,
    pub key: input::Key,
    pub state: KeyButtonState,
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

impl ControlPanel {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let wb = WindowBuilder::new().with_size(WINDOW_WIDTH, WINDOW_HEIGHT);
        let mut window = wb.build()?;
        let mut renderer = Renderer::new((),
            &RendererOptions {
                size: (WINDOW_WIDTH, WINDOW_HEIGHT),
                clear_colour: Colour::new(220.0 / 255.0, 220.0 / 255.0, 220.0 / 255.0),
                vsync: false,
            },
            &window,
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
        renderer.push_atlases(atlases)?;

        renderer.finish(WINDOW_WIDTH, WINDOW_HEIGHT);
        Ok(Self {
            window,
            renderer,
            key_buttons: vec![
                KeyButton { x: 103, y: 100, key: input::Key::Left, state: KeyButtonState::Neutral },
                KeyButton { x: 151, y: 100, key: input::Key::Down, state: KeyButtonState::Neutral },
                KeyButton { x: 199, y: 100, key: input::Key::Right, state: KeyButtonState::Neutral },
                KeyButton { x: 151, y: 52, key: input::Key::Up, state: KeyButtonState::Neutral },
            ],
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
        })
    }

    pub fn update(&mut self) {
        for event in self.window.process_events() {
            match event {
                Event::MouseMove(x, y) => {
                    self.mouse_x = *x;
                    self.mouse_y = *y;
                },

                Event::MouseButtonDown(input::MouseButton::Left) => {
                    for button in self.key_buttons.iter_mut() {
                        if button.contains_point(self.mouse_x, self.mouse_y) {
                            button.state = match button.state {
                                KeyButtonState::Neutral => KeyButtonState::NeutralWillPress,
                                KeyButtonState::NeutralWillPress | KeyButtonState::NeutralWillPR | KeyButtonState::NeutralWillPRP => KeyButtonState::Neutral,
                                KeyButtonState::Held => KeyButtonState::HeldWillRelease,
                                KeyButtonState::HeldWillRelease | KeyButtonState::HeldWillRP | KeyButtonState::HeldWillRPR => KeyButtonState::Held,
                            };
                        }
                    }
                },

                _ => (),
            }
        }
    }

    pub fn draw(&mut self) {
        self.renderer.set_view(WINDOW_WIDTH, WINDOW_HEIGHT, WINDOW_WIDTH, WINDOW_HEIGHT, 0, 0, WINDOW_WIDTH as _, WINDOW_HEIGHT as _, 0.0, 0, 0, WINDOW_WIDTH as _, WINDOW_HEIGHT as _);
        for button in self.key_buttons.iter() {
            let alpha = if button.contains_point(self.mouse_x, self.mouse_y) { 1.0 } else { 0.6 };
            let atlas_ref_l = match button.state {
                KeyButtonState::Neutral | KeyButtonState::NeutralWillPress | KeyButtonState::NeutralWillPR | KeyButtonState::NeutralWillPRP => &self.key_button_l_neutral,
                KeyButtonState::Held | KeyButtonState::HeldWillRelease | KeyButtonState::HeldWillRP | KeyButtonState::HeldWillRPR => &self.key_button_l_neutral,
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
            self.renderer.draw_sprite(atlas_ref_r, (button.x + atlas_ref_l.w) as _, button.y as _, 1.0, 1.0, 0.0, 0xFFFFFF, alpha);
        }
        self.renderer.finish(WINDOW_WIDTH, WINDOW_HEIGHT)
    }

    // Little helper function, input MUST be a BMP file in 32-bit RGBA format. Best used with include_bytes!()
    fn upload_bmp(atlases: &mut AtlasBuilder, bmp: &[u8]) -> AtlasRef {
        fn read_u32(data: &[u8], pos: usize) -> u32 {
            let bytes = &data[pos..pos+4];
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
        atlases.texture(w as _, h as _, 0, 0, corrected_rgba.into_boxed_slice())
            .expect("Failed to pack a texture for control panel")
    }
}
