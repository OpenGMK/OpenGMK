use crate::{
    imgui_utils::*,
    input,
    game::recording::{
        self,
        KeyState,
        InputMode,
        keybinds::Binding,
        window::{Window, EmulatorContext},
    },
    types::Colour,
};
use imgui::StyleColor;
use ramen::input::Key;

pub enum ContextMenuType {
    Key(Key), Mouse(i8)
}

pub struct InputWindows {
    context_menu_type: Option<ContextMenuType>,
    request_context_menu: bool,
}

// Keyboard & Mouse window
impl Window for InputWindows {
    fn name(&self) -> String {
        "Input".to_owned()
    }

    fn show_window(&mut self, info: &mut EmulatorContext) {
        self.show_input_windows(info);
    }

    fn is_open(&self) -> bool { true }
    
    fn show_context_menu(&mut self, info: &mut EmulatorContext) -> bool {
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
            request_context_menu: false,
        }
    }

    fn request_context_menu(&mut self, menu_type: ContextMenuType) {
        self.context_menu_type = Some(menu_type);
        self.request_context_menu = true;
    }

    fn show_input_windows(&mut self, info: &mut EmulatorContext) {
        let config = &info.config;
        let frame = info.frame;

        frame
            .window(if config.full_keyboard { "Keyboard###FullKeyboard" } else { "Keyboard###SimpleKeyboard" })
            .resizable(false)
            .scroll_bar(false)
            .scrollable(false)
            .position(if config.full_keyboard { [8.0, 350.0] } else { [50.0, 354.0] }, imgui::Condition::FirstUseEver)
            .size(if config.full_keyboard { [917.0, 362.0] } else { [365.0, 192.0] }, imgui::Condition::Always)
            .size_constraints(if config.full_keyboard { [440.0, 200.0] } else { [201.0, 122.0] }, [-1.0, -1.0])
            .build(|| {
                self.render_keyboard_window(info);
            });

        frame
            .window("Mouse")
            .position([2.0, 210.0], imgui::Condition::FirstUseEver)
            .size([300.0, 138.0], imgui::Condition::Always)
            .build(|| self.render_mouse_window(info));

        if self.request_context_menu {
            if !info.request_context_menu() {
                self.context_menu_type = None;
            }
            self.request_context_menu = false;
        }
    }

    /// Renders the keyboard state menu into an imgui window
    fn render_keyboard_window(&mut self, info: &mut EmulatorContext) {
        let win_frame_height = info.win_frame_height;
        let win_padding = info.win_padding;
        let frame = info.frame;

        if info.config.full_keyboard {
            frame.rect(
                Vec2(0.0, win_frame_height) + Vec2::from(frame.window_pos()),
                Vec2(frame.window_size()[0], win_frame_height + 20.0) + Vec2::from(frame.window_pos()),
                Colour::new(0.14, 0.14, 0.14),
                255,
            );
            
            let content_min = win_padding + Vec2(0.0, win_frame_height * 2.0);
            let content_max = Vec2::from(frame.window_size()) - win_padding;

            let mut cur_x = content_min.0;
            let mut cur_y = content_min.1;
            let left_part_edge = ((content_max.0 - content_min.0) * (15.0 / 18.5)).floor();
            let button_width = ((left_part_edge - content_min.0 - 14.0) / 15.0).floor();
            let button_height = ((content_max.1 - content_min.1 - 4.0 - (win_padding.1 * 2.0)) / 6.5).floor();
            let button_size = Vec2(button_width, button_height);
            self.render_keyboard_button(
                info,
                "Esc",
                Vec2((button_width * 1.5).floor(), button_height),
                cur_x,
                cur_y,
                Key::Escape,
            );
            cur_x = left_part_edge - (button_width * 12.0 + 11.0);
            self.render_keyboard_button(info, "F1", button_size, cur_x, cur_y, Key::F1);
            cur_x += button_width + 1.0;
            self.render_keyboard_button(info, "F2", button_size, cur_x, cur_y, Key::F2);
            cur_x += button_width + 1.0;
            self.render_keyboard_button(info, "F3", button_size, cur_x, cur_y, Key::F3);
            cur_x += button_width + 1.0;
            self.render_keyboard_button(info, "F4", button_size, cur_x, cur_y, Key::F4);
            cur_x += button_width + 1.0;
            self.render_keyboard_button(info, "F5", button_size, cur_x, cur_y, Key::F5);
            cur_x += button_width + 1.0;
            self.render_keyboard_button(info, "F6", button_size, cur_x, cur_y, Key::F6);
            cur_x += button_width + 1.0;
            self.render_keyboard_button(info, "F7", button_size, cur_x, cur_y, Key::F7);
            cur_x += button_width + 1.0;
            self.render_keyboard_button(info, "F8", button_size, cur_x, cur_y, Key::F8);
            cur_x += button_width + 1.0;
            self.render_keyboard_button(info, "F9", button_size, cur_x, cur_y, Key::F9);
            cur_x += button_width + 1.0;
            self.render_keyboard_button(info, "F10", button_size, cur_x, cur_y, Key::F10);
            cur_x += button_width + 1.0;
            self.render_keyboard_button(info, "F11", button_size, cur_x, cur_y, Key::F11);
            cur_x += button_width + 1.0;
            self.render_keyboard_button(info, "F12", button_size, cur_x, cur_y, Key::F12);
            cur_x = content_max.0 - (button_width * 3.0 + 2.0);
            self.render_keyboard_button(info, "PrSc", button_size, cur_x, cur_y, Key::PrintScreen);
            cur_x += button_width + 1.0;
            self.render_keyboard_button(info, "ScrLk", button_size, cur_x, cur_y, Key::ScrollLock);
            cur_x += button_width + 1.0;
            self.render_keyboard_button(info, "Pause", button_size, cur_x, cur_y, Key::Pause);
            cur_x = content_min.0;
            cur_y = (content_max.1 - (win_padding.1 * 2.0)).ceil() - (button_height * 5.0 + 4.0);
            self.render_dummy_button(frame, "`", button_size, cur_x, cur_y);
            cur_x += button_width + 1.0;
            self.render_keyboard_button(info, "1", button_size, cur_x, cur_y, Key::Alpha1);
            cur_x += button_width + 1.0;
            self.render_keyboard_button(info, "2", button_size, cur_x, cur_y, Key::Alpha2);
            cur_x += button_width + 1.0;
            self.render_keyboard_button(info, "3", button_size, cur_x, cur_y, Key::Alpha3);
            cur_x += button_width + 1.0;
            self.render_keyboard_button(info, "4", button_size, cur_x, cur_y, Key::Alpha4);
            cur_x += button_width + 1.0;
            self.render_keyboard_button(info, "5", button_size, cur_x, cur_y, Key::Alpha5);
            cur_x += button_width + 1.0;
            self.render_keyboard_button(info, "6", button_size, cur_x, cur_y, Key::Alpha6);
            cur_x += button_width + 1.0;
            self.render_keyboard_button(info, "7", button_size, cur_x, cur_y, Key::Alpha7);
            cur_x += button_width + 1.0;
            self.render_keyboard_button(info, "8", button_size, cur_x, cur_y, Key::Alpha8);
            cur_x += button_width + 1.0;
            self.render_keyboard_button(info, "9", button_size, cur_x, cur_y, Key::Alpha9);
            cur_x += button_width + 1.0;
            self.render_keyboard_button(info, "0", button_size, cur_x, cur_y, Key::Alpha0);
            cur_x += button_width + 1.0;
            self.render_keyboard_button(info, "-", button_size, cur_x, cur_y, Key::Minus);
            cur_x += button_width + 1.0;
            self.render_dummy_button(frame, "=", button_size, cur_x, cur_y);
            cur_x += button_width + 1.0;
            self.render_keyboard_button(
                info,
                "Back",
                Vec2(left_part_edge - cur_x, button_height),
                cur_x,
                cur_y,
                Key::Backspace,
            );
            cur_x = content_max.0 - (button_width * 3.0 + 2.0);
            self.render_keyboard_button(info, "Ins", button_size, cur_x, cur_y, Key::Insert);
            cur_x += button_width + 1.0;
            self.render_keyboard_button(info, "Home", button_size, cur_x, cur_y, Key::Home);
            cur_x += button_width + 1.0;
            self.render_keyboard_button(info, "PgUp", button_size, cur_x, cur_y, Key::PageUp);
            cur_x = content_min.0;
            cur_y += button_height + 1.0;
            self.render_keyboard_button(
                info,
                "Tab",
                Vec2((button_width * 1.5).floor(), button_height),
                cur_x,
                cur_y,
                Key::Tab,
            );
            cur_x += (button_width * 1.5).floor() + 1.0;
            self.render_keyboard_button(info, "Q", button_size, cur_x, cur_y, Key::Q);
            cur_x += button_width + 1.0;
            self.render_keyboard_button(info, "W", button_size, cur_x, cur_y, Key::W);
            cur_x += button_width + 1.0;
            self.render_keyboard_button(info, "E", button_size, cur_x, cur_y, Key::E);
            cur_x += button_width + 1.0;
            self.render_keyboard_button(info, "R", button_size, cur_x, cur_y, Key::R);
            cur_x += button_width + 1.0;
            self.render_keyboard_button(info, "T", button_size, cur_x, cur_y, Key::T);
            cur_x += button_width + 1.0;
            self.render_keyboard_button(info, "Y", button_size, cur_x, cur_y, Key::Y);
            cur_x += button_width + 1.0;
            self.render_keyboard_button(info, "U", button_size, cur_x, cur_y, Key::U);
            cur_x += button_width + 1.0;
            self.render_keyboard_button(info, "I", button_size, cur_x, cur_y, Key::I);
            cur_x += button_width + 1.0;
            self.render_keyboard_button(info, "O", button_size, cur_x, cur_y, Key::O);
            cur_x += button_width + 1.0;
            self.render_keyboard_button(info, "P", button_size, cur_x, cur_y, Key::P);
            cur_x += button_width + 1.0;
            self.render_dummy_button(frame, "[", button_size, cur_x, cur_y);
            cur_x += button_width + 1.0;
            self.render_dummy_button(frame, "]", button_size, cur_x, cur_y);
            cur_x += button_width + 1.0;
            self.render_keyboard_button(
                info,
                "Enter",
                Vec2(left_part_edge - cur_x, button_height * 2.0 + 1.0),
                cur_x,
                cur_y,
                Key::Return,
            );
            cur_x = content_max.0 - (button_width * 3.0 + 2.0);
            self.render_keyboard_button(info, "Del", button_size, cur_x, cur_y, Key::Delete);
            cur_x += button_width + 1.0;
            self.render_keyboard_button(info, "End", button_size, cur_x, cur_y, Key::End);
            cur_x += button_width + 1.0;
            self.render_keyboard_button(info, "PgDn", button_size, cur_x, cur_y, Key::PageDown);
            cur_x = content_min.0;
            cur_y += button_height + 1.0;
            self.render_keyboard_button(
                info,
                "Caps",
                Vec2((button_width * 1.5).floor(), button_height),
                cur_x,
                cur_y,
                Key::CapsLock,
            );
            cur_x += (button_width * 1.5).floor() + 1.0;
            self.render_keyboard_button(info, "A", button_size, cur_x, cur_y, Key::A);
            cur_x += button_width + 1.0;
            self.render_keyboard_button(info, "S", button_size, cur_x, cur_y, Key::S);
            cur_x += button_width + 1.0;
            self.render_keyboard_button(info, "D", button_size, cur_x, cur_y, Key::D);
            cur_x += button_width + 1.0;
            self.render_keyboard_button(info, "F", button_size, cur_x, cur_y, Key::F);
            cur_x += button_width + 1.0;
            self.render_keyboard_button(info, "G", button_size, cur_x, cur_y, Key::G);
            cur_x += button_width + 1.0;
            self.render_keyboard_button(info, "H", button_size, cur_x, cur_y, Key::H);
            cur_x += button_width + 1.0;
            self.render_keyboard_button(info, "J", button_size, cur_x, cur_y, Key::J);
            cur_x += button_width + 1.0;
            self.render_keyboard_button(info, "K", button_size, cur_x, cur_y, Key::K);
            cur_x += button_width + 1.0;
            self.render_keyboard_button(info, "L", button_size, cur_x, cur_y, Key::L);
            cur_x += button_width + 1.0;
            self.render_dummy_button(frame, ";", button_size, cur_x, cur_y);
            cur_x += button_width + 1.0;
            self.render_dummy_button(frame, "'", button_size, cur_x, cur_y);
            cur_x += button_width + 1.0;
            self.render_dummy_button(frame, "#", button_size, cur_x, cur_y);
            cur_x = content_min.0;
            cur_y += button_height + 1.0;
            self.render_keyboard_button(
                info,
                "Shift",
                Vec2(button_width * 2.0, button_height),
                cur_x,
                cur_y,
                Key::LeftShift,
            );
            cur_x += button_width * 2.0 + 1.0;
            self.render_dummy_button(frame, "\\", button_size, cur_x, cur_y);
            cur_x += button_width + 1.0;
            self.render_keyboard_button(info, "Z", button_size, cur_x, cur_y, Key::Z);
            cur_x += button_width + 1.0;
            self.render_keyboard_button(info, "X", button_size, cur_x, cur_y, Key::X);
            cur_x += button_width + 1.0;
            self.render_keyboard_button(info, "C", button_size, cur_x, cur_y, Key::C);
            cur_x += button_width + 1.0;
            self.render_keyboard_button(info, "V", button_size, cur_x, cur_y, Key::V);
            cur_x += button_width + 1.0;
            self.render_keyboard_button(info, "B", button_size, cur_x, cur_y, Key::B);
            cur_x += button_width + 1.0;
            self.render_keyboard_button(info, "N", button_size, cur_x, cur_y, Key::N);
            cur_x += button_width + 1.0;
            self.render_keyboard_button(info, "M", button_size, cur_x, cur_y, Key::M);
            cur_x += button_width + 1.0;
            self.render_keyboard_button(info, ",", button_size, cur_x, cur_y, Key::Comma);
            cur_x += button_width + 1.0;
            self.render_keyboard_button(info, ".", button_size, cur_x, cur_y, Key::Period);
            cur_x += button_width + 1.0;
            self.render_dummy_button(frame, "/", button_size, cur_x, cur_y);
            cur_x += button_width + 1.0;
            self.render_keyboard_button(
                info,
                "RShift",
                Vec2(left_part_edge - cur_x, button_height),
                cur_x,
                cur_y,
                Key::RightShift,
            );
            cur_x = content_min.0;
            cur_y += button_height + 1.0;
            self.render_keyboard_button(
                info,
                "Ctrl",
                Vec2((button_width * 1.5).floor(), button_height),
                cur_x,
                cur_y,
                Key::LeftControl,
            );
            cur_x += (button_width * 1.5).floor() + 1.0;
            self.render_keyboard_button(info, "Win", button_size, cur_x, cur_y, Key::LeftSuper);
            cur_x += button_width + 1.0;
            self.render_keyboard_button(info, "Alt", button_size, cur_x, cur_y, Key::LeftAlt);
            cur_x += button_width + 1.0;
            self.render_keyboard_button(
                info,
                "Space",
                Vec2((left_part_edge - cur_x) - (button_width * 3.5 + 3.0).floor(), button_height),
                cur_x,
                cur_y,
                Key::Space,
            );
            cur_x = left_part_edge - (button_width * 3.5 + 2.0).floor();
            self.render_keyboard_button(info, "RAlt", button_size, cur_x, cur_y, Key::RightAlt);
            cur_x += button_width + 1.0;
            self.render_keyboard_button(info, "Pg", button_size, cur_x, cur_y, Key::Applications);
            cur_x += button_width + 1.0;
            self.render_keyboard_button(
                info,
                "RCtrl",
                Vec2(left_part_edge - cur_x, button_height),
                cur_x,
                cur_y,
                Key::RightControl,
            );
            cur_x = content_max.0 - (button_width * 3.0 + 2.0);
            self.render_keyboard_button(info, "<", button_size, cur_x, cur_y, Key::LeftArrow);
            cur_x += button_width + 1.0;
            self.render_keyboard_button(info, "v", button_size, cur_x, cur_y, Key::DownArrow);
            cur_y -= button_height + 1.0;
            self.render_keyboard_button(info, "^", button_size, cur_x, cur_y, Key::UpArrow);
            cur_x += button_width + 1.0;
            cur_y += button_height + 1.0;
            self.render_keyboard_button(info, ">", button_size, cur_x, cur_y, Key::RightArrow);
        } else {
            frame.rect(
                Vec2(0.0, win_frame_height) + Vec2::from(frame.window_pos()),
                Vec2(frame.window_size()[0], win_frame_height + 20.0) + Vec2::from(frame.window_pos()),
                Colour::new(0.14, 0.14, 0.14),
                255,
            );
            let content_min = win_padding + Vec2(0.0, win_frame_height * 2.0);
            let content_max = Vec2::from(frame.window_size()) - win_padding;

            let button_width = (((content_max.0 - content_min.0) - 2.0) / 6.0).floor();
            let button_height = ((content_max.1 - content_min.1) / 2.5).floor();
            let button_size = Vec2(button_width, button_height);
            let arrows_left_bound =
                content_min.0 + ((content_max.0 - content_min.0) / 2.0 - (button_width * 1.5)).floor();
            self.render_keyboard_button(
                info,
                "<",
                button_size,
                arrows_left_bound,
                content_max.1 - button_height - 8.0,
                Key::LeftArrow,
            );
            self.render_keyboard_button(
                info,
                "v",
                button_size,
                arrows_left_bound + button_width + 1.0,
                content_max.1 - button_height - 8.0,
                Key::DownArrow,
            );
            self.render_keyboard_button(
                info,
                ">",
                button_size,
                arrows_left_bound + (button_width * 2.0 + 2.0),
                content_max.1 - button_height - 8.0,
                Key::RightArrow,
            );
            self.render_keyboard_button(
                info,
                "^",
                button_size,
                arrows_left_bound + button_width + 1.0,
                content_max.1 - (button_height * 2.0) - 9.0,
                Key::UpArrow,
            );
            self.render_keyboard_button(info, "R", button_size, content_min.0, content_min.1, Key::R);
            self.render_keyboard_button(
                info,
                "Shift",
                button_size,
                content_min.0,
                content_max.1 - button_height - 8.0,
                Key::LeftShift,
            );
            self.render_keyboard_button(info, "F2", button_size, content_max.0 - button_width, content_min.1, Key::F2);
            self.render_keyboard_button(
                info,
                "Z",
                button_size,
                content_max.0 - button_width,
                content_max.1 - button_height - 8.0,
                Key::Z,
            );
        }
    }

    /// Renders the mouse state menu into an imgui window
    fn render_mouse_window(&mut self, info: &mut EmulatorContext) {
        let frame = info.frame;

        frame.rect(
            Vec2(0.0, info.win_frame_height) + Vec2::from(frame.window_pos()),
            Vec2(frame.window_size()[0], info.win_frame_height + 20.0) + Vec2::from(frame.window_pos()),
            Colour::new(0.14, 0.14, 0.14),
            255,
        );

        let button_size = Vec2(40.0, 40.0);
        self.render_mouse_button(info, "Left", button_size, 4.0, 65.0, 0);
        self.render_mouse_button(info, "Middle", button_size, 48.0, 65.0, 2);
        self.render_mouse_button(info, "Right", button_size, 92.0, 65.0, 1);
        if frame.button_with_size_and_pos("Set Mouse", Vec2(150.0, 20.0), Vec2(150.0, 50.0)) || info.keybind_pressed(Binding::SetMouse) {
            if *info.game_running {
                *info.setting_mouse_pos = true;
            } else {
                *info.err_string = Some("The game is not running. Please load a savestate.".into());
            }
        }

        if let Some((x, y)) = info.new_mouse_pos {
            let color_stack = frame.push_style_color(StyleColor::Text, [1.0, 0.5, 0.5, 1.0]);
            frame.text_centered(&format!("x: {}*", x), Vec2(225.0, 80.0));
            frame.text_centered(&format!("y: {}*", y), Vec2(225.0, 96.0));
            color_stack.end();
        } else {
            frame.text_centered(&format!("x: {}", info.game.input.mouse_x()), Vec2(225.0, 80.0));
            frame.text_centered(&format!("y: {}", info.game.input.mouse_y()), Vec2(225.0, 96.0));
        }
    }

    /// Renders a single keyboard control button
    fn render_keyboard_button(
        &mut self,
        info: &mut EmulatorContext,
        name: &str,
        size: Vec2<f32>,
        x: f32,
        y: f32,
        code: ramen::input::Key,
    ) {
        let EmulatorContext {
            frame,
            keyboard_state,
            config,
            ..
        } = info;

        let vk = input::ramen2vk(code);
        let imgui_key = input::ramen2imgui(code).unwrap(); // We only display keys that actually have a mapping
        let state = &mut keyboard_state[usize::from(vk)];
        let clicked = frame.invisible_button_with_size_and_pos(name, size, Vec2(x, y));
        let hovered = frame.is_item_hovered();
        match config.input_mode {
            InputMode::Mouse => {
                if clicked {
                    state.click();
                }
                if frame.is_mouse_clicked(imgui::MouseButton::Right) && hovered {
                    frame.focus_current_window();
                    self.request_context_menu(ContextMenuType::Key(code));
                }
                if frame.is_mouse_clicked(imgui::MouseButton::Middle) && hovered {
                    frame.focus_current_window();
                    *state = if state.is_held() { KeyState::HeldWillDouble } else { KeyState::NeutralWillDouble };
                }
            },
            InputMode::Direct => {
                if frame.is_key_pressed(imgui_key) {
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
                } else if frame.is_key_released(imgui_key) {
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
        state.draw_keystate(*frame, Vec2(x, y), size);
        frame.text_centered(name, Vec2(x, y) + Vec2(size.0 / 2.0, size.1 / 2.0));
        if hovered {
            frame.set_cursor_pos([8.0, 22.0]);
            frame.text(state.repr());
        }
    }

    /// Renders a single mouse control button
    fn render_mouse_button(&mut self, info: &mut EmulatorContext, name: &str, size: Vec2<f32>, x: f32, y: f32, button: i8) {
        let EmulatorContext {
            frame,
            mouse_state,
            ..
        } = info;

        let state: &mut KeyState = &mut mouse_state[button as usize];
        if frame.invisible_button_with_size_and_pos(name, size, Vec2(x, y)) {
            state.click();
        }
        let hovered = frame.is_item_hovered();
        if frame.is_mouse_clicked(imgui::MouseButton::Right) && hovered {
            frame.focus_current_window();
            self.request_context_menu(ContextMenuType::Mouse(button));
        }
        if frame.is_mouse_clicked(imgui::MouseButton::Middle) && hovered {
            frame.focus_current_window();
            *state = if state.is_held() { KeyState::HeldWillDouble } else { KeyState::NeutralWillDouble };
        }
        state.draw_keystate(*frame, Vec2(x, y), size);
        frame.text_centered(name, Vec2(x, y) + Vec2(size.0 / 2.0, size.1 / 2.0));
        if hovered {
            frame.set_cursor_pos([8.0, 22.0]);
            frame.text(state.repr());
        }
    }

    /// Renders a single "dummy" button which does nothing, used only to fill space on the keyboard layout
    fn render_dummy_button(&mut self, frame: &imgui::Ui, name: &str, size: Vec2<f32>, x: f32, y: f32) {
        let pos = Vec2::from(frame.window_pos());
        frame.invisible_button_with_size_and_pos(name, size, Vec2(x, y));
        frame.rect(Vec2(x, y) + pos, Vec2(x, y) + size + pos, recording::BTN_NEUTRAL_COL, 190);
        frame.rect_outline(Vec2(x, y) + pos, Vec2(x, y) + size + pos, Colour::new(0.4, 0.4, 0.65), u8::MAX);
        frame.text_centered(name, Vec2(x, y) + Vec2(size.0 / 2.0, size.1 / 2.0));
    }
    
    fn display_context_menu(&mut self, info: &mut EmulatorContext) -> bool {
        match self.context_menu_type {
            Some(ContextMenuType::Key(key)) => {
                let key_state = &mut info.keyboard_state[usize::from(input::ramen2vk(key))];
                key_state.menu(info.frame)
            },
            Some(ContextMenuType::Mouse(button)) => {
                let key_state = &mut info.mouse_state[button as usize];
                key_state.menu(info.frame)
            },
            None => { false },
        }
    }
}
