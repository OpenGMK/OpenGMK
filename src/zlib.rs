use flate2::write::ZlibEncoder;
use minio::WritePrimitives;
use std::io::{self, Write};

/// Takes some data and writes the compressed data to the cursor in GM8 format.
pub struct ZlibWriter {
    pub encoder: ZlibEncoder<Vec<u8>>,
}

impl ZlibWriter {
    pub fn new() -> ZlibWriter {
        ZlibWriter { encoder: ZlibEncoder::new(Vec::new(), flate2::Compression::default()) }
    }

    pub fn finish<W: Write>(self, cursor: &mut W) -> io::Result<usize> {
        let encoded = self.encoder.finish()?;
        let size_len = cursor.write_u32_le(encoded.len() as u32)?;
        cursor.write_all(&encoded)?;
        Ok(size_len + encoded.len())
    }
}

impl Write for ZlibWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.encoder.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.encoder.flush()
    }
}
