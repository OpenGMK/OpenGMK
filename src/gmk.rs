use minio::WritePrimitives;
use std::io;

pub fn write_header<W>(writer: &mut W) -> io::Result<usize>
where
    W: io::Write,
{
    let result = writer.write_u32_le(1234321)?;
    // todo
    Ok(result)
}
