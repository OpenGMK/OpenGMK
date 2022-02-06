use crate::asset::ByteString;

#[derive(Debug)]
pub struct HelpDialog {
    pub bg_colour: u32,
    pub new_window: bool,
    pub caption: ByteString,
    pub left: i32,
    pub top: i32,
    pub width: u32,
    pub height: u32,
    pub border: bool,
    pub resizable: bool,
    pub window_on_top: bool,
    pub freeze_game: bool,
    pub info: ByteString,
}
