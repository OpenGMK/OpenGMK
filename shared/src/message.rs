use crate::{input, types::ID};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, io};

/// A message sent from the controller to the client.
#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    /// Initializes a recording session, asking the client for an update prior to any inputs
    Hello { keys_requested: Vec<input::Key>, mouse_buttons_requested: Vec<input::MouseButton> },

    /// Tells the game to advance a frame and then send us an update
    Advance {
        key_inputs: Vec<(input::Key, bool)>,
        mouse_inputs: Vec<(input::MouseButton, bool)>,
        mouse_location: (f64, f64),
        keys_requested: Vec<input::Key>,
        mouse_buttons_requested: Vec<input::MouseButton>,
    },

    /// Tells the game to create a savestate in the given index
    Save { index: usize },

    /// Tells the game to load the savestate with the given index and then send us an update
    Load { index: usize, keys_requested: Vec<input::Key>, mouse_buttons_requested: Vec<input::MouseButton> },
}

/// A message sent from the client to the controller.
#[derive(Debug, Serialize, Deserialize)]
pub enum Information {
    /// Updates the controller on the state of the game
    Update {
        keys_held: Vec<input::Key>,
        mouse_buttons_held: Vec<input::MouseButton>,
        mouse_location: (f64, f64),
        seed: i32,
        instance: Option<InstanceDetails>,
    },

    /// Tells the controller that a key was pressed while the game window was focused
    KeyPressed { key: input::Key },

    /// Tells the controller that the user clicked on an instance and provides its details
    InstanceClicked { details: InstanceDetails },

    /// Tells the controller that the game encountered an error
    GameError { error: String },

    /// Sends the controller some general info which should be shown to the user
    General { message: String },
}

/// The details of an instance sent to the control panel for display.
#[derive(Debug, Serialize, Deserialize)]
pub struct InstanceDetails {
    pub id: ID,
    pub object_name: String,
    pub x: f64,
    pub y: f64,
    pub speed: f64,
    pub direction: f64,
    pub timeline_info: Option<(i32, f64, f64)>,
    pub path_info: Option<(i32, f64, f64)>,
    pub alarms: HashMap<u32, i32>,
    pub bbox_top: i32,
    pub bbox_left: i32,
    pub bbox_right: i32,
    pub bbox_bottom: i32,
}

pub trait MessageStream {
    /// Serializes an object using bincode, then writes it as a length-tagged byte stream.
    fn send_message<S>(&mut self, s: S) -> io::Result<()>
    where
        S: Serialize;

    /// Receives a length-tagged byte stream, then deserializes it using bincode.
    /// This function does not block and will return Ok(Some(None)) if there is nothing in the pipe to read,
    /// and Ok(None) if the pipe is closed.
    /// A byte buffer must be provided for bincode. The buffer must outlive deserialized objects.
    fn receive_message<'de, D>(&mut self, read_buffer: &'de mut Vec<u8>) -> io::Result<Option<Option<D>>>
    where
        D: Deserialize<'de>;
}

impl<T> MessageStream for T
where
    T: io::Read + io::Write,
{
    fn send_message<S>(&mut self, s: S) -> io::Result<()>
    where
        S: Serialize,
    {
        let message = bincode::serialize(&s).expect("Failed to serialize message");
        self.write(&(message.len() as u32).to_le_bytes())?;
        self.write_all(&message)
    }

    fn receive_message<'de, D>(&mut self, read_buffer: &'de mut Vec<u8>) -> io::Result<Option<Option<D>>>
    where
        D: Deserialize<'de>,
    {
        let mut len_buffer = [0; 4];

        match self.read(&mut len_buffer) {
            Ok(0) => Ok(None),
            Ok(len) => {
                assert_eq!(len, 4);
                read_buffer.resize_with(u32::from_le_bytes(len_buffer) as usize, Default::default);
                loop {
                    match self.read(read_buffer) {
                        Ok(0) => break Ok(None),
                        Ok(len) => {
                            assert_eq!(len, read_buffer.len());
                            let d: D = bincode::deserialize::<D>(read_buffer).expect("Failed to deserialize message");
                            break Ok(Some(Some(d)))
                        },
                        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => (),
                        Err(e) => break Err(e),
                    }
                }
            },
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => Ok(Some(None)),
            Err(e) => Err(e),
        }
    }
}
