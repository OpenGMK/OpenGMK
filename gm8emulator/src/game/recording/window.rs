use crate::{ 
    imgui,  game::{
        Game,
        recording::{ KeyState, ContextMenu, ProjectConfig, InstanceReport, },
        replay::Replay,
        savestate::{self, SaveState},
    },
    gml::rand::Random,
    render::RendererState,
};
use std::path::PathBuf;

 pub struct DisplayInformation<'a, 'f> {
    pub game: &'a mut Game,
    pub frame: &'a mut imgui::Frame<'f>,
    pub context_menu: &'a mut Option<ContextMenu>,
    pub game_running: &'a mut bool,
    pub setting_mouse_pos: &'a mut bool,
    pub new_mouse_pos: &'a mut Option<(i32, i32)>,
    pub new_rand: &'a mut Option<Random>,
    pub config: &'a mut ProjectConfig,
    pub err_string: &'a mut Option<String>,
    pub replay: &'a mut Replay,

    pub keyboard_state: &'a mut [KeyState; 256],
    pub mouse_state: &'a mut [KeyState; 3],
    pub savestate: &'a mut SaveState,
    pub renderer_state: &'a mut RendererState,
    pub save_buffer: &'a mut savestate::Buffer,
    pub instance_reports: &'a mut Vec<(i32, Option<InstanceReport>)>,
    
    pub save_paths: &'a Vec<PathBuf>,
    pub fps_text: &'a String,
    pub ui_renderer_state: &'a RendererState,
    pub startup_successful: &'a bool,
    pub project_path: &'a PathBuf,

    pub win_padding: imgui::Vec2<f32>,
    pub win_frame_height: f32,
    pub win_border_size: f32,
}

pub trait Window {
    fn show_window(&mut self, info: &mut DisplayInformation);
}
