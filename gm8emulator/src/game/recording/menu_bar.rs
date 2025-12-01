use crate::{
    game::recording::{
        UIState,
        keybinds::KeybindWindow,
        input_edit::InputEditWindow,
        console::ConsoleWindow,
        macro_window::MacroWindow,
        window::Openable,
    },
};

impl UIState<'_> {
    pub fn show_menu_bar(&mut self, frame: &mut imgui::Ui) -> bool {
        if let Some(main_menu_bar_token) = frame.begin_main_menu_bar() {
            if let Some(file_menu_token) = frame.begin_menu("File") {
                if frame.menu_item("Close") {
                    return false;
                }
                file_menu_token.end();
            }
            
            if let Some(window_menu_token) = frame.begin_menu("Windows") {
                if let Some(active_window_menu_token) = frame.begin_menu("Active Windows") {
                    for (window, focus) in &mut self.windows {
                        if frame.menu_item(&window.name()) {
                            *focus = true;
                        }
                    }
                    active_window_menu_token.end();
                }

                macro_rules! openable {
                    (@single $type:ty) => {
                        // see if a window of this type is already open
                        if let Some((_, focus)) = self.windows.iter_mut().find(|(win, _)| win.window_type_self() == <$type>::window_type()) {
                            // focus the window if it's already open
                            *focus = true;
                        } else {
                            // or create it
                            self.windows.push((Box::new(<$type>::open(0)), true));
                        }
                    };
                    (@multi $type:ty) => {{
                        // figure out all used ids for the windows
                        let mut ids: Vec<usize> = self.windows.iter().filter_map(|(win, _)| {
                            if win.window_type_self() == <$type>::window_type() {
                                Some(win.window_id())
                            } else {
                                None
                            }
                        }).collect();
                        // and select the smallest number not in use
                        let mut id: usize = 0;
                        if ids.len() > 0 {
                            ids.sort();
                            if let Some(new_id) = ids.iter().enumerate().position(|(i, id)| i != *id) {
                                id = new_id;
                            } else {
                                id = ids.len();
                            }
                        }
                        self.windows.push((Box::new(<$type>::open(id)), true));
                    }};
                    ($($id:ident $type:ty),* $(,)?) => {{
                        $(
                            // create the menu item for the window
                            if frame.menu_item(<$type>::window_name()) {
                                // and add the code for clicking on it, depending on whether multiple instances are allowed or not
                                openable!(@$id $type)
                            }
                        )*
                    }};
                }
                
                if let Some(open_menu_token) = frame.begin_menu("Open") {
                    openable! {
                        single KeybindWindow,
                        single InputEditWindow,
                        multi ConsoleWindow,
                        multi MacroWindow,
                    }
                    
                    open_menu_token.end();
                }

                window_menu_token.end();
            }
            
            main_menu_bar_token.end();
        }
        true
    }
}
