use imgui::ListBox;

use crate::{
    game::recording::window::{EmulatorContext, Openable, Window},
    gml::{Context, Value},
    imgui_utils::UiCustomFunction,
};

pub struct ConsoleWindow {
    input_string: String,
    gml_context: Context,
    output: Vec<String>,

    scroll_to_bottom: bool,
    run_code: bool,
    last_frame: usize,
    last_rerecords: u64,

    is_open: bool,
    id: usize,
}

impl Openable<Self> for ConsoleWindow {
    fn window_name() -> &'static str {
        "Console"
    }

    fn open(id: usize) -> Self {
        let mut new_console = Self::new();
        new_console.id = id;

        new_console
    }
}

impl Window for ConsoleWindow {
    fn stored_kind(&self) -> Option<super::WindowKind> {
        Some(super::WindowKind::Console(self.id))
    }

    fn window_id(&self) -> usize {
        self.id
    }

    fn name(&self) -> String {
        format!("Console {}", self.id + 1)
    }

    fn show_window(&mut self, info: &mut EmulatorContext) {
        let EmulatorContext {
            config,
            frame,
            game,
            keybindings,
            clean_state,
            ..
        } = info;

        frame
            .window(self.name())
            .opened(&mut self.is_open)
            .position([100.0, 100.0], imgui::Condition::FirstUseEver)
            .size([600.0, 250.0], imgui::Condition::FirstUseEver)
            .build(|| {
                let window_size = frame.window_size();
                let content_position = frame.window_content_region_min();
                ListBox::new("GMLConsoleOutput")
                    .size([window_size[0] - content_position[0] * 2.0, window_size[1] - 60.0])
                    .build(frame, || {
                        for text in &self.output {
                            frame.text(&text);
                        }
                        if self.scroll_to_bottom {
                            self.scroll_to_bottom = false;
                            frame.set_scroll_here_y_with_ratio(1.0);
                        }
                    });

                let item_spacing_x = frame.item_spacing().0;

                // ImGui uses frame_height to determine both width and height of the checkbox.
                //  Without any label there is no additional horizontal padding to consider.
                let checkbox_width = frame.frame_height() + item_spacing_x;
                let run_button_width = Self::RUN_BUTTON_WIDTH + item_spacing_x;

                // width = window width - padding - (width of checkbox + item x spacing) - (width of run button + item x spacing)
                let width = window_size[0]
                    - content_position[0] * 2.0
                    - checkbox_width
                    - run_button_width
                    - Self::TEXTBOX_MIN_WIDTH;
                frame.set_next_item_width(if width >= 0.0 {
                    // checkbox and run button are visible
                    width + Self::TEXTBOX_MIN_WIDTH
                } else if width >= -run_button_width {
                    // only checkbox is visible
                    width + run_button_width + Self::TEXTBOX_MIN_WIDTH
                } else if width >= -run_button_width - checkbox_width {
                    // nothing is visible
                    width + checkbox_width + run_button_width + Self::TEXTBOX_MIN_WIDTH
                } else {
                    Self::TEXTBOX_MIN_WIDTH // Shouldn't really happen since the minimum content width is bigger than the current minimum textbox width
                });

                let pressed_enter = frame.input_text("##consoleinput", &mut self.input_string)
                                        .enter_returns_true(true)
                                        .build();
                if pressed_enter {
                    frame.set_keyboard_focus_here();
                }

                if frame.is_item_focused() {
                    keybindings.disable_bindings();
                }

                frame.same_line();
                let mut run_code = pressed_enter;
                if width >= 0.0 {
                    run_code = frame.button_with_size("Run", [Self::RUN_BUTTON_WIDTH, 20.0]) || run_code;
                    frame.same_line();
                }

                if width >= -run_button_width {
                    frame.checkbox("##runcode", &mut self.run_code);
                    if self.last_frame != config.current_frame || self.last_rerecords != config.rerecords {
                        run_code = run_code || self.run_code;
                        self.last_frame = config.current_frame;
                        self.last_rerecords = config.rerecords;
                    }
                }

                if run_code {
                    if self.input_string.starts_with('/') || self.input_string.starts_with('.') {
                        // see if it's a known command
                        match self.input_string.split_at(1).1 {
                            "clear" => self.output.clear(),
                            _ => {
                                self.run_code = false;
                                self.output.push(format!("Unknown command: {}\n", self.input_string));
                            },
                        }
                    } else {
                        // run input as gml code
                        if !self.run_code {
                            self.output.push(format!(">>> {}\n", self.input_string));
                        }

                        let mut new_args: [Value; 16] = Default::default();
                        new_args[0] = self.input_string.clone().into();
                        match game.execute_string(&mut self.gml_context, &new_args) {
                            Ok(value) => match value {
                                Value::Str(string) => {
                                    if !self.run_code {
                                        self.output.push(format!("\"{}\"\n", string));
                                    }
                                },
                                Value::Real(real) => {
                                    if !self.run_code {
                                        self.output.push(format!("{}\n", real));
                                    }
                                },
                            },
                            Err(error) => {
                                self.run_code = false;
                                self.output.push(format!("Error: {}\n", error));
                            },
                        }

                        **clean_state = false;
                    }
                    if pressed_enter {
                        // only clear the input buffer if the user pressed enter
                        self.input_string.clear();
                    }
                    self.scroll_to_bottom = true;
                }
            });
    }

    fn is_open(&self) -> bool {
        self.is_open
    }
}

impl ConsoleWindow {
    const RUN_BUTTON_WIDTH: f32 = 50.0;
    const TEXTBOX_MIN_WIDTH: f32 = 15.0;

    pub fn new() -> Self {
        Self {
            input_string: String::with_capacity(256),
            gml_context: Context::with_single_instance(0),
            output: Vec::<String>::new(),

            scroll_to_bottom: false,
            run_code: false,
            last_frame: 0,
            last_rerecords: 0,

            is_open: true,

            id: 0,
        }
    }
}
