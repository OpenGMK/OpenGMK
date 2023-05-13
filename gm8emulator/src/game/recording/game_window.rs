use crate::{
    imgui,
    game::{
        Renderer,
        recording::{
            instance_report::InstanceReport,
            set_mouse_dialog::{SetMouseDialog, MouseDialogResult},
            window::{Window, DisplayInformation},
        },
    },
};

// for imgui callback
struct GameViewData {
    renderer: *mut Renderer,
    x: i32,
    y: i32,
    w: u32,
    h: u32,
}

pub struct GameWindow {
    callback_data: GameViewData,
    context_menu_options: Option<Vec<(String, i32)>>,
    mouse_dialog: SetMouseDialog,
    set_screencover_focus: bool,
}

// Game window
impl Window for GameWindow {
    fn name(&self) -> String {
        "Game".to_owned()
    }

    fn show_window(&mut self, info: &mut DisplayInformation) {
        if *info.game_running {
            self.display_window(info);
        } else {
            *info.setting_mouse_pos = false;
        }
    }

    fn is_open(&self) -> bool { true }

    fn show_context_menu(&mut self, info: &mut DisplayInformation) -> bool {
        self.display_context_menu(info)
    }
}

impl GameWindow {
    pub fn new() -> GameWindow {
        GameWindow {
            callback_data: GameViewData {
                w: 0,
                h: 0,
                x: 0,
                y: 0,
                renderer: std::ptr::null_mut(),
            },
            context_menu_options: None,
            mouse_dialog: SetMouseDialog::new(),
            set_screencover_focus: true,
        }
    }

    fn display_window(&mut self, info: &mut DisplayInformation) {
        if *info.setting_mouse_pos {
            if self.set_screencover_focus {
                info.frame.set_next_window_focus();
            }
            
            info.frame.begin_screen_cover();
            let screencover_focused = info.frame.window_focused();
            info.frame.end();

            if self.set_screencover_focus {
                unsafe {
                    cimgui_sys::igSetNextWindowCollapsed(false, 0);
                    cimgui_sys::igSetNextWindowFocus();
                }
            }

            if info.config.set_mouse_using_textbox {
                self.mouse_dialog.init_if_closed(info.new_mouse_pos.unwrap_or((0,0)));
                self.mouse_dialog.show_window(info);

                match self.mouse_dialog.get_result() {
                    Some(MouseDialogResult::Ok(new_mouse_pos)) => {
                        *info.new_mouse_pos = new_mouse_pos;
                        *info.setting_mouse_pos = false;
                    },
                    Some(MouseDialogResult::Cancel) => {
                        *info.setting_mouse_pos = false;
                    },
                    None => if screencover_focused && !self.set_screencover_focus {
                        // if we clicked outside the window, cancel setting the mouse
                        *info.setting_mouse_pos = false;
                    },
                }
            }

            self.set_screencover_focus = false;
        } else {
            // Next time we open a screencover we need to initially focus that one again
            self.set_screencover_focus = true;
        }
        
        let (w, h) = info.game.renderer.stored_size();
        info.frame.setup_next_window(imgui::Vec2(f32::from(info.config.ui_width) - w as f32 - 8.0, 8.0), None, None);
        info.frame.begin_window(
            &format!("{}###{}", info.game.get_window_title(), self.name()),
            Some(imgui::Vec2(
                w as f32 + (2.0 * info.win_border_size),
                h as f32 + info.win_border_size + info.win_frame_height
            )),
            false,
            false,
            None,
        );
        let imgui::Vec2(x, y) = info.frame.window_position();
        self.callback_data = GameViewData {
            renderer: (&mut info.game.renderer) as *mut _,
            x: (x + info.win_border_size) as i32,
            y: (y + info.win_frame_height) as i32,
            w: w,
            h: h,
        };

        unsafe extern "C" fn callback(
            _draw_list: *const cimgui_sys::ImDrawList,
            ptr: *const cimgui_sys::ImDrawCmd
        ) {
            let data = &*((*ptr).UserCallbackData as *mut GameViewData);
            (*data.renderer).draw_stored(data.x, data.y, data.w, data.h);
        }
        
        if !info.frame.window_collapsed() {
            info.frame.callback(callback, &mut self.callback_data);
            
            if *info.setting_mouse_pos && !info.config.set_mouse_using_textbox {
                let imgui::Vec2(mouse_x, mouse_y) = info.frame.mouse_pos();
                let position = (-(x + info.win_border_size - mouse_x) as i32, -(y + info.win_frame_height - mouse_y) as i32);
                info.frame.text_centered_float(&format!("{}, {}", position.0, position.1), imgui::Vec2(mouse_x, mouse_y-15.0));
                if info.frame.left_clicked() || info.frame.right_clicked() || info.frame.middle_clicked() {
                    *info.setting_mouse_pos = false;
                    *info.new_mouse_pos = Some(position);
                }
            }
            
            if info.frame.window_hovered() && info.frame.right_clicked() {
                self.set_context_menu_instances(info);
            }
        }
        
        info.frame.end();
    }

    fn display_context_menu(&mut self, info: &mut DisplayInformation) -> bool {
        for (label, id) in self.context_menu_options.as_ref().unwrap() {
            if info.frame.menu_item(&label) {
                if !info.config.watched_ids.contains(&id) {
                    info.config.watched_ids.push(*id);
                    info.instance_reports.push((*id, InstanceReport::new(info.game, *id)));
                    info.config.save();
                }
                self.context_menu_options = None;
                break
            }
        }

        self.context_menu_options.is_some()
    }

    /// Gets all the instances the mouse is hovered over and puts them in a context menu
    fn set_context_menu_instances(&mut self, info: &mut DisplayInformation) {
        unsafe {
            cimgui_sys::igSetWindowFocusNil();
        }
        let offset = info.frame.window_position() + imgui::Vec2(info.win_border_size, info.win_frame_height);
        let imgui::Vec2(x, y) = info.frame.mouse_pos() - offset;
        let (x, y) = info.game.translate_screen_to_room(x as _, y as _);
        
        let mut options: Vec<(String, i32)> = Vec::new();
        let mut iter = info.game.room.instance_list.iter_by_drawing();
        while let Some(handle) = iter.next(&info.game.room.instance_list) {
            let instance = info.game.room.instance_list.get(handle);
            instance.update_bbox(info.game.get_instance_mask_sprite(handle));
            if x >= instance.bbox_left.get()
            && x <= instance.bbox_right.get()
            && y >= instance.bbox_top.get()
            && y <= instance.bbox_bottom.get()
            {
                use crate::game::GetAsset;
                let id = instance.id.get();
                let description = match info.game.assets.objects.get_asset(instance.object_index.get()) {
                    Some(obj) => format!("{} ({})", obj.name, id.to_string()),
                    None => format!("<deleted object> ({})", id.to_string()),
                };
                options.push((description, id));
            }
        }
        
        if options.len() > 0 {
            if info.request_context_menu() {
                self.context_menu_options = Some(options);
            }
        }
    }
}
