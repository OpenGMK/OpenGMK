/// Helper macro to easily assert version and print errors in GMK/EXE reading.
macro_rules! read_version {
    (
        $reader:expr,               // the reader object
        $asset_name:expr,           // dyn fmt::Display
        $format_is_gmk:expr,        // bool `is_gmk` ("GMK" else "EXE")
        $asset_type_name:literal,   // literal like "object"
        $valid:pat $(,)?            // pattern like "Gm800 | Gm810"
    ) => {{
        use crate::asset::Version::*; // for matching `$valid` without requiring `Version::`
        use log::error;

        let format = if $format_is_gmk { "GMK" } else { "EXE" };
        let num = ($reader).read_u32::<LE>()?;
        if let Ok(version) = <Version as ::std::convert::TryFrom<u32>>::try_from(num) {
            if matches!(version, $valid) {
                ::std::io::Result::Ok(version)
            } else {
                error!(
                    "Invalid version {} for {} \"{}\" in {}!",
                    version as u32, $asset_type_name, $asset_name, format,
                );
                ::std::io::Result::Err(::std::io::ErrorKind::InvalidData.into())
            }
        } else {
            error!(
                "Unknown version {} for {} \"{}\" in {}!",
                num, $asset_type_name, $asset_name, format,
            );
            ::std::io::Result::Err(::std::io::ErrorKind::InvalidData.into())
        }
    }};
}

mod script;
pub use script::Script;

use byteorder::{LE, ReadBytesExt, WriteBytesExt};
use std::{convert::TryFrom, fmt, io};

pub trait Asset: Sized {
    fn name(&self) -> &[u8];
    fn timestamp(&self) -> Timestamp;
    fn version(&self) -> Version;

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

impl fmt::Display for ByteString {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&*String::from_utf8_lossy(self.0.as_slice()))
    }
}

impl ByteString {
    pub(crate) fn read<R: io::Read>(mut reader: R) -> io::Result<Self> {
        let length = reader.read_u32::<LE>()? as usize;
        let mut bytes = Vec::with_capacity(length);
        unsafe {
            bytes.set_len(length);
        }
        reader.read_exact(bytes.as_mut_slice())?;
        Ok(Self(bytes))
    }

    pub(crate) fn write<W: io::Write>(&self, mut writer: W) -> io::Result<()> {
        let length = u32::try_from(self.0.len()).unwrap_or(u32::max_value());
        writer.write_u32::<LE>(length)?;
        writer.write_all(self.0.as_slice())
    }
}

#[derive(Copy, Clone, Default)]
pub struct Timestamp(pub f64);

impl fmt::Debug for Timestamp {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("Timestamp")
            .field(&"FUCK")
            .finish()
    }
}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
#[repr(u32)]
pub enum Version {
    Gm800 = 800,
    Gm810 = 810,
}

impl TryFrom<u32> for Version {
    type Error = ();
    fn try_from(x: u32) -> Result<Self, Self::Error> {
        match x {
            800 => Ok(Self::Gm800),
            810 => Ok(Self::Gm810),
            _ => Err(Default::default()),
        }
    }
}
