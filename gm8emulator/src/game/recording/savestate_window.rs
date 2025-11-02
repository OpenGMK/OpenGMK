use crate::{
    imgui_utils::*,
    game::recording::{
        window::{Window, DisplayInformation},
        keybinds::Binding,
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
    fn name(&self) -> String {
        "Savestates".to_owned()
    }

    fn show_window(&mut self, info: &mut DisplayInformation) {
        info.frame
            .window("Savestates")
            .position([306.0, 8.0], imgui::Condition::FirstUseEver)
            .size([225.0, 330.0], imgui::Condition::FirstUseEver)
            .build(|| {
                if info.keybind_pressed(Binding::SelectNext) {
                    let mut slot = info.config.quicksave_slot;
                    if slot == self.capacity-1 {
                        slot = 0;
                    } else {
                        slot += 1;
                    }
                    while slot != info.config.quicksave_slot && !info.savestate_set_quicksave_slot(slot) {
                        if slot == self.capacity-1 {
                            slot = 0;
                        } else {
                            slot += 1;
                        }    
                    }
                }
                if info.keybind_pressed(Binding::SelectPrevious) {
                    let mut slot = info.config.quicksave_slot;
                    if slot == 0 {
                        slot = self.capacity-1;
                    } else {
                        slot -= 1;
                    }
                    while slot != info.config.quicksave_slot && !info.savestate_set_quicksave_slot(slot) {
                        if slot == 0 {
                            slot = self.capacity-1;
                        } else {
                            slot -= 1;
                        }
                    }
                }

                let rect_size = Vec2(info.frame.window_size()[0], 24.0);
                let pos = Vec2::<f32>::from(info.frame.window_pos()) + Vec2::<f32>::from(info.frame.window_content_region_min()) - Vec2(8.0, 8.0);
                for i in 0..(self.capacity/2) {
                    let min = Vec2(0.0, ((i * 2 + 1) * 24) as f32);
                    info.frame.rect(min + pos, min + rect_size + pos, Colour::new(1.0, 1.0, 1.0), 15);
                }
                for i in 0..self.capacity {
                    unsafe {
                        imgui::sys::igPushStyleColor_Vec4(
                            imgui::sys::ImGuiCol_Button as _,
                            imgui::sys::ImVec4 { x: 0.98, y: 0.59, z: 0.26, w: 0.4 });
                        imgui::sys::igPushStyleColor_Vec4(
                            imgui::sys::ImGuiCol_ButtonHovered as _,
                            imgui::sys::ImVec4 { x: 0.98, y: 0.59, z: 0.26, w: 1.0 }
                        );
                        imgui::sys::igPushStyleColor_Vec4(
                            imgui::sys::ImGuiCol_ButtonActive as _,
                            imgui::sys::ImVec4 { x: 0.98, y: 0.53, z: 0.06, w: 1.0 }
                        );
                    }
                    let y = (24 * i + 21) as f32;
                    if i == info.config.quicksave_slot {
                        let min = Vec2(0.0, (i * 24) as f32);
                        info.frame.rect(min + pos, min + rect_size + pos, Colour::new(0.1, 0.4, 0.2), 255);
                    }
                    if info.frame.button_with_size_and_pos(&self.save_text[i], Vec2(60.0, 20.0), Vec2(4.0, y))
                        && *info.game_running
                    {
                        info.savestate_save(i);
                    }

                    unsafe {
                        imgui::sys::igPopStyleColor(3);
                    }

                    if info.savestate_exists(i) {
                        if info.frame.button_with_size_and_pos(&self.load_text[i], Vec2(60.0, 20.0), Vec2(75.0, y)) && *info.startup_successful {
                            info.savestate_load(i);
                        }

                        if info.frame.button_with_size_and_pos(&self.select_text[i], Vec2(60.0, 20.0), Vec2(146.0, y)) && info.config.quicksave_slot != i {
                            info.savestate_set_quicksave_slot(i);
                        }
                    }
                }
            }
        );
    }

    fn is_open(&self) -> bool { true }
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
