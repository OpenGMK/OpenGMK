// A lightweight Rust 1.32 replacement for `byteorder` functions.

use std::io;

use std::f64;
use std::u32;
use std::u64;

pub trait ReadBytes: io::Read {
    /// Reads a `u32` (little-endian) from the underlying reader.
    fn read_u32_le(&mut self) -> io::Result<u32> {
        let mut data = [0u8; 4];
        self.read_exact(&mut data)?;
        Ok(u32::from_le_bytes(data))
    }

    /// Reads an `f64` (little-endian) from the underlying reader.
    fn read_f64_le(&mut self) -> io::Result<f64> {
        let mut data = [0u8; 8];
        self.read_exact(&mut data)?;
        Ok(f64::from_bits(u64::from_le_bytes(data)))
    }
}

/// Helper trait for reading strings.
pub trait ReadString: io::Read {
    /// Reads a pascal-style string from the underlying reader.
    fn read_pas_string(&mut self) -> io::Result<String> {
        let len = self.read_u32_le()? as usize;
        let mut buf = vec![0u8; len];
        self.read(&mut buf)?;
        Ok(String::from_utf8_lossy(&buf).into_owned())
    }
}

impl<R> ReadBytes for R where R: io::Read + ?Sized {}
impl<R> ReadString for R where R: io::Read + ?Sized {}
