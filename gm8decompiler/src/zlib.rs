use flate2::{write::ZlibEncoder, Compression};

use byteorder::{WriteBytesExt, LE};
use std::io::{self, Write};

/// Takes some data and writes the compressed data to the cursor in GM8 format.
pub struct ZlibWriter {
    encoder: ZlibEncoder<Vec<u8>>,
}

impl ZlibWriter {
    #[inline]
    pub fn new() -> ZlibWriter {
        // TODO: Make a PR for flate2 and make Compression a const fn with the ctors and yeah .
        ZlibWriter { encoder: ZlibEncoder::new(Vec::new(), Compression::default()) }
    }

    pub fn finish(self, mut writer: impl io::Write) -> io::Result<()> {
        let encoded = self.encoder.finish()?;
        writer.write_u32::<LE>(encoded.len().try_into().expect("zlib block len > u32 max"))?;
        writer.write_all(&encoded)?;
        Ok(())
    }
}

impl Write for ZlibWriter {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.encoder.write_all(buf)?;
        Ok(buf.len())
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        self.encoder.flush()
    }

    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.encoder.write_all(buf)
    }
}

impl Default for ZlibWriter {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
