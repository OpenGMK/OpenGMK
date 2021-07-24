use crate::{ 
    imgui,  game::{
        Game,
        recording::{
            ContextMenu,
            ProjectConfig,
        },
    },
    gml::rand::Random,
};

 pub struct DisplayInformation<'a, 'f> {
    pub game: &'a mut Game,
    pub frame: &'a mut imgui::Frame<'f>,
    pub context_menu: &'a mut Option<ContextMenu>,
    pub game_running: &'a mut bool,
    pub setting_mouse_pos: &'a mut bool,
    pub new_mouse_pos: &'a mut Option<(i32, i32)>,
    pub new_rand: &'a mut Option<Random>,
    pub config: &'a mut ProjectConfig,

    pub win_padding: imgui::Vec2<f32>,
    pub win_frame_height: f32,
    pub win_border_size: f32,
}

pub trait Window {
    fn show_window(&mut self, info: &mut DisplayInformation);
}
