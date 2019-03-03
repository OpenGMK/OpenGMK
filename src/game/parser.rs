use super::Game;
use crate::assets::{
    GMBackground, GMFont, GMPath, GMPathKind, GMPathPoint, GMScript, GMSound, GMSprite,
};
use crate::bytes::{ReadBytes, ReadString};
use crate::types::{BoundingBox, CollisionMap, Dimensions, Point, Version};
use crate::util::bgra2rgba;
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
    InvalidVersion(String, f64, f64),
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
                "Invalid version number while reading {} (expected: {:.1}, got: {:.1})",
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

// Helper function so I don't have to type the else-case for every check.
fn verify_ver(what: &str, who: &str, expected: u32, got: u32) -> Result<(), Error> {
    if expected == got {
        Ok(())
    } else {
        let what = if who != "" {
            format!("{} '{}'", what, who)
        } else {
            format!("{} (HEADER)", what)
        };
        Err(Error::from(ErrorKind::InvalidVersion(
            what,
            expected as f64 / 100.0,
            got as f64 / 100.0,
        )))
    }
}

impl Game {
    // TODO: functionify a lot of this.
    pub fn from_exe(exe: Vec<u8>, verbose: bool) -> Result<Game, Error> {
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
        if exe.read_u32_le()? != GM80_MAGIC {
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
        let settings_len = exe.read_u32_le()? as usize;
        verbose!("Inflating settings chunk... (size: {})\n", settings_len);
        let pos = exe.position() as usize;
        exe.seek(SeekFrom::Current(settings_len as i64))?;
        let _settings = inflate(&exe.get_ref()[pos..pos + settings_len])?; // TODO: parse
        verbose!("Inflated successfully (new size: {})\n", _settings.len());

        // Embedded DirectX DLL
        // we obviously don't need this, so we skip over it
        // if we're verbose logging, read the dll name (usually D3DX8.dll, but...)
        if verbose {
            let dllname = exe.read_pas_string()?;
            verbose!("Skipping embedded DLL '{}'", dllname);
        } else {
            // otherwise, skip dll name string
            let dllname_len = exe.read_u32_le()? as i64;
            exe.seek(SeekFrom::Current(dllname_len))?;
        }

        // skip embedded dll data chunk
        let dll_len = exe.read_u32_le()? as i64;
        verbose!(" (size: {})\n", dll_len);
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
            P: Fn(io::Cursor<&mut [u8]>) -> Result<T, Error>,
        {
            let assets_version = src.read_u32_le()?;
            verify_ver(name, "", ver, assets_version)?;
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
                    let mut inflated = inflate(&src_ref[pos..pos + len])?;
                    let mut data = io::Cursor::new(&mut inflated[..]);
                    if data.read_u32_le()? != 0 {
                        let result = parser(data)?;
                        assets.push(Some(Box::new(result)));
                    } else {
                        assets.push(None);
                    }
                }
                Ok(assets)
            } else {
                Ok(Vec::new()) // Identical to with_capacity(0)
            }
        }

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

        // Sounds
        let sounds = read_asset(&mut exe, "sounds", 800, verbose, |mut data| {
            let name = data.read_pas_string()?;
            let version = data.read_u32_le()? as Version;
            verify_ver("sound", &name, 800, version)?;
            let kind = data.read_u32_le()?;
            let file_type = data.read_pas_string()?;
            let file_name = data.read_pas_string()?;
            let file_data = if data.read_u32_le()? != 0 {
                let len = data.read_u32_le()? as usize;
                let pos = data.position() as usize;
                data.seek(SeekFrom::Current(len as i64))?;
                Some(data.get_ref()[pos..pos + len].to_vec().into_boxed_slice())
            } else {
                None
            };
            let _ = data.read_u32_le()?; // TODO: unused? no clue what this is
            let volume = data.read_f64_le()?;
            let pan = data.read_f64_le()?;
            let preload = data.read_u32_le()? != 0;

            if verbose {
                println!(" + Added sound '{}' ({})", name, file_name);
            }

            Ok(GMSound {
                name,
                kind,
                file_type,
                file_name,
                file_data,
                volume,
                pan,
                preload,
            })
        })?;

        // Sprites
        let sprites = read_asset(&mut exe, "sprites", 800, verbose, |mut data| {
            let name = data.read_pas_string()?;
            let version = data.read_u32_le()? as Version;
            verify_ver("sprite", &name, 800, version)?;
            let origin_x = data.read_u32_le()?;
            let origin_y = data.read_u32_le()?;
            let frame_count = data.read_u32_le()?;
            let mut width = 0u32;
            let mut height = 0u32;
            let (frames, colliders, per_frame_colliders) = if frame_count != 0 {
                let mut frames: Vec<Box<[u8]>> = Vec::with_capacity(frame_count as usize);
                for _ in 0..frame_count {
                    let fversion = data.read_u32_le()? as Version;
                    verify_ver("frame", &name, 800, fversion)?;
                    let frame_width = data.read_u32_le()?;
                    let frame_height = data.read_u32_le()?;

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

                    let pixeldata_len = data.read_u32_le()?;
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
                    let mut buf = data.get_ref()[pos..pos + len].to_vec();
                    bgra2rgba(&mut buf);

                    // RMakeImage lol
                    frames.push(buf.into_boxed_slice());
                }

                let read_collision =
                    |data: &mut io::Cursor<&mut [u8]>| -> Result<CollisionMap, Error> {
                        let version = data.read_u32_le()? as Version;
                        verify_ver("collision map", &name, 800, version)?;
                        let width = data.read_u32_le()?;
                        let height = data.read_u32_le()?;
                        let left = data.read_u32_le()?;
                        let right = data.read_u32_le()?;
                        let bottom = data.read_u32_le()?;
                        let top = data.read_u32_le()?;

                        let mask_size = width as usize * height as usize;
                        let mut pos = data.position() as usize;
                        data.seek(SeekFrom::Current(4 * mask_size as i64))?;
                        let mut mask = vec![0u8; mask_size];
                        let src = data.get_mut();
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
                let per_frame_colliders = data.read_u32_le()? != 0;
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

            if verbose {
                println!(
                    " + Added sprite '{}' ({}x{}, {} frames)",
                    name, width, height, frame_count
                );
            }

            Ok(GMSprite {
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
            })
        })?;

        // Backgrounds
        let backgrounds = read_asset(&mut exe, "backgrounds", 800, verbose, |mut data| {
            let name = data.read_pas_string()?;
            let version1 = data.read_u32_le()?;
            let version2 = data.read_u32_le()?;
            verify_ver("background (verno. 1)", &name, 710, version1)?;
            verify_ver("background (verno. 2)", &name, 800, version2)?;
            let width = data.read_u32_le()?;
            let height = data.read_u32_le()?;
            if width > 0 && height > 0 {
                let data_len = data.read_u32_le()?;

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
                let mut buf = data.get_ref()[pos..pos + len].to_vec();
                bgra2rgba(&mut buf);

                if verbose {
                    println!(" + Added background '{}' ({}x{})", name, width, height);
                }

                Ok(GMBackground {
                    name,
                    size: Dimensions { width, height },
                    data: Some(buf.into_boxed_slice()),
                })
            } else {
                if verbose {
                    println!(" + Added background (blank) '{}' (0x0)", name);
                }

                Ok(GMBackground {
                    name,
                    size: Dimensions {
                        width: 0,
                        height: 0,
                    },
                    data: None,
                })
            }
        })?;

        // Paths
        let paths = read_asset(&mut exe, "paths", 800, verbose, |mut data| {
            let name = data.read_pas_string()?;
            let version = data.read_u32_le()?;
            verify_ver("path", &name, 530, version)?;
            let kind = if data.read_u32_le()? == 0 {
                GMPathKind::StraightLines
            } else {
                GMPathKind::SmoothCurve
            };
            let closed = data.read_u32_le()? != 0;
            let precision = data.read_u32_le()?;
            let point_count = data.read_u32_le()?;
            let mut points = Vec::new();
            for _ in 0..point_count {
                points.push(GMPathPoint {
                    x: data.read_f64_le()?,
                    y: data.read_f64_le()?,
                    speed: data.read_f64_le()?,
                });
            }

            if verbose {
                println!(
                    " + Added path '{}' ({}, {}, {} points, precision: {})",
                    name,
                    if kind == GMPathKind::StraightLines {
                        "straight"
                    } else {
                        "smooth" // Minecraft Double Smooth Stone Slab
                    },
                    if closed { "closed" } else { "open" },
                    point_count,
                    precision
                );
            }

            Ok(GMPath {
                name,
                kind,
                closed,
                precision,
                points,
            })
        })?;

        // Scripts
        let scripts = read_asset(&mut exe, "scripts", 800, verbose, |mut data| {
            let name = data.read_pas_string()?;
            let version = data.read_u32_le()?;
            verify_ver("script", &name, 800, version)?;
            let source = data.read_pas_string()?;

            if verbose {
                println!(
                    " + Added script '{}' (source length: {})",
                    name,
                    source.len()
                );
            }

            Ok(GMScript { name, source })
        })?;

        // Fonts
        let fonts = read_asset(&mut exe, "fonts", 800, verbose, |mut data| {
            let name = data.read_pas_string()?;
            let version = data.read_u32_le()?;
            verify_ver("font", &name, 800, version)?;
            let sys_name = data.read_pas_string()?;
            let size = data.read_u32_le()?;
            let bold = data.read_u32_le()? != 0;
            let italic = data.read_u32_le()? != 0;
            let range_start = data.read_u32_le()?;
            let range_end = data.read_u32_le()?;

            // TODO: 8.1 specific magic

            let dmap = [0u32; 0x600];
            let width = data.read_u32_le()?;
            let height = data.read_u32_le()?;
            let len = data.read_u32_le()? as usize;
            if width as usize * height as usize != len {
                // TODO: bad data.
            }

            // convert f64 map to RGBA data
            // Step 1) Fill entire thing with 0xFF (WHITE)
            // Step 2) Read every byte into every 4th byte (Alpha)
            let mut pixels = vec![0xFFu8; len * 4];
            let pos = data.position() as usize;
            data.seek(SeekFrom::Current(len as i64))?;
            let src = data.get_ref();
            let mut pixel_pos = 3;
            for i in pos..pos + len {
                pixels[pixel_pos] = src[i];
                pixel_pos += 4;
            }

            if verbose {
                println!(
                    " + Added font '{}' ({}, {}px{}{})",
                    name,
                    sys_name,
                    size,
                    if bold { ", bold" } else { "" },
                    if italic { ", italic" } else { "" }
                );
            }

            Ok(GMFont {
                name,
                sys_name,
                size,
                bold,
                italic,
                range_start,
                range_end,
                dmap: Box::new(dmap),
                image_size: Dimensions { width, height },
                image_data: pixels.into_boxed_slice(),
            })
        })?;

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
