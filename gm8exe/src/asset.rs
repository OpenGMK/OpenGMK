pub mod background;
pub mod code_action;
pub mod constant;
pub mod extension;
pub mod font;
pub mod included_file;
pub mod object;
pub mod path;
pub mod room;
pub mod script;
pub mod sound;
pub mod sprite;
pub mod timeline;
pub mod trigger;

pub use self::{
    background::Background,
    code_action::CodeAction,
    constant::Constant,
    extension::Extension,
    font::Font,
    included_file::IncludedFile,
    object::Object,
    path::Path,
    room::Room,
    script::Script,
    sound::{Sound, SoundKind},
    sprite::Sprite,
    timeline::Timeline,
    trigger::{Trigger, TriggerKind},
};

use crate::GameVersion;
use byteorder::LE;
use std::{
    fmt::{self, Display},
    io,
};

pub trait Asset: Sized {
    /// Deserializes the asset from the format used in game executables.
    fn deserialize_exe(reader: impl io::Read, version: GameVersion, strict: bool) -> Result<Self, Error>;
    /// Serializes the asset to the format used in game executables.
    fn serialize_exe(&self, writer: impl io::Write, version: GameVersion) -> io::Result<()>;
}

#[derive(Debug)]
pub enum Error {
    IO(io::Error),
    MalformedData,
    VersionError { expected: u32, got: u32 },
}

impl std::error::Error for Error {}
impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            Error::IO(err) => format!("io error: {}", err),
            Error::VersionError { expected, got } => format!(
                "version error: expected {} ({}), found {} ({})",
                *expected,
                *expected as f32 / 100.0,
                *got,
                *got as f32 / 100.0
            ),
            Error::MalformedData => "malformed data while reading".into(),
        })
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::IO(err)
    }
}

impl From<(u32, u32)> for Error {
    fn from(version_error: (u32, u32)) -> Self {
        Error::VersionError { expected: version_error.0, got: version_error.1 }
    }
}

#[inline(always)]
fn assert_ver(got: u32, expected: u32) -> Result<(), Error> {
    if got != expected { Err(Error::VersionError { expected, got }) } else { Ok(()) }
}

#[inline(always)]
fn assert_ver_multiple(got: u32, expected: &[u32]) -> Result<(), Error> {
    if expected.contains(&got) { Ok(()) } else { Err(Error::VersionError { expected: expected[0], got }) }
}

#[derive(Debug, Default)]
pub struct PascalString(pub Box<[u8]>);

impl Display for PascalString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        String::from_utf8_lossy(self.0.as_ref()).fmt(f)
    }
}

impl From<&str> for PascalString {
    fn from(s: &str) -> Self {
        PascalString(s.to_owned().into_boxed_str().into_boxed_bytes())
    }
}

/// Helper trait to read big blocks of raw data.
pub trait ReadChunk: io::Read {
    fn read_chunk(&mut self, len: usize) -> io::Result<Vec<u8>> {
        // safety: read_exact specifies to expect buf to be uninitialized and never read from it
        let mut buf = Vec::with_capacity(len);
        unsafe { buf.set_len(len) };
        self.read_exact(&mut buf[..])?;
        Ok(buf)
    }
}
impl<R> ReadChunk for R where R: io::Read {}

// pascal-string extension for easy use
pub trait ReadPascalString: byteorder::ReadBytesExt + io::Read + ReadChunk {
    fn read_pas_string(&mut self) -> io::Result<PascalString> {
        let len = self.read_u32::<LE>()? as usize;
        let buf = self.read_chunk(len)?;
        Ok(PascalString(buf.into_boxed_slice()))
    }
}
impl<R> ReadPascalString for R where R: byteorder::ReadBytesExt + io::Read + ReadChunk {}

pub trait WritePascalString: byteorder::WriteBytesExt + io::Write {
    fn write_pas_string(&mut self, s: &PascalString) -> io::Result<()> {
        self.write_u32::<LE>(s.0.len() as u32)?;
        self.write_all(&s.0)?;
        Ok(())
    }
}
impl<W> WritePascalString for W where W: byteorder::WriteBytesExt + io::Write {}
