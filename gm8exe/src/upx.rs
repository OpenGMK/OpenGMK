use crate::reader::ReaderError;
use byteorder::{ReadBytesExt, LE};
use std::io;

/// Unpack the bytecode of a UPX-protected exe into a separate buffer
pub fn unpack<F>(
    data: &mut io::Cursor<&mut [u8]>,
    max_size: u32,
    disk_offset: u32,
    logger: Option<F>,
) -> Result<Vec<u8>, ReaderError>
where
    F: Copy + Fn(&str),
{
    log!(logger, "Unpacking UPX with output size {}, data starting at {}", max_size, disk_offset);

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
            let v = data.read_u32::<LE>()?;
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
    let v = data.read_u32::<LE>()?;
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
            continue
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
                break
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
                break // This is the only exit point
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
                        break
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
