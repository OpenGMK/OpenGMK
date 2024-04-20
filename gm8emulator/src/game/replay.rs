use crate::gml::Value;
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use lzzzz::lz4;
use serde::{Deserialize, Serialize};
use std::{
    fs::{File, OpenOptions},
    io::{self, Read, Write},
    path::PathBuf,
};

// Represents an entire replay (TAS) file
#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
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

#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum FrameRng {
    Override(i32),
    Increment(i32),
}

impl FrameRng {
    pub fn increase(&mut self) {
        *self = match self {
            FrameRng::Increment(amount) => FrameRng::Increment(*amount+1),
            FrameRng::Override(seed) => FrameRng::Override(seed.wrapping_add(1))
        }
    }

    pub fn decrease(&mut self) -> bool {
        *self = match self {
            FrameRng::Increment(amount) => if *amount <= 1 { FrameRng::Increment(0) } else { FrameRng::Increment(*amount-1) }
            FrameRng::Override(seed) => FrameRng::Override(seed.wrapping_sub(1))
        };

        !matches!(*self, FrameRng::Increment(0)) // Return false if we would not actually change the seed at all (0 increments). We can't identify this situation for Override since we don't know the original seed
    }
}

// Associated data for a single frame of playback
#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct Frame {
    pub mouse_x: i32,
    pub mouse_y: i32,
    pub inputs: Vec<Input>,
    pub events: Vec<Event>,
    pub new_seed: Option<FrameRng>,
    pub new_time: Option<u128>,
}

// Stored events for certain things which must always happen the same way during replay
#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum Event {
    GetInteger(Value),   // value returned from get_integer()
    GetString(Value),    // value returned from get_string()
    Randomize(i32),      // value assigned to seed by randomize()
    ShowMenu(Value),     // value returned from show_menu()
    ShowMessage,         // acknowledges that a show_message() does not need to be shown during replay
    ShowQuestion(Value), // value returned from show_question()
}

// An input event which takes place during a frame
#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum Input {
    KeyPress(u8),
    KeyRelease(u8),
    MousePress(i8),
    MouseRelease(i8),
    MouseWheelUp,
    MouseWheelDown,
}

#[derive(Debug)]
pub enum ReadError {
    IOErr(io::Error),
    DecompressErr(lzzzz::Error),
    DeserializeErr(Box<bincode::ErrorKind>),
    UnknownVersion(u32),
}

#[derive(Debug)]
pub enum WriteError {
    IOErr(io::Error),
    CompressErr(lzzzz::Error),
    SerializeErr(Box<bincode::ErrorKind>),
}

impl Replay {
    pub fn new(start_time: u128, start_seed: i32) -> Self {
        Self { start_time, start_seed, startup_events: Vec::new(), frames: Vec::new() }
    }

    // Loads a Replay from a gmtas-format file (doesn't check the file extension)
    pub fn from_file(path: &PathBuf) -> Result<Self, ReadError> {
        let mut lz4_buf = Vec::new();
        let mut bin_buf = Vec::new();
        let mut file = File::open(path).map_err(ReadError::IOErr)?;

        match file.read_u32::<LE>() {
            Ok(1) => {
                let init_size = file.metadata().map(|m| m.len() as usize + 1).unwrap_or(0);
                lz4_buf.reserve(init_size);
                match file.read_to_end(&mut lz4_buf) {
                    Ok(_) => match (lz4_buf.as_slice().read_u64::<LE>().map(|x| x as usize), lz4_buf.get(8..)) {
                        (Ok(len), Some(block)) => {
                            bin_buf.reserve(len);
                            unsafe { bin_buf.set_len(len) };
                            match lz4::decompress(block, bin_buf.as_mut_slice()) {
                                Ok(len) => {
                                    unsafe { bin_buf.set_len(len) };
                                    bincode::deserialize::<'_, Self>(bin_buf.as_slice())
                                        .map_err(ReadError::DeserializeErr)
                                },
                                Err(err) => Err(ReadError::DecompressErr(err)),
                            }
                        },
                        (Ok(_), None) => Err(ReadError::IOErr(io::Error::from(io::ErrorKind::UnexpectedEof))),
                        (Err(err), _) => Err(ReadError::IOErr(err)),
                    },
                    Err(err) => Err(ReadError::IOErr(err)),
                }
            },
            Ok(v) => Err(ReadError::UnknownVersion(v)),
            Err(e) => Err(ReadError::IOErr(e)),
        }
    }

    // Serializes this replay into a file
    pub fn to_file(&self, path: &PathBuf) -> Result<(), WriteError> {
        let mut lz4_buf = Vec::new();
        let mut bin_buf = Vec::new();
        match bincode::serialize_into(&mut bin_buf, self) {
            Ok(()) => match lz4::compress_to_vec(bin_buf.as_slice(), lz4_buf.as_mut(), lz4::ACC_LEVEL_DEFAULT) {
                Ok(_length) => {
                    match OpenOptions::new().create(true).write(true).truncate(true).open(path).and_then(|mut f| {
                        f.write_u32::<LE>(1).and_then(|_| {
                            f.write_u64::<LE>(bin_buf.len() as u64).and_then(|_| f.write_all(lz4_buf.as_slice()))
                        })
                    }) {
                        Ok(()) => Ok(()),
                        Err(e) => Err(WriteError::IOErr(e)),
                    }
                },
                Err(err) => Err(WriteError::CompressErr(err)),
            },
            Err(err) => Err(WriteError::SerializeErr(err)),
        }
    }

    // Adds a new frame of input to the end of the replay.
    // Mouse position will be the same as the previous frame unless this is the first frame,
    // in which case it will be (0, 0)
    pub fn new_frame(&mut self) -> &mut Frame {
        let (mouse_x, mouse_y) = match self.frames.last() {
            Some(frame) => (frame.mouse_x, frame.mouse_y),
            None => (0, 0),
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

    // Adds a new frame of input to the specified position of the replay.
    // Mouse position will be the same as the previous frame unless this is the first frame,
    // in which case it will be (0, 0)
    pub fn insert_new_frame(&mut self, index: usize) -> &mut Frame {
        let (mouse_x, mouse_y) = match self.frames.get(index-1) {
            Some(frame) => (frame.mouse_x, frame.mouse_y),
            None => (0, 0),
        };
        self.frames.insert(index, Frame {
            mouse_x,
            mouse_y,
            inputs: Vec::new(),
            events: Vec::new(),
            new_seed: None,
            new_time: None,
        });
        self.frames.get_mut(index).unwrap()
    }

    pub fn delete_frame(&mut self, index: usize) -> Frame {
        self.frames.remove(index)
    }

    // Gets the data associated with a given frame, if any
    pub fn get_frame(&self, index: usize) -> Option<&Frame> {
        self.frames.get(index)
    }

    // Gets the data associated with a given frame, if any
    pub fn get_frame_mut(&mut self, index: usize) -> Option<&mut Frame> {
        self.frames.get_mut(index)
    }

    // Gets the replay's frame count
    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }

    pub fn truncate_frames(&mut self, len: usize) {
        self.frames.truncate(len)
    }

    // Returns whether this replay begins the same way as the other one.
    pub fn contains_part(&self, other: &Replay) -> bool {
        if self.frame_count() > other.frame_count() {
            let mut part = self.clone();
            part.frames.truncate(other.frame_count());
            part == *other
        } else if self.frame_count() == other.frame_count() {
            *self == *other
        } else {
            false
        }
    }
}
