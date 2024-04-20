use crate::{
    game::recording::window::{Window, Openable, DisplayInformation},
    gml::{ Context, Value },
    imgui,
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
        format!("Console {}", self.id+1)
    }

    fn show_window(&mut self, info: &mut DisplayInformation) {
        let DisplayInformation {
            config,
            frame,
            game,
            keybindings,
            clean_state,
            ..
        } = info;

        frame.setup_next_window(imgui::Vec2(100.0, 100.0), Some(imgui::Vec2(600.0, 250.0)), None);
        if frame.begin_window(&self.name(), None, true, false, Some(&mut self.is_open)) {
            let window_size = frame.window_size();
            let content_position = frame.content_position();
            if frame.begin_listbox(&"GMLConsoleOutput", window_size - imgui::Vec2(content_position.0*2.0, 60.0)) {
                for text in &self.output {
                    frame.text(&text);
                }
                if self.scroll_to_bottom {
                    self.scroll_to_bottom = false;
                    frame.set_scroll_here_y(1.0);
                }
                frame.end_listbox();
            }

            // width = window width - padding - (width of checkbox + item x spacing) - (width of run button + item x spacing)
            let width = window_size.0 - content_position.0 * 2.0 - 58.0 - 28.0;
            if width > 0.0 {
                // checkbox and run button are visible
                frame.set_next_item_width(width);
            } else if width > -58.0 {
                // only checkbox is visible
                frame.set_next_item_width(width+58.0);
            } else if width > -58.0 - 28.0 {
                // nothing is visible
                frame.set_next_item_width(width+58.0+28.0)
            }

            let pressed_enter = frame.input_text(&"##consoleinput", &mut self.input_string, cimgui_sys::ImGuiInputTextFlags__ImGuiInputTextFlags_EnterReturnsTrue as _, None);
            if pressed_enter {
                frame.set_keyboard_focus_here(0);
            }

            if frame.is_item_focused() {
                keybindings.disable_bindings();
            }

            frame.same_line(0.0, -1.0);
            let mut run_code = pressed_enter;
            if width > 0.0 {
                run_code = frame.button(&"Run", imgui::Vec2(50.0, 20.0), None) || run_code;
                frame.same_line(0.0, -1.0);
            }

            frame.checkbox("##runcode", &mut self.run_code);
            if self.last_frame != config.current_frame || self.last_rerecords != config.rerecords {
                run_code = run_code || self.run_code;
                self.last_frame = config.current_frame;
                self.last_rerecords = config.rerecords;
            }

            if run_code {
                if self.input_string.starts_with('/') || self.input_string.starts_with('.') {
                    // see if it's a known command
                    match self.input_string.split_at(1).1 {
                        "clear" => self.output.clear(),
                        _ => {
                            self.run_code = false;
                            self.output.push(format!("Unknown command: {}\n", self.input_string));
                        }
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
                            Value::Str(string) => if !self.run_code { self.output.push(format!("\"{}\"\n", string)); },
                            Value::Real(real) => if !self.run_code { self.output.push(format!("{}\n", real)); },
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
        }
        frame.end();
    }

    fn is_open(&self) -> bool {
        self.is_open
    }
}

impl ConsoleWindow {
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
