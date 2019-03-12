use super::Game;
use crate::assets::{self, *};

use crate::bytes::{ReadBytes, ReadString};
use flate2::read::ZlibDecoder;
use rayon::prelude::*;

use std::error;
use std::fmt::{self, Display};
use std::io::{self, Read, Seek, SeekFrom};
use std::option::NoneError;
use std::u32;

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
            ErrorKind::ReadError => write!(f, "Error while reading input data. Likely EOF"),
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

/// Helper function for inflating zlib data. A preceding u32 indicating size is assumed.
fn inflate<I>(data: &I) -> Result<Vec<u8>, Error>
where
    I: AsRef<[u8]> + ?Sized,
{
    let slice = data.as_ref();
    let mut decoder = ZlibDecoder::new(slice);
    let mut buf: Vec<u8> = Vec::with_capacity(slice.len());
    decoder.read_to_end(&mut buf)?;
    Ok(buf)
}

impl Game {
    // TODO: functionify a lot of this.
    pub fn from_exe<I>(mut exe: I, strict: bool, verbose: bool) -> Result<Game, Error>
    where
        I: AsRef<[u8]> + AsMut<[u8]>,
    {
        let exe = exe.as_mut();

        // verify executable header
        if exe.get(0..2)? != b"MZ" {
            return Err(Error::from(ErrorKind::InvalidExeHeader));
        }

        // comfy wrapper for byteorder I/O
        let mut exe = io::Cursor::new(exe);

        // detect GameMaker version
        // TODO: support gm8.1 here later obviously
        exe.set_position(GM80_MAGIC_POS);
        if exe.read_u32_le()? != GM80_MAGIC {
            return Err(Error::from(ErrorKind::InvalidMagic));
        }

        if verbose {
            println!(
                "Detected GameMaker 8.0 magic '{}' @ {:#X}\n",
                GM80_MAGIC, GM80_MAGIC_POS
            );
        }

        // version version blahblah - I should do something with this later.
        exe.seek(SeekFrom::Current(12))?;

        // Game Settings
        let settings_len = exe.read_u32_le()? as usize;
        let pos = exe.position() as usize;
        exe.seek(SeekFrom::Current(settings_len as i64))?;
        let _settings = inflate(&exe.get_ref()[pos..pos + settings_len])?; // TODO: parse

        if verbose {
            println!(
                "Reading settings chunk... (size: {} ({} deflated))",
                _settings.len(),
                settings_len
            );
        }

        // Embedded DirectX DLL
        // we obviously don't need this, so we skip over it
        // if we're verbose logging, read the dll name (usually D3DX8.dll, but...)
        if verbose {
            let dllname = exe.read_pas_string()?;
            if verbose {
                print!("Skipping embedded DLL '{}'", dllname);
            }
        } else {
            // otherwise, skip dll name string
            let dllname_len = exe.read_u32_le()? as i64;
            exe.seek(SeekFrom::Current(dllname_len))?;
        }

        // skip embedded dll data chunk
        let dll_len = exe.read_u32_le()? as i64;
        if verbose {
            // follwup to the print above
            print!(" (size: {})\n", dll_len);
        }
        exe.seek(SeekFrom::Current(dll_len))?;

        // Asset Data Decryption
        {
            let mut swap_table = [0u8; 256];
            let mut reverse_table = [0u8; 256];

            // the swap table is squished inbetween 2 chunks of useless garbage
            let garbage1_size = exe.read_u32_le()? as i64 * 4;
            let garbage2_size = exe.read_u32_le()? as i64 * 4;
            exe.seek(SeekFrom::Current(garbage1_size))?;
            assert_eq!(exe.read(&mut swap_table)?, 256);
            exe.seek(SeekFrom::Current(garbage2_size))?;

            // fill up reverse table
            for i in 0..256 {
                reverse_table[swap_table[i] as usize] = i as u8;
            }

            // asset data length
            let len = exe.read_u32_le()? as usize;

            // simplifying for expressions below
            let pos = exe.position() as usize; // stream position
            let data = exe.get_mut(); // mutable ref for writing
            if verbose {
                println!(
                    "Decrypting asset data... (size: {}, garbage1: {}, garbage2: {})\n",
                    len, garbage1_size, garbage2_size
                );
            }

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
        // (there's 6 more u32's than it claims to contain, hence (n+6)*4)
        let garbage = ((exe.read_u32_le()? + 6) * 4) as i64;
        exe.seek(SeekFrom::Current(garbage))?;

        fn read_asset<I, T, P>(
            src: &mut io::Cursor<I>,
            name: &str,
            ver: u32,
            log: bool,
            parser: P,
        ) -> Result<Vec<Option<Box<T>>>, Error>
        where
            I: AsRef<[u8]>,
            P: Fn(&[u8]) -> Result<T, Error>,
        {
            use std::u32;

            let assets_version = src.read_u32_le()?;
            // verify_ver(name, ver, assets_version)?;
            let asset_count = src.read_u32_le()? as usize;
            if asset_count != 0 {
                if log {
                    println!(
                        "Reading {}... (ver: {:.1}, count: {})",
                        name,
                        assets_version as f64 / 100f64,
                        asset_count
                    );
                }
                let mut assets = Vec::with_capacity(asset_count);
                for _ in 0..asset_count {
                    let len = src.read_u32_le()? as usize;
                    let pos = src.position() as usize;
                    src.seek(SeekFrom::Current(len as i64))?;
                    let src_ref = src.get_ref().as_ref();

                    // Replace this once I remove flate2
                    let inflated = inflate(&src_ref[pos..pos + len])?;
                    let mut data: &[u8] = inflated.as_ref();
                    if data.len() > 4 {
                        let mut buf = [0u8; 4];
                        data.read(&mut buf)?;
                        if u32::from_le_bytes(buf) != 0 {
                            let result = parser(&data)?;
                            assets.push(Some(Box::new(result)));
                        } else {
                            assets.push(None);
                        }
                    } else {
                        assets.push(None);
                    }
                }
                Ok(assets)
            } else {
                Ok(Vec::new()) // Identical to with_capacity(0)
            }
        }

        // TODO: make this not hacky-looking..
        let p = exe.position();
        let mut exe = io::Cursor::new(exe.into_inner() as &[u8]);
        exe.set_position(p);

        // Extensions
        let _extensions = read_asset(&mut exe, "extensions", 700, verbose, |_| {
            Ok(()) // TODO: Implement
        })?;

        // Triggers
        let _triggers = read_asset(&mut exe, "triggers", 800, verbose, |_| {
            Ok(()) // TODO: Implement
        })?;

        // Constants
        let _constants = read_asset(&mut exe, "constants", 800, verbose, |_| {
            Ok(()) // TODO: Implement
        })?;

        fn get_assets<'a>(src: &mut io::Cursor<&'a [u8]>) -> io::Result<Vec<&'a [u8]>> {
            let count = src.read_u32_le()? as usize;
            let mut refs = Vec::with_capacity(count);
            for _ in 0..count {
                let len = src.read_u32_le()? as usize;
                let pos = src.position() as usize;
                src.seek(SeekFrom::Current(len as i64))?; // try.
                let data = src.get_ref();
                refs.push(&data[pos..pos + len]);
            }
            Ok(refs)
        }

        // Sounds
        assert_eq!(800, exe.read_u32_le()?);
        let sounds = get_assets(&mut exe)?
            .par_iter()
            .map(|data| inflate(&data))
            .map(|data| {
                data.and_then(|data| {
                    if &data[0..4] != &[0, 0, 0, 0] {
                        Ok(Some(Box::new(Sound::deserialize(&data[4..], strict)?)))
                    } else {
                        Ok(None)
                    }
                })
            })
            .collect::<Result<Vec<_>, Error>>()?;

        if verbose {
            for sound in sounds.iter() {
                if let Some(sound) = sound {
                    println!(" + Added sound '{}' ({})", sound.name, sound.source);
                }
            }
        }

        // Sprites
        assert_eq!(800, exe.read_u32_le()?);
        let sprites = get_assets(&mut exe)?
            .par_iter()
            .map(|data| inflate(&data))
            .map(|data| {
                data.and_then(|data| {
                    if &data[0..4] != &[0, 0, 0, 0] {
                        Ok(Some(Box::new(Sprite::deserialize(&data[4..], strict)?)))
                    } else {
                        Ok(None)
                    }
                })
            })
            .collect::<Result<Vec<_>, Error>>()?;

        if verbose {
            for sprite in sprites.iter() {
                if let Some(sprite) = sprite {
                    let framecount = if let Some(frames) = &sprite.frames {
                        frames.len()
                    } else {
                        0
                    };
                    println!(
                        " + Added sprite '{}' ({}x{}, {} frame{})",
                        sprite.name,
                        sprite.size.width,
                        sprite.size.height,
                        framecount,
                        if framecount > 1 { "s" } else { "" }
                    );
                }
            }
        }

        // Backgrounds
        assert_eq!(800, exe.read_u32_le()?);
        let backgrounds = get_assets(&mut exe)?
            .par_iter()
            .map(|data| inflate(&data))
            .map(|data| {
                data.and_then(|data| {
                    if &data[0..4] != &[0, 0, 0, 0] {
                        Ok(Some(Box::new(Background::deserialize(&data[4..], strict)?)))
                    } else {
                        Ok(None)
                    }
                })
            })
            .collect::<Result<Vec<_>, Error>>()?;

        if verbose {
            for background in backgrounds.iter() {
                if let Some(background) = background {
                    println!(
                        " + Added background '{}' ({}x{})",
                        background.name, background.size.width, background.size.height
                    );
                }
            }
        }

        // Paths
        assert_eq!(800, exe.read_u32_le()?);
        let paths = get_assets(&mut exe)?
            .par_iter()
            .map(|data| inflate(&data))
            .map(|data| {
                data.and_then(|data| {
                    if &data[0..4] != &[0, 0, 0, 0] {
                        Ok(Some(Box::new(Path::deserialize(&data[4..], strict)?)))
                    } else {
                        Ok(None)
                    }
                })
            })
            .collect::<Result<Vec<_>, Error>>()?;

        if verbose {
            use assets::path::ConnectionKind;
            for path in paths.iter() {
                if let Some(path) = path {
                    println!(
                        " + Added path '{}' ({}, {}, {} point{}, precision: {})",
                        path.name,
                        match path.connection {
                            ConnectionKind::StraightLine => "straight",
                            ConnectionKind::SmoothCurve => "smooth",
                        },
                        if path.closed { "closed" } else { "open" },
                        path.points.len(),
                        if path.points.len() > 1 { "s" } else { "" },
                        path.precision
                    );
                }
            }
        }

        // Scripts
        assert_eq!(800, exe.read_u32_le()?);
        let scripts = get_assets(&mut exe)?
            .par_iter()
            .map(|data| inflate(&data))
            .map(|data| {
                data.and_then(|data| {
                    if &data[0..4] != &[0, 0, 0, 0] {
                        Ok(Some(Box::new(Script::deserialize(&data[4..], strict)?)))
                    } else {
                        Ok(None)
                    }
                })
            })
            .collect::<Result<Vec<_>, Error>>()?;

        if verbose {
            for script in scripts.iter() {
                if let Some(script) = script {
                    println!(
                        " + Added script '{}' (source length: {})",
                        script.name,
                        script.source.len()
                    );
                }
            }
        }

        assert_eq!(800, exe.read_u32_le()?);
        let fonts = get_assets(&mut exe)?
            .par_iter()
            .map(|data| inflate(&data))
            .map(|data| {
                data.and_then(|data| {
                    if &data[0..4] != &[0, 0, 0, 0] {
                        Ok(Some(Box::new(Font::deserialize(&data[4..], false, strict)?)))
                    } else {
                        Ok(None)
                    }
                })
            })
            .collect::<Result<Vec<_>, Error>>()?;
        
        if verbose {
            for font in fonts.iter() {
                if let Some(font) = font {
                    println!(
                        " + Added font '{}' ({}, {}px{}{})",
                        font.name,
                        font.sys_name,
                        font.size,
                        if font.bold { ", bold" } else { "" },
                        if font.italic { ", italic" } else { "" }
                    );
                }
            }
        }

        Ok(Game {
            sprites,
            sounds,
            backgrounds,
            paths,
            scripts,
            fonts,
        })
    }
}
