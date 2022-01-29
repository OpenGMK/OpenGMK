use byteorder::{ReadBytesExt, LE};
use std::io::{self, Read, Seek, SeekFrom};

/// The settings used to decrypt antidec2-protected data, usually extracted from machine code
#[derive(Copy, Clone)]
pub struct Metadata {
    pub exe_load_offset: u32,
    pub header_start: u32,
    pub xor_mask: u32,
    pub add_mask: u32,
    pub sub_mask: u32,
}

/// Helper function for checking whether a data stream looks like an antidec2-protected exe (GM8.0).
/// If so, returns the relevant vars to decrypt the data stream (AntidecMetadata struct)
pub fn check80(exe: &mut io::Cursor<&mut [u8]>) -> io::Result<Option<Metadata>> {
    // Verify size is large enough to do the following checks - otherwise it can't be antidec
    if exe.get_ref().len() < 0x144AC0usize + 4 {
        return Ok(None)
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
        let exe_load_offset = exe.read_u32::<LE>()? ^ dword_xor_mask;
        // Now the header_start from later in the file
        exe.set_position(0x00144AC0);
        let header_start = exe.read_u32::<LE>()?;
        // xor mask
        exe.set_position(0x000322D3);
        let xor_mask = exe.read_u32::<LE>()? ^ dword_xor_mask;
        // add mask
        exe.set_position(0x000322D8);
        let add_mask = exe.read_u32::<LE>()? ^ dword_xor_mask;
        // sub mask
        exe.set_position(0x000322E4);
        let sub_mask = exe.read_u32::<LE>()? ^ dword_xor_mask;
        Ok(Some(Metadata { exe_load_offset, header_start, xor_mask, add_mask, sub_mask }))
    } else {
        Ok(None)
    }
}

/// Helper function for checking whether a data stream looks like an antidec2-protected exe (GM8.1).
/// If so, returns the relevant vars to decrypt the data stream (AntidecMetadata struct)
pub fn check81(exe: &mut io::Cursor<&mut [u8]>) -> io::Result<Option<Metadata>> {
    // Verify size is large enough to do the following checks - otherwise it can't be antidec
    if exe.get_ref().len() < 0x1F0C53usize {
        return Ok(None)
    }

    // Check for the loading sequence
    exe.set_position(0x000462CC);
    let mut buf = [0u8; 7];
    exe.read_exact(&mut buf)?;

    let byte_xor_mask = buf[3];
    if buf == [0x80, 0x34, 0x08, byte_xor_mask, 0xE2, 0xFA, 0xE9] {
        // Convert mask into a u32 mask so we can apply it easily to dwords
        let dword_xor_mask = u32::from_ne_bytes([byte_xor_mask, byte_xor_mask, byte_xor_mask, byte_xor_mask]);
        // Next, the file offset for loading gamedata bytes
        exe.set_position(0x00046255);
        let exe_load_offset = exe.read_u32::<LE>()? ^ dword_xor_mask;
        // Now the header_start from later in the file
        exe.set_position(0x001F0C53);
        let header_start = exe.read_u32::<LE>()?;
        // xor mask
        exe.set_position(0x00046283);
        let xor_mask = exe.read_u32::<LE>()? ^ dword_xor_mask;
        // add mask
        exe.set_position(0x00046274);
        let add_mask = exe.read_u32::<LE>()? ^ dword_xor_mask;
        // sub mask
        exe.set_position(0x00046293);
        let sub_mask = exe.read_u32::<LE>()? ^ dword_xor_mask;
        Ok(Some(Metadata { exe_load_offset, header_start, xor_mask, add_mask, sub_mask }))
    } else {
        Ok(None)
    }
}

/// Removes antidec2 encryption from gamedata, given the IVs required to do so.
/// Also sets the cursor to the start of the gamedata.
/// Returns true on success, or false indicating that the provided settings are incompatible with the data.
pub fn decrypt(data: &mut io::Cursor<&mut [u8]>, settings: Metadata) -> io::Result<bool> {
    // Offset in the file where the header is
    let offset = settings.exe_load_offset + settings.header_start;
    // Subtract 4 from that position to make sure the first chunk gets decrypted, in case it isn't 4-byte aligned
    let game_data = match data.get_mut().get_mut((offset - 4) as usize..) {
        Some(d) => d,
        None => return Ok(false),
    };

    let mut xor_mask = settings.xor_mask;
    let mut add_mask = settings.add_mask;

    // panic is unreachable, optimized out. const generics should make this cleaner
    for chunk in game_data.rchunks_exact_mut(4).map(|x| <&mut [u8] as TryInto<&mut [u8; 4]>>::try_into(x).unwrap()) {
        let mut value = u32::from_le_bytes(*chunk);

        // apply masks, bswap
        value = (value ^ xor_mask).wrapping_add(add_mask).swap_bytes();

        // cycle masks
        xor_mask = xor_mask.wrapping_sub(settings.sub_mask);
        add_mask = add_mask.swap_bytes().wrapping_add(1);

        // write decrypted value
        *chunk = value.to_le_bytes();
    }

    data.set_position(offset as u64);
    Ok(true)
}
