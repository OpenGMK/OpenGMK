mod constant;
mod room;
mod script;
mod sound;
mod trigger;

pub use self::constant::Constant;
pub use self::room::Room;
pub use self::script::Script;
pub use self::sound::{Sound, SoundKind};
pub use self::trigger::{Trigger, TriggerKind};

use std::{
    error::Error,
    fmt::{self, Display},
    io,
};

pub trait Asset {
    fn deserialize<B>(bytes: B, strict: bool, version: u32) -> Result<Self, AssetDataError>
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
    VersionError { expected: u32, got: u32 },
}

impl Error for AssetDataError {}
impl Display for AssetDataError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                AssetDataError::IO(err) => format!("io error: {}", err),
                AssetDataError::VersionError { expected, got } => format!(
                    "version error: expected {} ({}), found {} ({})",
                    *expected,
                    *expected as f32 / 100.0,
                    *got,
                    *got as f32 / 100.0
                ),
            }
        )
    }
}

impl From<io::Error> for AssetDataError {
    fn from(err: io::Error) -> Self {
        AssetDataError::IO(err)
    }
}

impl From<(u32, u32)> for AssetDataError {
    fn from(version_error: (u32, u32)) -> Self {
        AssetDataError::VersionError {
            expected: version_error.0,
            got: version_error.1,
        }
    }
}

#[inline(always)]
fn assert_ver(got: u32, expected: u32) -> Result<(), AssetDataError> {
    if got != expected {
        Err(AssetDataError::VersionError { expected, got })
    } else {
        Ok(())
    }
}
