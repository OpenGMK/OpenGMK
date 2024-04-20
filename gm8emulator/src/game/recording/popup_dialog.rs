pub mod string_input;

use super::window::DisplayInformation;

pub enum DialogState {
    Submit,
    Open,
    Closed,
    Cancelled,
    Invalid,
}

pub trait Dialog {
    /// Displays the dialog as a modal popup
    fn show(&mut self, info: &mut DisplayInformation) -> DialogState;
    /// Gets the name of the popup. Each popup name used within a window must be unique.
    fn get_name(&self) -> &'static str;
    /// Clears all input fields from the dialog, called when requesting this dialog
    fn reset(&mut self);
}
