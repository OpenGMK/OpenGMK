use crate::{
    game::recording::window::{Window, DisplayInformation},
    gml::{ Context, Value },
    imgui,
};

pub struct ConsoleWindow {
    input_buffer: Vec<u8>,
    gml_context: Context,
    output: Vec<String>,

    scroll_to_bottom: bool,
    run_code: bool,
    last_frame: usize,
    last_rerecords: u64,
}

// Keyboard & Mouse window
impl Window for ConsoleWindow {
    fn show_window(&mut self, info: &mut DisplayInformation) {
        let DisplayInformation {
            config,
            frame,
            game,
            keybindings,
            ..
        } = info;

        if frame.begin_window(&"GML Console", None, true, false, None) {
            let window_size = frame.window_size();
            let content_position = frame.content_position();
            frame.begin_listbox(&"GMLConsoleOutput", window_size - imgui::Vec2(content_position.0*2.0, 60.0));
            for text in &self.output {
                frame.text(&text);
            }
            if self.scroll_to_bottom {
                self.scroll_to_bottom = false;
                frame.set_scroll_here_y(1.0);
            }
            frame.end_listbox();
            let width = window_size.0 - content_position.0 * 0.2 - 100.0;
            if width > 0.0 {
                // checkbox and run button are visible
                frame.set_next_item_width(width);
            } else if width > -50.0 {
                // only checkbox is visible
                frame.set_next_item_width(width+50.0);
            }
            let pressed_enter = frame.input_text(&"###consoleinput", self.input_buffer.as_mut_ptr(), self.input_buffer.len(), cimgui_sys::ImGuiInputTextFlags__ImGuiInputTextFlags_EnterReturnsTrue as _);
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
                let mut new_args: [Value; 16] = Default::default();
                new_args[0] = self.input_buffer.iter().take_while(|x| **x != 0u8).copied().collect::<Vec<u8>>().into();
                if !self.run_code {
                    if let Some(Value::Str(s)) = new_args.get(0) {
                        self.output.push(format!(">>> {}\n", s));
                    }
                }
                if pressed_enter {
                    // only clear the input buffer if the user pressed enter
                    self.input_buffer.fill(0);
                }
                match game.execute_string(&mut self.gml_context, &new_args) {
                    Ok(value) => match value {
                        Value::Str(string) => if !self.run_code { self.output.push(format!("\"{}\"\n", string)); },
                        Value::Real(real) => if !self.run_code { self.output.push(format!("{}\n", real)); },
                    },
                    Err(error) => {
                        self.run_code = false;
                        self.output.push(format!("Error: {0}\n", error));
                    },
                }
                self.scroll_to_bottom = true;
            }
        }
        frame.end();
    }

    fn is_open(&self) -> bool { true }
}

impl ConsoleWindow {
    pub fn new() -> Self {
        Self {
            input_buffer: vec![0 as u8; 1024],
            gml_context: Context::with_single_instance(0),
            output: Vec::<String>::new(),

            scroll_to_bottom: false,
            run_code: false,
            last_frame: 0,
            last_rerecords: 0,
        }
    }
}
