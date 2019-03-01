use super::Game;
use crate::assets::{GMBackground, GMSound, GMSprite};
use crate::types::{BoundingBox, CollisionMap, Dimensions, Point, Version};
use crate::util::bgra2rgba;
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
    InvalidVersion(&'static str, f64, f64),
    ReadError,
    ImageParseError(String, &'static str),
}

impl error::Error for Error {}
impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.kind {
            ErrorKind::ImageParseError(name, cause) => {
                write!(f, "Failed to parse sprite '{}': {}", name, cause)
            }
            ErrorKind::IO(err) => write!(f, "IO Error: {}", err),
            ErrorKind::InvalidExeHeader => write!(f, "Invalid .exe header (missing 'MZ')"),
            ErrorKind::InvalidMagic => write!(f, "Invalid magic number (missing 1234321)"),
            ErrorKind::InvalidVersion(what, expected, got) => write!(
                f,
                "Invalid version number while reading {} (expected: {} got: {})",
                what, expected, got
            ),
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
fn inflate(data: &io::Cursor<Vec<u8>>, pos: usize, len: usize) -> Result<Vec<u8>, Error> {
    let slice = data.get_ref().get(pos..pos + len)?;
    let mut decoder = ZlibDecoder::new(slice);
    let mut buf: Vec<u8> = Vec::with_capacity(len);
    decoder.read_to_end(&mut buf)?;
    Ok(buf)
}

/// Helper trait for reading pascal-style strings.
trait ReadString: io::Read {
    /// Reads a pascal-style string from the underlying reader.
    fn read_string(&mut self) -> io::Result<String> {
        let len = self.read_u32::<LE>()? as usize;
        let mut buf = vec![0u8; len];
        self.read(&mut buf)?;
        Ok(String::from_utf8_lossy(&buf).into_owned())
    }
}

impl<R: io::Read + ?Sized> ReadString for R {}

impl Game {
    // TODO: functionify a lot of this.
    pub fn from_exe(exe: Vec<u8>, verbose: bool) -> Result<(), Error> {
        // Helper macro so I don't have to type `if verbose {}` for every print.
        // It's also easy to modify later.
        macro_rules! verbose {
            ($($arg:tt)*) => {{
                if verbose {
                    print!($($arg)*);
                }
            }};
        }

        // verify executable header
        if exe.get(0..2)? != b"MZ" {
            return Err(Error::from(ErrorKind::InvalidExeHeader));
        }

        // comfy wrapper for byteorder I/O
        let mut exe = io::Cursor::new(exe);

        // detect GameMaker version
        // TODO: support gm8.1 here later obviously
        exe.set_position(GM80_MAGIC_POS);
        if exe.read_u32::<LE>()? != GM80_MAGIC {
            return Err(Error::from(ErrorKind::InvalidMagic));
        }
        verbose!(
            "Detected GameMaker 8.0 magic '{}' @ {:#X}\n",
            GM80_MAGIC,
            GM80_MAGIC_POS
        );

        // version version blahblah - I should do something with this later.
        exe.seek(SeekFrom::Current(12))?;

        // Game Settings
        let settings_len = exe.read_u32::<LE>()? as usize;
        verbose!("Inflating settings chunk... (size: {})\n", settings_len);
        let _settings = inflate(&exe, exe.position() as usize, settings_len)?; // TODO: parse
        exe.seek(SeekFrom::Current(settings_len as i64))?;
        verbose!("Inflated successfully (new size: {})\n", _settings.len());

        // Embedded DirectX DLL
        // we obviously don't need this, so we skip over it
        // if we're verbose logging, read the dll name (usually D3DX8.dll, but...)
        if verbose {
            let dllname = exe.read_string()?;
            verbose!("Skipping embedded DLL '{}'", dllname);
        } else {
            // otherwise, skip dll name string
            let dllname_len = exe.read_u32::<LE>()? as i64;
            exe.seek(SeekFrom::Current(dllname_len))?;
        }

        // skip embedded dll data chunk
        let dll_len = exe.read_u32::<LE>()? as i64;
        verbose!(" (size: {})\n", dll_len);
        exe.seek(SeekFrom::Current(dll_len))?;

        // Asset Data Decryption
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
        // (there's 6 more u32's than it claims to contain, hence (n+6)*4)
        let garbage = ((exe.read_u32::<LE>()? + 6) * 4) as i64;
        exe.seek(SeekFrom::Current(garbage))?;

        // Extensions
        let _extensions_ver = exe.read_u32::<LE>()? as Version;
        let extension_count = exe.read_u32::<LE>()? as usize;
        if extension_count != 0 {
            // verbose!(
            //     "Reading extensions... (ver: {:.1}, count: {})\n",
            //     extensions_ver as f64 / 100.0,
            //     extension_count
            // );

            // let _char_table = [0u8; 512]; // 512 = 0x200
            // for _ in 0..extension_count {}
            // TODO: implement
        }

        // Triggers
        let triggers_ver = exe.read_u32::<LE>()? as Version;
        let trigger_count = exe.read_u32::<LE>()? as usize;
        if trigger_count != 0 {
            verbose!(
                "Reading triggers... (ver: {:.1}, count: {})\n",
                triggers_ver as f64 / 100.0,
                trigger_count
            );
            // TODO: implement
        }

        // Constants
        let constants_ver = exe.read_u32::<LE>()? as Version;
        let constant_count = exe.read_u32::<LE>()? as usize;
        if constant_count != 0 {
            verbose!(
                "Reading constants... (ver: {:.1}, count: {})\n",
                constants_ver as f64 / 100.0,
                constant_count
            );
            // TODO: implement
        }

        // Sounds
        let sounds_ver = exe.read_u32::<LE>()? as Version;
        if sounds_ver != 800 {
            return Err(Error::from(ErrorKind::InvalidVersion("sounds header", 800f64 / 100.0, sounds_ver as f64 / 100.0)));
        }
        let sound_count = exe.read_u32::<LE>()? as usize;
        let _sounds = if sound_count != 0 {
            verbose!(
                "Reading sounds... (ver: {:.1}, count: {})\n",
                sounds_ver as f64 / 100.0,
                sound_count
            );

            let mut sounds: Vec<Option<Box<GMSound>>> = Vec::with_capacity(sound_count);
            for _ in 0..sound_count {
                let len = exe.read_u32::<LE>()? as usize;
                let mut data = inflate(&exe, exe.position() as usize, len)?;
                exe.seek(SeekFrom::Current(len as i64))?;
                let sound = GMSound::from_raw(&mut data)?;
                if let Some(sound) = &sound {
                    verbose!("+ Added sound '{}' ({})\n", sound.name, sound.file_name);
                }
                sounds.push(sound);
            }
            sounds
        } else {
            Vec::with_capacity(0)
        };

        // Sprites
        let sprites_ver = exe.read_u32::<LE>()? as Version;
        if sprites_ver != 800 {
            return Err(Error::from(ErrorKind::InvalidVersion("sprites header", 800f64 / 100.0, sprites_ver as f64 / 100.0)));
        }
        let sprite_count = exe.read_u32::<LE>()? as usize;
        let _sprites = if sprite_count != 0 {
            verbose!(
                "Reading sprites... (ver: {:.1}, count: {})\n",
                sprites_ver as f64 / 100.0,
                sprite_count
            );

            let mut sprites: Vec<Option<Box<GMSprite>>> = Vec::with_capacity(sprite_count);
            for _ in 0..sprite_count {
                let len = exe.read_u32::<LE>()? as usize;
                let mut data = inflate(&exe, exe.position() as usize, len)?;
                exe.seek(SeekFrom::Current(len as i64))?;
                let sprite = GMSprite::from_raw(&mut data)?;
                if let Some(sprite) = &sprite {
                    verbose!(
                        " + Added sprite '{}' ({}x{}, {} frame{})\n",
                        sprite.name,
                        sprite.size.width,
                        sprite.size.height,
                        sprite.frame_count,
                        if sprite.frame_count > 1 { "s" } else { "" }
                    );
                }
                sprites.push(sprite);
            }

            sprites
        } else {
            Vec::with_capacity(0)
        };

        let backgrounds_ver = exe.read_u32::<LE>()? as Version;
        if backgrounds_ver != 800 {
            return Err(Error::from(ErrorKind::InvalidVersion("backgrounds header", 800f64 / 100.0, backgrounds_ver as f64 / 100.0)));
        }
        let background_count = exe.read_u32::<LE>()? as usize;
        let _backgrounds = if background_count != 0 {
            verbose!(
                "Reading backgrounds... (ver: {:.1}, count: {})\n",
                backgrounds_ver as f64 / 100.0,
                background_count
            );

            let mut backgrounds: Vec<Option<Box<GMBackground>>> = Vec::with_capacity(sprite_count);
            for _ in 0..background_count {
                let len = exe.read_u32::<LE>()? as usize;
                let mut data = inflate(&exe, exe.position() as usize, len)?;
                exe.seek(SeekFrom::Current(len as i64))?;
                let background = GMBackground::from_raw(&mut data)?;
                if let Some(background) = &background {
                    verbose!(
                        " + Added background '{}' ({}x{})\n",
                        background.name,
                        background.size.width,
                        background.size.height,
                    );
                }
                backgrounds.push(background);
            }
            backgrounds
        } else {
            Vec::with_capacity(0)
        };

        Ok(())
    }
}

impl GMBackground {
    fn from_raw<R>(src: &mut R) -> Result<Option<Box<GMBackground>>, Error>
    where
        R: AsMut<[u8]>,
    {
        let src = src.as_mut();
        let mut data = io::Cursor::new(src.as_ref());
        if data.read_u32::<LE>()? != 0 {
            let name = data.read_string()?;
            let version1 = data.read_u32::<LE>()?;
            let version2 = data.read_u32::<LE>()?;
            if version1 != 710 {
                return Err(Error::from(ErrorKind::InvalidVersion("background", 710f64 / 100.0, version1 as f64 / 100.0)));
            } else if version2 != 800 {
                return Err(Error::from(ErrorKind::InvalidVersion("background", 800f64 / 100.0, version2 as f64 / 100.0)));
            }
            let width = data.read_u32::<LE>()?;
            let height = data.read_u32::<LE>()?;
            if width > 0 && height > 0 {
                let data_len = data.read_u32::<LE>()?;

                // sanity check
                if data_len != (width * height * 4) {
                    return Err(Error::from(ErrorKind::ImageParseError(
                        name,
                        "Inconsistent pixel data length with dimensions",
                    )));
                }

                // BGRA -> RGBA
                let pos = data.position() as usize;
                let len = data_len as usize;
                data.seek(SeekFrom::Current(len as i64))?;
                let mut buf = src[pos..pos + len].to_vec();
                bgra2rgba(&mut buf);

                Ok(Some(Box::new(GMBackground {
                    name,
                    size: Dimensions { width, height },
                    data: Some(buf.into_boxed_slice()),
                })))
            } else {
                Ok(Some(Box::new(GMBackground {
                    name,
                    size: Dimensions {
                        width: 0,
                        height: 0,
                    },
                    data: None,
                })))
            }
        } else {
            Ok(None)
        }
    }
}

impl GMSound {
    fn from_raw<R>(src: &mut R) -> Result<Option<Box<GMSound>>, Error>
    where
        R: AsMut<[u8]>,
    {
        let src = src.as_mut();
        let mut data = io::Cursor::new(src.as_ref());
        if data.read_u32::<LE>()? != 0 {
            let name = data.read_string()?;
            let version = data.read_u32::<LE>()? as Version;
            if version != 800 {
                return Err(Error::from(ErrorKind::InvalidVersion("sound", 800f64 / 100.0, version as f64 / 100.0)));
            }
            let kind = data.read_u32::<LE>()?;
            let file_type = data.read_string()?;
            let file_name = data.read_string()?;
            let file_data = if data.read_u32::<LE>()? != 0 {
                let len = data.read_u32::<LE>()? as usize;
                let pos = data.position() as usize;
                data.seek(SeekFrom::Current(len as i64))?;
                Some(src[pos..pos + len].to_vec().into_boxed_slice())
            } else {
                None
            };
            let _ = data.read_u32::<LE>()?; // TODO: unused? no clue what this is
            let volume = data.read_f64::<LE>()?;
            let pan = data.read_f64::<LE>()?;
            let preload = data.read_u32::<LE>()? != 0;

            Ok(Some(Box::new(GMSound {
                name,
                kind,
                file_type,
                file_name,
                file_data,
                volume,
                pan,
                preload,
            })))
        } else {
            Ok(None)
        }
    }
}

impl GMSprite {
    fn from_raw<R>(src: &mut R) -> Result<Option<Box<GMSprite>>, Error>
    where
        R: AsMut<[u8]>,
    {
        let src = src.as_mut();
        let mut data = io::Cursor::new(src.as_ref());
        if data.read_u32::<LE>()? != 0 {
            let name = data.read_string()?;
            let version = data.read_u32::<LE>()? as Version;
            if version != 800 {
                return Err(Error::from(ErrorKind::InvalidVersion("sprite", 800f64 / 100.0, version as f64 / 100.0)));
            }
            let origin_x = data.read_u32::<LE>()?;
            let origin_y = data.read_u32::<LE>()?;
            let frame_count = data.read_u32::<LE>()?;
            let mut width = 0u32;
            let mut height = 0u32;
            let (frames, colliders, per_frame_colliders) = if frame_count != 0 {
                let mut frames: Vec<Box<[u8]>> = Vec::with_capacity(frame_count as usize);
                for _ in 0..frame_count {
                    let version = data.read_u32::<LE>()? as Version; // TODO: Hm.
                    if version != 800 {
                        return Err(Error::from(ErrorKind::InvalidVersion("frame", 800f64 / 100.0, version as f64 / 100.0)));
                    }
                    let frame_width = data.read_u32::<LE>()?;
                    let frame_height = data.read_u32::<LE>()?;

                    // sanity check 1
                    if width != 0 && height != 0 {
                        if width != frame_width || height != frame_height {
                            return Err(Error::from(ErrorKind::ImageParseError(
                                name,
                                "Inconsistent width/height across frames",
                            )));
                        }
                    } else {
                        width = frame_width;
                        height = frame_height;
                    }

                    let pixeldata_len = data.read_u32::<LE>()?;
                    let pixeldata_pixels = width * height;

                    // sanity check 2
                    if pixeldata_len != (pixeldata_pixels * 4) {
                        return Err(Error::from(ErrorKind::ImageParseError(
                            name,
                            "Inconsistent pixel data length with dimensions",
                        )));
                    }

                    // BGRA -> RGBA
                    let pos = data.position() as usize;
                    let len = pixeldata_len as usize;
                    data.seek(SeekFrom::Current(len as i64))?;
                    let mut buf = src[pos..pos + len].to_vec();
                    bgra2rgba(&mut buf);

                    // RMakeImage lol
                    frames.push(buf.into_boxed_slice());
                }

                let read_collision = |data: &mut io::Cursor<&[u8]>| -> Result<CollisionMap, Error> {
                    let version = data.read_u32::<LE>()? as Version;
                    if version != 800 {
                        return Err(Error::from(ErrorKind::InvalidVersion("collision map", 800f64 / 100.0, version as f64 / 100.0)));
                    }
                    let width = data.read_u32::<LE>()?;
                    let height = data.read_u32::<LE>()?;
                    let left = data.read_u32::<LE>()?;
                    let right = data.read_u32::<LE>()?;
                    let bottom = data.read_u32::<LE>()?;
                    let top = data.read_u32::<LE>()?;

                    let mask_size = width as usize * height as usize;
                    let mut mask = vec![0u8; mask_size];
                    let mut pos = data.position() as usize;
                    data.seek(SeekFrom::Current(4 * mask_size as i64))?;
                    for i in 0..mask_size {
                        mask[i] = src[pos];
                        pos += 4;
                    }

                    Ok(CollisionMap {
                        bounds: BoundingBox {
                            width,
                            height,
                            top,
                            bottom,
                            left,
                            right,
                        },
                        data: mask.into_boxed_slice(),
                    })
                };

                let mut colliders: Vec<CollisionMap>;
                let per_frame_colliders = data.read_u32::<LE>()? != 0;
                if per_frame_colliders {
                    colliders = Vec::with_capacity(frame_count as usize);
                    for _ in 0..frame_count {
                        colliders.push(read_collision(&mut data)?);
                    }
                } else {
                    colliders = Vec::with_capacity(1);
                    colliders.push(read_collision(&mut data)?);
                }
                (Some(frames), Some(colliders), per_frame_colliders)
            } else {
                (None, None, false)
            };

            Ok(Some(Box::new(GMSprite {
                name,
                size: Dimensions { width, height },
                origin: Point {
                    x: origin_x,
                    y: origin_y,
                },
                frame_count,
                frames,
                colliders,
                per_frame_colliders,
            })))
        } else {
            Ok(None)
        }
    }
}
