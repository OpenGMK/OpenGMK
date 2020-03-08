use crate::reader::PESection;
use minio::{ReadPrimitives, WritePrimitives};
use std::io::{self, Seek, SeekFrom};

/// A windows icon from the .rsrc header
pub struct WindowsIcon {
    pub width: u32,
    pub height: u32,
    pub original_bpp: u16,
    pub bgra_data: Vec<u8>,
}

/// Finds the icon group from the exe file which will be used for the window icon.
/// Returns a tuple of the parsed WindowsIcon object and the raw .ico blob
pub fn find_icons(
    data: &mut io::Cursor<&mut [u8]>,
    pe_sections: &[PESection],
) -> io::Result<(Vec<WindowsIcon>, Vec<u8>)> {
    // top level header
    let rsrc_base = data.position();
    data.seek(SeekFrom::Current(12))?;
    let name_count = data.read_u16_le()?;
    let id_count = data.read_u16_le()?;
    // skip over any names in the top-level
    data.seek(SeekFrom::Current((name_count as i64) * 8))?;

    let mut icons: Vec<(u32, u32, u32)> = Vec::new(); // id, rva, size

    // read IDs until we find 3 (RT_ICON) or 14 (RT_GROUP_ICON)
    // Windows guarantees that these IDs will be in ascending order, so we'll find 3 before 14.
    for _ in 0..id_count {
        let id = data.read_u32_le()?;
        let offset = data.read_u32_le()? & 0x7FFFFFFF; // high bit is 1

        if id == 3 {
            // 3 = RT_ICON
            let top_level_pos = data.position();
            // Go down to next layer
            data.set_position((offset as u64) + rsrc_base + 14);
            let leaf_count = data.read_u16_le()?;
            if leaf_count == 0 {
                // No leaves under RT_ICON, so no icon
                return Ok((vec![], vec![]));
            }

            // Get each leaf
            for _ in 0..leaf_count {
                // Store where we are in the leaf index
                let leaf_pos = data.position();

                // Go down yet another layer
                let icon_id = data.read_u32_le()?;
                let language_offset = data.read_u32_le()? & 0x7FFFFFFF; // high bit is 1
                data.set_position((language_offset as u64) + rsrc_base + 20);
                let leaf = data.read_u32_le()?;

                // Finally we get to the leaf, which has a pointer to our icon data + size
                data.set_position((leaf as u64) + rsrc_base);
                let rva = data.read_u32_le()?;
                let size = data.read_u32_le()?;
                icons.push((icon_id, rva, size));

                // Go back to the leaf index and go to the next item
                data.set_position(leaf_pos);
                data.seek(SeekFrom::Current(8))?;
            }
            data.set_position(top_level_pos);
        } else if id == 14 {
            // 14 = RT_GROUP_ICON
            data.set_position((offset as u64) + rsrc_base + 12);
            let leaf_count = data.read_u16_le()? + data.read_u16_le()?;
            if leaf_count == 0 {
                // No leaves under RT_GROUP_ICON, so no icon
                return Ok((vec![], vec![]));
            }

            data.seek(SeekFrom::Current(4))?;
            let language_offset = data.read_u32_le()? & 0x7FFFFFFF; // high bit is 1
            data.set_position((language_offset as u64) + rsrc_base + 20);
            let leaf = data.read_u32_le()?;

            // Finally the leaf
            data.set_position((leaf as u64) + rsrc_base);
            let rva = data.read_u32_le()?;
            let size = data.read_u32_le()?;

            if let Some(v) = extract_virtual_bytes(data, pe_sections, rva, size as usize)? {
                // Read the ico header
                let mut ico_header = io::Cursor::new(&v);
                ico_header.seek(SeekFrom::Current(4))?;
                let image_count = usize::from(ico_header.read_u16_le()?);

                let mut icon_group: Vec<WindowsIcon> = vec![];

                let raw_header_size = (6 + (image_count * 16)) as usize;
                let raw_body_size: u32 = icons.iter().map(|t| t.2).sum();
                let mut raw_file: Vec<u8> = Vec::with_capacity(raw_header_size + (raw_body_size as usize));
                let mut raw_file_body: Vec<u8> = Vec::with_capacity(raw_body_size as usize);
                raw_file.extend_from_slice(&v[0..6]);
                for _ in 0..image_count {
                    // Copy data to raw file header
                    let pos = ico_header.position() as usize;
                    raw_file.extend_from_slice(&v[pos..pos + 12]);
                    raw_file.write_u32_le((raw_header_size + raw_file_body.len()) as u32)?;

                    // Skip over the ICO file header
                    // This contains width, height, bpp etc - but these are allowed to be wrong, so we ignore them
                    ico_header.seek(SeekFrom::Current(12))?;
                    let ordinal = ico_header.read_u16_le()?;

                    // Match this ordinal name with an icon resource
                    for icon in &icons {
                        if icon.0 == ordinal as u32 && icon.2 >= 40 {
                            if let Some(v) = extract_virtual_bytes(data, pe_sections, icon.1, icon.2 as usize)? {
                                raw_file_body.extend_from_slice(&v);
                                if let Some(i) = make_icon(v)? {
                                    icon_group.push(i);
                                } else {
                                    println!("WARNING: Failed to recover an icon: id {}, rva 0x{:X}", icon.0, icon.1);
                                }
                            }
                            break;
                        }
                    }
                }
                raw_file.append(&mut raw_file_body);
                return Ok((icon_group, raw_file));
            }
        }
    }

    Ok((vec![], vec![]))
}

fn make_icon(blob: Vec<u8>) -> io::Result<Option<WindowsIcon>> {
    let mut data = io::Cursor::new(&blob);
    let data_start = data.read_u32_le()? as usize;
    let width = data.read_u32_le()?;
    let double_height = data.read_u32_le()?;
    let reserved = data.read_u16_le()?;
    let bpp = data.read_u16_le()?;
    data.set_position(data_start as u64);

    // Checks to make sure this is a valid icon
    if width * 2 != double_height {
        return Ok(None);
    }
    if reserved != 1 {
        return Ok(None);
    }

    // Rename this for clarity
    let ico_wh = width;

    match bpp {
        32 => {
            // 32 bpp: just BGRA pixels followed by mask data.
            // Mask is pointless as far as I can see.
            match blob.get(data_start..data_start + (ico_wh as usize * ico_wh as usize * 4)) {
                Some(d) => {
                    Ok(Some(WindowsIcon { width: ico_wh, height: ico_wh, original_bpp: bpp, bgra_data: d.to_vec() }))
                },
                None => Ok(None),
            }
        },
        8 => {
            // 8 bpp: BGRX lookup table with 256 colours in it, followed by pixel bytes - each byte is a colour index.
            // After pixels is mask data, which indicates whether each pixel is visible or not.
            let pixel_count = ico_wh as usize * ico_wh as usize;
            let mut bgra_data = Vec::with_capacity(pixel_count * 4);
            data.seek(SeekFrom::Current(1024))?; // skip LUT

            for _ in 0..pixel_count {
                let lut_pos = data_start as usize + (data.read_u8()? as usize * 4);
                bgra_data.extend_from_slice(&blob[lut_pos..lut_pos + 4]);
            }

            // read alpha bits - start by reading bitmask bytes which will be used fully
            let mut cursor = 0;
            while cursor + (4 * 8) <= bgra_data.len() {
                let mut bitmask = data.read_u8()?;
                for _ in 0..8 {
                    let (m, b) = bitmask.overflowing_add(bitmask);
                    bitmask = m;
                    bgra_data[cursor + 3] = if b { 0x0 } else { 0xFF };
                    cursor += 4;
                }
            }

            // Apply any leftover bits
            if cursor < bgra_data.len() {
                let mut bitmask = data.read_u8()?;
                while cursor < bgra_data.len() {
                    let (m, b) = bitmask.overflowing_add(bitmask);
                    bitmask = m;
                    bgra_data[cursor + 3] = if b { 0x0 } else { 0xFF };
                    cursor += 4;
                }
            }

            Ok(Some(WindowsIcon { width: ico_wh, height: ico_wh, original_bpp: bpp, bgra_data }))
        },
        _ => Ok(None),
    }
}

/// Extracts some bytes from the file from their location in the initialized exe's memory
fn extract_virtual_bytes(
    data: &mut io::Cursor<&mut [u8]>,
    pe_sections: &[PESection],
    rva: u32,
    size: usize,
) -> io::Result<Option<Vec<u8>>> {
    for section in pe_sections {
        if rva >= section.virtual_address
            && ((rva as usize) + size) < ((section.virtual_address + section.virtual_size) as usize)
        {
            // data is in this section
            let offset_on_disk = rva - section.virtual_address;
            let data_location = (section.disk_address + offset_on_disk) as usize;
            return Ok(data.get_ref().get(data_location..data_location + size).map(|chunk| chunk.to_vec()));
        }
    }

    Ok(None)
}
