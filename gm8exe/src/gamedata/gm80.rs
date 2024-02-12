use byteorder::{ReadBytesExt, LE};
use std::{
    cmp::max,
    io::{self, Read, Seek, SeekFrom},
};

/// Check if this is a standard gm8.0 game by looking for the loading sequence
/// If so, sets the cursor to the start of the gamedata.
pub fn check<F>(exe: &mut io::Cursor<&mut [u8]>, logger: Option<F>) -> io::Result<bool>
where
    F: Copy + Fn(&str),
{
    log!(logger, "Checking for standard GM8.0 format...");

    // Verify size is large enough to do the following checks - otherwise it can't be this format
    if exe.get_ref().len() < 0x144AC0 + 4 {
        log!(logger, "File too short for this format (0x{:X} bytes)", exe.get_ref().len());
        return Ok(false)
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
                let magic = exe.read_u32::<LE>()?;
                let mut buf = [0u8; 6];
                exe.read_exact(&mut buf)?;
                if buf == [0x0F, 0x85, 0x18, 0x01, 0x00, 0x00] {
                    log!(logger, "GM8.0 magic check looks intact - value is {}", magic);
                    Some(magic)
                } else {
                    log!(logger, "GM8.0 magic check's JNZ is patched out");
                    None
                }
            },
            0x90 => {
                exe.seek(SeekFrom::Current(4))?;
                log!(logger, "GM8.0 magic check is patched out with NOP");
                None
            },
            i => {
                log!(logger, "Unknown instruction in place of magic CMP: {}", i);
                return Ok(false)
            },
        };

        // There should be a CMP for the next dword, it's usually a version header (0x320)
        let gm80_header_ver: Option<u32> = {
            exe.set_position(0x000A49E2);
            let mut buf = [0u8; 7];
            exe.read_exact(&mut buf)?;
            if buf == [0x8B, 0xC6, 0xE8, 0x07, 0xBD, 0xFD, 0xFF] {
                match exe.read_u8()? {
                    0x3D => {
                        let magic = exe.read_u32::<LE>()?;
                        let mut buf = [0u8; 6];
                        exe.read_exact(&mut buf)?;
                        if buf == [0x0F, 0x85, 0xF5, 0x00, 0x00, 0x00] {
                            log!(logger, "GM8.0 header version check looks intact - value is {}", magic);
                            Some(magic)
                        } else {
                            println!("GM8.0 header version check's JNZ is patched out");
                            None
                        }
                    },
                    0x90 => {
                        exe.seek(SeekFrom::Current(4))?;
                        log!(logger, "GM8.0 header version check is patched out with NOP");
                        None
                    },
                    i => {
                        log!(logger, "Unknown instruction in place of magic CMP: {}", i);
                        return Ok(false)
                    },
                }
            } else {
                log!(logger, "GM8.0 header version check appears patched out");
                None
            }
        };

        // Read header start pos
        exe.set_position(0x144AC0);
        let header_start = exe.read_u32::<LE>()?;
        log!(logger, "Reading header from 0x{:X}", header_start);
        exe.set_position(header_start as u64);

        // Check the header magic numbers are what we read them as
        match gm80_magic {
            Some(n) => {
                loop {
                    let header1 = match exe.read_u32::<LE>() {
                        Ok(h) => h,
                        _ => {
                            log!(logger, "Passed end of stream looking for GM8.0 header, so quitting");
                            return Ok(false)
                        },
                    };
                    if header1 == n {
                        break
                    } else {
                        log!(
                            logger,
                            "Didn't find GM8.0 header at {}: expected {}, got {}",
                            exe.position() - 4,
                            n,
                            header1
                        );
                        // Skip ahead 10000 bytes in the file and try again - this is what the GM8 runner does
                        exe.seek(SeekFrom::Current(10000 - 4))?;
                    }
                }
            },
            None => {
                exe.seek(SeekFrom::Current(4))?;
            },
        }
        match gm80_header_ver {
            Some(n) => {
                let header2 = exe.read_u32::<LE>()?;
                if header2 != n {
                    log!(logger, "Failed to read GM8.0 header: expected version {}, got {}", n, header2);
                    return Ok(false)
                }
            },
            None => {
                exe.seek(SeekFrom::Current(4))?;
            },
        }

        exe.seek(SeekFrom::Current(8))?;
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Removes GameMaker 8.0 protection in-place.
pub fn decrypt<F>(data: &mut io::Cursor<&mut [u8]>, logger: Option<F>) -> io::Result<()>
where
    F: Copy + Fn(&str),
{
    let mut swap_table = [0u8; 256];
    let mut reverse_table = [0u8; 256];

    // the swap table is squished inbetween 2 chunks of useless garbage
    let garbage1_size = data.read_u32::<LE>()? as i64 * 4;
    let garbage2_size = data.read_u32::<LE>()? as i64 * 4;
    data.seek(SeekFrom::Current(garbage1_size))?;
    assert_eq!(data.read(&mut swap_table)?, 256);
    data.seek(SeekFrom::Current(garbage2_size))?;

    // fill up reverse table
    for i in 0..256 {
        reverse_table[swap_table[i] as usize] = i as u8;
    }

    // asset data length
    let len = data.read_u32::<LE>()? as usize;

    // simplifying for expressions below
    let pos = data.position() as usize; // stream position
    let data = data.get_mut(); // mutable ref for writing
    log!(logger, "Decrypting asset data... (size: {}, garbage1: {}, garbage2: {})", len, garbage1_size, garbage2_size);

    // decryption: first pass
    //   in reverse, data[i-1] = rev[data[i-1]] - (data[i-2] + (i - (pos+1)))
    for i in ((pos + 2)..=(pos + len)).rev() {
        data[i - 1] =
            reverse_table[data[i - 1] as usize].wrapping_sub(data[i - 2].wrapping_add((i.wrapping_sub(pos + 1)) as u8));
    }

    // decryption: second pass
    //   for each byte from end of the file, calculate a byte position to swap with, then swap them over
    for i in (pos..pos + len).rev() {
        let b = max(i as u32 - swap_table[(i - pos) & 0xFF] as u32, pos as u32);
        data.swap(i, b as usize);
    }

    Ok(())
}
