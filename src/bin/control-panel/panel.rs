use crate::{
    atlas::{AtlasBuilder, AtlasRef},
    window::{Window, WindowBuilder},
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
    pub buttons: Vec<Button>,
    button_graphic: AtlasRef,
}

pub struct Button {
    pub x: isize,
    pub y: isize,
    pub key: input::Key,
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
        let mut bytes: Vec<u8> = Vec::with_capacity(KEY_BUTTON_SIZE * KEY_BUTTON_SIZE * 4);
        for _ in 0..KEY_BUTTON_SIZE {
            bytes.push(0);
            bytes.push(0);
            bytes.push(0);
            bytes.push(0xFF);
        }
        for _ in 0..(KEY_BUTTON_SIZE - 2) {
            bytes.push(0);
            bytes.push(0);
            bytes.push(0);
            bytes.push(0xFF);

            for _ in 0..((KEY_BUTTON_SIZE - 2) * 4) {
                bytes.push(0xFF);
            }

            bytes.push(0);
            bytes.push(0);
            bytes.push(0);
            bytes.push(0xFF);
        }
        for _ in 0..KEY_BUTTON_SIZE {
            bytes.push(0);
            bytes.push(0);
            bytes.push(0);
            bytes.push(0xFF);
        }
        let atlas_ref = atlases.texture(KEY_BUTTON_SIZE as _, KEY_BUTTON_SIZE as _, 0, 0, bytes.into_boxed_slice()).ok_or("Couldn't pack images for control panel")?;
        renderer.push_atlases(atlases)?;

        renderer.finish(WINDOW_WIDTH, WINDOW_HEIGHT);
        Ok(Self {
            window,
            renderer,
            buttons: vec![
                Button { x: 103, y: 100, key: input::Key::Left },
                Button { x: 151, y: 100, key: input::Key::Down },
                Button { x: 199, y: 100, key: input::Key::Right },
                Button { x: 151, y: 52, key: input::Key::Up },
            ],
            button_graphic: atlas_ref,
        })
    }

    pub fn update(&mut self) {
        self.window.process_events();
    }

    pub fn draw(&mut self) {
        self.renderer.set_view(WINDOW_WIDTH, WINDOW_HEIGHT, WINDOW_WIDTH, WINDOW_HEIGHT, 0, 0, WINDOW_WIDTH as _, WINDOW_HEIGHT as _, 0.0, 0, 0, WINDOW_WIDTH as _, WINDOW_HEIGHT as _);
        for button in self.buttons.iter() {
            self.renderer.draw_sprite(&self.button_graphic, button.x as _, button.y as _, 1.0, 1.0, 0.0, 0xFFFFFF, 1.0);
        }
        self.renderer.finish(WINDOW_WIDTH, WINDOW_HEIGHT)
    }
}