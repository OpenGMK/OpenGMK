use crate::{
    imgui,
    game::{
        recording::{
            window::{Window, DisplayInformation},
        },
    },
    types::Colour,
};

// Savestates window
pub struct SaveStateWindow {
    capacity: usize,
    save_text: Vec<String>,
    load_text: Vec<String>,
    select_text: Vec<String>,
}

impl Window for SaveStateWindow {
    fn show_window(&mut self, info: &mut DisplayInformation) {
        info.frame.setup_next_window(imgui::Vec2(306.0, 8.0), Some(imgui::Vec2(225.0, 330.0)), None);
        info.frame.begin_window("Savestates", None, true, false, None);
        let rect_size = imgui::Vec2(info.frame.window_size().0, 24.0);
        let pos = info.frame.window_position() + info.frame.content_position() - imgui::Vec2(8.0, 8.0);
        for i in 0..(self.capacity/2) {
            let min = imgui::Vec2(0.0, ((i * 2 + 1) * 24) as f32);
            info.frame.rect(min + pos, min + rect_size + pos, Colour::new(1.0, 1.0, 1.0), 15);
        }
        for i in 0..self.capacity {
            unsafe {
                cimgui_sys::igPushStyleColorVec4(cimgui_sys::ImGuiCol__ImGuiCol_Button as _, cimgui_sys::ImVec4 { x: 0.98, y: 0.59, z: 0.26, w: 0.4 });
                cimgui_sys::igPushStyleColorVec4(cimgui_sys::ImGuiCol__ImGuiCol_ButtonHovered as _, cimgui_sys::ImVec4 { x: 0.98, y: 0.59, z: 0.26, w: 1.0 });
                cimgui_sys::igPushStyleColorVec4(cimgui_sys::ImGuiCol__ImGuiCol_ButtonActive as _, cimgui_sys::ImVec4 { x: 0.98, y: 0.53, z: 0.06, w: 1.0 });
            }
            let y = (24 * i + 21) as f32;
            if i == info.config.quicksave_slot {
                let min = imgui::Vec2(0.0, (i * 24) as f32);
                info.frame.rect(min + pos, min + rect_size + pos, Colour::new(0.1, 0.4, 0.2), 255);
            }
            if info.frame.button(&self.save_text[i], imgui::Vec2(60.0, 20.0), Some(imgui::Vec2(4.0, y))) && *info.game_running {
                info.savestate_save(i);
            }
            unsafe {
                cimgui_sys::igPopStyleColor(3);
            }

            if info.savestate_exists(i) {
                if info.frame.button(&self.load_text[i], imgui::Vec2(60.0, 20.0), Some(imgui::Vec2(75.0, y))) && *info.startup_successful {
                    info.savestate_load(i);
                }

                if info.frame.button(&self.select_text[i], imgui::Vec2(60.0, 20.0), Some(imgui::Vec2(146.0, y))) && info.config.quicksave_slot != i {
                    info.savestate_set_quicksave_slot(i);
                }
            }
        }
        info.frame.end();
    }
}

impl SaveStateWindow {
    pub fn new(capacity: usize) -> Self {
        SaveStateWindow {
            capacity: capacity,
            save_text: (0..capacity).map(|i| format!("Save {}", i + 1)).collect::<Vec<_>>(),
            load_text: (0..capacity).map(|i| format!("Load {}", i + 1)).collect::<Vec<_>>(),
            select_text: (0..capacity).map(|i| format!("Select###Select{}", i + 1)).collect::<Vec<_>>(),
        }
    }
}
