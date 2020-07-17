use image::{ImageError, ImageFormat, Pixel, RgbaImage};
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
    ImageError(ImageError),
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Self::IOError(e)
    }
}

impl From<ImageError> for Error {
    fn from(e: ImageError) -> Self {
        Self::ImageError(e)
    }
}

impl From<Error> for String {
    fn from(e: Error) -> Self {
        match e {
            Error::InvalidFile(handle) => format!("invalid file handle {}", handle),
            Error::IOError(err) => format!("io error: {}", err),
            Error::OutOfFiles => "out of files".into(),
            Error::WrongContent => "invalid operation".into(),
            Error::ImageError(err) => format!("image error: {}", err),
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

pub fn load_image(path: &str) -> Result<RgbaImage> {
    Ok(image::open(path)?.into_rgba())
}

pub fn load_image_strip(path: &str, imgnumb: usize) -> Result<Vec<RgbaImage>> {
    let image = load_image(path.as_ref())?;
    let sprite_width = image.width() as usize / imgnumb;
    let sprite_height = image.height() as usize;
    // get pixel data for each frame
    if imgnumb > 1 {
        let mut images = Vec::with_capacity(imgnumb);
        for i in 0..imgnumb {
            let mut pixels = Vec::with_capacity(sprite_width * sprite_height * 4);
            for row in image.rows() {
                for p in row.skip(i * sprite_width).take(sprite_width) {
                    pixels.extend_from_slice(p.channels());
                }
            }
            images.push(RgbaImage::from_vec(sprite_width as _, sprite_height as _, pixels).unwrap());
        }
        Ok(images)
    } else {
        Ok(vec![image])
    }
}

pub fn save_image<P: AsRef<Path>>(path: P, width: u32, height: u32, data: Box<[u8]>) -> Result<()> {
    let image = RgbaImage::from_vec(width, height, data.into_vec()).unwrap();
    // save to png if the filename is .png otherwise bmp regardless of filename
    if path.as_ref().extension().and_then(|s| s.to_str()).map(|s| s.eq_ignore_ascii_case("png")).unwrap_or(false) {
        image.save_with_format(path, ImageFormat::Png)?;
    } else {
        image.save_with_format(path, ImageFormat::Bmp)?;
    }
    Ok(())
}
