use std::{
    fs::File,
    io::{self, Read, Seek, Write},
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
        }
    }
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
            Some(Some(f)) => {
                Ok(f.file.stream_position()?)
            },
            _ => Err(Error::InvalidFile(handle)),
        }
    }

    pub fn seek(&mut self, handle: i32, pos: i32) -> Result<()> {
        match self.handles.get_mut((handle - 1) as usize) {
            Some(Some(f)) => {
                f.file.seek(io::SeekFrom::Start(pos as u64))?;
                Ok(())
            },
            _ => Err(Error::InvalidFile(handle)),
        }
    }

    pub fn size(&mut self, handle: i32) -> Result<u64> {
        match self.handles.get_mut((handle - 1) as usize) {
            Some(Some(f)) => {
                Ok(f.file.stream_len()?)
            },
            _ => Err(Error::InvalidFile(handle)),
        }
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
