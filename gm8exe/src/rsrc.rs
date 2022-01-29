use crate::reader::PESection;
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use std::io::{self, Seek, SeekFrom};

/*
/// A windows icon from the .rsrc header
pub struct WindowsIcon {
    pub width: u32,
    pub height: u32,
    pub original_bpp: u16,
    pub bgra_data: Vec<u8>,
}
*/

/// Finds the icon group from the exe file which will be used for the window icon.
/// Returns an entire rebuilt .ico file, or None if there isn't one associated with this exe.
pub fn find_icons(data: &mut io::Cursor<&mut [u8]>, pe_sections: &[PESection]) -> io::Result<Option<Vec<u8>>> {
    // top level header
    let rsrc_base = data.position();
    data.seek(SeekFrom::Current(12))?;
    let name_count = data.read_u16::<LE>()?;
    let id_count = data.read_u16::<LE>()?;
    // skip over any names in the top-level
    data.seek(SeekFrom::Current((name_count as i64) * 8))?;

    let mut icons: Vec<(u32, u32, u32)> = Vec::new(); // id, rva, size

    // read IDs until we find 3 (RT_ICON) or 14 (RT_GROUP_ICON)
    // Windows guarantees that these IDs will be in ascending order, so we'll find 3 before 14.
    for _ in 0..id_count {
        let id = data.read_u32::<LE>()?;
        let offset = data.read_u32::<LE>()? & 0x7FFFFFFF; // high bit is 1

        if id == 3 {
            // 3 = RT_ICON
            let top_level_pos = data.position();
            // Go down to next layer
            data.set_position((offset as u64) + rsrc_base + 14);
            let leaf_count = data.read_u16::<LE>()?;
            if leaf_count == 0 {
                // No leaves under RT_ICON, so no icon
                return Ok(None)
            }

            // Get each leaf
            for _ in 0..leaf_count {
                // Store where we are in the leaf index
                let leaf_pos = data.position();

                // Go down yet another layer
                let icon_id = data.read_u32::<LE>()?;
                let language_offset = data.read_u32::<LE>()? & 0x7FFFFFFF; // high bit is 1
                data.set_position((language_offset as u64) + rsrc_base + 20);
                let leaf = data.read_u32::<LE>()?;

                // Finally we get to the leaf, which has a pointer to our icon data + size
                data.set_position((leaf as u64) + rsrc_base);
                let rva = data.read_u32::<LE>()?;
                let size = data.read_u32::<LE>()?;
                icons.push((icon_id, rva, size));

                // Go back to the leaf index and go to the next item
                data.set_position(leaf_pos);
                data.seek(SeekFrom::Current(8))?;
            }
            data.set_position(top_level_pos);
        } else if id == 14 {
            // 14 = RT_GROUP_ICON
            data.set_position((offset as u64) + rsrc_base + 12);
            let leaf_count = data.read_u16::<LE>()? + data.read_u16::<LE>()?;
            if leaf_count == 0 {
                // No leaves under RT_GROUP_ICON, so no icon
                return Ok(None)
            }

            data.seek(SeekFrom::Current(4))?;
            let language_offset = data.read_u32::<LE>()? & 0x7FFFFFFF; // high bit is 1
            data.set_position((language_offset as u64) + rsrc_base + 20);
            let leaf = data.read_u32::<LE>()?;

            // Finally the leaf
            data.set_position((leaf as u64) + rsrc_base);
            let rva = data.read_u32::<LE>()?;
            let size = data.read_u32::<LE>()?;

            if let Some(v) = extract_virtual_bytes(data, pe_sections, rva, size as usize)? {
                // Read the ico header
                let mut ico_header = io::Cursor::new(&v);
                ico_header.seek(SeekFrom::Current(4))?;
                let image_count = usize::from(ico_header.read_u16::<LE>()?);

                let raw_header_size = (6 + (image_count * 16)) as usize;
                let raw_body_size: u32 = icons.iter().map(|t| t.2).sum();
                let mut raw_file: Vec<u8> = Vec::with_capacity(raw_header_size + (raw_body_size as usize));
                let mut raw_file_body: Vec<u8> = Vec::with_capacity(raw_body_size as usize);
                raw_file.extend_from_slice(&v[0..6]);
                for _ in 0..image_count {
                    // Copy data to raw file header
                    let pos = ico_header.position() as usize;
                    raw_file.extend_from_slice(&v[pos..pos + 12]);
                    raw_file.write_u32::<LE>((raw_header_size + raw_file_body.len()) as u32)?;

                    // Skip over the ICO file header
                    // This contains width, height, bpp etc - but these are allowed to be wrong, so we ignore them
                    ico_header.seek(SeekFrom::Current(12))?;
                    let ordinal = ico_header.read_u16::<LE>()?;

                    // Match this ordinal name with an icon resource
                    for icon in &icons {
                        if icon.0 == ordinal as u32 && icon.2 >= 40 {
                            if let Some(v) = extract_virtual_bytes(data, pe_sections, icon.1, icon.2 as usize)? {
                                raw_file_body.extend_from_slice(&v);
                            }
                            break
                        }
                    }
                }
                raw_file.append(&mut raw_file_body);
                return Ok(Some(raw_file))
            }
        }
    }

    Ok(None)
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
            return Ok(data.get_ref().get(data_location..data_location + size).map(|chunk| chunk.to_vec()))
        }
    }

    Ok(None)
}
