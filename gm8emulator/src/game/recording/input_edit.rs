use crate::{
    imgui,
    input::Button,
    game::{
        replay::Input,
        recording::{
            KeyState,
            window::{
                Window,
                Openable,
                DisplayInformation
            },
        },
    },
};
use std::convert::TryInto;

pub struct InputEditWindow {
    is_open: bool,
    keys: Vec<u8>,
    states: Vec<Vec<KeyState>>,
    last_frame: usize,
    scroll_y: f32,
    hovered_text: Option<&'static str>,
    scroll_to_current_frame: bool
}

const INPUT_TABLE_WIDTH: f32 = 50.0;
const INPUT_TABLE_HEIGHT: f32 = 20.0;
const INPUT_TABLE_YPOS: f32 = 44.0;
const TABLE_CLIPPING: f32 = INPUT_TABLE_HEIGHT*5.0;

macro_rules! rgb {
    ($r:expr, $g:expr, $b:expr) => {
        // create rgb value with 0.5 alpha
        0x80000000|($b&0xFF)<<16|($g&0xFF)<<8|($r&0xFF)
    };
}
const BGCOLOR_CURRENT: u32 = rgb!(145, 210, 145);
const BGCOLOR_CURRENT_ALT: u32 = rgb!(120, 165, 121);
const BGCOLOR_DISABLED: u32 = rgb!(210, 145, 145);
const BGCOLOR_DISABLED_ALT: u32 = rgb!(165, 120, 120);

impl Window for InputEditWindow {
    fn show_window(&mut self, info: &mut DisplayInformation) {
        // todo: figure out a better system on when to update this.
        if self.last_frame != info.config.current_frame {
            self.last_frame = info.config.current_frame;
            self.scroll_to_current_frame = true;
            self.update_keys(info);
        }
        unsafe { cimgui_sys::igPushStyleVarVec2(cimgui_sys::ImGuiStyleVar__ImGuiStyleVar_WindowPadding.try_into().unwrap(), imgui::Vec2(0.0, 0.0).into()); }
        info.frame.begin_window(Self::window_name(), None, true, false, Some(&mut self.is_open));

        unsafe { cimgui_sys::igSetCursorPos(cimgui_sys::ImVec2 { x: 0.0, y: INPUT_TABLE_YPOS }); }
        let table_size = info.frame.window_size() - imgui::Vec2(0.0, INPUT_TABLE_YPOS + 50.0);
        if info.frame.begin_table(
            "Input",
            self.keys.len() as i32 + 1,
            (cimgui_sys::ImGuiTableFlags__ImGuiTableFlags_RowBg
                | cimgui_sys::ImGuiTableFlags__ImGuiTableFlags_Borders
                | cimgui_sys::ImGuiTableFlags__ImGuiTableFlags_NoPadOuterX
                | cimgui_sys::ImGuiTableFlags__ImGuiTableFlags_NoPadInnerX
                | cimgui_sys::ImGuiTableFlags__ImGuiTableFlags_ScrollY) as i32,
            table_size,
            0.0
        ) {
            info.frame.table_setup_column("Frame", 0, 0.0);
            for key in self.keys.iter() {
                if let Some(button) = Button::try_from_u8(*key) {
                    info.frame.table_setup_column(&format!("{}", button), cimgui_sys::ImGuiTableColumnFlags__ImGuiTableColumnFlags_WidthFixed as i32, INPUT_TABLE_WIDTH);
                }
            }
            info.frame.table_setup_scroll_freeze(0, 1); // freeze header row
            info.frame.table_headers_row();

            if self.scroll_to_current_frame {
                self.scroll_to_current_frame = false;
                info.frame.set_scroll_y(info.config.current_frame as f32 * INPUT_TABLE_HEIGHT - INPUT_TABLE_HEIGHT * 2.0);
            }

            self.draw_input_rows(info);

            self.scroll_y = info.frame.get_scroll_y();
            let scroll_max_y = info.frame.get_scroll_max_y();

            info.frame.end_table();

            info.frame.text(&format!("===  Y Scroll: {}/{}", self.scroll_y, scroll_max_y));
        }

        if let Some(text) = self.hovered_text {
            unsafe {
                cimgui_sys::igSetCursorPos(cimgui_sys::ImVec2 { x: 8.0, y: 22.0 });
            }
            info.frame.text(text);
            self.hovered_text = None;
        }

        unsafe { cimgui_sys::igPopStyleVar(1); }
        info.frame.end();
    }

    fn is_open(&self) -> bool {
        self.is_open
    }

    fn name(&self) -> String {
        Self::window_name().to_owned()
    }
}
impl Openable<Self> for InputEditWindow {
    fn window_name() -> &'static str {
        "Input Editor"
    }

    fn open() -> Self {
        Self::new()
    }
}

impl InputEditWindow {
    fn new() -> Self {
        Self {
            is_open: true,
            keys: Vec::new(),
            states: Vec::new(),
            last_frame: 0,
            scroll_y: 0.0,
            hovered_text: None,
            scroll_to_current_frame: false,
        }
    }

    fn update_keys(&mut self, info: &mut DisplayInformation) {
        let DisplayInformation {
            replay,
            ..
        } = info;

        self.keys.clear();
        for i in 0..replay.frame_count() {
            if let Some(frame) = replay.get_frame(i) {
                for input in &frame.inputs {
                    match input {
                        Input::KeyPress(btn) | Input::KeyRelease(btn) => {
                            if !self.keys.contains(btn) {
                                self.keys.push(*btn);
                            }
                        },
                        _ => {},
                    }
                }
            }
        }
        self.states.clear();
        self.states.reserve(replay.frame_count());
        self.states.push(vec![KeyState::Neutral; self.keys.len()]);
        
        for i in 0..replay.frame_count() {
            if let Some(frame) = replay.get_frame(i) {
                for input in &frame.inputs {
                    match input {
                        Input::KeyPress(current_key) | Input::KeyRelease(current_key)  => {
                            if let Some(index) = self.keys.iter().position(|k| k == current_key) {
                                self.update_keystate(i, index, matches!(input, Input::KeyPress(_)));
                            }
                        },
                        _ => {},
                    }
                }

                self.states.push(self.states[i].clone());
                for state in self.states[i + 1].iter_mut() {
                    *state = match state {
                        KeyState::NeutralWillPress
                            | KeyState::NeutralWillTriple
                            | KeyState::HeldWillDouble
                            | KeyState::HeldDoubleEveryFrame
                            | KeyState::Held
                            => KeyState::Held,
                        KeyState::NeutralWillDouble
                            | KeyState::NeutralDoubleEveryFrame
                            | KeyState::HeldWillRelease
                            | KeyState::NeutralWillCactus
                            | KeyState::HeldWillTriple
                            | KeyState::Neutral
                            => KeyState::Neutral
                    }
                }
            }
        }
    }

    fn draw_input_rows(&mut self, info: &mut DisplayInformation) {
        let DisplayInformation {
            replay,
            frame,
            config,
            ..
        } = info;

        let visible_height = frame.window_size().1 - INPUT_TABLE_YPOS;
        let float_count = replay.frame_count() as f32;

        let clipped_above = (f32::max(self.scroll_y - TABLE_CLIPPING, 0.0) / INPUT_TABLE_HEIGHT).floor();
        let clipped_below = (f32::min(self.scroll_y + visible_height + TABLE_CLIPPING, float_count * INPUT_TABLE_HEIGHT) / INPUT_TABLE_HEIGHT).floor();

        frame.table_next_row(0, clipped_above * INPUT_TABLE_HEIGHT);

        for i in (clipped_above as usize)..(clipped_below as usize) {
            if i < config.current_frame {
                unsafe {
                    cimgui_sys::igPushStyleColorU32(cimgui_sys::ImGuiCol__ImGuiCol_TableRowBg as _, BGCOLOR_DISABLED);
                    cimgui_sys::igPushStyleColorU32(cimgui_sys::ImGuiCol__ImGuiCol_TableRowBgAlt as _, BGCOLOR_DISABLED_ALT);
                }
            } else if i == config.current_frame {
                unsafe {
                    cimgui_sys::igPushStyleColorU32(cimgui_sys::ImGuiCol__ImGuiCol_TableRowBg as _, BGCOLOR_CURRENT);
                    cimgui_sys::igPushStyleColorU32(cimgui_sys::ImGuiCol__ImGuiCol_TableRowBgAlt as _, BGCOLOR_CURRENT_ALT);
                }
            }
            frame.table_next_row(0, INPUT_TABLE_HEIGHT);

            frame.table_set_column_index(0);
            frame.text(&format!("{}", i + 1));

            for j in 0..self.keys.len() {
                frame.table_set_column_index(j as i32 + 1);
                let keystate = &self.states[i][j];
                frame.invisible_button(keystate.repr(), imgui::Vec2(INPUT_TABLE_WIDTH, INPUT_TABLE_HEIGHT), None);

                let hovered = frame.item_hovered();
                keystate.draw_keystate(frame, frame.get_item_rect_min()-frame.window_position(), frame.get_item_rect_size());
                if hovered {
                    self.hovered_text = Some(keystate.repr());
                }
            }

            if i <= config.current_frame {
                unsafe { cimgui_sys::igPopStyleColor(2);}
            }
        }

        frame.table_next_row(0, (float_count - clipped_above) * INPUT_TABLE_HEIGHT);
    }

    fn update_keystate(&mut self, frame_index: usize, key_index: usize, pressed: bool) {
        let state = &mut self.states[frame_index][key_index];

        macro_rules! invalid {
            () => {{
                    println!("Warning: Invalid input order {}: {}", state.repr(), pressed);

                KeyState::Neutral
            }};
        }
        
        *state = match state {
                    KeyState::Held => if pressed { /* this one is not currently possible to enter in tas mode */ KeyState::Held } else { KeyState::HeldWillRelease },
                    KeyState::HeldWillRelease => if pressed { KeyState::HeldWillDouble } else { invalid!() },
                    KeyState::HeldWillDouble => if pressed { invalid!() } else { KeyState::HeldWillTriple },
                    KeyState::HeldWillTriple => invalid!(),

                    KeyState::Neutral => if pressed { KeyState::NeutralWillPress } else { KeyState::NeutralWillCactus },
                    KeyState::NeutralWillPress => if pressed { invalid!() } else { KeyState::NeutralWillDouble },
                    KeyState::NeutralWillDouble => if pressed { KeyState::NeutralWillTriple } else { invalid!() },
                    KeyState::NeutralWillTriple => invalid!(),

                    _ => if pressed { KeyState::NeutralWillPress } else { KeyState::HeldWillRelease },
                };
    }
}
