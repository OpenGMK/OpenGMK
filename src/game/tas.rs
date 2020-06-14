use crate::{
    game::window::{Window, WindowBuilder},
    input,
};

const WINDOW_WIDTH: u32 = 300;
const WINDOW_HEIGHT: u32 = 750;

pub struct ControlPanel {
    pub window: Window,
    pub buttons: Vec<Button>,
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
        window.set_visible(true);
        Ok(Self {
            window,
            buttons: vec![
                Button { x: 126, y: 100, key: input::Key::Left },
                Button { x: 174, y: 100, key: input::Key::Down },
                Button { x: 222, y: 100, key: input::Key::Right },
                Button { x: 174, y: 52, key: input::Key::Up },
            ],
        })
    }
}
