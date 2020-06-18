use serde::{Deserialize, Serialize};
use std::io;

pub trait MessageStream {
    /// Serializes an object using bincode, then writes it as a length-tagged byte stream.
    fn send_message<S>(&mut self, s: S) -> io::Result<()>
    where
        S: Serialize;

    /// Receives a length-tagged byte stream, then deserializes it using bincode.
    /// This function does not block and will return Ok(None) if there is nothing in the pipe to read.
    /// A byte buffer must be provided for bincode. The buffer must outlive deserialized objects.
    fn receive_message<'de, D>(&mut self, read_buffer: &'de mut Vec<u8>) -> io::Result<Option<D>>
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

    fn receive_message<'de, D>(&mut self, read_buffer: &'de mut Vec<u8>) -> io::Result<Option<D>>
    where
        D: Deserialize<'de>,
    {
        let mut len_buffer = [0; 4];

        match self.read(&mut len_buffer) {
            Ok(len) => {
                assert_eq!(len, 4);
                read_buffer.resize_with(u32::from_le_bytes(len_buffer) as usize, Default::default);
                loop {
                    match self.read(read_buffer) {
                        Ok(len) => {
                            assert_eq!(len, read_buffer.len());
                            let d: D = bincode::deserialize::<D>(read_buffer).expect("Failed to deserialize message");
                            break Ok(Some(d))
                        },
                        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => (),
                        Err(e) => break Err(e),
                    }
                }
            },
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => Ok(None),
            Err(e) => Err(e),
        }
    }
}
