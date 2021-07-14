use crate::{asset::PascalString, colour::Colour};

/// The Settings header for a GM8 game
pub struct Settings {
    /// Start in full-screen mode
    pub fullscreen: bool,

    /// Scaling
    ///
    /// Fixed scale, in %.
    /// If it's negative (usually `-1`), Keep aspect ratio.
    /// Otherwise if it's `0`, Full scale.
    pub scaling: i32,

    /// Interpolate colours between pixels
    pub interpolate_pixels: bool,

    /// Colour outside the room region (RGBA)
    pub clear_colour: u32,

    /// Allow the player to resize the game window
    pub allow_resize: bool,

    /// Let the game window always stay on top
    pub window_on_top: bool,

    /// Don't draw a border in windowed mode
    pub dont_draw_border: bool,

    /// Don't show the buttons in the window captions
    pub dont_show_buttons: bool,

    /// Display the cursor
    pub display_cursor: bool,

    /// Freeze the game window when the window loses focus
    pub freeze_on_lose_focus: bool,

    /// Disable screensavers and power saving actions
    pub disable_screensaver: bool,

    /// Force Direct3D software vertex processing (GM 8.0 behaviour)
    pub force_cpu_render: bool,

    /// Set the resolution of the screen
    pub set_resolution: bool,

    /// Sub-var of `set_resolution` - Color Depth
    ///
    /// 0 - No Change
    ///
    /// 1 - 16-Bit
    ///
    /// 2 - 32-Bit
    pub colour_depth: u32,

    /// Sub-var of `set_resolution` - Resolution
    ///
    /// 0 - No change
    ///
    /// 1 - 320x240
    ///
    /// 2 - 640x480
    ///
    /// 3 - 800x600
    ///
    /// 4 - 1024x768
    ///
    /// 5 - 1280x1024
    ///
    /// 6 - 1600x1200
    pub resolution: u32,

    /// Sub-var of `set_resolution` - Frequency
    ///
    /// 0 - No Change
    ///
    /// 1 - 60Hz
    ///
    /// 2 - 70Hz
    ///
    /// 3 - 85Hz
    ///
    /// 4 - 100Hz
    ///
    /// 5 - 120Hz
    pub frequency: u32,

    /// Use synchronization to avoid tearing
    pub vsync: bool,

    /// Let <Esc> end the game
    pub esc_close_game: bool,

    /// Treat the close button as the <Esc> key
    pub treat_close_as_esc: bool,

    /// Let <F1> show the game information
    pub f1_help_menu: bool,

    /// Let <F4> switch between screen modes
    pub f4_fullscreen_toggle: bool,

    /// Let <F5> save the game and <F6> load a game
    pub f5_save_f6_load: bool,

    /// Let <F9> take a screenshot of the game
    pub f9_screenshot: bool,

    /// Game Process Priority
    ///
    /// 0 - Normal
    ///
    /// 1 - High
    ///
    /// 2 - Highest
    ///
    pub priority: u32,

    /// Show your own image while loading (data)
    pub custom_load_image: Option<Box<[u8]>>,

    /// Sub-value of `custom_load_image`:
    /// Make image partially translucent
    pub transparent: bool,

    /// Sub-value of `custom_load_image` + `transparent`
    ///
    /// Make translucent with alpha value: x
    pub translucency: u32,

    /// 0 - No loading progress bar
    ///
    /// 1 - Default loading progress bar
    ///
    /// 2 - Own loading progress bar
    pub loading_bar: u32,

    /// Loading bar - (Custom) Back Image
    pub backdata: Option<Box<[u8]>>,

    /// Loading bar - (Custom) Front Image
    pub frontdata: Option<Box<[u8]>>,

    /// Scale progress bar image
    pub scale_progress_bar: bool,

    /// Display error messages
    pub show_error_messages: bool,

    /// Write error messages to file game_errors.log
    pub log_errors: bool,

    /// Abort on all error messages
    pub always_abort: bool,

    /// Treat uninitialized variables as value 0
    pub zero_uninitialized_vars: bool,

    /// Throw an error when arguments aren't initialized correctly
    pub error_on_uninitialized_args: bool,

    /// Run create events before instance creation code (not available in base 8.1)
    pub swap_creation_events: bool,
}

/// The help dialog box associated with a GM8 game
#[derive(Debug)]
pub struct GameHelpDialog {
    pub bg_colour: Colour,
    pub new_window: bool,
    pub caption: PascalString,
    pub left: i32,
    pub top: i32,
    pub width: u32,
    pub height: u32,
    pub border: bool,
    pub resizable: bool,
    pub window_on_top: bool,
    pub freeze_game: bool,
    pub info: PascalString,
}
