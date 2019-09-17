//! A lightweight Rust 1.32 replacement to byteorder-style functions for std::io Read/Write.
// TODO: Make this its own crate please

use std::io;

use std::f64;
use std::i32;
use std::u32;
use std::u64;

pub trait ReadBytes: io::Read {
    /// Reads an `i32` (little-endian) from the underlying reader.
    #[inline(always)]
    fn read_i32_le(&mut self) -> io::Result<i32> {
        let mut data = [0u8; 4];
        self.read_exact(&mut data)?;
        Ok(i32::from_le_bytes(data))
    }

    /// Reads a `u32` (little-endian) from the underlying reader.
    #[inline(always)]
    fn read_u32_le(&mut self) -> io::Result<u32> {
        let mut data = [0u8; 4];
        self.read_exact(&mut data)?;
        Ok(u32::from_le_bytes(data))
    }

    /// Reads an `i16` (little-endian) from the underlying reader.
    #[inline(always)]
    fn read_i16_le(&mut self) -> io::Result<i16> {
        let mut data = [0u8; 2];
        self.read_exact(&mut data)?;
        Ok(i16::from_le_bytes(data))
    }

    /// Reads a `u16` (little-endian) from the underlying reader.
    #[inline(always)]
    fn read_u16_le(&mut self) -> io::Result<u16> {
        let mut data = [0u8; 2];
        self.read_exact(&mut data)?;
        Ok(u16::from_le_bytes(data))
    }

    /// Reads an `f64` (little-endian) from the underlying reader.
    #[inline(always)]
    fn read_f64_le(&mut self) -> io::Result<f64> {
        let mut data = [0u8; 8];
        self.read_exact(&mut data)?;
        Ok(f64::from_bits(u64::from_le_bytes(data)))
    }

    /// Reads a `u8` from the underlying reader.
    #[inline(always)]
    fn read_u8(&mut self) -> io::Result<u8> {
        let mut data = [0u8; 1];
        self.read_exact(&mut data)?;
        Ok(u8::from_le_bytes(data))
    }
}

pub trait WriteBytes: io::Write {
    /// Writes a `i32` (little-endian) to the underlying writer.
    #[inline(always)]
    fn write_i32_le(&mut self, n: i32) -> io::Result<usize> {
        Ok(self.write(&n.to_le_bytes())?)
    }

    /// Writes a `u32` (little-endian) to the underlying writer.
    #[inline(always)]
    fn write_u32_le(&mut self, n: u32) -> io::Result<usize> {
        Ok(self.write(&n.to_le_bytes())?)
    }

    /// Writes an `f64` (little-endian) to the underlying writer.
    #[inline(always)]
    fn write_f64_le(&mut self, n: f64) -> io::Result<usize> {
        Ok(self.write(&(n.to_bits()).to_le_bytes())?)
    }
}

/// Helper trait for reading strings.
pub trait ReadString: io::Read {
    /// Reads a pascal-style string from the underlying reader.
    /// A preceding little-endian u32 indicating size is assumed.
    #[inline(always)]
    fn read_pas_string(&mut self) -> io::Result<String> {
        let len = self.read_u32_le()? as usize;
        let mut buf = vec![0u8; len];
        self.read(&mut buf)?;
        Ok(String::from_utf8_lossy(&buf).into_owned())
    }
}

pub trait WriteString: io::Write {
    /// Writes a pascal-style string to the underlying writer.
    /// A preceding little-endian u32 indicating size will be included.
    /// Returns the bytes written, including the size prefix.
    #[inline(always)]
    fn write_pas_string(&mut self, s: &str) -> io::Result<usize> {
        Ok(self.write(&(s.len() as u32).to_le_bytes())? + self.write(s.as_bytes())?)
    }
}

impl<R> ReadBytes for R where R: io::Read + ?Sized {}
impl<R> ReadString for R where R: io::Read + ?Sized {}
impl<W> WriteBytes for W where W: io::Write + ?Sized {}
impl<W> WriteString for W where W: io::Write + ?Sized {}
