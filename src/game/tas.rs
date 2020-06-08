use crate::{
    atlas::{AtlasBuilder, AtlasRef},
    game::window::{Window, WindowBuilder},
    input,
    render::{opengl::OpenGLRenderer, Renderer, RendererOptions},
    types::Colour,
};

const WINDOW_WIDTH: u32 = 300;
const WINDOW_HEIGHT: u32 = 750;

pub struct ControlPanel {
    pub window: Window,
    pub renderer: Box<dyn Renderer>,
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
        let mut renderer = OpenGLRenderer::new(
            RendererOptions {
                title: "",
                size: (WINDOW_WIDTH, WINDOW_HEIGHT),
                icons: vec![],
                global_clear_colour: Colour::new(1.0, 142.0 / 255.0, 250.0 / 255.0),
                resizable: false,
                on_top: true,
                decorations: true,
                fullscreen: false,
                vsync: false,
            },
            &window,
        )?;
        window.set_visible(true);
        renderer.swap_interval(0);

        let mut atlases = AtlasBuilder::new(1024);
        let mut bytes: Vec<u8> = Vec::with_capacity(48 * 48 * 4);
        for _ in 0..48 {
            bytes.push(0);
            bytes.push(0);
            bytes.push(0);
            bytes.push(0xFF);
        }
        for _ in 0..46 {
            bytes.push(0);
            bytes.push(0);
            bytes.push(0);
            bytes.push(0xFF);

            for _ in 0..(46 * 4) {
                bytes.push(0xFF);
            }

            bytes.push(0);
            bytes.push(0);
            bytes.push(0);
            bytes.push(0xFF);
        }
        for _ in 0..48 {
            bytes.push(0);
            bytes.push(0);
            bytes.push(0);
            bytes.push(0xFF);
        }
        let atlas_ref =
            atlases.texture(48, 48, 0, 0, bytes.into_boxed_slice()).ok_or("Couldn't pack images for control panel")?;
        renderer.upload_atlases(atlases)?;

        renderer.finish(WINDOW_WIDTH, WINDOW_HEIGHT);
        Ok(Self {
            window,
            renderer: Box::new(renderer),
            buttons: vec![
                Button { x: 126, y: 100, key: input::Key::Left },
                Button { x: 174, y: 100, key: input::Key::Down },
                Button { x: 222, y: 100, key: input::Key::Right },
                Button { x: 174, y: 52, key: input::Key::Up },
            ],
            button_graphic: atlas_ref,
        })
    }

    pub fn draw(&mut self) {
        for button in self.buttons.iter() {
            self.renderer.draw_sprite(&self.button_graphic, button.x as _, button.y as _, 1.0, 1.0, 0.0, 0xFFFFFF, 1.0);
        }
        self.renderer.finish(WINDOW_WIDTH, WINDOW_HEIGHT)
    }

    pub fn set_current(&self) -> bool {
        self.renderer.set_current()
    }
}
