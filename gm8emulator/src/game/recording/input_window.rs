use crate::{
    imgui, input,
    game::recording::{
        self,
        KeyState,
        InputMode,
        keybinds::Binding,
        window::{Window, DisplayInformation},
    },
    types::Colour,
};
use ramen::input::Key;

pub enum ContextMenuType {
    Key(Key), Mouse(i8)
}

pub struct InputWindows {
    context_menu_type: Option<ContextMenuType>,
}

// Keyboard & Mouse window
impl Window for InputWindows {
    fn name(&self) -> String {
        "Input".to_owned()
    }

    fn show_window(&mut self, info: &mut DisplayInformation) {
        self.show_input_windows(info);
    }

    fn is_open(&self) -> bool { true }
    
    fn show_context_menu(&mut self, info: &mut DisplayInformation) -> bool {
        self.display_context_menu(info)
    }

    fn context_menu_close(&mut self) {
        self.context_menu_type = None;
    }
}

// Keyboard window
impl InputWindows {
    pub fn new() -> Self {
        InputWindows {
            context_menu_type: None,
        }
    }

    fn show_input_windows(&mut self, info: &mut DisplayInformation) {
        let set_mouse_bind_pressed = info.keybind_pressed(Binding::SetMouse);

        let DisplayInformation {
            frame,
            game,
            err_string,
            config,
            win_padding,
            mouse_state,
            keyboard_state,
            game_running,
            new_mouse_pos,
            setting_mouse_pos,
            win_frame_height,
            ..
        } = info;

        let mut request_context_menu = false;
        // Macro for keyboard keys and mouse buttons...
        macro_rules! kb_btn {
            ($name: expr, $size: expr, $x: expr, $y: expr, key $code: expr) => {
                let vk = input::ramen2vk($code);
                let state = &mut keyboard_state[usize::from(vk)];
                let clicked = frame.invisible_button($name, $size, Some(imgui::Vec2($x, $y)));
                let hovered = frame.item_hovered();
                match config.input_mode {
                    InputMode::Mouse => {
                        if clicked {
                            state.click();
                        }
                        if frame.right_clicked() && hovered {
                            unsafe {
                                cimgui_sys::igSetWindowFocusNil();
                            }
                            
                            self.context_menu_type = Some(ContextMenuType::Key($code));
                            request_context_menu = true;
                        }
                        if frame.middle_clicked() && hovered {
                            unsafe {
                                cimgui_sys::igSetWindowFocusNil();
                            }
                            *state = if state.is_held() {
                                KeyState::HeldWillDouble
                            } else {
                                KeyState::NeutralWillDouble
                            };
                        }
                    },
                    InputMode::Direct => {
                        if frame.key_pressed(vk) {
                            *state = match state {
                                // if neutral and setting would stay neutral => will press
                                KeyState::Neutral | KeyState::NeutralWillDouble | KeyState::NeutralWillCactus => {
                                    KeyState::NeutralWillPress
                                },
                                // if held but would release => keep held
                                KeyState::HeldWillRelease | KeyState::HeldWillTriple => KeyState::Held,
                                // otherwise just keep the state
                                _ => *state,
                            };
                        } else if frame.key_released(vk) {
                            *state = match state {
                                // if held and setting would stay held => will release
                                KeyState::Held | KeyState::HeldWillDouble | KeyState::HeldDoubleEveryFrame => {
                                    KeyState::HeldWillRelease
                                },
                                // if neutral but would press => keep neutral
                                KeyState::NeutralWillPress | KeyState::NeutralWillTriple => KeyState::Neutral,
                                // otherwise just keep the state
                                _ => *state,
                            };
                        }
                    },
                }
                state.draw_keystate(frame, imgui::Vec2($x, $y), $size);
                frame.text_centered($name, imgui::Vec2($x, $y) + imgui::Vec2($size.0 / 2.0, $size.1 / 2.0));
                if hovered {
                    unsafe {
                        cimgui_sys::igSetCursorPos(cimgui_sys::ImVec2 { x: 8.0, y: 22.0 });
                    }
                    frame.text(state.repr());
                }
            };
            
            ($name: expr, $size: expr, $x: expr, $y: expr, mouse $code: expr) => {
                let state = &mut mouse_state[$code as usize];
                if frame.invisible_button($name, $size, Some(imgui::Vec2($x, $y))) {
                    state.click();
                }
                let hovered = frame.item_hovered();
                if frame.right_clicked() && hovered {
                    unsafe { cimgui_sys::igSetWindowFocusNil(); }

                    self.context_menu_type = Some(ContextMenuType::Mouse($code));
                    request_context_menu = true;
                }
                if frame.middle_clicked() && hovered {
                    unsafe {
                        cimgui_sys::igSetWindowFocusNil();
                    }
                    *state = if state.is_held() { KeyState::HeldWillDouble } else { KeyState::NeutralWillDouble };
                }
                state.draw_keystate(frame, imgui::Vec2($x, $y), $size);
                frame.text_centered($name, imgui::Vec2($x, $y) + imgui::Vec2($size.0 / 2.0, $size.1 / 2.0));
                if hovered {
                    unsafe {
                        cimgui_sys::igSetCursorPos(cimgui_sys::ImVec2 { x: 8.0, y: 22.0 });
                    }
                    frame.text(state.repr());
                }
            };
            
            ($name: expr, $size: expr, $x: expr, $y: expr) => {
                let pos = frame.window_position();
                frame.invisible_button($name, $size, Some(imgui::Vec2($x, $y)));
                frame.rect(imgui::Vec2($x, $y) + pos, imgui::Vec2($x, $y) + $size + pos, recording::BTN_NEUTRAL_COL, 190);
                frame.rect_outline(
                    imgui::Vec2($x, $y) + pos,
                    imgui::Vec2($x, $y) + $size + pos,
                    Colour::new(0.4, 0.4, 0.65),
                    u8::MAX
                );
                frame.text_centered($name, imgui::Vec2($x, $y) + imgui::Vec2($size.0 / 2.0, $size.1 / 2.0));
            };
        }

        if config.full_keyboard {
            frame.setup_next_window(
                imgui::Vec2(8.0, 350.0),
                Some(imgui::Vec2(917.0, 362.0)),
                Some(imgui::Vec2(440.0, 200.0))
            );
            frame.begin_window("Keyboard###FullKeyboard", None, true, false, None);
            if !frame.window_collapsed() {
                frame.rect(
                    imgui::Vec2(0.0, *win_frame_height) + frame.window_position(),
                    imgui::Vec2(frame.window_size().0, *win_frame_height + 20.0) + frame.window_position(),
                    Colour::new(0.14, 0.14, 0.14),
                    255,
                );
                let content_min = *win_padding + imgui::Vec2(0.0, *win_frame_height * 2.0);
                let content_max = frame.window_size() - *win_padding;
                
                let mut cur_x = content_min.0;
                let mut cur_y = content_min.1;
                let left_part_edge = ((content_max.0 - content_min.0) * (15.0 / 18.5)).floor();
                let button_width = ((left_part_edge - content_min.0 - 14.0) / 15.0).floor();
                let button_height = ((content_max.1 - content_min.1 - 4.0 - (win_padding.1 * 2.0)) / 6.5).floor();
                let button_size = imgui::Vec2(button_width, button_height);
                kb_btn!("Esc", imgui::Vec2((button_width * 1.5).floor(), button_height), cur_x, cur_y, key Key::Escape);
                cur_x = left_part_edge - (button_width * 12.0 + 11.0);
                kb_btn!("F1", button_size, cur_x, cur_y, key Key::F1);
                cur_x += button_width + 1.0;
                kb_btn!("F2", button_size, cur_x, cur_y, key Key::F2);
                cur_x += button_width + 1.0;
                kb_btn!("F3", button_size, cur_x, cur_y, key Key::F3);
                cur_x += button_width + 1.0;
                kb_btn!("F4", button_size, cur_x, cur_y, key Key::F4);
                cur_x += button_width + 1.0;
                kb_btn!("F5", button_size, cur_x, cur_y, key Key::F5);
                cur_x += button_width + 1.0;
                kb_btn!("F6", button_size, cur_x, cur_y, key Key::F6);
                cur_x += button_width + 1.0;
                kb_btn!("F7", button_size, cur_x, cur_y, key Key::F7);
                cur_x += button_width + 1.0;
                kb_btn!("F8", button_size, cur_x, cur_y, key Key::F8);
                cur_x += button_width + 1.0;
                kb_btn!("F9", button_size, cur_x, cur_y, key Key::F9);
                cur_x += button_width + 1.0;
                kb_btn!("F10", button_size, cur_x, cur_y, key Key::F10);
                cur_x += button_width + 1.0;
                kb_btn!("F11", button_size, cur_x, cur_y, key Key::F11);
                cur_x += button_width + 1.0;
                kb_btn!("F12", button_size, cur_x, cur_y, key Key::F12);
                cur_x = content_max.0 - (button_width * 3.0 + 2.0);
                kb_btn!("PrSc", button_size, cur_x, cur_y, key Key::PrintScreen);
                cur_x += button_width + 1.0;
                kb_btn!("ScrLk", button_size, cur_x, cur_y, key Key::ScrollLock);
                cur_x += button_width + 1.0;
                kb_btn!("Pause", button_size, cur_x, cur_y, key Key::Pause);
                cur_x = content_min.0;
                cur_y = (content_max.1 - (win_padding.1 * 2.0)).ceil() - (button_height * 5.0 + 4.0);
                kb_btn!("`", button_size, cur_x, cur_y);
                cur_x += button_width + 1.0;
                kb_btn!("1", button_size, cur_x, cur_y, key Key::Alpha1);
                cur_x += button_width + 1.0;
                kb_btn!("2", button_size, cur_x, cur_y, key Key::Alpha2);
                cur_x += button_width + 1.0;
                kb_btn!("3", button_size, cur_x, cur_y, key Key::Alpha3);
                cur_x += button_width + 1.0;
                kb_btn!("4", button_size, cur_x, cur_y, key Key::Alpha4);
                cur_x += button_width + 1.0;
                kb_btn!("5", button_size, cur_x, cur_y, key Key::Alpha5);
                cur_x += button_width + 1.0;
                kb_btn!("6", button_size, cur_x, cur_y, key Key::Alpha6);
                cur_x += button_width + 1.0;
                kb_btn!("7", button_size, cur_x, cur_y, key Key::Alpha7);
                cur_x += button_width + 1.0;
                kb_btn!("8", button_size, cur_x, cur_y, key Key::Alpha8);
                cur_x += button_width + 1.0;
                kb_btn!("9", button_size, cur_x, cur_y, key Key::Alpha9);
                cur_x += button_width + 1.0;
                kb_btn!("0", button_size, cur_x, cur_y, key Key::Alpha0);
                cur_x += button_width + 1.0;
                kb_btn!("-", button_size, cur_x, cur_y, key Key::Minus);
                cur_x += button_width + 1.0;
                kb_btn!("=", button_size, cur_x, cur_y);
                cur_x += button_width + 1.0;
                kb_btn!("Back", imgui::Vec2(left_part_edge - cur_x, button_height), cur_x, cur_y, key Key::Backspace);
                cur_x = content_max.0 - (button_width * 3.0 + 2.0);
                kb_btn!("Ins", button_size, cur_x, cur_y, key Key::Insert);
                cur_x += button_width + 1.0;
                kb_btn!("Home", button_size, cur_x, cur_y, key Key::Home);
                cur_x += button_width + 1.0;
                kb_btn!("PgUp", button_size, cur_x, cur_y, key Key::PageUp);
                cur_x = content_min.0;
                cur_y += button_height + 1.0;
                kb_btn!("Tab", imgui::Vec2((button_width * 1.5).floor(), button_height), cur_x, cur_y, key Key::Tab);
                cur_x += (button_width * 1.5).floor() + 1.0;
                kb_btn!("Q", button_size, cur_x, cur_y, key Key::Q);
                cur_x += button_width + 1.0;
                kb_btn!("W", button_size, cur_x, cur_y, key Key::W);
                cur_x += button_width + 1.0;
                kb_btn!("E", button_size, cur_x, cur_y, key Key::E);
                cur_x += button_width + 1.0;
                kb_btn!("R", button_size, cur_x, cur_y, key Key::R);
                cur_x += button_width + 1.0;
                kb_btn!("T", button_size, cur_x, cur_y, key Key::T);
                cur_x += button_width + 1.0;
                kb_btn!("Y", button_size, cur_x, cur_y, key Key::Y);
                cur_x += button_width + 1.0;
                kb_btn!("U", button_size, cur_x, cur_y, key Key::U);
                cur_x += button_width + 1.0;
                kb_btn!("I", button_size, cur_x, cur_y, key Key::I);
                cur_x += button_width + 1.0;
                kb_btn!("O", button_size, cur_x, cur_y, key Key::O);
                cur_x += button_width + 1.0;
                kb_btn!("P", button_size, cur_x, cur_y, key Key::P);
                cur_x += button_width + 1.0;
                kb_btn!("[", button_size, cur_x, cur_y);
                cur_x += button_width + 1.0;
                kb_btn!("]", button_size, cur_x, cur_y);
                cur_x += button_width + 1.0;
                kb_btn!("Enter", imgui::Vec2(left_part_edge - cur_x, button_height * 2.0 + 1.0), cur_x, cur_y, key Key::Return);
                cur_x = content_max.0 - (button_width * 3.0 + 2.0);
                kb_btn!("Del", button_size, cur_x, cur_y, key Key::Delete);
                cur_x += button_width + 1.0;
                kb_btn!("End", button_size, cur_x, cur_y, key Key::End);
                cur_x += button_width + 1.0;
                kb_btn!("PgDn", button_size, cur_x, cur_y, key Key::PageDown);
                cur_x = content_min.0;
                cur_y += button_height + 1.0;
                kb_btn!("Caps", imgui::Vec2((button_width * 1.5).floor(), button_height), cur_x, cur_y, key Key::CapsLock);
                cur_x += (button_width * 1.5).floor() + 1.0;
                kb_btn!("A", button_size, cur_x, cur_y, key Key::A);
                cur_x += button_width + 1.0;
                kb_btn!("S", button_size, cur_x, cur_y, key Key::S);
                cur_x += button_width + 1.0;
                kb_btn!("D", button_size, cur_x, cur_y, key Key::D);
                cur_x += button_width + 1.0;
                kb_btn!("F", button_size, cur_x, cur_y, key Key::F);
                cur_x += button_width + 1.0;
                kb_btn!("G", button_size, cur_x, cur_y, key Key::G);
                cur_x += button_width + 1.0;
                kb_btn!("H", button_size, cur_x, cur_y, key Key::H);
                cur_x += button_width + 1.0;
                kb_btn!("J", button_size, cur_x, cur_y, key Key::J);
                cur_x += button_width + 1.0;
                kb_btn!("K", button_size, cur_x, cur_y, key Key::K);
                cur_x += button_width + 1.0;
                kb_btn!("L", button_size, cur_x, cur_y, key Key::L);
                cur_x += button_width + 1.0;
                kb_btn!(";", button_size, cur_x, cur_y);
                cur_x += button_width + 1.0;
                kb_btn!("'", button_size, cur_x, cur_y);
                cur_x += button_width + 1.0;
                kb_btn!("#", button_size, cur_x, cur_y);
                cur_x = content_min.0;
                cur_y += button_height + 1.0;
                kb_btn!("Shift", imgui::Vec2(button_width * 2.0, button_height), cur_x, cur_y, key Key::LeftShift);
                cur_x += button_width * 2.0 + 1.0;
                kb_btn!("\\", button_size, cur_x, cur_y);
                cur_x += button_width + 1.0;
                kb_btn!("Z", button_size, cur_x, cur_y, key Key::Z);
                cur_x += button_width + 1.0;
                kb_btn!("X", button_size, cur_x, cur_y, key Key::X);
                cur_x += button_width + 1.0;
                kb_btn!("C", button_size, cur_x, cur_y, key Key::C);
                cur_x += button_width + 1.0;
                kb_btn!("V", button_size, cur_x, cur_y, key Key::V);
                cur_x += button_width + 1.0;
                kb_btn!("B", button_size, cur_x, cur_y, key Key::B);
                cur_x += button_width + 1.0;
                kb_btn!("N", button_size, cur_x, cur_y, key Key::N);
                cur_x += button_width + 1.0;
                kb_btn!("M", button_size, cur_x, cur_y, key Key::M);
                cur_x += button_width + 1.0;
                kb_btn!(",", button_size, cur_x, cur_y, key Key::Comma);
                cur_x += button_width + 1.0;
                kb_btn!(".", button_size, cur_x, cur_y, key Key::Period);
                cur_x += button_width + 1.0;
                kb_btn!("/", button_size, cur_x, cur_y);
                cur_x += button_width + 1.0;
                kb_btn!("RShift", imgui::Vec2(left_part_edge - cur_x, button_height), cur_x, cur_y, key Key::RightShift);
                cur_x = content_min.0;
                cur_y += button_height + 1.0;
                kb_btn!("Ctrl", imgui::Vec2((button_width * 1.5).floor(), button_height), cur_x, cur_y, key Key::LeftControl);
                cur_x += (button_width * 1.5).floor() + 1.0;
                kb_btn!("Win", button_size, cur_x, cur_y, key Key::LeftSuper);
                cur_x += button_width + 1.0;
                kb_btn!("Alt", button_size, cur_x, cur_y, key Key::LeftAlt);
                cur_x += button_width + 1.0;
                kb_btn!("Space", imgui::Vec2((left_part_edge - cur_x) - (button_width * 3.5 + 3.0).floor(), button_height), cur_x, cur_y, key Key::Space);
                cur_x = left_part_edge - (button_width * 3.5 + 2.0).floor();
                kb_btn!("RAlt", button_size, cur_x, cur_y, key Key::RightAlt);
                cur_x += button_width + 1.0;
                kb_btn!("Pg", button_size, cur_x, cur_y, key Key::Applications);
                cur_x += button_width + 1.0;
                kb_btn!("RCtrl", imgui::Vec2(left_part_edge - cur_x, button_height), cur_x, cur_y, key Key::RightControl);
                cur_x = content_max.0 - (button_width * 3.0 + 2.0);
                kb_btn!("<", button_size, cur_x, cur_y, key Key::LeftArrow);
                cur_x += button_width + 1.0;
                kb_btn!("v", button_size, cur_x, cur_y, key Key::DownArrow);
                cur_y -= button_height + 1.0;
                kb_btn!("^", button_size, cur_x, cur_y, key Key::UpArrow);
                cur_x += button_width + 1.0;
                cur_y += button_height + 1.0;
                kb_btn!(">", button_size, cur_x, cur_y, key Key::RightArrow);
            }
            frame.end();
        } else {
            frame.setup_next_window(imgui::Vec2(50.0, 354.0), Some(imgui::Vec2(365.0, 192.0)), Some(imgui::Vec2(201.0, 122.0)));
            frame.begin_window("Keyboard###SimpleKeyboard", None, true, false, None);
            if !frame.window_collapsed() {
                frame.rect(
                    imgui::Vec2(0.0, *win_frame_height) + frame.window_position(),
                    imgui::Vec2(frame.window_size().0, *win_frame_height + 20.0) + frame.window_position(),
                    Colour::new(0.14, 0.14, 0.14),
                    255,
                );
                let content_min = *win_padding + imgui::Vec2(0.0, *win_frame_height * 2.0);
                let content_max = frame.window_size() - *win_padding;

                let button_width = (((content_max.0 - content_min.0) - 2.0) / 6.0).floor();
                let button_height = ((content_max.1 - content_min.1) / 2.5).floor();
                let button_size = imgui::Vec2(button_width, button_height);
                let arrows_left_bound = content_min.0 + ((content_max.0 - content_min.0) / 2.0 - (button_width * 1.5)).floor();
                kb_btn!("<", button_size, arrows_left_bound, content_max.1 - button_height - 8.0, key Key::LeftArrow);
                kb_btn!("v", button_size, arrows_left_bound + button_width + 1.0, content_max.1 - button_height - 8.0, key Key::DownArrow);
                kb_btn!(">", button_size, arrows_left_bound + (button_width * 2.0 + 2.0), content_max.1 - button_height - 8.0, key Key::RightArrow);
                kb_btn!("^", button_size, arrows_left_bound + button_width + 1.0, content_max.1 - (button_height * 2.0) - 9.0, key Key::UpArrow);
                kb_btn!("R", button_size, content_min.0, content_min.1, key Key::R);
                kb_btn!("Shift", button_size, content_min.0, content_max.1 - button_height - 8.0, key Key::LeftShift);
                kb_btn!("F2", button_size, content_max.0 - button_width, content_min.1, key Key::F2);
                kb_btn!("Z", button_size, content_max.0 - button_width, content_max.1 - button_height - 8.0, key Key::Z);
            }
            frame.end();
        }

        // Mouse window
        frame.setup_next_window(imgui::Vec2(2.0, 210.0), None, None);
        frame.begin_window("Mouse", Some(imgui::Vec2(300.0, 138.0)), false, false, None);
        if !frame.window_collapsed() {
            frame.rect(
                imgui::Vec2(0.0, *win_frame_height) + frame.window_position(),
                imgui::Vec2(frame.window_size().0, *win_frame_height + 20.0) + frame.window_position(),
                Colour::new(0.14, 0.14, 0.14),
                255,
            );

            let button_size = imgui::Vec2(40.0, 40.0);
            kb_btn!("Left", button_size, 4.0, 65.0, mouse 0);
            kb_btn!("Middle", button_size, 48.0, 65.0, mouse 2);
            kb_btn!("Right", button_size, 92.0, 65.0, mouse 1);
            if (frame.button("Set Mouse", imgui::Vec2(150.0, 20.0), Some(imgui::Vec2(150.0, 50.0)))
                || set_mouse_bind_pressed) && !config.is_read_only
            {
                if **game_running {
                    **setting_mouse_pos = true;
                } else {
                    **err_string = Some("The game is not running. Please load a savestate.".into());
                }
            }

            if let Some((x, y)) = *new_mouse_pos {
                unsafe { cimgui_sys::igPushStyleColorVec4(cimgui_sys::ImGuiCol__ImGuiCol_Text as _, cimgui_sys::ImVec4 { x: 1.0, y: 0.5, z: 0.5, w: 1.0 }); }
                frame.text_centered(&format!("x: {}*", x), imgui::Vec2(225.0, 80.0));
                frame.text_centered(&format!("y: {}*", y), imgui::Vec2(225.0, 96.0));
                unsafe { cimgui_sys::igPopStyleColor(1); }
            } else {
                frame.text_centered(&format!("x: {}", game.input.mouse_x()), imgui::Vec2(225.0, 80.0));
                frame.text_centered(&format!("y: {}", game.input.mouse_y()), imgui::Vec2(225.0, 96.0));
            }
        }
        frame.end();

        if request_context_menu {
            if !info.request_context_menu() {
                self.context_menu_type = None;
            }
        }
    }

    fn display_context_menu(&mut self, info: &mut DisplayInformation) -> bool {
        match self.context_menu_type {
            Some(ContextMenuType::Key(key)) => {
                let key_state = &mut info.keyboard_state[usize::from(input::ramen2vk(key))];
                key_state.menu(&mut info.frame)
            },
            Some(ContextMenuType::Mouse(button)) => {
                let key_state = &mut info.mouse_state[button as usize];
                key_state.menu(&mut info.frame)
            },
            None => { false },
        }
    }
}
