use imgui::StyleColor;

use crate::{
    imgui_utils::*,
    game::recording::{
        window::{Window, EmulatorContext},
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

    fn show_window(&mut self, info: &mut EmulatorContext) {
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
                    // Is this really the way to handle multiple values pushed on the stack in this lib?
                    let button_color = info.frame.push_style_color(StyleColor::Button, [0.98, 0.59, 0.26, 0.4]);
                    let button_hovered_color = info.frame.push_style_color(StyleColor::ButtonHovered, [0.98, 0.59, 0.26, 1.0]);
                    let button_active_color = info.frame.push_style_color(StyleColor::ButtonActive, [0.98, 0.53, 0.06, 1.0]);

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

                    button_active_color.end();
                    button_hovered_color.end();
                    button_color.end();

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

    fn is_open(&self) -> bool {
        true
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
