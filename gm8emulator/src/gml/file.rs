use image::{gif::GifDecoder, AnimationDecoder, ImageError, ImageFormat, Pixel, RgbaImage};
use std::{
    fs::{File, OpenOptions},
    io::{self, BufReader, Read, Seek, SeekFrom, Write},
    path::Path,
};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct TextHandle(File,);
#[derive(Debug)]
pub struct BinaryHandle(File,);

pub enum AccessMode {
    Read,
    Write,
    Special,  // 'append' for text files, 'read-write' for binary
}

#[derive(Debug)]
pub enum Error {
    LegacyFileUnopened,
    InvalidFile(i32),
    IOError(io::Error),
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

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LegacyFileUnopened => write!(f, "file is not opened"),
            Self::InvalidFile(handle) => write!(f, "invalid file handle {}", handle),
            Self::IOError(err) => write!(f, "io error: {}", err),
            Self::ImageError(err) => write!(f, "image error: {}", err),
        }
    }
}

// Helper functions

fn read_until<P>(file: &mut File, mut end_pred: P) -> io::Result<Vec<u8>>
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
fn skip_until<P>(file: &mut File, end_pred: P) -> io::Result<bool>
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

impl TextHandle {
    pub fn open(path: &str, mode: AccessMode) -> io::Result<Self> {
        #[rustfmt::skip]
        let (read, write, append) = match mode {
            AccessMode::Read    => (true,  false, false),
            AccessMode::Write   => (false, true,  false),
            AccessMode::Special => (false, true,  true ),
        };

        Ok( Self ( OpenOptions::new()
            .create(!read)
            .read(read)
            .write(write)
            .append(append)
            .truncate(write && !append)
            .open(path)?
        ) )
    }

    pub fn read_real(&mut self) -> Result<f64> {
        Ok(read_real(&mut self.0)?)
    }

    pub fn read_string(&mut self) -> Result<Vec<u8>> {
        let mut bytes = read_until(&mut self.0, |c| c == 0x0a)?;
        if bytes.last() == Some(&0x0a) {
            // LF
            bytes.pop();
            self.0.seek(SeekFrom::Current(-1))?;
            if bytes.last() == Some(&0x0d) {
                // CR
                bytes.pop();
                self.0.seek(SeekFrom::Current(-1))?;
            }
        }
        Ok(bytes)
    }

    pub fn write_real(&mut self, real: f64) -> Result<()> {
        let text = if real.fract() == 0.0 {
            format!(" {:.0}", real)
        } else {
            format!(" {:.6}", real)
        };
        self.0.write_all(text.as_bytes())?;
        Ok(())
    }

    pub fn write_string(&mut self, text: &[u8]) -> Result<()> {
        self.0.write_all(text)?;
        Ok(())
    }

    pub fn write_newline(&mut self) -> Result<()> {
        self.0.write_all(b"\r\n")?;
        Ok(())
    }

    pub fn skip_line(&mut self) -> Result<()> {
        Ok(skip_line(&mut self.0)?)
    }

    pub fn is_eof(&mut self) -> Result<bool> {
        let mut buf: [u8; 1] = [0];
        let last_pos = self.0.stream_position()?;
        let bytes_read = self.0.read(&mut buf)?;
        self.0.seek(SeekFrom::Start(last_pos))?;
        Ok(bytes_read == 0)
    }

    pub fn is_eoln(&mut self) -> Result<bool> {
        let mut buf: [u8; 2] = [0, 0];
        let last_pos = self.0.stream_position()?;
        let bytes_read = self.0.read(&mut buf)?;
        self.0.seek(SeekFrom::Start(last_pos))?;
        Ok(bytes_read == 0 || (buf[0] == 0x0d && buf[1] == 0x0a))
    }
}

impl BinaryHandle {
    pub fn open(path: &str, mode: AccessMode) -> io::Result<Self> {
        #[rustfmt::skip]
        let (read, write) = match mode {
            AccessMode::Read    => (true,  false),
            AccessMode::Write   => (false, true ),
            AccessMode::Special => (true,  true ),
        };

        Ok( Self ( OpenOptions::new()
            .create(!read)
            .read(read)
            .write(write)
            .open(path)?
        ) )
    }

    pub fn clear(&mut self) -> Result<()> {
        self.0.seek(SeekFrom::Start(0))?;
        self.0.set_len(0)?;
        Ok(())
    }

    pub fn read_byte(&mut self) -> Result<u8> {
        let mut buf: [u8; 1] = [0];
        self.0.read_exact(&mut buf)?;
        Ok(buf[0])
    }

    pub fn write_byte(&mut self, byte: u8) -> Result<()> {
        self.0.write_all(&[byte])?;
        Ok(())
    }

    pub fn tell(&mut self) -> Result<u64> {
        Ok(self.0.stream_position()?)
    }

    pub fn seek(&mut self, pos: i32) -> Result<()> {
        self.0.seek(SeekFrom::Start(pos as u64))?;
        Ok(())
    }

    pub fn size(&mut self) -> Result<u64> {
        Ok(self.0.stream_len()?)
    }
}

pub fn read_real(f: &mut File) -> io::Result<f64> {
    // Read digits and at most one period or comma, plus one extra character
    let mut period_seen = false;
    let mut nonspace_seen = false;
    let mut bytes = read_until(f, |b| {
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
                f.seek(SeekFrom::Current(-1))?;
            }
        }
    }
    // Having done that, there may still be a trailing dot, so remove that
    if let Some(&b) = bytes.last() {
        if b == 0x2e || b == 0x2c {
            bytes.pop();
        }
    }
    // These bytes are guaranteed to be UTF-8 so no worries here
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
}

pub fn skip_line(f: &mut File) -> io::Result<()> {
    skip_until(f, |c| c == 0x0a)?;
    Ok(())
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

pub fn load_animation(path: &str, imgnumb: usize) -> Result<Vec<RgbaImage>> {
    if ImageFormat::from_path(path)? == ImageFormat::Gif {
        GifDecoder::new(BufReader::new(File::open(path)?))?
            .into_frames()
            .map(|r| r.map(|f| f.into_buffer()).map_err(Error::from))
            .collect()
    } else {
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
