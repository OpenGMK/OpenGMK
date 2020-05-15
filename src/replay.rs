use std::collections::HashMap;
use winit::event::VirtualKeyCode;

// Represents an entire replay (TAS) file
#[derive(Debug)]
pub struct Replay {
    // The different room speeds that are used in this replay, and how many frames of each.
    // Mainly used for calculating the real-time length with minimal floating point errors.
    speed_map: HashMap<u32, usize>,

    // System time to use at the beginning of this replay.
    // Will be used to spoof some GML variables such as `current_time`.
    pub start_time: u128,

    // RNG seed to use at the beginning of this replay.
    pub start_seed: i32,

    // List of frames in this replay.
    frames: Vec<Frame>,
}

// Associated data for a single frame of playback
#[derive(Debug)]
pub struct Frame {
    pub mouse_x: i32,
    pub mouse_y: i32,
    pub inputs: Vec<Input>,
}

// An input event which takes place during a frame
#[derive(Debug)]
pub enum Input {
    KeyPress(VirtualKeyCode),
    KeyRelease(VirtualKeyCode),
    MousePress(usize),
    MouseRelease(usize),
    MouseWheelUp,
    MouseWheelDown,
}

impl Replay {
    pub fn new(start_time: u128, start_seed: i32) -> Self {
        Self {
            speed_map: HashMap::new(),
            start_time,
            start_seed,
            frames: Vec::new(),
        }
    }

    // Adds a new frame of input to the end of the replay.
    // Mouse position will be the same as the previous frame unless this is the first frame,
    // in which case it will be (0, 0)
    pub fn add_frame(&mut self) -> &mut Frame {
        let (mouse_x, mouse_y) = match self.frames.last() {
            Some(frame) => (frame.mouse_x, frame.mouse_y),
            None => (0, 0),
        };
        self.frames.push(Frame {
            mouse_x,
            mouse_y,
            inputs: Vec::new(),
        });
        self.frames.last_mut().unwrap() // Last cannot be None since we just pushed an element
    }

    // Gets the data associated with a given frame, if any
    pub fn get_frame(&self, index: usize) -> Option<&Frame> {
        self.frames.get(index)
    }

    // Calculates the length of this replay in seconds
    pub fn get_length(&self) -> f64 {
        self.speed_map.iter().map(|(&fps, &count)| (count as f64) / f64::from(fps)).sum()
    }
}
