use minio::{ReadPrimitives, WritePrimitives};
use std::io::{self, Read, Seek, SeekFrom};
use std::iter::once;

pub enum XorMethod {
    Normal,
    Sudalv,
}

/// Check if this is a standard gm8.1 game by looking for the loading sequence
/// If so, removes gm81 encryption and sets the cursor to the start of the gamedata.
pub fn check<F>(exe: &mut io::Cursor<&mut [u8]>, logger: Option<F>) -> io::Result<bool>
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
                    log!(
                        logger,
                        "GM8.1 magic check looks intact - value is 0x{:X}",
                        magic
                    );
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
                XorMethod::Sudalv
            }
            _ => XorMethod::Normal,
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
                let found_header = seek_value(exe, n)?.is_some();
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

        decrypt(exe, logger, xor_method)?;
        exe.seek(SeekFrom::Current(20))?;
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Check if this is a standard gm8.1 game by looking for the default header (last-resort method)
/// If so, removes gm81 encryption and sets the cursor to the start of the gamedata.
pub fn check_lazy<F>(exe: &mut io::Cursor<&mut [u8]>, logger: Option<F>) -> io::Result<bool>
where
    F: Copy + Fn(&str),
{
    log!(logger, "Searching for default GM8.1 data header");
    exe.set_position(3800004);
    Ok(seek_value(exe, 0xF7140067)?.is_some())
}

/// Seeks for a GM81-style magic value from its current position.
/// Returns the associated xor value within the magic if it was found; returns None otherwise.
/// On success, the cursor will have been advanced just past the eight bytes from which the value was parsed.
pub fn seek_value(exe: &mut io::Cursor<&mut [u8]>, value: u32) -> io::Result<Option<u32>> {
    let mut pos = exe.position();
    loop {
        exe.set_position(pos);
        let d1 = exe.read_u32_le()?;
        let d2 = exe.read_u32_le()?;
        let parsed_value = (d1 & 0xFF00FF00) | (d2 & 0x00FF00FF);
        let parsed_xor = (d1 & 0x00FF00FF) | (d2 & 0xFF00FF00);
        if parsed_value == value {
            break Ok(Some(parsed_xor));
        }
        pos += 1;
        if ((pos + 8) as usize) >= exe.get_ref().len() {
            break Ok(None);
        }
    }
}

/// Removes GM8.1 encryption in-place.
pub fn decrypt<F>(
    data: &mut io::Cursor<&mut [u8]>,
    logger: Option<F>,
    xor_method: XorMethod,
) -> io::Result<()>
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
    let hash_key_utf16: Vec<u8> = hash_key
        .bytes()
        .flat_map(|c| once(c).chain(once(0)))
        .collect();

    // generate crc table
    let mut crc_table = [0u32; 256];
    let crc_polynomial: u32 = 0x04C11DB7;
    for (i, val) in crc_table.iter_mut().enumerate() {
        *val = crc_32_reflect(i as u32, 8) << 24;
        for _ in 0..8 {
            *val = (*val << 1)
                ^ if *val & (1 << 31) != 0 {
                    crc_polynomial
                } else {
                    0
                };
        }
        *val = crc_32_reflect(*val, 32);
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
        XorMethod::Normal => {
            // Normal xor generation
            while let Ok(dword) = data.read_u32_le() {
                data.set_position(data.position() - 4);
                seed1 = (0xFFFF & seed1) * 0x9069 + (seed1 >> 16);
                seed2 = (0xFFFF & seed2) * 0x4650 + (seed2 >> 16);
                let xor_mask = (seed1 << 16) + (seed2 & 0xFFFF);
                data.write_u32_le(xor_mask ^ dword)?;
            }
        }
        XorMethod::Sudalv => {
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

fn sudalv_magic(
    seed: u32,
    data: &mut io::Cursor<&mut [u8]>,
    magic_point: u32,
    x20: &mut u32,
) -> io::Result<u32> {
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
