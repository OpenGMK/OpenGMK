use super::{Game, GameHelpDialog, GameVersion};
use crate::assets::{
    path::ConnectionKind, Background, Constant, Font, Object, Path, Room, Script, Sound, Sprite, Timeline, Trigger,
};
use crate::bytes::{ReadBytes, ReadString, WriteBytes};
use crate::types::Dimensions;

use flate2::read::ZlibDecoder;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};

use std::convert::TryInto;
use std::error;
use std::fmt::{self, Display};
use std::fs;
use std::io::{self, Read, Seek, SeekFrom};
use std::iter::once;
use std::u32;

const GM80_HEADER_START_POS: u64 = 0x144AC0;

//const UPX_BYTES_START_POS: u64 = 0x20D;

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
    UnknownFormat,
    InvalidVersion(String, f64, f64), // name, expected, got
    PartialUPXPacking,
}

impl error::Error for Error {}
impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.kind {
            ErrorKind::IO(err) => write!(f, "IO Error: {}", err),
            ErrorKind::InvalidExeHeader => write!(f, "Invalid EXE or PE header"),
            ErrorKind::UnknownFormat => write!(f, "Unknown data format - no game version detected"),
            ErrorKind::InvalidVersion(n, e, g) => {
                write!(f, "Invalid version in {} (expected: {:.1}, got: {:.1})", n, e, g)
            }
            ErrorKind::PartialUPXPacking => write!(
                f,
                "Invalid packing: exe header is signed by UPX, but could not find packed data"
            ),
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

/// Unpack the bytecode of a UPX-protected exe into a separate buffer
fn unpack_upx(data: &mut io::Cursor<&mut [u8]>, options: &ParserOptions) -> Result<Vec<u8>, Error> {
    // Locate PE header and read code entry point
    // Note: I am not sure how to read the exact length of the data section, but UPX's entry point is
    // always after the area it extracts to, so it should always suffice as an output size.
    data.set_position(0x3C);
    let pe_header = data.read_u8()?;
    data.set_position(pe_header as u64 + 40);
    let entry_point = data.read_u32_le()?;
    data.seek(SeekFrom::Current(361))?;

    if options.log {
        println!("UPX entry point: 0x{:X}", entry_point);
    }

    // set up output vector
    let mut output: Vec<u8> = Vec::with_capacity(entry_point as usize); // = vec![0u8; entry_point as usize];
    output.extend_from_slice(&[0u8; 0x400]);

    // helper function to pull a new bit from the mask buffer and pull a new mask if we exhaust the current one
    fn pull_new_bit(
        mask_buffer: &mut u32,
        next_bit_buffer: &mut bool,
        data: &mut io::Cursor<&mut [u8]>,
    ) -> Result<(), Error> {
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

    output.shrink_to_fit();
    Ok(output)
}

/// Helper function for checking whether a data stream looks like an antidec2-protected exe.
/// If so, returns the relevant vars to decrypt the data stream:
/// (exe_load_offset, header_start, xor_mask, add_mask, sub_mask).
fn check_antidec(exe: &mut io::Cursor<&mut [u8]>) -> Result<Option<(u32, u32, u32, u32, u32)>, Error> {
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

/// Check if this is a standard gm8.0 game by looking for the loading sequence
/// If so, sets the cursor to the start of the gamedata.
fn check_gm80(exe: &mut io::Cursor<&mut [u8]>, options: &ParserOptions) -> Result<bool, Error> {
    if options.log {
        println!("Checking for standard GM8.0 format");
    }

    // Verify size is large enough to do the following checks - otherwise it can't be this format
    if exe.get_ref().len() < (GM80_HEADER_START_POS as usize) + 4 {
        if options.log {
            println!("File too short for this format (0x{:X} bytes)", exe.get_ref().len());
        }
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
                    if options.log {
                        println!("GM8.0 magic check looks intact - value is {}", magic);
                    }
                    Some(magic)
                } else {
                    println!("GM8.0 magic check's JNZ is patched out");
                    None
                }
            }
            0x90 => {
                exe.seek(SeekFrom::Current(4))?;
                if options.log {
                    println!("GM8.0 magic check is patched out with NOP");
                }
                None
            }
            i => {
                if options.log {
                    println!("Unknown instruction in place of magic CMP: {}", i);
                }
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
                            if options.log {
                                println!("GM8.0 header version check looks intact - value is {}", magic);
                            }
                            Some(magic)
                        } else {
                            println!("GM8.0 header version check's JNZ is patched out");
                            None
                        }
                    }
                    0x90 => {
                        exe.seek(SeekFrom::Current(4))?;
                        if options.log {
                            println!("GM8.0 header version check is patched out with NOP");
                        }
                        None
                    }
                    i => {
                        if options.log {
                            println!("Unknown instruction in place of magic CMP: {}", i);
                        }
                        return Ok(false);
                    }
                }
            } else {
                if options.log {
                    println!("GM8.0 header version check appears patched out");
                }
                None
            }
        };

        // Read header start pos
        exe.set_position(GM80_HEADER_START_POS);
        let header_start = exe.read_u32_le()?;
        if options.log {
            println!("Reading header from 0x{:X}", header_start);
        }
        exe.set_position(header_start as u64);

        // Check the header magic numbers are what we read them as
        match gm80_magic {
            Some(n) => {
                let header1 = exe.read_u32_le()?;
                if header1 != n {
                    if options.log {
                        println!(
                            "Failed to read GM8.0 header: expected {} at {}, got {}",
                            n, header_start, header1
                        );
                    }
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
                    if options.log {
                        println!("Failed to read GM8.0 header: expected version {}, got {}", n, header2);
                    }
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
fn check_gm81(exe: &mut io::Cursor<&mut [u8]>, options: &ParserOptions) -> Result<bool, Error> {
    if options.log {
        println!("Checking for standard GM8.1 format");
    }

    // Verify size is large enough to do the following checks - otherwise it can't be this format
    if exe.get_ref().len() < 0x226D8A {
        if options.log {
            println!("File too short for this format (0x{:X} bytes)", exe.get_ref().len());
        }
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
                    if options.log {
                        println!("GM8.1 magic check looks intact - value is 0x{:X}", magic);
                    }
                    Some(magic)
                } else {
                    println!("GM8.1 magic check's JE is patched out");
                    None
                }
            }
            b => {
                println!("GM8.1 magic check's CMP is patched out ({:?})", b);
                None
            }
        };

        // Search for header
        exe.set_position(header_start as u64);
        match gm81_magic {
            Some(n) => {
                if options.log {
                    println!("Searching for GM8.1 magic number {} from position {}", n, header_start);
                }
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
                    if options.log {
                        println!("Didn't find GM81 magic value (0x{:X}) before EOF, so giving up", n);
                    }
                    return Ok(false);
                }
            }
            None => {
                exe.seek(SeekFrom::Current(8))?;
            }
        }

        decrypt_gm81(exe, options)?;
        exe.seek(SeekFrom::Current(20))?;
        Ok(true)
    } else {
        Ok(false)
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
) -> Result<(), Error> {
    let game_data = data.get_mut().get_mut(exe_load_offset as usize..).unwrap(); // <- TODO
    for chunk in game_data.rchunks_exact_mut(4) {
        let chunk: &mut [u8; 4] = chunk.try_into().unwrap(); // unreachable
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

/// Identifies the game version and start of gamedata header, given a data cursor.
/// Also removes any version-specific encryptions.
fn find_gamedata(exe: &mut io::Cursor<&mut [u8]>, options: &ParserOptions) -> Result<GameVersion, Error> {
    // Check for UPX-signed PE header
    exe.set_position(0x170);
    // Check for "UPX0" header
    if exe.read_u32_le()? == 0x30585055 {
        if options.log {
            println!("Found UPX0 header");
        }

        exe.seek(SeekFrom::Current(36))?;
        // Check for "UPX1" header
        if exe.read_u32_le()? == 0x31585055 {
            exe.seek(SeekFrom::Current(76))?;

            // Read the UPX version which is a null-terminated string.
            if options.log {
                let mut upx_ver = String::with_capacity(4); // Usually "3.03"
                while let Ok(ch) = exe.read_u8() {
                    if ch != 0 {
                        upx_ver.push(ch as char);
                    } else {
                        break;
                    }
                }
                println!("Found UPX version {}", upx_ver);
            } else {
                while exe.read_u8()? != 0 {}
            }

            if exe.read_u32_le()? == 0x21585055 {
                //"UPX!"
                exe.seek(SeekFrom::Current(28))?;

                let mut unpacked = unpack_upx(exe, options)?;
                if options.log {
                    println!("Successfully unpacked UPX - output is {} bytes", unpacked.len());
                }
                let mut unpacked = io::Cursor::new(&mut *unpacked);

                // UPX unpacked, now check if this is a supported data format
                if let Some((exe_load_offset, header_start, xor_mask, add_mask, sub_mask)) =
                    check_antidec(&mut unpacked)?
                {
                    if options.log {
                        println!("Found antidec2 loading sequence, decrypting with the following values:");
                        println!(
                            "exe_load_offset:0x{:X} header_start:0x{:X} xor_mask:0x{:X} add_mask:0x{:X} sub_mask:0x{:X}",
                            exe_load_offset, header_start, xor_mask, add_mask, sub_mask
                        );
                    }
                    decrypt_antidec(exe, exe_load_offset, header_start, xor_mask, add_mask, sub_mask)?;

                    // 8.0-specific header, but no point strict-checking it because antidec puts random garbage there.
                    exe.seek(SeekFrom::Current(12))?;
                    return Ok(GameVersion::GameMaker80);
                } else {
                    return Err(Error::from(ErrorKind::UnknownFormat));
                }
            } else {
                return Err(Error::from(ErrorKind::PartialUPXPacking));
            }
        }
    }

    // Check for antidec2 protection in the base exe (so without UPX on top of it)
    if let Some((exe_load_offset, header_start, xor_mask, add_mask, sub_mask)) = check_antidec(exe)? {
        if options.log {
            println!("Found antidec2 loading sequence [no UPX], decrypting with the following values:");
            println!(
                "exe_load_offset:0x{:X} header_start:0x{:X} xor_mask:0x{:X} add_mask:0x{:X} sub_mask:0x{:X}",
                exe_load_offset, header_start, xor_mask, add_mask, sub_mask
            );
        }
        decrypt_antidec(exe, exe_load_offset, header_start, xor_mask, add_mask, sub_mask)?;

        // 8.0-specific header, but no point strict-checking it because antidec puts random garbage there.
        exe.seek(SeekFrom::Current(12))?;
        return Ok(GameVersion::GameMaker80);
    }

    // Standard formats
    if check_gm80(exe, options)? {
        Ok(GameVersion::GameMaker80)
    } else if check_gm81(exe, options)? {
        Ok(GameVersion::GameMaker81)
    } else {
        Err(Error::from(ErrorKind::UnknownFormat))
    }
}

impl Game {
    // TODO: functionify a lot of this.
    pub fn from_exe<I>(mut exe: I, options: &ParserOptions) -> Result<Game, Error>
    where
        I: AsRef<[u8]> + AsMut<[u8]>,
    {
        let exe = exe.as_mut();

        // comfy wrapper for byteorder I/O
        let mut exe = io::Cursor::new(exe);

        // verify executable header
        if options.strict {
            // Windows EXE must always start with "MZ"
            if exe.get_ref().get(0..2).unwrap_or(b"XX") != b"MZ" {
                return Err(Error::from(ErrorKind::InvalidExeHeader));
            }
            // Byte 0x3C indicates the start of the PE header
            exe.set_position(0x3C);
            let pe_header_loc = exe.read_u32_le()? as usize;
            // PE header must begin with PE\0\0, then 0x14C which means i386.
            if exe
                .get_ref()
                .get(pe_header_loc..(pe_header_loc + 6))
                .unwrap_or(b"XXXXXX")
                != b"PE\0\0\x4C\x01"
            {
                return Err(Error::from(ErrorKind::InvalidExeHeader));
            }
        }

        let game_ver = find_gamedata(&mut exe, options)?;

        // little helper thing
        let assert_ver = |name: &str, expect, ver| -> Result<(), Error> {
            if options.strict {
                if ver == expect {
                    Ok(())
                } else {
                    Err(Error::from(ErrorKind::InvalidVersion(
                        name.to_string(),
                        expect as f64 / 100.0f64,
                        ver as f64 / 100.0f64,
                    )))
                }
            } else {
                Ok(())
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
                println!(" + Added script '{}'", script.name);
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
