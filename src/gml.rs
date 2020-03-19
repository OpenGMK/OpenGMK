pub mod compiler;
pub mod context;
pub mod kernel;
pub mod rand;
pub mod runtime;
pub mod value;

pub use compiler::Compiler;
pub use context::Context;
pub use value::Value;

pub type Result<T> = std::result::Result<T, runtime::Error>;
pub use runtime::Error as Error;

pub const TRUE: f64 = 1.0;
pub const FALSE: f64 = 0.0;

pub const SELF: i32 = -1;
pub const OTHER: i32 = -2;
pub const ALL: i32 = -3;
pub const NOONE: i32 = -4;
pub const GLOBAL: i32 = -5;
pub const LOCAL: i32 = -7;

pub mod ev {
    pub const CREATE: usize = 0;
    pub const DESTROY: usize = 1;
    pub const ALARMS: usize = 2;
    pub const STEP: usize = 3;
    pub const COLLISION: usize = 4;
    pub const KEYBOARD: usize = 5;
    pub const MOUSE: usize = 6;
    pub const OTHER: usize = 7;
    pub const DRAW: usize = 8;
    pub const KEYPRESS: usize = 9;
    pub const KEYRELEASE: usize = 10;
    pub const TRIGGER: usize = 11;
}

/// Enum for each instance variable
#[derive(Clone, Copy, Debug)]
pub enum InstanceVariable {
    Id,
    ObjectIndex,
    Solid,
    Visible,
    Persistent,
    Depth,
    Alarm,
    SpriteIndex,
    ImageAlpha,
    ImageBlend,
    ImageIndex,
    ImageSpeed,
    ImageXscale,
    ImageYscale,
    ImageAngle,
    MaskIndex,
    Direction,
    Friction,
    Gravity,
    GravityDirection,
    Hspeed,
    Vspeed,
    Speed,
    X,
    Y,
    Xprevious,
    Yprevious,
    Xstart,
    Ystart,
    PathIndex,
    PathPosition,
    PathPositionprevious,
    PathSpeed,
    PathScale,
    PathOrientation,
    PathEndaction,
    TimelineIndex,
    TimelineRunning,
    TimelineSpeed,
    TimelinePosition,
    TimelineLoop,

    SpriteWidth,
    SpriteHeight,
    BboxLeft,
    BboxRight,
    BboxBottom,
    BboxTop,
}

/// Enum for each game variable
#[derive(Clone, Copy, Debug)]
pub enum GameVariable {
    Argument,
    Argument0,
    Argument1,
    Argument2,
    Argument3,
    Argument4,
    Argument5,
    Argument6,
    Argument7,
    Argument8,
    Argument9,
    Argument10,
    Argument11,
    Argument12,
    Argument13,
    Argument14,
    Argument15,
    CurrentTime,
    Health,
    InstanceCount,
    Lives,
    MouseX,
    MouseY,
    Room,
    RoomSpeed,
    RoomHeight,
    RoomWidth,
    RoomCaption,
    ViewEnabled,
    ViewVisible,
    ViewXview,
    ViewYview,
    ViewWview,
    ViewHview,
    ViewXport,
    ViewYport,
    ViewWport,
    ViewHport,
    ViewAngle,
    ViewHborder,
    ViewVborder,
    ViewHspeed,
    ViewVspeed,
    ViewObject,
}
