use crate::asset::{AssetDataError, *};
use crate::color::Color;
use crate::GameVersion;

use flate2::read::ZlibDecoder;
use minio::{ReadPrimitives, WritePrimitives};
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};

use std::{
    convert::TryInto,
    error::Error,
    fmt::{self, Display},
    fs,
    io::{self, Read, Seek, SeekFrom},
    iter::once,
    path,
};

macro_rules! log {
    ($logger: expr, $x: expr) => {
        if let Some(logger) = &$logger {
            logger($x.into());
        }
    };
    ($logger: expr, $format: expr, $($x: expr),*) => {
        if let Some(logger) = &$logger {
            logger(&format!(
                $format,
                $($x),*
            ));
        }
    };
    ($($x:expr,)*) => (log![$($x),*]); // leveraged from vec![]
}

pub struct GameAssets {
    pub sprites: Vec<Option<Box<Sprite>>>,
    pub sounds: Vec<Option<Box<Sound>>>,
    pub backgrounds: Vec<Option<Box<Background>>>,
    pub paths: Vec<Option<Box<Path>>>,
    pub scripts: Vec<Option<Box<Script>>>,
    pub fonts: Vec<Option<Box<Font>>>,
    pub timelines: Vec<Option<Box<Timeline>>>,
    pub objects: Vec<Option<Box<Object>>>,
    pub rooms: Vec<Option<Box<Room>>>,
    pub included_files: Vec<IncludedFile>,
    pub triggers: Vec<Option<Box<Trigger>>>,
    pub constants: Vec<Constant>,
    // Extensions
    pub version: GameVersion,

    pub help_dialog: GameHelpDialog,
    pub last_instance_id: i32, // TODO: type
    pub last_tile_id: i32,     // TODO: type
    pub room_order: Vec<i32>,  // TODO: type?
}

#[derive(Debug)]
pub struct GameHelpDialog {
    pub bg_color: Color,
    pub new_window: bool,
    pub caption: String,
    pub left: i32,
    pub top: i32,
    pub width: u32,
    pub height: u32,
    pub border: bool,
    pub resizable: bool,
    pub window_on_top: bool,
    pub freeze_game: bool,
    pub info: String,
}

#[derive(Debug)]
pub enum ReaderError {
    AssetError(AssetDataError),
    InvalidExeHeader,
    IO(io::Error),
    PartialUPXPacking,
    UnknownFormat,
}
impl Error for ReaderError {}
impl Display for ReaderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ReaderError::AssetError(err) => format!("asset data error: {}", err),
                ReaderError::InvalidExeHeader => format!("invalid exe header"),
                ReaderError::IO(err) => format!("io error: {}", err),
                ReaderError::PartialUPXPacking => format!("looks upx protected, can't locate headers"),
                ReaderError::UnknownFormat => format!("unknown format, could not identify file"),
            }
        )
    }
}

macro_rules! from_err {
    ($t: ident, $e: ty, $variant: ident) => {
        impl From<$e> for $t {
            fn from(err: $e) -> Self {
                $t::$variant(err)
            }
        }
    };
}

from_err!(ReaderError, AssetDataError, AssetError);
from_err!(ReaderError, io::Error, IO);

pub enum Gm81XorMethod {
    Normal,
    Sudalv,
}

const GM80_HEADER_START_POS: u64 = 0x144AC0;

/// Identifies the game version and start of gamedata header, given a data cursor.
/// Also removes any version-specific encryptions.
pub fn find_gamedata<F>(
    exe: &mut io::Cursor<&mut [u8]>,
    logger: Option<F>,
    upx_data: Option<(u32, u32)>,
) -> Result<GameVersion, ReaderError>
where
    F: Copy + Fn(&str),
{
    // Check if UPX is in use first
    match upx_data {
        Some((max_size, disk_offset)) => {
            // UPX in use, let's unpack it
            let mut unpacked = unpack_upx(exe, max_size, disk_offset, logger)?;
            log!(logger, "Successfully unpacked UPX - output is {} bytes", unpacked.len());
            let mut unpacked = io::Cursor::new(&mut *unpacked);

            // UPX unpacked, now check if this is a supported data format
            if let Some((exe_load_offset, header_start, xor_mask, add_mask, sub_mask)) = check_antidec(&mut unpacked)? {
                if logger.is_some() {
                    log!(
                        logger,
                        concat!(
                            "Found antidec2 loading sequence, decrypting with the following values:\n",
                            "exe_load_offset:0x{:X} header_start:0x{:X} ",
                            "xor_mask:0x{:X} add_mask:0x{:X} sub_mask:0x{:X}"
                        ),
                        exe_load_offset,
                        header_start,
                        xor_mask,
                        add_mask,
                        sub_mask
                    );
                }
                decrypt_antidec(exe, exe_load_offset, header_start, xor_mask, add_mask, sub_mask)?;

                // 8.0-specific header, but no point strict-checking it because antidec puts random garbage there.
                exe.seek(SeekFrom::Current(12))?;
                Ok(GameVersion::GameMaker8_0)
            } else {
                Err(ReaderError::UnknownFormat)
            }
        }
        None => {
            // Check for antidec2 protection in the base exe (so without UPX on top of it)
            if let Some((exe_load_offset, header_start, xor_mask, add_mask, sub_mask)) = check_antidec(exe)? {
                if logger.is_some() {
                    log!(
                        logger,
                        concat!(
                            "Found antidec2 loading sequence [no UPX], decrypting with the following values:\n",
                            "exe_load_offset:0x{:X} header_start:0x{:X} ",
                            "xor_mask:0x{:X} add_mask:0x{:X} sub_mask:0x{:X}",
                        ),
                        exe_load_offset,
                        header_start,
                        xor_mask,
                        add_mask,
                        sub_mask
                    );
                }
                decrypt_antidec(exe, exe_load_offset, header_start, xor_mask, add_mask, sub_mask)?;

                // 8.0-specific header, but no point strict-checking it because antidec puts random garbage there.
                exe.seek(SeekFrom::Current(12))?;
                Ok(GameVersion::GameMaker8_0)
            } else {
                // Standard formats
                if check_gm80(exe, logger)? {
                    Ok(GameVersion::GameMaker8_0)
                } else if check_gm81(exe, logger)? {
                    Ok(GameVersion::GameMaker8_1)
                } else {
                    Err(ReaderError::UnknownFormat)
                }
            }
        }
    }
}

/// Check if this is a standard gm8.0 game by looking for the loading sequence
/// If so, sets the cursor to the start of the gamedata.
fn check_gm80<F>(exe: &mut io::Cursor<&mut [u8]>, logger: Option<F>) -> Result<bool, ReaderError>
where
    F: Copy + Fn(&str),
{
    log!(logger, "Checking for standard GM8.0 format...");

    // Verify size is large enough to do the following checks - otherwise it can't be this format
    if exe.get_ref().len() < (GM80_HEADER_START_POS as usize) + 4 {
        log!(
            logger,
            "File too short for this format (0x{:X} bytes)",
            exe.get_ref().len()
        );
        return Ok(false);
    }

    // Check for the standard 8.0 loading sequence
    exe.set_position(0x000A49BE);
    let mut buf = [0u8; 8];
    exe.read_exact(&mut buf)?;
    if buf == [0x8B, 0x45, 0xF4, 0xE8, 0x2A, 0xBD, 0xFD, 0xFF] {
        // Looks like GM8.0 so let's parse the rest of loading sequence.
        // If the next byte isn't a CMP, the GM8.0 magic check has been patched out.
        let gm80_magic: Option<u32> = match exe.read_u8()? {
            0x3D => {
                let magic = exe.read_u32_le()?;
                let mut buf = [0u8; 6];
                exe.read_exact(&mut buf)?;
                if buf == [0x0F, 0x85, 0x18, 0x01, 0x00, 0x00] {
                    log!(logger, "GM8.0 magic check looks intact - value is {}", magic);
                    Some(magic)
                } else {
                    log!(logger, "GM8.0 magic check's JNZ is patched out");
                    None
                }
            }
            0x90 => {
                exe.seek(SeekFrom::Current(4))?;
                log!(logger, "GM8.0 magic check is patched out with NOP");
                None
            }
            i => {
                log!(logger, "Unknown instruction in place of magic CMP: {}", i);
                return Ok(false);
            }
        };

        // There should be a CMP for the next dword, it's usually a version header (0x320)
        let gm80_header_ver: Option<u32> = {
            exe.set_position(0x000A49E2);
            let mut buf = [0u8; 7];
            exe.read_exact(&mut buf)?;
            if buf == [0x8B, 0xC6, 0xE8, 0x07, 0xBD, 0xFD, 0xFF] {
                match exe.read_u8()? {
                    0x3D => {
                        let magic = exe.read_u32_le()?;
                        let mut buf = [0u8; 6];
                        exe.read_exact(&mut buf)?;
                        if buf == [0x0F, 0x85, 0xF5, 0x00, 0x00, 0x00] {
                            log!(logger, "GM8.0 header version check looks intact - value is {}", magic);
                            Some(magic)
                        } else {
                            println!("GM8.0 header version check's JNZ is patched out");
                            None
                        }
                    }
                    0x90 => {
                        exe.seek(SeekFrom::Current(4))?;
                        log!(logger, "GM8.0 header version check is patched out with NOP");
                        None
                    }
                    i => {
                        log!(logger, "Unknown instruction in place of magic CMP: {}", i);
                        return Ok(false);
                    }
                }
            } else {
                log!(logger, "GM8.0 header version check appears patched out");
                None
            }
        };

        // Read header start pos
        exe.set_position(GM80_HEADER_START_POS);
        let header_start = exe.read_u32_le()?;
        log!(logger, "Reading header from 0x{:X}", header_start);
        exe.set_position(header_start as u64);

        // Check the header magic numbers are what we read them as
        match gm80_magic {
            Some(n) => {
                let header1 = exe.read_u32_le()?;
                if header1 != n {
                    log!(
                        logger,
                        "Failed to read GM8.0 header: expected {} at {}, got {}",
                        n,
                        header_start,
                        header1
                    );
                    return Ok(false);
                }
            }
            None => {
                exe.seek(SeekFrom::Current(4))?;
            }
        }
        match gm80_header_ver {
            Some(n) => {
                let header2 = exe.read_u32_le()?;
                if header2 != n {
                    log!(
                        logger,
                        "Failed to read GM8.0 header: expected version {}, got {}",
                        n,
                        header2
                    );
                    return Ok(false);
                }
            }
            None => {
                exe.seek(SeekFrom::Current(4))?;
            }
        }

        exe.seek(SeekFrom::Current(8))?;
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Check if this is a standard gm8.1 game by looking for the loading sequence
/// If so, removes gm81 encryption and sets the cursor to the start of the gamedata.
fn check_gm81<F>(exe: &mut io::Cursor<&mut [u8]>, logger: Option<F>) -> Result<bool, ReaderError>
where
    F: Copy + Fn(&str),
{
    log!(logger, "Checking for standard GM8.1 format");

    // Verify size is large enough to do the following checks - otherwise it can't be this format
    if exe.get_ref().len() < 0x226D8A {
        log!(
            logger,
            "File too short for this format (0x{:X} bytes)",
            exe.get_ref().len()
        );
        return Ok(false);
    }

    // Check for the standard 8.1 loading sequence
    exe.set_position(0x00226CF3);
    let mut buf = [0u8; 8];
    exe.read_exact(&mut buf)?;
    if buf == [0xE8, 0x80, 0xF2, 0xDD, 0xFF, 0xC7, 0x45, 0xF0] {
        // Looks like GM8.1 so let's parse the rest of loading sequence.
        // Next dword is the point where we start reading the header
        let header_start = exe.read_u32_le()?;

        // Next we'll read the magic value
        exe.seek(SeekFrom::Current(125))?;
        let mut buf = [0u8; 3];
        exe.read_exact(&mut buf)?;
        let gm81_magic: Option<u32> = match buf {
            [0x81, 0x7D, 0xEC] => {
                let magic = exe.read_u32_le()?;
                if exe.read_u8()? == 0x74 {
                    log!(logger, "GM8.1 magic check looks intact - value is 0x{:X}", magic);
                    Some(magic)
                } else {
                    log!(logger, "GM8.1 magic check's JE is patched out");
                    None
                }
            }
            b => {
                log!(logger, "GM8.1 magic check's CMP is patched out ({:?})", b);
                None
            }
        };

        // Check if SUDALV's re-encryption is in use
        exe.set_position(0x0010BB83);
        let mut buf = [0u8; 8];
        exe.read_exact(&mut buf)?;
        let xor_method = match buf {
            [0x8B, 0x02, 0xC1, 0xE0, 0x10, 0x8B, 0x11, 0x81] => {
                log!(logger, "Found SUDALV re-encryption");
                Gm81XorMethod::Sudalv
            }
            _ => Gm81XorMethod::Normal,
        };

        // Search for header
        exe.set_position(header_start as u64);
        match gm81_magic {
            Some(n) => {
                log!(
                    logger,
                    "Searching for GM8.1 magic number {} from position {}",
                    n,
                    header_start
                );
                let found_header = {
                    let mut i = header_start as u64;
                    loop {
                        exe.set_position(i);
                        let val = (exe.read_u32_le()? & 0xFF00FF00) + (exe.read_u32_le()? & 0x00FF00FF);
                        if val == n {
                            break true;
                        }
                        i += 1;
                        if ((i + 8) as usize) >= exe.get_ref().len() {
                            break false;
                        }
                    }
                };
                if !found_header {
                    log!(
                        logger,
                        "Didn't find GM81 magic value (0x{:X}) before EOF, so giving up",
                        n
                    );
                    return Ok(false);
                }
            }
            None => {
                exe.seek(SeekFrom::Current(8))?;
            }
        }

        decrypt_gm81(exe, logger, xor_method)?;
        exe.seek(SeekFrom::Current(20))?;
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Removes GameMaker 8.0 protection in-place.
fn decrypt_gm80<F>(data: &mut io::Cursor<&mut [u8]>, logger: Option<F>) -> io::Result<()>
where
    F: Copy + Fn(&str),
{
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
    log!(
        logger,
        "Decrypting asset data... (size: {}, garbage1: {}, garbage2: {})",
        len,
        garbage1_size,
        garbage2_size
    );

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
fn decrypt_gm81<F>(data: &mut io::Cursor<&mut [u8]>, logger: Option<F>, xor_method: Gm81XorMethod) -> io::Result<()>
where
    F: Copy + Fn(&str),
{
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

    let sudalv_magic_point = (data.position() - 12) as u32;
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

    log!(
        logger,
        "Decrypting GM8.1 protection (hashkey: {}, seed1: {}, seed2: {})",
        hash_key,
        seed1,
        seed2
    );

    // skip to where gm81 encryption starts
    let old_position = data.position();
    data.seek(SeekFrom::Current(((seed2 & 0xFF) + 0xA) as i64))?;

    // Decrypt stream from here
    match xor_method {
        Gm81XorMethod::Normal => {
            // Normal xor generation
            while let Ok(dword) = data.read_u32_le() {
                data.set_position(data.position() - 4);
                seed1 = (0xFFFF & seed1) * 0x9069 + (seed1 >> 16);
                seed2 = (0xFFFF & seed2) * 0x4650 + (seed2 >> 16);
                let xor_mask = (seed1 << 16) + (seed2 & 0xFFFF);
                data.write_u32_le(xor_mask ^ dword)?;
            }
        }
        Gm81XorMethod::Sudalv => {
            // SUDALV xor generation
            let pos = data.position();
            data.set_position(0x20);
            let mut x20: u32 = data.read_u32_le()?;
            data.set_position(pos);

            while let Ok(dword) = data.read_u32_le() {
                data.set_position(data.position() - 4);
                seed1 = sudalv_magic(seed1, data, sudalv_magic_point, &mut x20)?;
                seed2 = sudalv_magic(seed2, data, sudalv_magic_point, &mut x20)?;
                let xor_mask = (seed1 << 16) + (seed2 & 0xFFFF);
                data.write_u32_le(xor_mask ^ dword)?;
            }
        }
    }

    data.set_position(old_position);
    Ok(())
}

fn sudalv_magic(seed: u32, data: &mut io::Cursor<&mut [u8]>, magic_point: u32, x20: &mut u32) -> io::Result<u32> {
    let t = seed & 0xFFFF;
    let start_pos = data.position();

    if *x20 == 0 {
        *x20 = magic_point;
    }

    data.set_position(*x20 as u64);
    let ecx = data.read_u32_le()?;

    if ecx == 0 {
        *x20 = magic_point;
    } else {
        *x20 -= 2;
    }

    data.set_position(start_pos);
    Ok(t.wrapping_mul(ecx & 0xFFFF).wrapping_add(seed >> 16))
}

/// Helper function for inflating zlib data. A preceding u32 indicating size is assumed.
fn inflate<I>(data: &I) -> Result<Vec<u8>, ReaderError>
where
    I: AsRef<[u8]> + ?Sized,
{
    let slice = data.as_ref();
    let mut decoder = ZlibDecoder::new(slice);
    let mut buf: Vec<u8> = Vec::with_capacity(slice.len());
    decoder.read_to_end(&mut buf)?;
    Ok(buf)
}

/// Helper function for checking whether a data stream looks like an antidec2-protected exe.
/// If so, returns the relevant vars to decrypt the data stream
/// (exe_load_offset, header_start, xor_mask, add_mask, sub_mask).
fn check_antidec(exe: &mut io::Cursor<&mut [u8]>) -> Result<Option<(u32, u32, u32, u32, u32)>, ReaderError> {
    // Verify size is large enough to do the following checks - otherwise it can't be antidec
    if exe.get_ref().len() < (GM80_HEADER_START_POS as usize) + 4 {
        return Ok(None);
    }

    // Check for the loading sequence
    exe.set_position(0x00032337);
    let mut buf = [0u8; 8];
    exe.read_exact(&mut buf)?;
    if buf == [0xE2, 0xF7, 0xC7, 0x05, 0x2E, 0x2F, 0x43, 0x00] {
        // Looks like antidec's loading sequence, so let's extract values from it
        // First, the xor byte that's used to decrypt the decryption code (yes you read that right)
        exe.seek(SeekFrom::Current(-9))?;
        let byte_xor_mask = exe.read_u8()?;
        // Convert it into a u32 mask so we can apply it easily to dwords
        let dword_xor_mask = u32::from_ne_bytes([byte_xor_mask, byte_xor_mask, byte_xor_mask, byte_xor_mask]);
        // Next, the file offset for loading gamedata bytes
        exe.set_position(0x000322A9);
        let exe_load_offset = exe.read_u32_le()? ^ dword_xor_mask;
        // Now the header_start from later in the file
        exe.set_position(GM80_HEADER_START_POS);
        let header_start = exe.read_u32_le()?;
        // xor mask
        exe.set_position(0x000322D3);
        let xor_mask = exe.read_u32_le()? ^ dword_xor_mask;
        // add mask
        exe.set_position(0x000322D8);
        let add_mask = exe.read_u32_le()? ^ dword_xor_mask;
        // sub mask
        exe.set_position(0x000322E4);
        let sub_mask = exe.read_u32_le()? ^ dword_xor_mask;
        Ok(Some((exe_load_offset, header_start, xor_mask, add_mask, sub_mask)))
    } else {
        Ok(None)
    }
}

/// Removes antidec2 encryption from gamedata, given the IVs required to do so.
/// Also sets the cursor to the start of the gamedata.
fn decrypt_antidec(
    data: &mut io::Cursor<&mut [u8]>,
    exe_load_offset: u32,
    header_start: u32,
    mut xor_mask: u32,
    mut add_mask: u32,
    sub_mask: u32,
) -> Result<(), ReaderError> {
    let game_data = data.get_mut().get_mut(exe_load_offset as usize..).unwrap(); // <- TODO
    for chunk in game_data.rchunks_exact_mut(4) {
        // TODO: fix this when const generics start existing
        let chunk: &mut [u8; 4] = chunk
            .try_into()
            .unwrap_or_else(|_| unsafe { std::hint::unreachable_unchecked() });
        let mut value = u32::from_le_bytes(*chunk);

        // apply masks, bswap
        value ^= xor_mask;
        value = value.wrapping_add(add_mask);
        value = value.swap_bytes();

        // cycle masks
        xor_mask = xor_mask.wrapping_sub(sub_mask);
        add_mask = add_mask.swap_bytes().wrapping_add(1);

        // write decrypted value
        *chunk = value.to_le_bytes();
    }

    data.set_position((exe_load_offset + header_start + 4) as u64);
    Ok(())
}

/// Unpack the bytecode of a UPX-protected exe into a separate buffer
fn unpack_upx<F>(
    data: &mut io::Cursor<&mut [u8]>,
    max_size: u32,
    disk_offset: u32,
    logger: Option<F>,
) -> Result<Vec<u8>, ReaderError>
where
    F: Copy + Fn(&str),
{
    log!(
        logger,
        "Unpacking UPX with output size {}, data starting at {}",
        max_size,
        disk_offset
    );

    // set up output vector
    let mut output: Vec<u8> = Vec::with_capacity(max_size as usize);
    output.extend_from_slice(&[0u8; 0x400]);
    data.set_position((disk_offset as u64) + 0xD); // yeah it starts 13 bytes into the section

    // helper function to pull a new bit from the mask buffer and pull a new mask if we exhaust the current one
    fn pull_new_bit(
        mask_buffer: &mut u32,
        next_bit_buffer: &mut bool,
        data: &mut io::Cursor<&mut [u8]>,
    ) -> Result<(), ReaderError> {
        let (b, w) = mask_buffer.overflowing_add(*mask_buffer);
        if b == 0 {
            let v = data.read_u32_le()?;
            let (b, w) = v.overflowing_add(v);
            *mask_buffer = b + 1;
            *next_bit_buffer = w;
            Ok(())
        } else {
            *mask_buffer = b;
            *next_bit_buffer = w;
            Ok(())
        }
    }

    // Data always starts with a bitmask, so let's pull it in and assign our buffers their IVs
    let v = data.read_u32_le()?;
    let (b, w) = v.overflowing_add(v);
    let mut mask_buffer = b + 1;
    let mut next_bit_buffer = w;

    // This value also gets stored between loops
    let mut u_var12: u32 = 0xFFFFFFFF;

    // Main loop
    loop {
        if next_bit_buffer {
            // Instruction bit 1 means to copy a byte directly from input to output.
            output.push(data.read_u8()?);
            pull_new_bit(&mut mask_buffer, &mut next_bit_buffer, data)?;
            continue;
        }

        // We pulled a 0. u_var6 is a value calculated from the instruction bits following a 0.
        let mut u_var6: u32 = 1;
        loop {
            // Pull a bit and push it into u_var6
            pull_new_bit(&mut mask_buffer, &mut next_bit_buffer, data)?;
            u_var6 <<= 1;
            u_var6 |= next_bit_buffer as u32;

            // Next bit is an instruction bit. If it's 1, it means stop reading.
            pull_new_bit(&mut mask_buffer, &mut next_bit_buffer, data)?;
            if next_bit_buffer {
                break;
            }
            // Otherwise, it means pull another bit and push it into u_var6
            pull_new_bit(&mut mask_buffer, &mut next_bit_buffer, data)?;
            u_var6 -= 1; // Decrements here, not sure why.
            u_var6 <<= 1;
            u_var6 |= next_bit_buffer as u32;
        }

        // The minimum possible value of u_var6 is 2, since it starts at 1, is immediately shifted, then
        // has a bit added to it. I guess this check is for whether that's the case (and the bit was 0)?
        if u_var6 < 3 {
            // Just grabs a new instruction-bit normally.
            pull_new_bit(&mut mask_buffer, &mut next_bit_buffer, data)?;
        } else {
            // This is weird because it copies a byte into AL then xors all of EAX, which has a dead value left in it.
            u_var12 = ((((u_var6 - 3) << 8) & 0xFFFFFF00) + (data.read_u8()? as u32 & 0xFF)) ^ 0xFFFFFFFF;
            if u_var12 == 0 {
                break; // This is the only exit point
            }
            // Next instruction bit is pulled from the byte we read above, then shifted out of that byte
            next_bit_buffer = (u_var12 & 1) != 0;
            u_var12 = ((u_var12 as i32) >> 1) as u32;
        }

        // next, we're going to calculate the number of bytes to copy from somewhere else in the output vec.
        let mut byte_count: u32 = 0;
        let mut do_push_bit: bool = true;
        if !next_bit_buffer {
            // Instruction to start byte_count at 1, then pull bits into it.
            byte_count = 1;
            pull_new_bit(&mut mask_buffer, &mut next_bit_buffer, data)?;
            if !next_bit_buffer {
                // Loop pulling bits into byte_count
                loop {
                    // Pull bit, push it into byte_count
                    pull_new_bit(&mut mask_buffer, &mut next_bit_buffer, data)?;
                    byte_count <<= 1;
                    byte_count += next_bit_buffer as u32;
                    // Instruction bit - 1 means stop
                    pull_new_bit(&mut mask_buffer, &mut next_bit_buffer, data)?;
                    if next_bit_buffer {
                        break;
                    }
                }
                // Add 2 to the byte count for some reason?
                byte_count += 2;
                do_push_bit = false;
            }
        }
        if do_push_bit {
            // We didn't do the loop above, so instead we just pull one bit into byte_count
            pull_new_bit(&mut mask_buffer, &mut next_bit_buffer, data)?;
            byte_count <<= 1;
            byte_count += next_bit_buffer as u32;
        }

        // Again, add 2 to the byte count for some reason.
        byte_count += 2;
        if u_var12 < 0xfffffb00 {
            // Add another 1 only if our cursor is more than 1280 bytes behind the head. Not sure why.
            byte_count += 1;
        }

        // Cursor into the output vector. We're going to read some bytes from here and push them again.
        let mut cursor = (output.len() as u32).wrapping_add(u_var12) as usize;
        // Do the byte-copying.
        for _ in 0..byte_count {
            output.push(output[cursor]);
            cursor += 1;
        }

        // Finally, pull a new instruction bit and start the loop again.
        pull_new_bit(&mut mask_buffer, &mut next_bit_buffer, data)?;
    }

    Ok(output)
}

fn find_rsrc_icon(data: &mut io::Cursor<&mut [u8]>) -> Result<Option<(u32, u32)>, ReaderError> {
    // top level header
    let rsrc_base = data.position();
    data.seek(SeekFrom::Current(12))?;
    let name_count = data.read_u16_le()?;
    let id_count = data.read_u16_le()?;
    // skip over any names in the top-level
    data.seek(SeekFrom::Current((name_count as i64) * 8))?;
    // read IDs until we find 3 (RT_ICON)
    for _ in 0..id_count {
        let id = data.read_u32_le()?;
        let offset = data.read_u32_le()? & 0x7FFFFFFF; // high bit is 1
        if id == 3 {
            // Go down to next layer
            data.set_position((offset as u64) + rsrc_base + 14);
            let leaf_count = data.read_u16_le()?;
            if leaf_count == 0 {
                // No leaves under RT_ICON, so no icon
                return Ok(None);
            }
            // And another layer...
            data.seek(SeekFrom::Current(4))?;
            let language_offset = data.read_u32_le()? & 0x7FFFFFFF; // high bit is 1
            data.set_position((language_offset as u64) + rsrc_base + 20);
            let leaf = data.read_u32_le()?;
            // Finally we get to the leaf, which has a pointer to our icon data + size
            data.set_position((leaf as u64) + rsrc_base);
            let rva = data.read_u32_le()?;
            let size = data.read_u32_le()?;
            return Ok(Some((rva, size)));
        }
    }
    // No RT_ICON group, so no icon
    Ok(None)
}

pub struct PESection {
    pub virtual_size: u32,
    pub virtual_address: u32,
    pub disk_size: u32,
    pub disk_address: u32,
}

pub fn from_exe<I, F>(
    mut exe: I,
    strict: bool,
    logger: Option<F>,
    dump_dll: Option<&path::Path>,
) -> Result<GameAssets, ReaderError>
where
    F: Copy + Fn(&str),
    I: AsRef<[u8]> + AsMut<[u8]>,
{
    let exe = exe.as_mut();

    // comfy wrapper for byteorder I/O
    let mut exe = io::Cursor::new(exe);

    // verify executable header
    // Windows EXE must always start with "MZ"
    if exe.get_ref().get(0..2).unwrap_or(b"XX") != b"MZ" {
        return Err(ReaderError::InvalidExeHeader);
    }
    // Dword at 0x3C indicates the start of the PE header
    exe.set_position(0x3C);
    let pe_header_loc = exe.read_u32_le()? as usize;
    // PE header must begin with PE\0\0, then 0x14C which means i386.
    match exe.get_ref().get(pe_header_loc..(pe_header_loc + 6)) {
        Some(b"PE\0\0\x4C\x01") => (),
        _ => return Err(ReaderError::InvalidExeHeader),
    }
    // Read number of sections
    exe.set_position((pe_header_loc + 6) as u64);
    let section_count = exe.read_u16_le()?;
    // Read length of optional header
    exe.seek(SeekFrom::Current(12))?;
    let optional_len = exe.read_u16_le()?;
    // Skip over PE characteristics (2 bytes) + optional header
    exe.seek(SeekFrom::Current((optional_len as i64) + 2))?;

    // Read all sections, noting these 3 values from certain sections if they exist
    let mut upx0_virtual_len: Option<u32> = None;
    let mut upx1_data: Option<(u32, u32)> = None; // virtual size, position on disk
    let mut rsrc_ico_data: Option<(u32, u32)> = None;

    let mut sections: Vec<PESection> = Vec::with_capacity(section_count as usize);

    for _ in 0..section_count {
        let mut sect_name = [0u8; 8];
        exe.read_exact(&mut sect_name)?;

        let virtual_size = exe.read_u32_le()?;
        let virtual_address = exe.read_u32_le()?;
        let disk_size = exe.read_u32_le()?;
        let disk_address = exe.read_u32_le()?;
        exe.seek(SeekFrom::Current(16))?;

        // See if this is a section we want to do something with
        match sect_name {
            [0x55, 0x50, 0x58, 0x30, 0x00, 0x00, 0x00, 0x00] => {
                // UPX0 section
                upx0_virtual_len = Some(virtual_size);
                log!(logger, "UPX0 section found, virtual len: {}", virtual_size);
            }
            [0x55, 0x50, 0x58, 0x31, 0x00, 0x00, 0x00, 0x00] => {
                // UPX1 section
                upx1_data = Some((virtual_size, disk_address));
                log!(logger, "UPX1 section found, virtual len: {}", virtual_size);
            }
            [0x2E, 0x72, 0x73, 0x72, 0x63, 0x00, 0x00, 0x00] => {
                // .rsrc section
                log!(logger, "Reading .rsrc");
                let temp_pos = exe.position();
                exe.set_position(disk_address as u64);
                rsrc_ico_data = find_rsrc_icon(&mut exe)?;
                exe.set_position(temp_pos);
            }
            _ => {}
        }
        sections.push(PESection {
            virtual_size,
            virtual_address,
            disk_size,
            disk_address,
        })
    }

    // Find icon data if we can
    let mut icon_data: Option<Vec<u8>> = None;
    match rsrc_ico_data {
        Some((rva, size)) => {
            for section in sections {
                if rva >= section.virtual_address && (rva + size) < (section.virtual_address + section.virtual_size) {
                    // icon data is in this section
                    let offset_on_disk = rva - section.virtual_address;
                    let icon_location = section.disk_address + offset_on_disk;
                    exe.set_position(icon_location as u64);
                    let mut data = vec![0u8; size as usize];
                    exe.read_exact(&mut data)?;
                    icon_data = Some(data);
                    break;
                }
            }
        }
        None => {}
    }
    match icon_data {
        Some(v) => log!(logger, "Loaded icon data ({} bytes)", v.len()),
        None => log!(logger, "Couldn't find an icon"),
    }

    // Decide if UPX is in use based on PE section names
    // This is None if there is no UPX, obviously, otherwise it's (max_size, offset_on_disk)
    let upx_data: Option<(u32, u32)> = match upx0_virtual_len {
        Some(len0) => match upx1_data {
            Some((len1, offset)) => Some((len0 + len1, offset)),
            None => None,
        },
        None => None,
    };

    // Identify the game version in use and locate the gamedata header
    let game_ver = find_gamedata(&mut exe, logger, upx_data)?;

    // little helper thing
    macro_rules! assert_ver {
        ($name: literal, $expect: expr, $ver: expr) => {{
            let expected = $expect;
            let got = $ver;
            if strict {
                if got == expected {
                    Ok(())
                } else {
                    Err(ReaderError::AssetError(AssetDataError::VersionError { expected, got }))
                }
            } else {
                Ok(())
            }
        }};
    }

    // Game Settings
    let settings_len = exe.read_u32_le()? as usize;
    let pos = exe.position() as usize;
    exe.seek(SeekFrom::Current(settings_len as i64))?;
    let _settings = inflate(&exe.get_ref()[pos..pos + settings_len])?; // TODO: parse

    log!(
        logger,
        "Reading settings chunk... (size: {} ({} deflated))",
        _settings.len(),
        settings_len
    );

    // Embedded DirectX DLL
    // we obviously don't need this, so we skip over it
    // if we're verbose logging, read the dll name (usually D3DX8.dll, but...)
    if logger.is_some() {
        let dllname = exe.read_pas_string()?;
        log!(logger, "Skipping embedded DLL '{}'", dllname);
    } else {
        // otherwise, skip dll name string
        let dllname_len = exe.read_u32_le()? as i64;
        exe.seek(SeekFrom::Current(dllname_len))?;
    }

    // skip or dump embedded dll data chunk
    let dll_len = exe.read_u32_le()? as i64;
    if let Some(out_path) = dump_dll {
        println!("Dumping DirectX DLL to {}...", out_path.display());
        let mut dll_data = vec![0u8; dll_len as usize];
        exe.read(&mut dll_data)?;
        fs::write(out_path, &dll_data)?;
    } else {
        exe.seek(SeekFrom::Current(dll_len))?;
    }

    // yeah
    decrypt_gm80(&mut exe, logger)?;

    // Garbage field - random bytes
    let garbage_dwords = exe.read_u32_le()?;
    exe.seek(SeekFrom::Current((garbage_dwords * 4) as i64))?;
    log!(logger, "Skipped {} garbage DWORDs", garbage_dwords);

    // GM8 Pro flag, game ID
    let pro_flag: bool = exe.read_u32_le()? != 0;
    let game_id = exe.read_u32_le()?;
    log!(logger, "Pro flag: {}", pro_flag);
    log!(logger, "Game ID: {}", game_id);

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

    fn get_assets<T, F>(src: &mut io::Cursor<&[u8]>, deserializer: F) -> Result<Vec<Option<Box<T>>>, ReaderError>
    where
        T: Send,
        F: Fn(&[u8]) -> Result<T, AssetDataError> + Sync,
    {
        get_asset_refs(src)?
            .par_iter()
            .map(|chunk| {
                inflate(&chunk).and_then(|data| {
                    // If the first u32 is 0 then the underlying data doesn't exist (is a None asset).
                    match data.get(..4) {
                        Some(&[0, 0, 0, 0]) => Ok(None),
                        Some(_) => Ok(Some(Box::new(deserializer(
                            data.get(4..).unwrap_or_else(|| unreachable!()),
                        )?))),
                        None => Err(ReaderError::AssetError(AssetDataError::MalformedData)),
                    }
                })
            })
            .collect::<Result<Vec<_>, ReaderError>>()
    }

    // stuff to pass to asset deserializers
    let a_strict = strict;
    let a_version = game_ver;

    // TODO: Extensions
    assert_ver!("extensions header", 700, exe.read_u32_le()?)?;
    let _extensions = get_assets(&mut exe, |_data| Ok(()));

    // Triggers
    assert_ver!("triggers header", 800, exe.read_u32_le()?)?;
    let triggers = get_assets(&mut exe, |data| Trigger::deserialize(data, a_strict, a_version))?;
    if logger.is_some() {
        triggers.iter().flatten().for_each(|trigger| {
            log!(
                logger,
                " + Added trigger '{}' (moment: {}, condition: {})",
                trigger.name,
                trigger.moment,
                trigger.condition
            );
        });
    }

    // Constants
    assert_ver!("constants header", 800, exe.read_u32_le()?)?;
    let constant_count = exe.read_u32_le()? as usize;
    let mut constants = Vec::with_capacity(constant_count);
    for _ in 0..constant_count {
        let name = exe.read_pas_string()?;
        let expression = exe.read_pas_string()?;
        log!(logger, " + Added constant '{}' (expression: {})", name, expression);
        constants.push(Constant { name, expression });
    }

    // Sounds
    assert_ver!("sounds header", 800, exe.read_u32_le()?)?;
    let sounds = get_assets(&mut exe, |data| Sound::deserialize(data, a_strict, a_version))?;
    if logger.is_some() {
        sounds.iter().flatten().for_each(|sound| {
            log!(logger, " + Added sound '{}' ({})", sound.name, sound.source);
        });
    }

    // Sprites
    assert_ver!("sprites header", 800, exe.read_u32_le()?)?;
    let sprites = get_assets(&mut exe, |data| Sprite::deserialize(data, a_strict, a_version))?;
    if logger.is_some() {
        sprites.iter().flatten().for_each(|sprite| {
            let framecount = sprite.frames.len();
            let (width, height) = match sprite.frames.first() {
                Some(frame) => (frame.width, frame.height),
                None => (0, 0),
            };
            log!(
                logger,
                " + Added sprite '{}' ({}x{}, {} frame{})",
                sprite.name,
                width,
                height,
                framecount,
                if framecount > 1 { "s" } else { "" }
            );
        });
    }

    // Backgrounds
    assert_ver!("backgrounds header", 800, exe.read_u32_le()?)?;
    let backgrounds = get_assets(&mut exe, |data| Background::deserialize(data, a_strict, a_version))?;
    if logger.is_some() {
        backgrounds.iter().flatten().for_each(|background| {
            log!(
                logger,
                " + Added background '{}' ({}x{})",
                background.name,
                background.width,
                background.height
            );
        });
    }

    // Paths
    assert_ver!("paths header", 800, exe.read_u32_le()?)?;
    let paths = get_assets(&mut exe, |data| Path::deserialize(data, a_strict, a_version))?;
    if logger.is_some() {
        use crate::asset::path::ConnectionKind;

        paths.iter().flatten().for_each(|path| {
            log!(
                logger,
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
    assert_ver!("scripts header", 800, exe.read_u32_le()?)?;
    let scripts = get_assets(&mut exe, |data| Script::deserialize(data, a_strict, a_version))?;
    if logger.is_some() {
        scripts.iter().flatten().for_each(|script| {
            log!(logger, " + Added script '{}'", script.name);
        });
    }

    // Fonts
    assert_ver!("fonts header", 800, exe.read_u32_le()?)?;
    let fonts = get_assets(&mut exe, |data| Font::deserialize(data, a_strict, a_version))?;
    if logger.is_some() {
        fonts.iter().flatten().for_each(|font| {
            log!(
                logger,
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
    assert_ver!("timelines header", 800, exe.read_u32_le()?)?;
    let timelines = get_assets(&mut exe, |data| Timeline::deserialize(data, a_strict, a_version))?;
    if logger.is_some() {
        timelines.iter().flatten().for_each(|timeline| {
            log!(
                logger,
                " + Added timeline '{}' (moments: {})",
                timeline.name,
                timeline.moments.len()
            );
        });
    }

    // Objects
    assert_ver!("objects header", 800, exe.read_u32_le()?)?;
    let objects = get_assets(&mut exe, |data| Object::deserialize(data, a_strict, a_version))?;
    if logger.is_some() {
        objects.iter().flatten().for_each(|object| {
            log!(
                logger,
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
    assert_ver!("rooms header", 800, exe.read_u32_le()?)?;
    let rooms = get_assets(&mut exe, |data| Room::deserialize(data, a_strict, a_version))?;
    if logger.is_some() {
        rooms.iter().flatten().for_each(|room| {
            log!(
                logger,
                " + Added room '{}' ({}x{}, {}FPS{})",
                room.name,
                room.width,
                room.height,
                room.speed,
                if room.persistent { ", persistent" } else { "" },
            );
        });
    }

    let last_instance_id = exe.read_i32_le()?;
    let last_tile_id = exe.read_i32_le()?;

    // Included Files
    assert_ver!("included files header", 800, exe.read_u32_le()?)?;
    let included_files = get_asset_refs(&mut exe)?
        .iter()
        .map(|chunk| {
            // AssetDataError -> ReaderError
            inflate(chunk).and_then(|data| IncludedFile::deserialize(data, a_strict, a_version).map_err(|e| e.into()))
        })
        .collect::<Result<Vec<_>, _>>()?;
    if logger.is_some() {
        use crate::asset::includedfile::ExportSetting;
        for file in &included_files {
            log!(
                logger,
                " + Added included file '{}' (len: {}, export mode: {})",
                file.file_name,
                file.source_length,
                match &file.export_settings {
                    ExportSetting::NoExport => "no export".into(),
                    ExportSetting::TempFolder => "temp folder".into(),
                    ExportSetting::GameFolder => "game folder".into(),
                    ExportSetting::CustomFolder(p) => format!("custom path: '{}'", p),
                }
            );
        }
    }

    // Help Dialog
    assert_ver!("help dialog", 800, exe.read_u32_le()?)?;
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
            width: data.read_u32_le()?,
            height: data.read_u32_le()?,
            border: data.read_u32_le()? != 0,
            resizable: data.read_u32_le()? != 0,
            window_on_top: data.read_u32_le()? != 0,
            freeze_game: data.read_u32_le()? != 0,
            info: data.read_pas_string()?,
        };
        log!(logger, " + Help Dialog: {:#?}", hdg);
        exe.seek(SeekFrom::Current(len as i64))?;
        hdg
    };

    // Action library initialization code. We don't need to store this.
    assert_ver!("action library initialization code header", 500, exe.read_u32_le()?)?;
    let str_count = exe.read_u32_le()? as usize;
    for _ in 0..str_count {
        let str_len = exe.read_u32_le()?;
        exe.seek(SeekFrom::Current(str_len as i64))?;
    }
    log!(logger, " + Skipped {} action library initialization strings", str_count);

    // Room Order
    assert_ver!("room order lookup", 700, exe.read_u32_le()?)?;
    let room_order = {
        let ro_count = exe.read_u32_le()? as usize;
        let mut room_order = Vec::with_capacity(ro_count);
        for _ in 0..ro_count {
            room_order.push(exe.read_i32_le()?);
        }
        log!(logger, " + Added Room Order LUT: {:?}", room_order);

        room_order
    };

    Ok(GameAssets {
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
        included_files,

        version: game_ver,
        help_dialog,
        last_instance_id,
        last_tile_id,
        room_order,
    })
}
