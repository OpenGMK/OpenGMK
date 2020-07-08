use std::{
    fs::File,
    io::{self, Read, Seek, SeekFrom, Write},
    path::Path,
};
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct FileManager {
    handles: [Option<Handle>; 32],
}

#[derive(Debug)]
pub struct Handle {
    file: File,
    content: Content,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Content {
    Binary,
    Text,
}

#[derive(Debug)]
pub enum Error {
    InvalidFile(i32),
    IOError(io::Error),
    OutOfFiles,
    WrongContent,
    Other(String),
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Self::IOError(e)
    }
}

impl From<Error> for String {
    fn from(e: Error) -> Self {
        match e {
            Error::InvalidFile(handle) => format!("invalid file handle {}", handle),
            Error::IOError(err) => format!("io error: {}", err),
            Error::OutOfFiles => "out of files".into(),
            Error::WrongContent => "invalid operation".into(),
            Error::Other(s) => s,
        }
    }
}

// Helper functions

fn read_until<P>(file: &mut File, mut end_pred: P) -> Result<Vec<u8>>
where
    P: FnMut(u8) -> bool,
{
    let mut out = Vec::new();
    for byte_maybe in file.bytes() {
        let byte = byte_maybe?;
        out.push(byte);
        if end_pred(byte) {
            break
        }
    }
    Ok(out)
}

// Returns Ok(false) on EOF
fn skip_until<P>(file: &mut File, end_pred: P) -> Result<bool>
where
    P: Fn(u8) -> bool,
{
    for byte in file.bytes() {
        if end_pred(byte?) {
            return Ok(true)
        }
    }
    Ok(false)
}

impl FileManager {
    pub fn new() -> Self {
        Self { handles: Default::default() }
    }

    pub fn open(&mut self, path: &str, content: Content, read: bool, write: bool, append: bool) -> Result<i32> {
        let file = File::with_options()
            .create(!read)
            .read(read)
            .write(write)
            .append(append)
            .truncate(content == Content::Text && write && !append)
            .open(path)?;

        match self.handles.iter_mut().enumerate().find(|(_, x)| x.is_none()) {
            Some((i, handle)) => {
                *handle = Some(Handle { file, content });
                Ok((i + 1) as i32)
            },
            None => Err(Error::OutOfFiles),
        }
    }

    pub fn close(&mut self, handle: i32, content: Content) -> Result<()> {
        if handle > 0 {
            match self.handles.get((handle - 1) as usize) {
                Some(Some(f)) => {
                    if f.content == content {
                        Ok(self.handles[(handle - 1) as usize] = None)
                    } else {
                        Err(Error::WrongContent)
                    }
                },
                _ => Err(Error::InvalidFile(handle)),
            }
        } else {
            Ok(())
        }
    }

    pub fn clear(&mut self, handle: i32) -> Result<()> {
        if handle > 0 {
            match self.handles.get_mut((handle - 1) as usize) {
                Some(Some(f)) => {
                    f.file.seek(SeekFrom::Start(0))?;
                    f.file.set_len(0)?;
                    Ok(())
                },
                _ => Err(Error::InvalidFile(handle)),
            }
        } else {
            Ok(())
        }
    }

    pub fn read_real(&mut self, handle: i32) -> Result<f64> {
        match self.handles.get_mut((handle - 1) as usize) {
            Some(Some(f)) => {
                // Read digits and at most one period or comma, plus one extra character
                let mut period_seen = false;
                let mut nonspace_seen = false;
                let mut bytes = read_until(&mut f.file, |b| {
                    // If you read spaces or dashes at the start, skip them
                    if b == 0x20 || b == 0x2d {
                        return nonspace_seen
                    }
                    nonspace_seen = true;
                    // Comma or period
                    if b == 0x2e || b == 0x2c {
                        if period_seen {
                            true
                        } else {
                            period_seen = true;
                            false
                        }
                    } else {
                        b < 0x30 || b > 0x39
                    }
                })?;
                // read_until leaves a trailing character, so remove that
                if let Some(&b) = bytes.last() {
                    if b < 0x30 || b > 0x39 {
                        // Remove the trailing character and step back if it's a CR
                        if bytes.pop().unwrap() == 0x0d {
                            f.file.seek(SeekFrom::Current(-1))?;
                        }
                    }
                }
                // Having done that, there may still be a trailing dot, so remove that
                if let Some(&b) = bytes.last() {
                    if b == 0x2e || b == 0x2c {
                        bytes.pop();
                    }
                }
                let mut text = String::from_utf8_lossy(bytes.as_slice()).replace(",", ".");
                // Remove spaces and all dashes but one
                let mut minus_seen = false;
                text.retain(|c| {
                    if c == '-' {
                        if minus_seen {
                            return false
                        }
                        minus_seen = true;
                    }
                    c != ' '
                });
                text.parse().or(Ok(0.0))
            },
            _ => Err(Error::InvalidFile(handle)),
        }
    }

    pub fn read_string(&mut self, handle: i32) -> Result<String> {
        match self.handles.get_mut((handle - 1) as usize) {
            Some(Some(f)) => {
                let mut bytes = read_until(&mut f.file, |c| c == 0x0a)?;
                if bytes.last() == Some(&0x0a) {
                    // LF
                    bytes.pop();
                    f.file.seek(SeekFrom::Current(-1))?;
                    if bytes.last() == Some(&0x0d) {
                        // CR
                        bytes.pop();
                        f.file.seek(SeekFrom::Current(-1))?;
                    }
                }
                Ok(String::from_utf8_lossy(bytes.as_slice()).into())
            },
            _ => Err(Error::InvalidFile(handle)),
        }
    }

    pub fn write_string(&mut self, handle: i32, text: &str) -> Result<()> {
        match self.handles.get_mut((handle - 1) as usize) {
            Some(Some(f)) => {
                f.file.write_all(text.as_bytes())?;
                Ok(())
            },
            _ => Err(Error::InvalidFile(handle)),
        }
    }

    pub fn skip_line(&mut self, handle: i32) -> Result<()> {
        match self.handles.get_mut((handle - 1) as usize) {
            Some(Some(f)) => {
                skip_until(&mut f.file, |c| c == 0x0a)?;
                Ok(())
            },
            _ => Err(Error::InvalidFile(handle)),
        }
    }

    pub fn is_eof(&mut self, handle: i32) -> Result<bool> {
        match self.handles.get_mut((handle - 1) as usize) {
            Some(Some(f)) => {
                let mut buf: [u8; 1] = [0];
                let last_pos = f.file.stream_position()?;
                let bytes_read = f.file.read(&mut buf)?;
                f.file.seek(SeekFrom::Start(last_pos))?;
                Ok(bytes_read == 0)
            },
            _ => Err(Error::InvalidFile(handle)),
        }
    }

    pub fn is_eoln(&mut self, handle: i32) -> Result<bool> {
        match self.handles.get_mut((handle - 1) as usize) {
            Some(Some(f)) => {
                let mut buf: [u8; 2] = [0, 0];
                let last_pos = f.file.stream_position()?;
                let bytes_read = f.file.read(&mut buf)?;
                f.file.seek(SeekFrom::Start(last_pos))?;
                Ok(bytes_read == 0 || (buf[0] == 0x0d && buf[1] == 0x0a))
            },
            _ => Err(Error::InvalidFile(handle)),
        }
    }

    pub fn read_byte(&mut self, handle: i32) -> Result<u8> {
        match self.handles.get_mut((handle - 1) as usize) {
            Some(Some(f)) => {
                let mut buf: [u8; 1] = [0];
                f.file.read_exact(&mut buf)?;
                Ok(buf[0])
            },
            _ => Err(Error::InvalidFile(handle)),
        }
    }

    pub fn write_byte(&mut self, handle: i32, byte: u8) -> Result<()> {
        match self.handles.get_mut((handle - 1) as usize) {
            Some(Some(f)) => {
                f.file.write_all(&[byte])?;
                Ok(())
            },
            _ => Err(Error::InvalidFile(handle)),
        }
    }

    pub fn tell(&mut self, handle: i32) -> Result<u64> {
        match self.handles.get_mut((handle - 1) as usize) {
            Some(Some(f)) => Ok(f.file.stream_position()?),
            _ => Err(Error::InvalidFile(handle)),
        }
    }

    pub fn seek(&mut self, handle: i32, pos: i32) -> Result<()> {
        match self.handles.get_mut((handle - 1) as usize) {
            Some(Some(f)) => {
                f.file.seek(SeekFrom::Start(pos as u64))?;
                Ok(())
            },
            _ => Err(Error::InvalidFile(handle)),
        }
    }

    pub fn size(&mut self, handle: i32) -> Result<u64> {
        match self.handles.get_mut((handle - 1) as usize) {
            Some(Some(f)) => Ok(f.file.stream_len()?),
            _ => Err(Error::InvalidFile(handle)),
        }
    }
}

impl Default for FileManager {
    fn default() -> Self {
        Self { handles: Default::default() }
    }
}

pub fn file_exists(path: &str) -> bool {
    Path::new(path).is_file()
}

pub fn rename(from: &str, to: &str) -> Result<()> {
    if !Path::new(to).exists() {
        std::fs::rename(from, to)?;
    }
    Ok(())
}

pub fn copy(from: &str, to: &str) -> Result<()> {
    std::fs::copy(from, to)?;
    Ok(())
}

pub fn dir_exists(path: &str) -> bool {
    Path::new(path).is_dir()
}

pub fn dir_create(path: &str) -> Result<()> {
    std::fs::create_dir_all(path)?;
    Ok(())
}

pub fn delete(path: &str) -> Result<()> {
    if Path::new(path).exists() {
        std::fs::remove_file(path)?;
    }
    Ok(())
}

pub struct LoadedImage {
    pub width: u32,
    pub height: u32,
    pub data: Box<[u8]>,
}

pub fn load_image(path: &str, removeback: bool, smooth: bool) -> Result<LoadedImage> {
    // looks a little silly rn but i did it like this because more formats will be added
    let mut image = {
        if !path.ends_with(".png") {
            return Err(Error::Other("Attempted to load an unsupported image file type".into()))
        }
        let decoder = png::Decoder::new(std::fs::File::open(path)?);
        let (info, mut reader) =
            decoder.read_info().map_err(|e| Error::Other(format!("PNG info decoding error: {}", e)))?;
        if info.color_type != png::ColorType::RGBA {
            return Err(Error::Other(format!("Colour format {:?} is not supported yet", info.color_type)))
        }
        if info.bit_depth != png::BitDepth::Eight {
            return Err(Error::Other(format!("Bit depth {:?} is not supported yet", info.bit_depth)))
        }
        let mut buf = vec![0; info.buffer_size()];
        reader.next_frame(&mut buf).map_err(|e| Error::Other(format!("PNG decoding error: {}", e)))?;
        LoadedImage { width: info.width, height: info.height, data: buf.into_boxed_slice() }
    };
    // processing
    let w = image.width;
    let px_to_off = |x, y| (y * w + x) as usize * 4;
    if removeback {
        // remove background colour
        let bottom_left = (image.width * (image.height - 1) * 4) as usize;
        for px in (0..image.data.len()).step_by(4) {
            if image.data[px..px + 3] == image.data[bottom_left..bottom_left + 3] {
                image.data[px + 3] = 0;
            }
        }
    }
    if smooth {
        // smooth
        for y in 0..image.height {
            for x in 0..image.width {
                // if pixel is transparent
                if image.data[px_to_off(x, y) + 3] == 0 {
                    // for all surrounding pixels
                    for y in y.saturating_sub(1)..(y + 2).min(image.height) {
                        for x in x.saturating_sub(1)..(x + 2).min(image.width) {
                            // subtract 32 if possible
                            let b = px_to_off(x, y) + 3;
                            if image.data[b] >= 32 {
                                image.data[b] -= 32;
                            }
                        }
                    }
                }
            }
        }
    }
    if removeback {
        // make lerping less ugly
        for y in 0..image.height {
            for x in 0..image.width {
                if image.data[px_to_off(x, y) + 3] == 0 {
                    let (sx, sy) = if x > 0 && image.data[px_to_off(x - 1, y) + 3] != 0 {
                        (x - 1, y)
                    } else if x < image.width - 1 && image.data[px_to_off(x + 1, y) + 3] != 0 {
                        (x + 1, y)
                    } else if y > 0 && image.data[px_to_off(x, y - 1) + 3] != 0 {
                        (x, y - 1)
                    } else if y < image.height - 1 && image.data[px_to_off(x, y + 1) + 3] != 0 {
                        (x, y + 1)
                    } else {
                        continue
                    };
                    image.data[px_to_off(x, y)] = image.data[px_to_off(sx, sy)];
                    image.data[px_to_off(x, y) + 1] = image.data[px_to_off(sx, sy) + 1];
                    image.data[px_to_off(x, y) + 2] = image.data[px_to_off(sx, sy) + 2];
                }
            }
        }
    }
    Ok(image)
}

pub fn save_image(path: &str, width: u32, height: u32, data: Box<[u8]>) -> Result<()> {
    let w = std::io::BufWriter::new(std::fs::File::create(path)?);
    let mut encoder = png::Encoder::new(w, width, height);
    encoder.set_color(png::ColorType::RGBA);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();
    writer.write_image_data(&data).unwrap();
    Ok(Default::default())
}
