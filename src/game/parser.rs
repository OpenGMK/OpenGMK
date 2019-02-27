use super::Game;
use byteorder::{ReadBytesExt, LE};
use flate2::read::ZlibDecoder;
use std::error;
use std::fmt::{self, Display};
use std::io::{self, Read, Seek, SeekFrom};
use std::option::NoneError;

const GM80_MAGIC_POS: u64 = 2000000;
const GM80_MAGIC: u32 = 1234321;

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
}

#[derive(Debug)]
pub enum ErrorKind {
    IO(io::Error),
    InvalidExeHeader,
    InvalidMagic,
    ReadError,
}

impl error::Error for Error {}
impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.kind {
            ErrorKind::IO(err) => write!(f, "IO Error: {}", err),
            ErrorKind::InvalidExeHeader => write!(f, "Invalid .exe header (missing 'MZ')"),
            ErrorKind::InvalidMagic => write!(f, "Invalid magic number (missing 1234321)"),
            ErrorKind::ReadError => write!(f, "Error while reading input data. Likely EOF."),
        }
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Error {
        Error { kind }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error {
            kind: ErrorKind::IO(err),
        }
    }
}

impl From<NoneError> for Error {
    fn from(_: NoneError) -> Error {
        Error {
            kind: ErrorKind::ReadError,
        }
    }
}

/// Convenience function for inflating zlib data. A preceding u32 indicating size is assumed.
fn inflate(data: &mut io::Cursor<Vec<u8>>, len: usize) -> Result<Vec<u8>, Error> {
    let pos = data.position() as usize;
    let slice = data.get_ref().get(pos..pos + len)?;
    let mut decoder = ZlibDecoder::new(slice);
    let mut buf: Vec<u8> = Vec::with_capacity(len);
    decoder.read_to_end(&mut buf)?;
    data.seek(SeekFrom::Current(len as i64))?;
    Ok(buf)
}

impl Game {
    pub fn from_exe(exe: Vec<u8>, verbose: bool) -> Result<(), Error> {
        // small macro so I don't have to type "if verbose {}" for every print
        // it's also easy to modify later
        macro_rules! verbose {
            ($($arg:tt)*) => {{
                if verbose {
                    print!($($arg)*);
                }
            }};
        }

        // -- begin exe reading --

        // verify executable header
        if exe.get(0..2)? != b"MZ" {
            return Err(Error::from(ErrorKind::InvalidExeHeader));
        }

        // comfy wrapper for byteorder I/O
        let mut exe = io::Cursor::new(exe);

        // detect GameMaker version
        exe.set_position(GM80_MAGIC_POS);
        if exe.read_u32::<LE>()? != GM80_MAGIC {
            // support gm8.1 here later
            return Err(Error::from(ErrorKind::InvalidMagic));
        }
        verbose!(
            "Detected GameMaker 8.0 magic '{}' @ {:#X}\n",
            GM80_MAGIC,
            GM80_MAGIC_POS
        );

        // version version blahblah - I should do something with this later.
        exe.seek(SeekFrom::Current(12))?;

        // -- settings --

        // settings data chunk
        let settings_len = exe.read_u32::<LE>()? as usize;
        verbose!("Inflating settings chunk... (size: {})\n", settings_len);
        let _settings = inflate(&mut exe, settings_len)?; // TODO: don't ignore this
        verbose!("Inflated successfully (new size: {})\n", _settings.len());

        // -- directx shared library --

        // we obviously don't need this, so we skip over it
        let dllname_len = exe.read_u32::<LE>()? as i64;
        if verbose {
            // if we're verbose logging, read the dll name (usually D3DX8.dll, but...)
            let dllname_len = dllname_len as usize;
            let mut dllname = vec![0u8; dllname_len];
            assert_eq!(exe.read(&mut dllname)?, dllname_len);
            let dllname = String::from_utf8(dllname).unwrap_or("<INVALID UTF8>".to_string());
            verbose!("Skipping embedded DLL '{}'", dllname);
        } else {
            exe.seek(SeekFrom::Current(dllname_len))?; // skip dllname string
        }

        // skip embedded dll data chunk
        let dll_len = exe.read_u32::<LE>()? as i64;
        exe.seek(SeekFrom::Current(dll_len))?;
        verbose!(" (size: {})\n", dll_len);

        // -- asset data decryption --
        {
            let mut swap_table = [0u8; 256];
            let mut reverse_table = [0u8; 256];

            // the swap table is squished inbetween 2 chunks of useless garbage
            let garbage1_size = exe.read_u32::<LE>()? as i64 * 4;
            let garbage2_size = exe.read_u32::<LE>()? as i64 * 4;
            exe.seek(SeekFrom::Current(garbage1_size))?;
            assert_eq!(exe.read(&mut swap_table)?, 256);
            exe.seek(SeekFrom::Current(garbage2_size))?;

            // fill up reverse table
            for i in 0..256 {
                reverse_table[swap_table[i] as usize] = i as u8;
            }

            // asset data length
            let len = exe.read_u32::<LE>()? as usize;

            // simplifying for expressions below
            let pos = exe.position() as usize; // stream position
            let data = exe.get_mut(); // mutable ref for writing
            verbose!(
                "Decrypting asset data... (size: {}, garbage1: {}, garbage2: {})\n",
                len,
                garbage1_size,
                garbage2_size
            );

            // decryption: first pass
            //   in reverse, data[i-1] = rev[data[i-1]] - (data[i-2] + (i - (pos+1)))
            for i in (pos..=pos + len).rev() {
                data[i - 1] = reverse_table[data[i - 1] as usize]
                    .wrapping_sub(data[i - 2].wrapping_add((i.wrapping_sub(pos + 1)) as u8));
            }

            // decryption: second pass
            //   .. it's complicated
            let mut a: u8;
            let mut b: u32;
            for i in (pos..pos + len - 1).rev() {
                b = i as u32 - swap_table[(i - pos) & 0xFF] as u32;
                if b < pos as u32 {
                    b = pos as u32;
                }
                a = data[i];
                data[i] = data[b as usize];
                data[b as usize] = a;
            }
        }

        // more garbage fields that do nothing
        let garbage = ((exe.read_u32::<LE>()? + 6) * 4) as i64;
        exe.seek(SeekFrom::Current(garbage))?;

        // -- extensions --

        let _ = exe.read_u32::<LE>()?; // data version '700'
        let extension_count = exe.read_u32::<LE>()?;

        // read extensions
        if extension_count != 0 {
            // stuff
        }

        Ok(())
    }
}
