use byteorder::{LE, ReadBytesExt};
use crate::{format, GameVersion, rsrc};
use log::{error, info};
use std::{borrow::Cow, io::{self, Read, Seek}};

pub struct Gmk {
    data: Box<[u8]>,
    ico_file_raw: Option<Vec<u8>>,
    game_version: GameVersion,

    settings_offset: usize,
    settings_len: usize,
    dll_name_offset: usize,
    dll_name_len: usize,
    dll_offset: usize,
    dll_len: usize,
}

impl Gmk {
    /// Reads a GMK from a whole exe file.
    pub fn from_exe(mut data: Box<[u8]>) -> io::Result<Self> {
        let mut exe = io::Cursor::new(data.as_mut());

        // verify executable header
        // Windows EXE must always start with "MZ"
        match exe.get_ref().get(0..2) {
            Some(b"MZ") => (),
            Some(header) => {
                error!("Not a valid exe: should begin with [M, Z], but begins with {:?}", header);
                return Err(io::ErrorKind::InvalidData.into())
            },
            None => {
                error!("Not a valid exe: not long enough!");
                return Err(io::ErrorKind::InvalidData.into())
            }
        }
        // Dword at 0x3C indicates the start of the PE header
        exe.set_position(0x3C);
        let pe_header_loc = exe.read_u32::<LE>()? as usize;
        // PE header must begin with PE\0\0, then 0x14C which means i386.
        match exe.get_ref().get(pe_header_loc..(pe_header_loc + 6)) {
            Some(b"PE\0\0\x4C\x01") => (),
            _ => {
                error!("Not a valid exe: no PE section at 0x{:X}", pe_header_loc);
                return Err(io::ErrorKind::InvalidData.into())
            },
        }
        // Read number of sections
        exe.set_position((pe_header_loc + 6) as u64);
        let section_count = exe.read_u16::<LE>()?;
        // Read length of optional header
        exe.seek(io::SeekFrom::Current(12))?;
        let optional_len = exe.read_u16::<LE>()?;
        // Skip over PE characteristics (2 bytes) + optional header
        exe.seek(io::SeekFrom::Current((optional_len as i64) + 2))?;

        // Read all sections, noting these 3 values from certain sections if they exist
        let mut upx0_virtual_len: Option<u32> = None;
        let mut upx1_data: Option<(u32, u32)> = None; // virtual size, position on disk
        let mut rsrc_location: Option<u32> = None;

        let mut sections: Vec<rsrc::PESection> = Vec::with_capacity(section_count as usize);

        for _ in 0..section_count {
            let mut sect_name = [0u8; 8];
            exe.read_exact(&mut sect_name)?;

            let virtual_size = exe.read_u32::<LE>()?;
            let virtual_address = exe.read_u32::<LE>()?;
            let disk_size = exe.read_u32::<LE>()?;
            let disk_address = exe.read_u32::<LE>()?;
            exe.seek(io::SeekFrom::Current(16))?;

            // See if this is a section we want to do something with
            match sect_name {
                [0x55, 0x50, 0x58, 0x30, 0x00, 0x00, 0x00, 0x00] => {
                    // UPX0 section
                    upx0_virtual_len = Some(virtual_size);
                    info!("UPX0 section found, virtual len: {}", virtual_size);
                },
                [0x55, 0x50, 0x58, 0x31, 0x00, 0x00, 0x00, 0x00] => {
                    // UPX1 section
                    upx1_data = Some((virtual_size, disk_address));
                    info!("UPX1 section found, virtual len: {}", virtual_size);
                },
                [0x2E, 0x72, 0x73, 0x72, 0x63, 0x00, 0x00, 0x00] => {
                    // .rsrc section
                    info!("Found .rsrc section beginning at {}", disk_address);
                    rsrc_location = Some(disk_address);
                },
                _ => {},
            }
            sections.push(rsrc::PESection { virtual_size, virtual_address, disk_size, disk_address })
        }

        let ico_file_raw = rsrc_location.map(|x| {
            let temp_pos = exe.position();
            exe.set_position(u64::from(x));
            let ico = rsrc::find_icons(&mut exe, &sections);
            exe.set_position(temp_pos);
            ico
        }).transpose()?.flatten();

        // Decide if UPX is in use based on PE section names
        // This is None if there is no UPX, obviously, otherwise it's (max_size, offset_on_disk)
        let upx_data: Option<(u32, u32)> = match upx0_virtual_len {
            Some(len0) => upx1_data.map(|(len1, offset)| (len0 + len1, offset)),
            None => None,
        };

        // Identify the game version in use and locate the gamedata header
        let game_version = format::find_in_exe(&mut exe, upx_data)?;

        // Game Settings
        let settings_len = exe.read_u32::<LE>()? as usize;
        let settings_offset = exe.position() as usize;
        exe.seek(io::SeekFrom::Current(settings_len as i64))?;

        // Embedded DirectX DLL
        let dll_name_len = exe.read_u32::<LE>()? as usize;
        let dll_name_offset = exe.position() as usize;
        exe.seek(io::SeekFrom::Current(dll_name_len as i64))?;
        let dll_len = exe.read_u32::<LE>()? as usize;
        let dll_offset = exe.position() as usize;
        exe.seek(io::SeekFrom::Current(dll_len as i64))?;

        // Final decryption pass
        format::gm80::decrypt(&mut exe)?;

        Ok(Self {
            data,
            ico_file_raw,
            game_version,
            settings_offset,
            settings_len,
            dll_name_offset,
            dll_name_len,
            dll_offset,
            dll_len,
        })
    }

    /// Reports the GameVersion of this GMK.
    #[inline(always)]
    pub fn version(&self) -> GameVersion {
        self.game_version
    }

    /// Returns a reference to the whole .ico file associated with this GMK, if any.
    #[inline(always)]
    pub fn ico_file(&self) -> Option<&[u8]> {
        self.ico_file_raw.as_ref().map(|x| x.as_slice())
    }

    /// Returns the name and contents of this file's embedded DirectX DLL.
    /// The file is usually called D3DX8.dll and the contents do not usually change between games.
    #[inline(always)]
    pub fn directx_dll(&self) -> io::Result<(Cow<'_, str>, Vec<u8>)> {
        unsafe {
            let name_slice = self.data.get_unchecked(self.dll_name_offset..(self.dll_name_offset + self.dll_name_len));
            let name = String::from_utf8_lossy(name_slice);
            let data = self.data.get_unchecked(self.dll_offset..(self.dll_offset + self.dll_len));
            let mut contents = Vec::new();
            flate2::bufread::ZlibDecoder::new(data).read_to_end(&mut contents)?;
            Ok((name, contents))
        }
    }
}
