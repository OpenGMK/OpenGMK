use crate::{gml::Value};
use serde::{Deserialize, Serialize};

// Represents an entire replay (TAS) file
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Replay {
    // System time to use at the beginning of this replay.
    // Will be used to spoof some GML variables such as `current_time`.
    pub start_time: u128,

    // RNG seed to use at the beginning of this replay.
    pub start_seed: i32,

    // Special list of stored events used during startup (before frame 0)
    pub startup_events: Vec<Event>,

    // List of frames in this replay.
    frames: Vec<Frame>,
}

// Associated data for a single frame of playback
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Frame {
    pub mouse_x: f64,
    pub mouse_y: f64,
    pub inputs: Vec<Input>,
    pub events: Vec<Event>,
    pub new_seed: Option<i32>,
    pub new_time: Option<u128>,
}

// Stored events for certain things which must always happen the same way during replay
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Event {
    GetInteger(Value),    // value returned from get_integer()
    GetString(Value),     // value returned from get_string()
    Randomize(i32),       // value assigned to seed by randomize()
    ShowMenu(Value),      // value returned from show_menu()
    ShowMessage,          // acknowledges that a show_message() does not need to be shown during replay
    ShowQuestion(Value),  // value returned from show_question()
    SoundIsPlaying(bool), // value returned from sound_isplaying()
}

// An input event which takes place during a frame
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Input {
    KeyPress(u8),
    KeyRelease(u8),
    MousePress(i8),
    MouseRelease(i8),
    MouseWheelUp,
    MouseWheelDown,
}

impl Replay {
    pub fn new(start_time: u128, start_seed: i32) -> Self {
        Self { start_time, start_seed, startup_events: Vec::new(), frames: Vec::new() }
    }

    // Adds a new frame of input to the end of the replay.
    // Mouse position will be the same as the previous frame unless this is the first frame,
    // in which case it will be (0, 0)
    pub fn new_frame(&mut self) -> &mut Frame {
        let (mouse_x, mouse_y) = match self.frames.last() {
            Some(frame) => (frame.mouse_x, frame.mouse_y),
            None => (0.0, 0.0),
        };
        self.frames.push(Frame {
            mouse_x,
            mouse_y,
            inputs: Vec::new(),
            events: Vec::new(),
            new_seed: None,
            new_time: None,
        });
        self.frames.last_mut().unwrap() // Last cannot be None since we just pushed an element
    }

    // Gets the data associated with a given frame, if any
    pub fn get_frame(&self, index: usize) -> Option<&Frame> {
        self.frames.get(index)
    }

    // Gets the replay's frame count
    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }
}
