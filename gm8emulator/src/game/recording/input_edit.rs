use crate::{
    imgui,
    input::Button,
    game::{
        replay::{
            Input,
            Replay,
            FrameRng,
        },
        recording::{
            KeyState, ContextMenu,
            window::{
                Window,
                Openable,
                DisplayInformation
            },
        },
    },
};

#[derive(PartialEq, Eq)]
enum MouseSelection {
    None,
    Left,
    Middle,
    Right,
}
pub struct InputEditWindow {
    is_open: bool,
    updated: bool,
    keys: Vec<u8>,
    states: Vec<Vec<KeyState>>,
    last_frame: usize,
    scroll_y: f32,
    hovered_text: Option<&'static str>,
    scroll_to_current_frame: bool,

    is_selecting: MouseSelection,
    selection_start_indicies: (usize, usize),
    selection_end_indicies: (usize, usize),

    context_menu: bool,
    context_menu_pos: imgui::Vec2<f32>,
    context_menu_indicies: (usize, usize),
    context_menu_keystate: KeyState,
}

const INPUT_TABLE_WIDTH: f32 = 50.0;
const INPUT_TABLE_RNG_WIDTH: f32 = INPUT_TABLE_WIDTH * 2.0;
const INPUT_TABLE_HEIGHT: f32 = 20.0;
const INPUT_TABLE_YPOS: f32 = 44.0;
const TABLE_PADDING: f32 = 2.0;
const TOTAL_INPUT_TABLE_HEIGHT: f32 = INPUT_TABLE_HEIGHT + TABLE_PADDING * 2.0; // total height = table height + top padding + bottom padding
// draw 2 elements above and below the visible region.
const TABLE_CLIPPING: f32 = TOTAL_INPUT_TABLE_HEIGHT*2.0;


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
        if info.context_menu.is_none() {
            if self.context_menu {
                self.is_selecting = MouseSelection::None;
                self.context_menu = false;
            }
        }

        // todo: figure out a better system on when to update this.
        if self.last_frame != info.config.current_frame || !self.updated{
            self.updated = true;
            self.last_frame = info.config.current_frame;
            self.scroll_to_current_frame = true;
            self.update_keys(info);
        }

        unsafe { cimgui_sys::igPushStyleVarVec2(cimgui_sys::ImGuiStyleVar__ImGuiStyleVar_WindowPadding as _, imgui::Vec2(0.0, 0.0).into()); }
        info.frame.begin_window(Self::window_name(), None, true, false, Some(&mut self.is_open));

        unsafe {
            cimgui_sys::igSetCursorPos(imgui::Vec2(0.0, INPUT_TABLE_YPOS).into());
            cimgui_sys::igPushStyleVarVec2(cimgui_sys::ImGuiStyleVar__ImGuiStyleVar_CellPadding as _, imgui::Vec2(TABLE_PADDING, TABLE_PADDING).into());
        }
        self.push_current_row_colors();

        let table_size = info.frame.window_size() - imgui::Vec2(0.0, INPUT_TABLE_YPOS);

        if info.frame.begin_table(
            "Input",
            self.keys.len() as i32 + 2, // + Frame counter and RNG Seed columns
            (cimgui_sys::ImGuiTableFlags__ImGuiTableFlags_RowBg
                | cimgui_sys::ImGuiTableFlags__ImGuiTableFlags_Reorderable
                | cimgui_sys::ImGuiTableFlags__ImGuiTableFlags_Borders
                | cimgui_sys::ImGuiTableFlags__ImGuiTableFlags_NoPadOuterX
                | cimgui_sys::ImGuiTableFlags__ImGuiTableFlags_NoPadInnerX
                | cimgui_sys::ImGuiTableFlags__ImGuiTableFlags_ScrollY) as _,
                table_size,
                0.0
        ) {
            info.frame.table_setup_column("Frame", cimgui_sys::ImGuiTableColumnFlags__ImGuiTableColumnFlags_NoReorder as _, 0.0);
            for key in self.keys.iter() {
                if let Some(button) = Button::try_from_u8(*key) {
                    info.frame.table_setup_column(&format!("{}", button), cimgui_sys::ImGuiTableColumnFlags__ImGuiTableColumnFlags_WidthFixed as _, INPUT_TABLE_WIDTH);
                }
            }
            info.frame.table_setup_column("RNG", (cimgui_sys::ImGuiTableColumnFlags__ImGuiTableColumnFlags_NoReorder |  cimgui_sys::ImGuiTableColumnFlags__ImGuiTableColumnFlags_WidthFixed) as _, INPUT_TABLE_RNG_WIDTH);
            info.frame.table_setup_scroll_freeze(0, 1); // freeze header row
            info.frame.table_headers_row();

            if self.scroll_to_current_frame {
                self.scroll_to_current_frame = false;
                info.frame.set_scroll_y(info.config.current_frame as f32 * TOTAL_INPUT_TABLE_HEIGHT - TOTAL_INPUT_TABLE_HEIGHT * 2.0);
            }

            self.scroll_y = info.frame.get_scroll_y();

            self.draw_input_rows(info);
            self.check_selection(info.frame, info.context_menu, info.replay);

            info.frame.end_table();
        }
        unsafe {
            cimgui_sys::igPopStyleColor(2); // ImGuiCol_TableRowBg, ImGuiCol_TableRowBgAlt, pushed in self.push_current_row_colors()
            cimgui_sys::igPopStyleVar(1); // ImGuiStyleVar_CellPadding
        }

        if let Some(text) = self.hovered_text {
            unsafe {
                cimgui_sys::igSetCursorPos(imgui::Vec2(8.0, 22.0).into());
            }
            info.frame.text(text);
            self.hovered_text = None;
        }

        unsafe { cimgui_sys::igPopStyleVar(1); }
        info.frame.end();

        if self.context_menu {
            self.show_context_menu(info);
        }
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
            updated: false,
            keys: Vec::new(),
            states: Vec::new(),
            last_frame: 0,
            scroll_y: 0.0,
            hovered_text: None,
            scroll_to_current_frame: false,

            is_selecting: MouseSelection::None,
            selection_start_indicies: (0, 0),
            selection_end_indicies: (0, 0),

            context_menu: false,
            context_menu_pos: imgui::Vec2(0.0, 0.0),
            context_menu_indicies: (0, 0),
            context_menu_keystate: KeyState::Neutral,
        }
    }

    /// pushes the current row color and alternative row color on the color style stack.
    /// Used to be able to set the color directly to something else. Colors are applied on either the next next_row() call or the next end_table() call
    ///   depending on where the visible region ends. The active color needs to live long enough for either of them.
    ///
    /// (I'll be upset if there's no better way to do this, but I can't think of one right now so here we are.)
    fn push_current_row_colors(&self) {
        unsafe {
            cimgui_sys::igPushStyleColorU32(cimgui_sys::ImGuiCol__ImGuiCol_TableRowBg as _, cimgui_sys::igGetColorU32Col(cimgui_sys::ImGuiCol__ImGuiCol_TableRowBg as _, 1.0));
            cimgui_sys::igPushStyleColorU32(cimgui_sys::ImGuiCol__ImGuiCol_TableRowBgAlt as _, cimgui_sys::igGetColorU32Col(cimgui_sys::ImGuiCol__ImGuiCol_TableRowBgAlt as _, 1.0));
        }
    }
    /// Sets the row color and alternative row color. Expects 2 items to be on the current color style stack.
    /// This needs to be done so that the current color can live long enough.
    fn set_table_colors(&self, color: u32, color_alt: u32) {
        unsafe {
            cimgui_sys::igPopStyleColor(2);
            cimgui_sys::igPushStyleColorU32(cimgui_sys::ImGuiCol__ImGuiCol_TableRowBg as _, color);
            cimgui_sys::igPushStyleColorU32(cimgui_sys::ImGuiCol__ImGuiCol_TableRowBgAlt as _, color_alt);
        }
    }

    fn show_context_menu(&mut self, info: &mut DisplayInformation) {
        let DisplayInformation {
            frame,
            replay,
            context_menu,
            ..
        } = info;

        frame.begin_context_menu(self.context_menu_pos);

        if self.selection_start_indicies == self.selection_end_indicies {
            if !self.context_menu_keystate.menu(frame, self.context_menu_pos) {
                let frame_index = self.context_menu_indicies.0;
                let key_index = self.context_menu_indicies.1;
    
                self.update_replay_keystate(frame_index, key_index, self.context_menu_keystate, replay);
    
                self.is_selecting = MouseSelection::None;
                self.context_menu = false;
                **context_menu = None;
            }
        } else {
            if let Some(state) = self.any_button_menu(frame) {
                let start = usize::min(self.selection_start_indicies.0, self.selection_end_indicies.0);
                let end = usize::max(self.selection_start_indicies.0, self.selection_end_indicies.0);
                let key_index = self.selection_start_indicies.1;

                for frame_index in start..end {
                    self.update_replay_keystate(frame_index, key_index, state, replay);
                }

                self.is_selecting = MouseSelection::None;
                self.context_menu = false;
                **context_menu = None;

                // fix key states
                self.update_keys(info);
            }
        }

        info.frame.end();
    }

    fn any_button_menu(&self, frame: &mut imgui::Frame<'_>) -> Option<KeyState> {
        frame.begin_context_menu(self.context_menu_pos);
        let result = if frame.menu_item("Release") {
            Some(KeyState::HeldWillRelease)
        } else if frame.menu_item("Release, Press") {
            Some(KeyState::HeldWillDouble)
        } else if frame.menu_item("Release, Press, Release") {
            Some(KeyState::HeldWillTriple)
        } else if frame.menu_item("Tap Every Frame") {
            Some(KeyState::HeldDoubleEveryFrame)
        } else if frame.menu_item("Press") {
            Some(KeyState::NeutralWillPress)
        } else if frame.menu_item("Press, Release") {
            Some(KeyState::NeutralWillDouble)
        } else if frame.menu_item("Press, Release, Press") {
            Some(KeyState::NeutralWillTriple)
        } else if frame.menu_item("Tap Every Frame") {
            Some(KeyState::NeutralDoubleEveryFrame)
        } else if frame.menu_item("Cactus-Release") {
            Some(KeyState::NeutralWillCactus)
        } else {
            None
        };
        frame.end();

        result
    }

    fn update_replay_keystate(&mut self, frame_index: usize, key_index: usize, new_keystate: KeyState, replay: &mut Replay) {
        let old_keystate = &self.states[frame_index][key_index];
        let old_press = old_keystate.ends_in_press();
        let new_press = new_keystate.ends_in_press();

        if old_press != new_press {
            if let Some(next_keystates) = self.states.get_mut(frame_index + 1) {
                if let Some(new_state) = match next_keystates[key_index] {
                    KeyState::Held
                        | KeyState::HeldWillDouble
                        | KeyState::HeldDoubleEveryFrame
                        => if new_press { None } else { Some(KeyState::NeutralWillPress) },

                    KeyState::HeldWillRelease
                        => if new_press { None } else { Some(KeyState::Neutral) },

                    KeyState::HeldWillTriple
                        => if new_press { None } else { Some(KeyState::NeutralWillDouble) },

                    KeyState::Neutral
                        | KeyState::NeutralWillDouble
                        | KeyState::NeutralDoubleEveryFrame
                        | KeyState::NeutralWillCactus
                        => if new_press { Some(KeyState::HeldWillRelease) } else { None },

                    KeyState::NeutralWillPress
                        => if new_press { Some(KeyState::Held) } else { None }

                    KeyState::NeutralWillTriple
                        => if new_press { Some(KeyState::HeldWillDouble) } else { None },
                } {
                    // If the next frame will have to be adjusted, do that.
                    self.update_replay(frame_index + 1, key_index, replay, new_state);
                }
            }
        }
        
        self.update_replay(frame_index, key_index, replay, new_keystate);
    }

    fn update_keys(&mut self, info: &mut DisplayInformation) {
        let DisplayInformation {
            replay,
            ..
        } = info;

        // clear selection if keys need to be updated.
        self.is_selecting = MouseSelection::None;

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
        self.keys.sort_unstable();

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

        let clipped_above = (f32::max(self.scroll_y - TABLE_CLIPPING, 0.0) / TOTAL_INPUT_TABLE_HEIGHT).floor();
        let clipped_below = (f32::min(self.scroll_y + visible_height + TABLE_CLIPPING, float_count * TOTAL_INPUT_TABLE_HEIGHT) / TOTAL_INPUT_TABLE_HEIGHT).floor();

        if clipped_above > 0.0 {
            // placeholder row for everything above the visible region. To make sure the size stays the same.
            frame.table_next_row(0, clipped_above * TOTAL_INPUT_TABLE_HEIGHT);
        }

        let start_index = clipped_above as usize;
        if start_index < config.current_frame {
            self.set_table_colors(BGCOLOR_DISABLED, BGCOLOR_DISABLED_ALT);
        }
        for i in start_index..(clipped_below as usize) {
            frame.table_next_row(0, INPUT_TABLE_HEIGHT);
            if i == config.current_frame {
                self.set_table_colors(BGCOLOR_CURRENT, BGCOLOR_CURRENT_ALT);
            } else if i == config.current_frame + 1 {
                // remove whatever is currently on the stack
                unsafe { cimgui_sys::igPopStyleColor(2); }
                // and push the default colors
                self.push_current_row_colors();
            }

            frame.table_set_column_index(0);
            frame.text(&format!("{}", i));

            for j in 0..self.keys.len() {
                frame.table_set_column_index(j as i32 + 1);
                let keystate = &self.states[i][j];
                frame.invisible_button(keystate.repr(), imgui::Vec2(INPUT_TABLE_WIDTH, INPUT_TABLE_HEIGHT), None);

                let hovered = frame.item_hovered();
                let item_rect_min = frame.get_item_rect_min();
                let item_rect_size = frame.get_item_rect_size();

                let mut within_selection = false;
                if self.is_selecting != MouseSelection::None && i >= config.current_frame && j == self.selection_start_indicies.1 {
                    let mouse_pos = frame.mouse_pos();
                    if !self.context_menu && (hovered || (mouse_pos.1 >= item_rect_min.1 && mouse_pos.1 <= item_rect_min.1 + item_rect_size.1)) {
                        within_selection = true;
                        self.selection_end_indicies = (i, j);
                    } else {
                        within_selection = i >= usize::min(self.selection_start_indicies.0, self.selection_end_indicies.0) && i <= usize::max(self.selection_start_indicies.0, self.selection_end_indicies.0);
                    }
                }

                keystate.draw_keystate(frame, item_rect_min-frame.window_position(), item_rect_size);
                if within_selection {
                    frame.rect(item_rect_min, item_rect_min + item_rect_size, crate::types::Colour::new(0.3, 0.4, 0.7), 128);
                }

                if hovered {
                    self.hovered_text = Some(keystate.repr());
                    // if we clicked on an editable frame
                    if i >= config.current_frame {
                        if frame.left_clicked() {
                            self.is_selecting = MouseSelection::Left;
                            self.selection_start_indicies = (i, j);
                            self.selection_end_indicies = (i, j);
                        } else if frame.right_clicked() {
                            self.is_selecting = MouseSelection::Right;
                            self.selection_start_indicies = (i, j);
                            self.selection_end_indicies = (i, j);
                        }
                    }
                }
            }

            // RNG Changer
            let current_frame = replay.get_frame_mut(i).unwrap();
            frame.table_set_column_index(self.keys.len() as i32 + 1);
            let text = match current_frame.new_seed {
                None => String::from("-"),
                Some(FrameRng::Override(new_seed)) => format!("{}", new_seed),
                Some(FrameRng::Increment(count)) => format!("+{}", count),
            };
            
            frame.button(&text, imgui::Vec2(INPUT_TABLE_RNG_WIDTH, INPUT_TABLE_HEIGHT), None);
            let hovered = frame.item_hovered();
            // If this is a rng change we haven't reached yet and is hovered
            if i >= config.current_frame && hovered {
                if frame.left_clicked() {
                    current_frame.new_seed = Some(FrameRng::Increment(
                        match current_frame.new_seed {
                            None | Some(FrameRng::Override(_)) => 1,
                            Some(FrameRng::Increment(count)) => count + 1,
                        }
                    ));
                } else if frame.right_clicked() {
                    current_frame.new_seed = match current_frame.new_seed {
                        None => None,
                        Some(FrameRng::Override(new_seed)) => Some(FrameRng::Override(new_seed)),
                        Some(FrameRng::Increment(count)) => if count == 1 { None } else { Some(FrameRng::Increment(count - 1)) },
                    };
                } else if frame.middle_clicked() {
                    current_frame.new_seed = None;
                }
            }
        }

        if float_count - clipped_below > 0.0 {
            // placeholder row for everything below the visible region. To make sure the size stays the same.
            frame.table_next_row(0, (float_count - clipped_below) * TOTAL_INPUT_TABLE_HEIGHT);
        }
    }

    fn check_selection(&mut self, frame: &mut imgui::Frame<'_>, context_menu: &mut Option<ContextMenu>, replay: &mut Replay) {
        match self.is_selecting {
            MouseSelection::Left => {
                if frame.left_released() {
                    if self.selection_start_indicies == self.selection_end_indicies {
                        let (i, j) = self.selection_start_indicies;
                        let mut target_state = self.states[i][j].clone();
                        target_state.click();

                        self.update_replay_keystate(i, j, target_state, replay);
                    } else {
                        let key_index = self.selection_start_indicies.1;
                        let start = usize::min(self.selection_start_indicies.0, self.selection_end_indicies.0);
                        let end = usize::max(self.selection_start_indicies.0, self.selection_end_indicies.0);
                        for frame_index in start..end {
                            let mut target_state = self.states[frame_index][key_index].clone();
                            target_state.click();
    
                            self.update_replay_keystate(frame_index, key_index, target_state, replay);
                        }
                    }
                    self.is_selecting = MouseSelection::None;
                }
            },
            MouseSelection::Right => {
                if frame.right_released() {
                    self.context_menu = true;
                    self.context_menu_pos = frame.mouse_pos();
                    self.context_menu_indicies = self.selection_start_indicies;
                    self.context_menu_keystate = self.states[self.selection_start_indicies.0][self.selection_start_indicies.1].clone();
                    *context_menu = Some(ContextMenu::Any);
                }
            },
            _ => {
                if !frame.mouse_down() {
                    self.is_selecting = MouseSelection::None;
                }
            },
        }

        
    }

    fn update_replay(&mut self, frame_index: usize, key_index: usize, replay: &mut Replay, target_state: KeyState) {
        if let Some(replay_frame) = replay.get_frame_mut(frame_index) {
            let mut new_inputs: Vec<Input> = replay_frame.inputs.iter().filter(|input|
                match input {
                    Input::KeyPress(key) | Input::KeyRelease(key) => *key != self.keys[key_index],
                    _ => true,
                }
            ).cloned().collect();

            target_state.push_key_inputs(self.keys[key_index], &mut new_inputs);
            replay_frame.inputs = new_inputs;

            self.states[frame_index][key_index] = target_state;
        }
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
