pub mod antidec;
pub mod gm80;
pub mod gm81;

use crate::{reader::ReaderError, upx, GameVersion};
use std::io::{self, Seek, SeekFrom};

/// Identifies the game version and start of gamedata header, given a data cursor.
/// Also removes any version-specific encryptions.
pub fn find<F>(
    exe: &mut io::Cursor<&mut [u8]>,
    logger: Option<F>,
    upx_data: Option<(u32, u32)>,
) -> Result<GameVersion, ReaderError>
where
    F: Copy + Fn(&str),
{
    // helper fn for logging antidec settings
    let log_antidec = |data: antidec::Metadata| {
        log!(
            logger,
            "exe_load_offset:0x{:X} header_start:0x{:X} xor_mask:0x{:X} add_mask:0x{:X} sub_mask:0x{:X}",
            data.exe_load_offset,
            data.header_start,
            data.xor_mask,
            data.add_mask,
            data.sub_mask
        );
    };

    // Check if UPX is in use first
    match upx_data {
        Some((max_size, disk_offset)) => {
            // UPX in use, let's unpack it
            let mut unpacked = upx::unpack(exe, max_size, disk_offset, logger)?;
            log!(logger, "Successfully unpacked UPX - output is {} bytes", unpacked.len());
            let mut unpacked = io::Cursor::new(&mut *unpacked);

            // UPX unpacked, now check if this is a supported data format
            if let Some(antidec_settings) = antidec::check80(&mut unpacked)? {
                if logger.is_some() {
                    log!(logger, "Found GM8.0 antidec2 loading sequence, decrypting with these settings:");
                    log_antidec(antidec_settings);
                }
                if antidec::decrypt(exe, antidec_settings)? {
                    // 8.0-specific header, but no point strict-checking it because antidec puts random garbage there.
                    exe.seek(SeekFrom::Current(16))?;
                    Ok(GameVersion::GameMaker8_0)
                } else {
                    // Antidec couldn't be decrypted with the settings we read, so we must have got the format wrong
                    Err(ReaderError::UnknownFormat)
                }
            } else if let Some(antidec_settings) = antidec::check81(&mut unpacked)? {
                log!(logger, "Found GM8.1 antidec2 loading sequence, decrypting with these settings:");
                log_antidec(antidec_settings);
                if antidec::decrypt(exe, antidec_settings)? {
                    // Search for header
                    let found_header = gm81::seek_value(exe, 0xF7140067)?.is_some();

                    if found_header {
                        gm81::decrypt(exe, logger, gm81::XorMethod::Normal)?;
                        exe.seek(SeekFrom::Current(20))?;
                        Ok(GameVersion::GameMaker8_1)
                    } else {
                        log!(logger, "Didn't find GM81 magic value (0xF7140017) before EOF, so giving up");
                        Err(ReaderError::UnknownFormat)
                    }
                } else {
                    // Antidec couldn't be decrypted with the settings we read, so we must have got the format wrong
                    Err(ReaderError::UnknownFormat)
                }
            } else {
                Err(ReaderError::UnknownFormat)
            }
        },
        None => {
            if let Some(antidec_settings) = antidec::check80(exe)? {
                // antidec2 protection in the base exe (so without UPX on top of it)
                if logger.is_some() {
                    log!(logger, "Found GM8.0 antidec2 loading sequence [no UPX], decrypting with these settings:");
                    log_antidec(antidec_settings);
                }
                if antidec::decrypt(exe, antidec_settings)? {
                    // 8.0-specific header, but no point strict-checking it because antidec puts random garbage there.
                    exe.seek(SeekFrom::Current(16))?;
                    Ok(GameVersion::GameMaker8_0)
                } else {
                    // Antidec couldn't be decrypted with the settings we read, so we must have got the format wrong
                    Err(ReaderError::UnknownFormat)
                }
            } else if let Some(antidec_settings) = antidec::check81(exe)? {
                // antidec81 protection in the base exe (so without UPX on top of it)
                if logger.is_some() {
                    log!(logger, "Found GM8.1 antidec2 loading sequence [no UPX], decrypting with these settings:");
                    log_antidec(antidec_settings);
                }
                if antidec::decrypt(exe, antidec_settings)? {
                    let found_header = gm81::seek_value(exe, 0xF7140067)?.is_some();

                    if found_header {
                        gm81::decrypt(exe, logger, gm81::XorMethod::Normal)?;
                        exe.seek(SeekFrom::Current(20))?;
                        Ok(GameVersion::GameMaker8_1)
                    } else {
                        log!(logger, "Didn't find GM81 magic value (0xF7140017) before EOF, so giving up");
                        Err(ReaderError::UnknownFormat)
                    }
                } else {
                    // Antidec couldn't be decrypted with the settings we read, so we must have got the format wrong
                    Err(ReaderError::UnknownFormat)
                }
            } else {
                // Standard formats
                if gm80::check(exe, logger)? {
                    Ok(GameVersion::GameMaker8_0)
                } else if gm81::check(exe, logger)? || gm81::check_lazy(exe, logger)? {
                    Ok(GameVersion::GameMaker8_1)
                } else {
                    Err(ReaderError::UnknownFormat)
                }
            }
        },
    }
}
