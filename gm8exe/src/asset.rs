pub mod background;
pub mod constant;
pub mod extension;
pub mod font;
pub mod includedfile;
pub mod object;
pub mod path;
pub mod room;
pub mod script;
pub mod sound;
pub mod sprite;
pub mod timeline;
pub mod trigger;

pub mod etc;

pub use self::{
    background::Background,
    constant::Constant,
    extension::Extension,
    font::Font,
    includedfile::IncludedFile,
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
use std::{
    error::Error,
    fmt::{self, Display},
    io,
};

pub trait Asset {
    fn deserialize<B>(bytes: B, strict: bool, version: GameVersion) -> Result<Self, AssetDataError>
    where
        B: AsRef<[u8]>,
        Self: Sized;
    fn serialize<W>(&self, writer: &mut W) -> io::Result<usize>
    where
        W: io::Write;
}

#[derive(Debug)]
pub enum AssetDataError {
    IO(io::Error),
    MalformedData,
    VersionError { expected: u32, got: u32 },
}

impl Error for AssetDataError {}
impl Display for AssetDataError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            AssetDataError::IO(err) => format!("io error: {}", err),
            AssetDataError::VersionError { expected, got } => format!(
                "version error: expected {} ({}), found {} ({})",
                *expected,
                *expected as f32 / 100.0,
                *got,
                *got as f32 / 100.0
            ),
            AssetDataError::MalformedData => "malformed data while reading".into(),
        })
    }
}

impl From<io::Error> for AssetDataError {
    fn from(err: io::Error) -> Self {
        AssetDataError::IO(err)
    }
}

impl From<(u32, u32)> for AssetDataError {
    fn from(version_error: (u32, u32)) -> Self {
        AssetDataError::VersionError { expected: version_error.0, got: version_error.1 }
    }
}

#[inline(always)]
fn assert_ver(got: u32, expected: u32) -> Result<(), AssetDataError> {
    if got != expected { Err(AssetDataError::VersionError { expected, got }) } else { Ok(()) }
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

// pascal-string extension for easy use
pub trait ReadPascalString: io::Read + minio::ReadPrimitives + minio::ReadStrings {
    fn read_pas_string(&mut self) -> io::Result<PascalString> {
        let len = self.read_u32_le()? as usize;
        let mut buf = vec![0u8; len];
        self.read_exact(&mut buf)?;
        Ok(PascalString(buf.into_boxed_slice()))
    }
}

pub trait WritePascalString: io::Write + minio::WritePrimitives {
    fn write_pas_string(&mut self, s: &PascalString) -> io::Result<usize> {
        self.write_u32_le(s.0.len() as u32).and_then(|x| self.write_all(s.0.as_ref()).map(|()| s.0.len() + x))
    }
}

impl<R> ReadPascalString for R where R: io::Read {}
impl<W> WritePascalString for W where W: io::Write {}
