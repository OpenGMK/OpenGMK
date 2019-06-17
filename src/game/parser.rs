use super::{Game, GameHelpDialog, GameVersion};
use crate::assets::{
    path::ConnectionKind, Background, Constant, Font, Object, Path, Room, Script, Sound, Sprite, Timeline, Trigger,
};
use crate::bytes::{ReadBytes, ReadString, WriteBytes};
use crate::types::Dimensions;

use flate2::read::ZlibDecoder;
use rayon::prelude::*;

use std::error;
use std::fmt::{self, Display};
use std::fs;
use std::io::{self, Read, Seek, SeekFrom};
use std::iter::once;
use std::u32;

const GM80_MAGIC_POS: u64 = 2000000;
const GM80_MAGIC: u32 = 1234321;

const GM81_MAGIC_POS: u64 = 3800004;
const GM81_MAGIC_FIELD_SIZE: u32 = 1024;
const GM81_MAGIC_1: u32 = 0xF7000000;
const GM81_MAGIC_2: u32 = 0x00140067;

pub struct ParserOptions<'a> {
    /// Optionally dump DirectX dll to out path.
    pub dump_dll: Option<&'a std::path::Path>,

    /// Enable verbose logging.
    pub log: bool,

    /// Strict version checking.
    pub strict: bool,
}

impl<'a> ParserOptions<'a> {
    pub fn new() -> ParserOptions<'a> {
        ParserOptions {
            dump_dll: None,
            log: false,
            strict: true,
        }
    }
}

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
}

#[derive(Debug)]
pub enum ErrorKind {
    IO(io::Error),
    InvalidExeHeader,
    InvalidMagic,
    InvalidVersion(String, f64, f64), // name, expected, got
}

impl error::Error for Error {}
impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.kind {
            ErrorKind::IO(err) => write!(f, "IO Error: {}", err),
            ErrorKind::InvalidExeHeader => write!(f, "Invalid .exe header (missing 'MZ')"),
            ErrorKind::InvalidMagic => write!(f, "Invalid magic number (missing 1234321)"),
            ErrorKind::InvalidVersion(n, e, g) => {
                write!(f, "Invalid version in {} (expected: {:.1}, got: {:.1})", n, e, g)
            }
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

/// Removes GameMaker 8.0 protection in-place.
fn decrypt_gm80(data: &mut io::Cursor<&mut [u8]>, options: &ParserOptions) -> io::Result<()> {
    let mut swap_table = [0u8; 256];
    let mut reverse_table = [0u8; 256];

    // the swap table is squished inbetween 2 chunks of useless garbage
    let garbage1_size = data.read_u32_le()? as i64 * 4;
    let garbage2_size = data.read_u32_le()? as i64 * 4;
    data.seek(SeekFrom::Current(garbage1_size))?;
    assert_eq!(data.read(&mut swap_table)?, 256);
    data.seek(SeekFrom::Current(garbage2_size))?;

    // fill up reverse table
    for i in 0..256 {
        reverse_table[swap_table[i] as usize] = i as u8;
    }

    // asset data length
    let len = data.read_u32_le()? as usize;

    // simplifying for expressions below
    let pos = data.position() as usize; // stream position
    let data = data.get_mut(); // mutable ref for writing
    if options.log {
        println!(
            "Decrypting asset data... (size: {}, garbage1: {}, garbage2: {})",
            len, garbage1_size, garbage2_size
        );
    }

    // decryption: first pass
    //   in reverse, data[i-1] = rev[data[i-1]] - (data[i-2] + (i - (pos+1)))
    for i in (pos..=pos + len).rev() {
        data[i - 1] =
            reverse_table[data[i - 1] as usize].wrapping_sub(data[i - 2].wrapping_add((i.wrapping_sub(pos + 1)) as u8));
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

    Ok(())
}

/// Removes GM8.1 encryption in-place.
fn decrypt_gm81(data: &mut io::Cursor<&mut [u8]>, options: &ParserOptions) -> io::Result<()> {
    // YYG's crc32 implementation
    let crc_32 = |hash_key: &Vec<u8>, crc_table: &[u32; 256]| -> u32 {
        let mut result: u32 = 0xFFFFFFFF;
        for c in hash_key.iter() {
            result = (result >> 8) ^ crc_table[((result & 0xFF) as u8 ^ c) as usize];
        }
        result
    };
    let crc_32_reflect = |mut value: u32, c: i8| -> u32 {
        let mut rvalue: u32 = 0;
        for i in 1..=c {
            if value & 1 != 0 {
                rvalue |= 1 << (c - i);
            }
            value >>= 1;
        }
        rvalue
    };

    let hash_key = format!("_MJD{}#RWK", data.read_u32_le()?);
    let hash_key_utf16: Vec<u8> = hash_key.bytes().flat_map(|c| once(c).chain(once(0))).collect();

    // generate crc table
    let mut crc_table = [0u32; 256];
    let crc_polynomial: u32 = 0x04C11DB7;
    for i in 0..256 {
        crc_table[i] = crc_32_reflect(i as u32, 8) << 24;
        for _ in 0..8 {
            crc_table[i] = (crc_table[i] << 1)
                ^ (if crc_table[i] & (1 << 31) != 0 {
                    crc_polynomial
                } else {
                    0
                });
        }
        crc_table[i] = crc_32_reflect(crc_table[i], 32);
    }

    // get our two seeds for generating xor masks
    let mut seed1 = data.read_u32_le()?;
    let mut seed2 = crc_32(&hash_key_utf16, &crc_table);

    if options.log {
        println!(
            "Decrypting GM8.1 protection (hashkey: {}, seed1: {}, seed2: {})",
            hash_key, seed1, seed2
        )
    }

    // skip to where gm81 encryption starts
    let old_position = data.position();
    data.seek(SeekFrom::Current(((seed2 & 0xFF) + 0xA) as i64))?;

    // Decrypt stream from here
    while let Ok(dword) = data.read_u32_le() {
        data.set_position(data.position() - 4);
        seed1 = (0xFFFF & seed1) * 0x9069 + (seed1 >> 16);
        seed2 = (0xFFFF & seed2) * 0x4650 + (seed2 >> 16);
        let xor_mask = (seed1 << 16) + (seed2 & 0xFFFF);
        data.write_u32_le(xor_mask ^ dword)?;
    }

    data.set_position(old_position);
    Ok(())
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

impl<'a> Game<'a> {
    // TODO: functionify a lot of this.
    pub fn from_exe<I>(mut exe: I, options: &ParserOptions) -> Result<Game<'a>, Error>
    where
        I: AsRef<[u8]> + AsMut<[u8]>,
    {
        let exe = exe.as_mut();

        // verify executable header
        if options.strict {
            if exe.get(0..2).unwrap_or(b"XX") != b"MZ" {
                return Err(Error::from(ErrorKind::InvalidExeHeader));
            }
        }

        // comfy wrapper for byteorder I/O
        let mut exe = io::Cursor::new(exe);

        // detect GameMaker version
        let mut game_ver = None;
        // check for standard 8.0 header
        exe.set_position(GM80_MAGIC_POS);
        if exe.read_u32_le()? == GM80_MAGIC {
            if options.log {
                println!("Detected GameMaker 8.0 magic (pos: {:#X})", GM80_MAGIC_POS);
            }

            game_ver = Some(GameVersion::GameMaker80);
            // 8.0-specific header TODO: strict should probably check these values.
            exe.seek(SeekFrom::Current(12))?;
        } else {
            // check for standard 8.1 header
            exe.set_position(GM81_MAGIC_POS);

            for _ in 0..GM81_MAGIC_FIELD_SIZE {
                if (exe.read_u32_le()? & 0xFF00FF00) == GM81_MAGIC_1 {
                    if (exe.read_u32_le()? & 0x00FF00FF) == GM81_MAGIC_2 {
                        if options.log {
                            println!("Detected GameMaker 8.1 magic (pos: {:#X})", exe.position() - 8);
                        }

                        game_ver = Some(GameVersion::GameMaker81);
                        decrypt_gm81(&mut exe, options)?;
                        // 8.1-specific header TODO: strict should probably check these values.
                        exe.seek(SeekFrom::Current(20))?;
                        break;
                    } else {
                        exe.set_position(exe.position() - 4);
                    }
                }
            }

            // error if no version detected
            if let None = game_ver {
                return Err(Error::from(ErrorKind::InvalidMagic));
            }
        }

        // Technically, it shouldn't make it here with a `None`.
        let game_ver = match game_ver {
            Some(ver) => ver,
            None => return Err(Error::from(ErrorKind::InvalidMagic)),
        };

        // little helper thing
        let assert_ver = |name: &str, expect, ver| -> Result<(), Error> {
            if ver == expect {
                Ok(())
            } else {
                Err(Error::from(ErrorKind::InvalidVersion(
                    name.to_string(),
                    expect as f64 / 100.0f64,
                    ver as f64 / 100.0f64,
                )))
            }
        };

        // Game Settings
        let settings_len = exe.read_u32_le()? as usize;
        let pos = exe.position() as usize;
        exe.seek(SeekFrom::Current(settings_len as i64))?;
        let _settings = inflate(&exe.get_ref()[pos..pos + settings_len])?; // TODO: parse

        if options.log {
            println!(
                "Reading settings chunk... (size: {} ({} deflated))",
                _settings.len(),
                settings_len
            );
        }

        // Embedded DirectX DLL
        // we obviously don't need this, so we skip over it
        // if we're verbose logging, read the dll name (usually D3DX8.dll, but...)
        if options.log {
            let dllname = exe.read_pas_string()?;
            if options.log {
                print!("Skipping embedded DLL '{}'", dllname);
            }
        } else {
            // otherwise, skip dll name string
            let dllname_len = exe.read_u32_le()? as i64;
            exe.seek(SeekFrom::Current(dllname_len))?;
        }

        // skip or dump embedded dll data chunk
        let dll_len = exe.read_u32_le()? as i64;
        if options.log {
            // follwup to the print aboves
            print!(" (size: {})\n", dll_len);
        }
        if let Some(out_path) = options.dump_dll {
            println!("Dumping DirectX DLL to {}...", out_path.display());
            let mut dll_data = vec![0u8; dll_len as usize];
            exe.read(&mut dll_data)?;
            fs::write(out_path, &dll_data)?;
        } else {
            exe.seek(SeekFrom::Current(dll_len))?;
        }

        // yeah
        decrypt_gm80(&mut exe, options)?;

        // Garbage field - random bytes
        let garbage_dwords = exe.read_u32_le()?;
        exe.seek(SeekFrom::Current((garbage_dwords * 4) as i64))?;
        if options.log {
            println!("Skipped {} garbage DWORDs", garbage_dwords);
        }

        // GM8 Pro flag, game ID
        let pro_flag: bool = exe.read_u32_le()? != 0;
        let game_id = exe.read_u32_le()?;
        if options.log {
            println!("Pro flag: {}", pro_flag);
            println!("Game ID: {}", game_id);
        }

        // 16 random bytes...
        exe.seek(SeekFrom::Current(16))?;

        // Rewrap data immutable.
        let prev_pos = exe.position();
        let mut exe = io::Cursor::new(exe.into_inner() as &[u8]);
        exe.set_position(prev_pos);

        fn get_asset_refs<'a>(src: &mut io::Cursor<&'a [u8]>) -> io::Result<Vec<&'a [u8]>> {
            let count = src.read_u32_le()? as usize;
            let mut refs = Vec::with_capacity(count);
            for _ in 0..count {
                let len = src.read_u32_le()? as usize;
                let pos = src.position() as usize;
                src.seek(SeekFrom::Current(len as i64))?;
                let data = src.get_ref();
                refs.push(&data[pos..pos + len]);
            }
            Ok(refs)
        }

        fn get_assets<T, F>(src: &mut io::Cursor<&[u8]>, deserializer: F) -> Result<Vec<Option<Box<T>>>, Error>
        where
            T: Send,
            F: Fn(&[u8]) -> Result<T, io::Error> + Sync,
        {
            get_asset_refs(src)?
                .par_iter()
                .map(|data| inflate(&data))
                .map(|data| {
                    data.and_then(|data| {
                        if data.get(..4).unwrap_or(&[0, 0, 0, 0]) != &[0, 0, 0, 0] {
                            Ok(Some(Box::new(deserializer(data.get(4..).unwrap_or(&[]))?)))
                        } else {
                            Ok(None)
                        }
                    })
                })
                .collect::<Result<Vec<_>, Error>>()
        }

        // TODO: Extensions
        assert_ver("extensions header", 700, exe.read_u32_le()?)?;
        let _extensions = get_assets(&mut exe, |_data| Ok(()));

        // Triggers
        assert_ver("triggers header", 800, exe.read_u32_le()?)?;
        let triggers = get_assets(&mut exe, |data| Trigger::deserialize(data, options))?;
        if options.log {
            triggers.iter().flatten().for_each(|trigger| {
                println!(
                    " + Added trigger '{}' (moment: {}, condition: {})",
                    trigger.name, trigger.moment, trigger.condition
                );
            });
        }

        // Constants
        assert_ver("constants header", 800, exe.read_u32_le()?)?;
        let constant_count = exe.read_u32_le()? as usize;
        let mut constants = Vec::with_capacity(constant_count);
        for _ in 0..constant_count {
            let name = exe.read_pas_string()?;
            let expression = exe.read_pas_string()?;
            if options.log {
                println!(" + Added constant '{}' (expression: {})", name, expression);
            }
            constants.push(Constant { name, expression });
        }

        // Sounds
        assert_ver("sounds header", 800, exe.read_u32_le()?)?;
        let sounds = get_assets(&mut exe, |data| Sound::deserialize(data, options))?;
        if options.log {
            sounds.iter().flatten().for_each(|sound| {
                println!(" + Added sound '{}' ({})", sound.name, sound.source);
            });
        }

        // Sprites
        assert_ver("sprites header", 800, exe.read_u32_le()?)?;
        let sprites = get_assets(&mut exe, |data| Sprite::deserialize(data, options))?;
        if options.log {
            sprites.iter().flatten().for_each(|sprite| {
                let framecount = sprite.frames.len();
                println!(
                    " + Added sprite '{}' ({}x{}, {} frame{})",
                    sprite.name,
                    sprite.width(),
                    sprite.height(),
                    framecount,
                    if framecount > 1 { "s" } else { "" }
                );
            });
        }

        // Backgrounds
        assert_ver("backgrounds header", 800, exe.read_u32_le()?)?;
        let backgrounds = get_assets(&mut exe, |data| Background::deserialize(data, options))?;
        if options.log {
            backgrounds.iter().flatten().for_each(|background| {
                println!(
                    " + Added background '{}' ({}x{})",
                    background.name, background.size.width, background.size.height
                );
            });
        }

        // Paths
        assert_ver("paths header", 800, exe.read_u32_le()?)?;
        let paths = get_assets(&mut exe, |data| Path::deserialize(data, options))?;
        if options.log {
            paths.iter().flatten().for_each(|path| {
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
            });
        }

        // Scripts
        assert_ver("scripts header", 800, exe.read_u32_le()?)?;
        let scripts = get_assets(&mut exe, |data| Script::deserialize(data, options))?;
        if options.log {
            scripts.iter().flatten().for_each(|script| {
                println!(
                    " + Added script '{}' (source length: {})",
                    script.name,
                    script.source.len()
                );
            });
        }

        // Fonts
        assert_ver("fonts header", 800, exe.read_u32_le()?)?;
        let fonts = get_assets(&mut exe, |data| Font::deserialize(data, &game_ver, options))?;
        if options.log {
            fonts.iter().flatten().for_each(|font| {
                println!(
                    " + Added font '{}' ({}, {}px{}{})",
                    font.name,
                    font.sys_name,
                    font.size,
                    if font.bold { ", bold" } else { "" },
                    if font.italic { ", italic" } else { "" }
                );
            });
        }

        // Timelines
        assert_ver("timelines header", 800, exe.read_u32_le()?)?;
        let timelines = get_assets(&mut exe, |data| Timeline::deserialize(data, options))?;
        if options.log {
            timelines.iter().flatten().for_each(|timeline| {
                println!(
                    " + Added timeline '{}' (moments: {})",
                    timeline.name,
                    timeline.moments.len()
                );
            });
        }

        // Objects
        assert_ver("objects header", 800, exe.read_u32_le()?)?;
        let objects = get_assets(&mut exe, |data| Object::deserialize(data, options))?;
        if options.log {
            objects.iter().flatten().for_each(|object| {
                println!(
                    " + Added object {} ({}{}{}depth {})",
                    object.name,
                    if object.solid { "solid; " } else { "" },
                    if object.visible { "visible; " } else { "" },
                    if object.persistent { "persistent; " } else { "" },
                    object.depth,
                );
            });
        }

        // Rooms
        assert_ver("rooms header", 800, exe.read_u32_le()?)?;
        let rooms = get_assets(&mut exe, |data| Room::deserialize(data, options))?;
        if options.log {
            rooms.iter().flatten().for_each(|room| {
                println!(
                    " + Added room '{}' ({}x{}, {}FPS{})",
                    room.name,
                    room.size.width,
                    room.size.height,
                    room.speed,
                    if room.persistent { ", persistent" } else { "" },
                );
            });
        }

        let last_instance_id = exe.read_i32_le()?;
        let last_tile_id = exe.read_i32_le()?;

        // TODO: Included Files
        assert_ver("included files' header", 800, exe.read_u32_le()?)?;
        let _extensions = get_assets(&mut exe, |_data| Ok(()));

        // Help Dialog
        assert_ver("help dialog", 800, exe.read_u32_le()?)?;
        let help_dialog = {
            let len = exe.read_u32_le()? as usize;
            let pos = exe.position() as usize;
            let mut data = io::Cursor::new(inflate(exe.get_ref().get(pos..pos + len).unwrap_or(&[]))?);
            let hdg = GameHelpDialog {
                bg_color: data.read_u32_le()?.into(),
                new_window: data.read_u32_le()? != 0,
                caption: data.read_pas_string()?,
                left: data.read_i32_le()?,
                top: data.read_i32_le()?,
                size: Dimensions {
                    width: data.read_u32_le()?,
                    height: data.read_u32_le()?,
                },
                border: data.read_u32_le()? != 0,
                resizable: data.read_u32_le()? != 0,
                window_on_top: data.read_u32_le()? != 0,
                freeze_game: data.read_u32_le()? != 0,
                info: data.read_pas_string()?,
            };
            if options.log {
                println!(" + Help Dialog: {:#?}", hdg);
            }
            exe.seek(SeekFrom::Current(len as i64))?;
            hdg
        };

        // Garbage... ? TODO: What is this???
        assert_ver("garbage string collection header", 500, exe.read_u32_le()?)?;
        let _gs_count = exe.read_u32_le()? as usize;
        let mut _gstrings = Vec::with_capacity(_gs_count);
        for _ in 0.._gs_count {
            _gstrings.push(exe.read_pas_string()?);
        }
        if options.log {
            println!(" + Added Garbage Strings: {:?}", _gstrings);
        }

        // Room Order
        assert_ver("room order lookup", 700, exe.read_u32_le()?)?;
        let room_order = {
            let ro_count = exe.read_u32_le()? as usize;
            let mut room_order = Vec::with_capacity(ro_count);
            for _ in 0..ro_count {
                room_order.push(exe.read_i32_le()?);
            }
            if options.log {
                println!(" + Added Room Order LUT: {:?}", room_order);
            }
            room_order
        };

        Ok(Game {
            sprites,
            sounds,
            backgrounds,
            paths,
            scripts,
            fonts,
            timelines,
            objects,
            triggers,
            constants,
            rooms,

            version: game_ver,
            help_dialog,
            last_instance_id,
            last_tile_id,
            room_order,
        })
    }
}
