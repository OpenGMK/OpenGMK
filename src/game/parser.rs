use super::Game;
use byteorder::{ReadBytesExt, LE};
use flate2::read::ZlibDecoder;
use std::error;
use std::fmt::{self, Display};
use std::io::{self, Read, Seek, SeekFrom};
use std::option::NoneError;

const GM80_MAGIC_POS: u64 = 2000000;
const GM80_MAGIC: u32 = 1234321;

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
}

#[derive(Debug)]
pub enum ErrorKind {
    IO(io::Error),
    InvalidExeHeader,
    InvalidMagic,
    ReadError,
}

impl error::Error for Error {}
impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.kind {
            ErrorKind::IO(err) => write!(f, "IO Error: {}", err),
            ErrorKind::InvalidExeHeader => write!(f, "Invalid .exe header (missing 'MZ')"),
            ErrorKind::InvalidMagic => write!(f, "Invalid magic number (missing 1234321)"),
            ErrorKind::ReadError => write!(f, "Error while reading input data. Likely EOF."),
        }
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Error {
        Error { kind }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error {
            kind: ErrorKind::IO(err),
        }
    }
}

impl From<NoneError> for Error {
    fn from(_: NoneError) -> Error {
        Error {
            kind: ErrorKind::ReadError,
        }
    }
}

/// Convenience function for inflating zlib data. A preceding u32 indicating size is assumed.
fn inflate(data: &mut io::Cursor<Vec<u8>>) -> Result<Vec<u8>, Error> {
    let len = data.read_u32::<LE>()? as usize;
    let pos = data.position() as usize;
    let slice = data.get_ref().get(pos..pos + len)?;
    let mut decoder = ZlibDecoder::new(slice);
    let mut buf: Vec<u8> = Vec::with_capacity(len);
    decoder.read_to_end(&mut buf)?;
    data.seek(SeekFrom::Current(len as i64))?;
    Ok(buf)
}

impl Game {
    pub fn from_exe(exe: Vec<u8>) -> Result<(), Error> {
        // verify executable header
        if exe.get(0..2)? != b"MZ" {
            return Err(Error::from(ErrorKind::InvalidExeHeader));
        }

        // comfy wrapper for byteorder I/O
        let mut exe = io::Cursor::new(exe);

        // detect GameMaker version
        exe.set_position(GM80_MAGIC_POS);
        if exe.read_u32::<LE>()? != GM80_MAGIC {
            // support gm8.1 here later
            return Err(Error::from(ErrorKind::InvalidMagic));
        }

        // version version blahblah - I should do something with this later.
        exe.seek(SeekFrom::Current(12))?;

        // settings data chunk
        let _settings = inflate(&mut exe)?;

        // directx shared library (D3DX8.dll), we obviously don't need this...
        let dlln = exe.read_u32::<LE>()? as i64;
        exe.seek(SeekFrom::Current(dlln))?;
        let dll = exe.read_u32::<LE>()? as i64;
        exe.seek(SeekFrom::Current(dll))?;

        // time to decrypt the data!
        // oh_boy!()

        Ok(())
    }
}
