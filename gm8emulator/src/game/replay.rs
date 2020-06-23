use crate::gml::Value;
use serde::{Deserialize, Serialize};
use shared::input::{Key, MouseButton};

// Represents an entire replay (TAS) file
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Replay {
    // System time to use at the beginning of this replay.
    // Will be used to spoof some GML variables such as `current_time`.
    pub start_time: u128,

    // RNG seed to use at the beginning of this replay.
    pub start_seed: i32,

    // List of frames in this replay.
    frames: Vec<Frame>,
}

// Associated data for a single frame of playback
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Frame {
    pub fps: u32,
    pub mouse_x: i32,
    pub mouse_y: i32,
    pub inputs: Vec<Input>,
    pub events: Vec<Event>,
}

// Stored events for certain things which must always happen the same way during replay
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Event {
    GetInteger(Value),   // value returned from get_integer()
    GetString(Value),    // value returned from get_string()
    Randomize(i32),      // value assigned to seed by randomize()
    ShowMenu(Value),     // value returned from show_menu()
    ShowMessage,         // acknowledges that a show_message() does not need to be shown during replay
    ShowQuestion(Value), // value returned from show_question()
}

// An input event which takes place during a frame
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Input {
    KeyPress(Key),
    KeyRelease(Key),
    MousePress(MouseButton),
    MouseRelease(MouseButton),
    MouseWheelUp,
    MouseWheelDown,
}

impl Replay {
    pub fn new(start_time: u128, start_seed: i32) -> Self {
        Self { start_time, start_seed, frames: Vec::new() }
    }

    // Adds a new frame of input to the end of the replay.
    // Mouse position will be the same as the previous frame unless this is the first frame,
    // in which case it will be (0, 0)
    pub fn new_frame(&mut self, fps: u32) -> &mut Frame {
        let (mouse_x, mouse_y) = match self.frames.last() {
            Some(frame) => (frame.mouse_x, frame.mouse_y),
            None => (0, 0),
        };
        self.frames.push(Frame { fps, mouse_x, mouse_y, inputs: Vec::new(), events: Vec::new() });
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

    // Calculates the length of this replay in milliseconds
    pub fn get_length(&self) -> f64 {
        // We want to do this in a way that'll avoid FPI as much as possible (for example in a 60FPS game)
        let mut ms: f64 = 0.0;

        let mut iter = self.frames.iter().peekable();
        loop {
            let speed = match iter.next() {
                Some(s) => s.fps,
                None => break,
            };
            let mut count: usize = 1;
            while let Some(Frame { fps, .. }) = iter.peek() {
                if *fps == speed {
                    iter.next();
                    count += 1;
                } else {
                    break
                }
            }
            ms += (count * 1000) as f64 / (speed as f64)
        }
        ms
    }
}
