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
            KeyState,
            window::{
                Window,
                Openable,
                DisplayInformation
            },
            keybinds::Binding,
        },
    },
};

use super::popup_dialog::{string_input::RNGSelect, Dialog, DialogState};

#[derive(PartialEq, Eq)]
enum MouseSelection {
    None,
    Left,
    //Middle,
    Right,
    Fixed,
}
#[derive(PartialEq, Eq, Copy, Clone)]
enum TableColor {
    DISABLED, CURRENT, SELECTED, DEFAULT, NONE
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

    setting_mouse_pos_for_frame: Option<usize>,
    setting_mouse_pos_end_frame: Option<usize>,
    single_frame_mouse: bool,

    is_selecting: MouseSelection,
    selection_column: Option<usize>,
    selection_start_index: usize,
    selection_end_index: usize,

    context_menu: bool,
    /// The indicies for the context menu. First value contains the frame, second value is the column that was clicked on, if it existed
    context_menu_indicies: (usize, Option<usize>),
    context_menu_keystate: KeyState,

    rng_select: RNGSelect,

    last_table_color: TableColor,
}

const INPUT_TABLE_WIDTH: f32 = 50.0;
const INPUT_TABLE_RNG_WIDTH: f32 = INPUT_TABLE_WIDTH * 1.5;
const INPUT_TABLE_MOUSE_WIDTH: f32 = INPUT_TABLE_WIDTH * 1.5;
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
const BGCOLOR_SELECTED: u32 = rgb!(244, 231, 90);
const BGCOLOR_SELECTED_ALT: u32 = rgb!(232, 223, 127);

impl Window for InputEditWindow {
    fn stored_kind(&self) -> Option<super::WindowKind> {
        Some(super::WindowKind::InputEditor)
    }

    fn show_window(&mut self, info: &mut DisplayInformation) {
        // todo: figure out a better system on when to update this.
        if self.last_frame != info.config.current_frame || !self.updated {
            self.updated = true;
            self.last_frame = info.config.current_frame;
            self.scroll_to_current_frame = true;
            self.update_keys(info.replay);
        }

        if self.setting_mouse_pos_for_frame.is_some() && !*info.setting_mouse_pos {
            let frame = self.setting_mouse_pos_for_frame.unwrap();
            if info.config.current_frame <= frame && frame < info.replay.frame_count() {
                if let Some(new_mouse_pos) = info.new_mouse_pos {
                    // If we have a new mouse position, set it to that
                    self.update_mouse_position_for_frame(frame, self.setting_mouse_pos_end_frame, new_mouse_pos.0, new_mouse_pos.1, info.replay);
                } else {
                    // Otherwise if we pressed Escape to unset the mouse, use the previous frames mouse position
                    let new_mouse_pos = if frame == 0 {
                        (0, 0)
                    } else {
                        let replay_frame = info.replay.get_frame(frame-1).unwrap();
                        (replay_frame.mouse_x, replay_frame.mouse_y)
                    };
                    self.update_mouse_position_for_frame(frame, self.setting_mouse_pos_end_frame, new_mouse_pos.0, new_mouse_pos.1, info.replay);
                }
            }
            self.setting_mouse_pos_for_frame = None;
            self.setting_mouse_pos_end_frame = None;
            *info.new_mouse_pos = None;
        }

        unsafe { cimgui_sys::igPushStyleVarVec2(cimgui_sys::ImGuiStyleVar__ImGuiStyleVar_WindowPadding as _, imgui::Vec2(0.0, 0.0).into()); }
        info.frame.begin_window(Self::window_name(), None, true, false, Some(&mut self.is_open));

        unsafe {
            cimgui_sys::igSetCursorPos(imgui::Vec2(0.0, INPUT_TABLE_YPOS).into());
            cimgui_sys::igPushStyleVarVec2(cimgui_sys::ImGuiStyleVar__ImGuiStyleVar_CellPadding as _, imgui::Vec2(TABLE_PADDING, 0.0).into());
        }
        self.push_current_row_colors();

        let table_size = info.frame.window_size() - imgui::Vec2(0.0, INPUT_TABLE_YPOS);

        if info.frame.begin_table(
            "Input",
            self.keys.len() as i32 + 3, // + Frame counter, RNG Seed and Mouse columns
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
            info.frame.table_setup_column("Mouse", (cimgui_sys::ImGuiTableColumnFlags__ImGuiTableColumnFlags_NoReorder |  cimgui_sys::ImGuiTableColumnFlags__ImGuiTableColumnFlags_WidthFixed) as _, INPUT_TABLE_MOUSE_WIDTH);
            info.frame.table_setup_scroll_freeze(0, 1); // freeze header row
            info.frame.table_headers_row();

            if self.scroll_to_current_frame {
                self.scroll_to_current_frame = false;
                info.frame.set_scroll_y(info.config.current_frame as f32 * TOTAL_INPUT_TABLE_HEIGHT - TOTAL_INPUT_TABLE_HEIGHT * 2.0);
            }

            self.scroll_y = info.frame.get_scroll_y();

            self.draw_input_rows(info);
            self.check_selection(info);

            info.frame.end_table();
        }
        unsafe {
            cimgui_sys::igPopStyleColor(2); // ImGuiCol_TableRowBg, ImGuiCol_TableRowBgAlt, pushed in self.push_current_row_colors()
            cimgui_sys::igPopStyleVar(1); // ImGuiStyleVar_CellPadding
        }

        let hovered_text = if self.is_selecting != MouseSelection::None {
            let count = self.selection_start_index.abs_diff(self.selection_end_index)+1;

            if let Some(text) = self.hovered_text {
                Some(format!("{} selected; {}", count, text))
            } else {
                Some(format!("{} selected", count))
            }
        } else {
            self.hovered_text.map(String::from)
        };

        if let Some(text) = hovered_text {
            unsafe {
                cimgui_sys::igSetCursorPos(imgui::Vec2(8.0, 22.0).into());
            }
            info.frame.text(text.as_str());
            self.hovered_text = None;
        }

        let text = "Single Frame Mouse Editing:";
        unsafe {
            let c_text = std::ffi::CString::new(text).expect("CString::new failed");
            let mut size = std::mem::MaybeUninit::uninit();
            cimgui_sys::igCalcTextSize(size.as_mut_ptr(), c_text.as_ptr(), std::ptr::null(), false, -1.0);
            let size = size.assume_init();
            
            // Technically igGetFrameHeightWithSpacing returns it with ItemSpacing.y instead of ItemInnerSpacing.x but
            //   those are the same at the moment and i can't be bothered to figure out how to get ItemInnerSpacing.x right now. :)
            let x = info.frame.window_size().0 - cimgui_sys::igGetFrameHeightWithSpacing() - size.x;
            cimgui_sys::igSetCursorPos(imgui::Vec2(x-8.0, 22.0).into());
        }
        info.frame.text(text);
        info.frame.same_line(0.0, -1.0);
        info.frame.checkbox("##mouse", &mut self.single_frame_mouse);

        unsafe { cimgui_sys::igPopStyleVar(1); } // ImGuiStyleVar__ImGuiStyleVar_WindowPadding
        info.frame.end();
    }

    fn is_open(&self) -> bool {
        self.is_open
    }

    fn name(&self) -> String {
        Self::window_name().to_owned()
    }

    fn show_context_menu(&mut self, info: &mut DisplayInformation) -> bool {
        self.display_context_menu(info)
    }

    fn context_menu_close(&mut self) {
        self.context_menu = false;
        self.is_selecting = MouseSelection::None;
    }

    fn handle_modal(&mut self, info: &mut DisplayInformation) -> bool {
        let mut any_open = false;
        match self.rng_select.show(info) {
            DialogState::Submit => {
                let new_seed = self.rng_select.get_result();
                let start = usize::min(self.selection_start_index, self.selection_end_index);
                let end = usize::max(self.selection_start_index, self.selection_end_index);

                assert!(start >= info.config.current_frame);
                assert!(end < info.replay.frame_count());

                for frame_index in start..(end+1) {
                    info.replay.get_frame_mut(frame_index).unwrap().new_seed = new_seed.clone();
                }
                self.is_selecting = MouseSelection::None; // Once the dialog is submitted, stop displaying the selection
            },
            DialogState::Open => {
                self.is_selecting = MouseSelection::Fixed; // Kind of a hack but the dialog can only be opened with the right-click context menu by selecting certain frames. Closing the context menu hides the selection so we show it again for as long as the dialog is open
                any_open = true;
            }
            _ => self.is_selecting = MouseSelection::None, // Once the dialog is closed, stop displaying the selection (Closed, Cancelled, Invalid, etc)
        };

        any_open
    }
}
impl Openable<Self> for InputEditWindow {
    fn window_name() -> &'static str {
        "Input Editor"
    }

    fn open(_id: usize) -> Self {
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

            setting_mouse_pos_for_frame: None,
            setting_mouse_pos_end_frame: None,
            single_frame_mouse: false,

            is_selecting: MouseSelection::None,
            selection_column: None,
            selection_start_index: 0,
            selection_end_index: 0,

            context_menu: false,
            context_menu_indicies: (0, None),
            context_menu_keystate: KeyState::Neutral,

            rng_select: RNGSelect::new("Pick RNG"),

            last_table_color: TableColor::NONE,
        }
    }

    fn update_mouse_position_for_frame(&mut self, frame: usize, end_frame: Option<usize>, x: i32, y: i32, replay: &mut Replay) {
        // If we want to set a new mouse position and aren't setting the mouse position anymore, update the frames accordingly
        if let Some(replay_frame) = replay.get_frame_mut(frame) {
            if self.single_frame_mouse && end_frame.is_none() {
                // Update just this frame
                replay_frame.mouse_x = x;
                replay_frame.mouse_y = y;
            } else {
                // Update this and all following frames that have had the same mouse position
                let old_mouse_x = replay_frame.mouse_x;
                let old_mouse_y = replay_frame.mouse_y;
                replay_frame.mouse_x = x;
                replay_frame.mouse_y = y;
                
                let mut current_frame = frame+1;
                while let Some(next_frame) = replay.get_frame_mut(current_frame) {
                    if end_frame.is_some() && current_frame <= end_frame.unwrap()
                        || end_frame.is_none() && next_frame.mouse_x == old_mouse_x && next_frame.mouse_y == old_mouse_y
                    {
                        next_frame.mouse_x = x;
                        next_frame.mouse_y = y;
                        current_frame += 1;
                    } else {
                        break;
                    }
                }
            }
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

    fn set_table_color(&mut self, color: TableColor) {
        if self.last_table_color != color {
            self.last_table_color = color;
            match color {
                TableColor::NONE => {},
                TableColor::DISABLED => self.set_table_colors(BGCOLOR_DISABLED, BGCOLOR_DISABLED_ALT),
                TableColor::SELECTED => self.set_table_colors(BGCOLOR_SELECTED, BGCOLOR_SELECTED_ALT),
                TableColor::CURRENT => self.set_table_colors(BGCOLOR_CURRENT, BGCOLOR_CURRENT_ALT),
                TableColor::DEFAULT => {
                    // remove whatever is currently on the stack
                    unsafe { cimgui_sys::igPopStyleColor(2); }
                    // and push the default colors
                    self.push_current_row_colors();
                },
            }
        }
    }

    fn display_context_menu(&mut self, info: &mut DisplayInformation) -> bool {
        let DisplayInformation {
            frame,
            replay,
            new_mouse_pos,
            setting_mouse_pos,
            ..
        } = info;

        if let Some(key_index) = self.context_menu_indicies.1 {
            // Context menu is to modify input, show the key context menu
            if self.selection_start_index == self.selection_end_index {
                if !self.context_menu_keystate.menu(frame) {
                    let frame_index = self.context_menu_indicies.0;

                    self.update_replay_keystate(frame_index, key_index, self.context_menu_keystate, replay);

                    self.is_selecting = MouseSelection::None;
                    self.context_menu = false;
                }
            } else {
                if let Some(state) = self.any_button_menu(frame) {
                    let start = usize::min(self.selection_start_index, self.selection_end_index);
                    let end = usize::max(self.selection_start_index, self.selection_end_index);

                    for frame_index in start..end {
                        self.update_replay_keystate(frame_index, key_index, state, replay);
                    }

                    self.is_selecting = MouseSelection::None;
                    self.context_menu = false;

                    // fix key states
                    self.update_keys(info.replay);
                }
            }
        } else {
            // Context menu is not editing input, show general options
            let start = usize::min(self.selection_start_index, self.selection_end_index);
            let end = usize::max(self.selection_start_index, self.selection_end_index);

            if frame.menu_item("Add 1 frame before") {
                self.add_frames(info.replay, start, 1);
                self.context_menu = false;
            } else if frame.menu_item("Add 10 frames before") {
                self.add_frames(info.replay, start, 10);
                self.context_menu = false;
            } else if frame.menu_item("Add 50 frames before") {
                self.add_frames(info.replay, start, 50);
                self.context_menu = false;
            } else if frame.menu_item("Add 1 frame after") {
                self.add_frames(info.replay, start, 1);
                self.context_menu = false;
            } else if frame.menu_item("Add 10 frames after") {
                self.add_frames(info.replay, start, 10);
                self.context_menu = false;
            } else if frame.menu_item("Add 50 frames after") {
                self.add_frames(info.replay, start, 50);
                self.context_menu = false;
            } else if frame.menu_item("Delete frame(s)") {
                self.delete_frames(replay, start, end);
                self.context_menu = false;
            } else if frame.menu_item("Set Mouse") {
                if let Some(current_frame) = replay.get_frame(start) {
                    **setting_mouse_pos = true;
                    **new_mouse_pos = Some((current_frame.mouse_x, current_frame.mouse_y));
                    self.setting_mouse_pos_for_frame = Some(start);
                    self.setting_mouse_pos_end_frame = Some(end);
                }
                self.context_menu = false;
            } else if frame.menu_item("Run until last selected frame") {
                *info.run_until_frame = Some(end);
                self.context_menu = false;
            } else if frame.menu_item("Pick RNG") {
                info.request_modal(&mut self.rng_select);
                self.context_menu = false;
            }
        }

        self.context_menu
    }

    fn any_button_menu(&self, frame: &mut imgui::Frame<'_>) -> Option<KeyState> {
        if frame.menu_item("Release") {
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
        }
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

    fn update_keys(&mut self, replay: &mut Replay) {
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
        let set_mouse_bind_pressed = info.keybind_pressed(Binding::SetMouse);
        
        let DisplayInformation {
            replay,
            frame,
            config,
            new_mouse_pos,
            setting_mouse_pos,
            ..
        } = info;

        let mouse_pos = frame.mouse_pos();

        let visible_height = frame.window_size().1 - INPUT_TABLE_YPOS;
        let float_count = replay.frame_count() as f32;

        let clipped_above = (f32::max(self.scroll_y - TABLE_CLIPPING, 0.0) / TOTAL_INPUT_TABLE_HEIGHT).floor();
        let clipped_below = (f32::min(self.scroll_y + visible_height + TABLE_CLIPPING, float_count * TOTAL_INPUT_TABLE_HEIGHT) / TOTAL_INPUT_TABLE_HEIGHT).floor();

        if clipped_above > 0.0 {
            // placeholder row for everything above the visible region. To make sure the size stays the same.
            frame.table_next_row(0, clipped_above * TOTAL_INPUT_TABLE_HEIGHT);
        }

        self.last_table_color = TableColor::NONE;
        let start_index = clipped_above as usize;
        if start_index < config.current_frame {
            self.set_table_color(TableColor::DISABLED);
        }
        for i in start_index..(clipped_below as usize) {
            frame.table_next_row(0, TOTAL_INPUT_TABLE_HEIGHT);

            if self.is_selecting != MouseSelection::None && self.selection_column.is_none() 
                && i >= usize::min(self.selection_start_index, self.selection_end_index)
                && i <= usize::max(self.selection_start_index, self.selection_end_index) {
                    self.set_table_color(TableColor::SELECTED);
            } else if i == config.current_frame {
                self.set_table_color(TableColor::CURRENT);
            } else if i > config.current_frame {
                self.set_table_color(TableColor::DEFAULT)
            }

            frame.table_set_column_index(0);
            frame.text(&format!("{}", i));

            let mut any_button_hovered = false;
            for j in 0..self.keys.len() {
                frame.table_set_column_index(j as i32 + 1);
                let keystate = &self.states[i][j];
                frame.invisible_button(keystate.repr(), imgui::Vec2(INPUT_TABLE_WIDTH, TOTAL_INPUT_TABLE_HEIGHT), None);

                let hovered = frame.item_hovered();
                let item_rect_min = frame.get_item_rect_min();
                let item_rect_size = frame.get_item_rect_size();

                let mut within_selection = false;
                // If we are selecting, check if the button falls withing the range of selected items.
                if self.is_selecting != MouseSelection::None && i >= config.current_frame && j == self.selection_column.unwrap_or(j) {
                    if self.is_selecting != MouseSelection::Fixed && !self.context_menu && (hovered || (mouse_pos.1 >= item_rect_min.1 && mouse_pos.1 <= item_rect_min.1 + item_rect_size.1)) {
                        // If we don't have a context menu open and the mouse is vertically on this button, mark it as selected
                        within_selection = true;
                        // And set this as the ending index
                        self.selection_end_index = i;
                    } else {
                        // Otherwise check if the index falls within the selected indicies
                        within_selection = i >= usize::min(self.selection_start_index, self.selection_end_index) && i <= usize::max(self.selection_start_index, self.selection_end_index);
                    }
                }

                keystate.draw_keystate(frame, item_rect_min-frame.window_position()+imgui::Vec2(0.0, TABLE_PADDING), item_rect_size - imgui::Vec2(0.0, TABLE_PADDING * 2.0));
                if within_selection {
                    frame.rect(item_rect_min + imgui::Vec2(0.0, TABLE_PADDING), item_rect_min + item_rect_size - imgui::Vec2(0.0, TABLE_PADDING * 2.0 - 1.0), crate::types::Colour::new(0.3, 0.4, 0.7), 128);
                }

                if hovered {
                    any_button_hovered = true;
                    self.hovered_text = Some(keystate.repr());
                    // if we clicked on an editable frame
                    if i >= config.current_frame {
                        if frame.left_clicked() {
                            self.is_selecting = MouseSelection::Left;
                            self.selection_start_index = i;
                            self.selection_end_index = i;
                            self.selection_column = Some(j);
                        } else if frame.right_clicked() {
                            self.is_selecting = MouseSelection::Right;
                            self.selection_start_index = i;
                            self.selection_end_index = i;
                            self.selection_column = Some(j);
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
            if hovered {
                any_button_hovered = true;
            }
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
            let current_frame = replay.get_frame(i).unwrap();
            let prev_frame = if i == 0 { None } else { replay.get_frame(i-1) };

            // Mouse Column
            frame.table_set_column_index(self.keys.len() as i32 + 2);
            let mouse_text = if self.single_frame_mouse || prev_frame.is_none() {
                format!("{}, {}", current_frame.mouse_x, current_frame.mouse_y)
            } else {
                // If we are in multi-frame mode for mouse edits then only show mouse coords when it actually changed
                if prev_frame.unwrap().mouse_x == current_frame.mouse_x && prev_frame.unwrap().mouse_y == current_frame.mouse_y {
                    String::from("-")
                } else {
                    format!("{}, {}", current_frame.mouse_x, current_frame.mouse_y)
                }
            };
            
            frame.button(&mouse_text, imgui::Vec2(INPUT_TABLE_MOUSE_WIDTH, INPUT_TABLE_HEIGHT), None);
            let mouse_hovered = frame.item_hovered();
            if mouse_hovered {
                any_button_hovered = true;
            }

            // If we clicked on a mouse input we haven't reached yet
            if i >= config.current_frame {
                if (frame.left_clicked() && mouse_hovered) // If we left clicked and are hovering this button
                    || (i == config.current_frame && set_mouse_bind_pressed) // or it's the next frame to be run and we used the keybind
                {
                    **setting_mouse_pos = true;
                    **new_mouse_pos = Some((current_frame.mouse_x, current_frame.mouse_y));
                    self.setting_mouse_pos_for_frame = Some(i);
                } else if frame.middle_clicked() && mouse_hovered {
                    self.update_mouse_position_for_frame(i, None, prev_frame.map(|f| f.mouse_x).unwrap_or(0), prev_frame.map(|f| f.mouse_y).unwrap_or(0), replay);
                }
            }

            // If we aren't hovering any of the key buttons, check if we are hovering the current row
            if frame.right_clicked() && !any_button_hovered && frame.window_hovered() && i >= config.current_frame {
                let row_pos = frame.get_item_rect_min(); // Get last item position to figure out whether or not we are hovering the current row
                if  mouse_pos.1 >= row_pos.1 && mouse_pos.1 <= row_pos.1 + INPUT_TABLE_HEIGHT {
                    self.is_selecting = MouseSelection::Right;
                    self.selection_start_index = i;
                    self.selection_end_index = i;
                    self.selection_column = None;
                }
            }
        }

        if float_count - clipped_below > 0.0 {
            // placeholder row for everything below the visible region. To make sure the size stays the same.
            frame.table_next_row(0, (float_count - clipped_below) * TOTAL_INPUT_TABLE_HEIGHT);
        }
    }

    fn check_selection(&mut self, info: &mut DisplayInformation) {
        match self.is_selecting {
            MouseSelection::Left => {
                if info.frame.left_released() {
                    if self.selection_start_index == self.selection_end_index {
                        let mut target_state = self.states[self.selection_start_index][self.selection_column.unwrap()].clone();
                        target_state.click();

                        self.update_replay_keystate(self.selection_start_index, self.selection_column.unwrap(), target_state, info.replay);
                    } else {
                        let key_index = self.selection_column.unwrap();
                        let start = usize::min(self.selection_start_index, self.selection_end_index);
                        let end = usize::max(self.selection_start_index, self.selection_end_index);
                        for frame_index in start..end {
                            let mut target_state = self.states[frame_index][key_index].clone();
                            target_state.click();
    
                            self.update_replay_keystate(frame_index, key_index, target_state, info.replay);
                        }
                    }
                    self.is_selecting = MouseSelection::None;
                }
            },
            MouseSelection::Right => {
                if info.frame.right_released() {
                    if info.request_context_menu() {
                        self.is_selecting = MouseSelection::Fixed;
                        self.context_menu = true;
                        self.context_menu_indicies = (self.selection_start_index, self.selection_column);
                        if let Some(index) = self.selection_column {
                            self.context_menu_keystate = self.states[self.selection_start_index][index].clone();
                        }
                    }
                }
            },
            MouseSelection::Fixed => {}, // Don't update the selection automatically if the selection was released (i.e. while context menu or popup are open)
            _ => {
                if !info.frame.mouse_down() {
                    self.is_selecting = MouseSelection::None;
                }
            },
        }
    }

    fn add_frames(&mut self, replay: &mut Replay, index: usize, count: usize) {
        if count != 0 {
            let mut i = count;
            while i != 0 {
                i -= 1;
                replay.insert_new_frame(index);
            }

            self.update_keys(replay)
        }
    }

    fn delete_frames(&mut self, replay: &mut Replay, start: usize, end: usize) {
        assert!(start <= end, "delete_frames: start must be less or equal to end");

        if end == replay.frame_count()-1 {
            replay.truncate_frames(start);
        } else {
            for _ in start..end+1 {
                replay.delete_frame(start);
                self.states.remove(start);
            }

            // update all key states for the first non-deleted frame to match the previous frame.
            for key in 0..self.keys.len() {
                let start_pressed = if start == 0 {
                    false
                } else {
                    self.states[start-1][key].ends_in_press()
                };

                let state = self.update_keystate_front(start, key, start_pressed).clone();
                self.update_replay(start, key, replay, state);
            }
        }

        self.update_keys(replay);
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

    fn update_keystate_front(&mut self, frame_index: usize, key_index: usize, start_pressed: bool) -> &KeyState{
        let state = &mut self.states[frame_index][key_index];

        if state.starts_with_press() != start_pressed {
            *state = match state {
                KeyState::NeutralWillPress => KeyState::Held,
                KeyState::Neutral | KeyState::NeutralWillDouble | KeyState::NeutralDoubleEveryFrame => KeyState::HeldWillRelease,
                KeyState::NeutralWillTriple => KeyState::HeldWillDouble,
                KeyState::NeutralWillCactus => KeyState::HeldWillRelease,
                
                KeyState::HeldWillRelease => KeyState::Neutral,
                KeyState::Held | KeyState::HeldWillDouble | KeyState::HeldDoubleEveryFrame => KeyState::NeutralWillPress,
                KeyState::HeldWillTriple => KeyState::NeutralWillDouble,
            }
        }

        state
    }
}
