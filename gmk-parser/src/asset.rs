mod script;

use byteorder::{LE, ReadBytesExt, WriteBytesExt};
use std::{convert::TryFrom, fmt, io};

pub trait Asset: Sized {
    fn name(&self) -> &[u8];

    fn from_gmk<R: io::Read>(&self, r: R) -> io::Result<Self>;
    fn to_gmk<W: io::Write>(&self, w: W) -> io::Result<()>;

    fn from_exe<R: io::Read>(&self, r: R) -> io::Result<Self>;
    fn to_exe<W: io::Write>(&self, w: W) -> io::Result<()>;
}

/// Represents a GameMaker string which may or may not be valid UTF-8.
#[derive(Clone)]
pub struct ByteString(pub Vec<u8>);

impl fmt::Debug for ByteString {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("ByteString")
            .field(&&*String::from_utf8_lossy(self.0.as_slice()))
            .finish()
    }
}
