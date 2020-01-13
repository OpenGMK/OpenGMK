use crate::{
    asset::*,
    gamedata::{self, gm80},
    rsrc,
    settings::{GameHelpDialog, Settings},
    GameAssets, GameVersion,
};
use flate2::bufread::ZlibDecoder;
use minio::ReadPrimitives;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use std::{
    error::Error,
    fmt::{self, Display},
    io::{self, Read, Seek, SeekFrom},
};

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
                ReaderError::InvalidExeHeader => "invalid exe header".into(),
                ReaderError::IO(err) => format!("io error: {}", err),
                ReaderError::PartialUPXPacking => {
                    "looks upx protected, can't locate headers".into()
                }
                ReaderError::UnknownFormat => "unknown format, could not identify file".into(),
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

/// Helper function for inflating zlib data.
pub(crate) fn inflate<I>(data: &I) -> Result<Vec<u8>, ReaderError>
where
    I: AsRef<[u8]> + ?Sized,
{
    let slice = data.as_ref();
    let mut decoder = ZlibDecoder::new(slice);
    let mut buf: Vec<u8> = Vec::with_capacity(slice.len());
    decoder.read_to_end(&mut buf)?;
    Ok(buf)
}

/// A windows PE Section header - https://docs.microsoft.com/en-us/windows/win32/debug/pe-format#section-table-section-headers
pub struct PESection {
    pub virtual_size: u32,
    pub virtual_address: u32,
    pub disk_size: u32,
    pub disk_address: u32,
}

pub fn from_exe<I, F>(
    mut exe: I,
    logger: Option<F>,
    strict: bool,
    multithread: bool,
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
    let mut rsrc_location: Option<u32> = None;

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
                log!(logger, "Found .rsrc section beginning at {}", disk_address);
                rsrc_location = Some(disk_address);
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

    let (icon_data, ico_file_raw) = if let Some(rsrc) = rsrc_location {
        let temp_pos = exe.position();
        exe.set_position(rsrc as u64);
        let icons = rsrc::find_icons(&mut exe, &sections)?;
        exe.set_position(temp_pos);
        icons
    } else {
        (vec![], vec![])
    };

    log!(logger, "Loaded {} icon(s)", icon_data.len());

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
    let game_ver = gamedata::find(&mut exe, logger, upx_data)?;

    // little helper thing
    macro_rules! assert_ver {
        ($name: literal, $expect: expr, $ver: expr) => {{
            let expected = $expect;
            let got = $ver;
            if strict {
                if got == expected {
                    Ok(())
                } else {
                    Err(ReaderError::AssetError(AssetDataError::VersionError {
                        expected,
                        got,
                    }))
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
    let settings_chunk = inflate(&exe.get_ref()[pos..pos + settings_len])?;

    log!(
        logger,
        "Reading settings chunk... (size: {})",
        settings_chunk.len(),
    );

    let settings = {
        fn read_data_maybe(
            cfg: &mut io::Cursor<Vec<u8>>,
        ) -> Result<Option<Box<[u8]>>, ReaderError> {
            if cfg.read_u32_le()? != 0 {
                let len = cfg.read_u32_le()? as usize;
                let pos = cfg.position() as usize;
                cfg.seek(SeekFrom::Current(len as i64))?;
                Ok(Some(
                    inflate(
                        cfg.get_ref()
                            .get(pos..pos + len)
                            .unwrap_or_else(|| unreachable!()),
                    )?
                    .into_boxed_slice(),
                ))
            } else {
                Ok(None)
            }
        }

        let mut cfg = io::Cursor::new(settings_chunk);

        let fullscreen = cfg.read_u32_le()? != 0;
        let interpolate_pixels = cfg.read_u32_le()? != 0;
        let dont_draw_border = cfg.read_u32_le()? != 0;
        let display_cursor = cfg.read_u32_le()? != 0;
        let scaling = cfg.read_i32_le()?;
        let allow_resize = cfg.read_u32_le()? != 0;
        let window_on_top = cfg.read_u32_le()? != 0;
        let clear_colour = cfg.read_u32_le()?;
        let set_resolution = cfg.read_u32_le()? != 0;
        let colour_depth = cfg.read_u32_le()?;
        let resolution = cfg.read_u32_le()?;
        let frequency = cfg.read_u32_le()?;
        let dont_show_buttons = cfg.read_u32_le()? != 0;
        let vsync = cfg.read_u32_le()? != 0;
        let disable_screensaver = cfg.read_u32_le()? != 0;
        let f4_fullscreen_toggle = cfg.read_u32_le()? != 0;
        let f1_help_menu = cfg.read_u32_le()? != 0;
        let esc_close_game = cfg.read_u32_le()? != 0;
        let f5_save_f6_load = cfg.read_u32_le()? != 0;
        let f9_screenshot = cfg.read_u32_le()? != 0;
        let treat_close_as_esc = cfg.read_u32_le()? != 0;
        let priority = cfg.read_u32_le()?;
        let freeze_on_lose_focus = cfg.read_u32_le()? != 0;
        let loading_bar = cfg.read_u32_le()?;
        let (backdata, frontdata) = if loading_bar != 0 {
            (read_data_maybe(&mut cfg)?, read_data_maybe(&mut cfg)?)
        } else {
            (None, None)
        };
        let custom_load_image = read_data_maybe(&mut cfg)?;
        let transparent = cfg.read_u32_le()? != 0;
        let translucency = cfg.read_u32_le()?;
        let scale_progress_bar = cfg.read_u32_le()? != 0;
        let show_error_messages = cfg.read_u32_le()? != 0;
        let log_errors = cfg.read_u32_le()? != 0;
        let always_abort = cfg.read_u32_le()? != 0;
        let (zero_uninitalized_vars, error_on_uninitalized_args) =
            match (game_ver, cfg.read_u32_le()?) {
                (GameVersion::GameMaker8_0, x) => (x != 0, true),
                (GameVersion::GameMaker8_1, x) => ((x & 1) != 0, (x & 2) != 0),
            };

        log!(logger, " + Loaded settings structure");
        log!(logger, "   - Start in full-screen mode: {}", fullscreen);

        log!(
            logger,
            "   - Interpolate colors between pixels: {}",
            interpolate_pixels
        );

        log!(
            logger,
            "   - Don't draw a border in windowed mode: {}",
            dont_draw_border
        );

        log!(logger, "   - Display the cursor: {}", display_cursor);

        log!(logger, "   - Scaling: {}", scaling);

        log!(
            logger,
            "   - Allow the player to resize the game window: {}",
            allow_resize
        );

        log!(
            logger,
            "   - Let the game window always stay on top: {}",
            window_on_top
        );

        log!(
            logger,
            "   - Colour outside the room region (RGBA): #{:0>8X}",
            clear_colour
        );

        log!(
            logger,
            "   - Set the resolution of the screen: {}",
            set_resolution
        );

        log!(
            logger,
            "   -   -> Color Depth: {}",
            match colour_depth {
                0 => "No Change",
                1 => "16-Bit",
                2 | _ => "32-Bit",
            }
        );

        log!(
            logger,
            "   -   -> Resolution: {}",
            match resolution {
                0 => "No Change",
                1 => "320x240",
                2 => "640x480",
                3 => "800x600",
                4 => "1024x768",
                5 => "1280x1024",
                6 | _ => "1600x1200",
            }
        );

        log!(
            logger,
            "   -   -> Frequency: {}",
            match frequency {
                0 => "No Change",
                1 => "60Hz",
                2 => "70Hz",
                3 => "85Hz",
                4 => "100Hz",
                5 | _ => "120Hz",
            }
        );

        log!(
            logger,
            "   - Don't show the buttons in the window captions: {}",
            dont_show_buttons
        );

        log!(
            logger,
            "   - Use synchronization to avoid tearing: {}",
            vsync
        );

        log!(
            logger,
            "   - Disable screensavers and power saving actions: {}",
            disable_screensaver
        );

        log!(logger, "   - Let <Esc> end the game: {}", esc_close_game);

        log!(
            logger,
            "   - Treat the close button as the <Esc> key: {}",
            treat_close_as_esc
        );

        log!(
            logger,
            "   - Let <F1> show the game information: {}",
            f1_help_menu
        );

        log!(
            logger,
            "   - Let <F4> switch between screen modes: {}",
            f4_fullscreen_toggle
        );

        log!(
            logger,
            "   - Let <F5> save the game and <F6> load a game: {}",
            f5_save_f6_load
        );

        log!(
            logger,
            "   - Let <F9> take a screenshot of the game: {}",
            f9_screenshot
        );

        log!(
            logger,
            "   - Game Process Priority: {}",
            match priority {
                0 => "Normal",
                1 => "High",
                2 | _ => "Highest",
            }
        );

        log!(
            logger,
            "   - Freeze the game window when the window loses focus: {}",
            freeze_on_lose_focus
        );

        log!(
            logger,
            "   - Loading bar: {}",
            match loading_bar {
                0 => "No loading progress bar",
                1 => "Default loading progress bar",
                2 | _ => "Own loading progress bar",
            }
        );

        log!(
            logger,
            "   - Show your own image while loading: {}",
            custom_load_image.is_some()
        );

        log!(
            logger,
            "   -   -> Make image partially translucent: {}",
            transparent
        );

        log!(
            logger,
            "   -   -> Make translucent with alpha value: {}",
            translucency
        );

        log!(
            logger,
            "   - Scale progress bar image: {}",
            scale_progress_bar
        );

        log!(
            logger,
            "   - Display error messages: {}",
            show_error_messages
        );

        log!(
            logger,
            "   - Write error messages to file game_errors.log: {}",
            log_errors
        );

        log!(logger, "   - Abort on all error messages: {}", always_abort);

        log!(
            logger,
            "   - Treat uninitialized variables as value 0: {}",
            zero_uninitalized_vars
        );

        log!(
            logger,
            "   - Throw an error when arguments aren't initialized correctly: {}",
            error_on_uninitalized_args
        );

        Settings {
            fullscreen,
            interpolate_pixels,
            dont_draw_border,
            display_cursor,
            scaling,
            allow_resize,
            window_on_top,
            clear_colour,
            set_resolution,
            colour_depth,
            resolution,
            frequency,
            dont_show_buttons,
            vsync,
            disable_screensaver,
            f4_fullscreen_toggle,
            f1_help_menu,
            esc_close_game,
            f5_save_f6_load,
            f9_screenshot,
            treat_close_as_esc,
            priority,
            freeze_on_lose_focus,
            loading_bar,
            backdata,
            frontdata,
            custom_load_image,
            transparent,
            translucency,
            scale_progress_bar,
            show_error_messages,
            log_errors,
            always_abort,
            zero_uninitalized_vars,
            error_on_uninitalized_args,
        }
    };

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
    let mut dx_dll = vec![0u8; dll_len as usize];
    exe.read_exact(&mut dx_dll)?;

    // yeah
    gm80::decrypt(&mut exe, logger)?;

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
    let guid = [
        exe.read_u32_le()?,
        exe.read_u32_le()?,
        exe.read_u32_le()?,
        exe.read_u32_le()?,
    ];

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

    fn get_assets<T, F>(
        src: &mut io::Cursor<&[u8]>,
        deserializer: F,
        multithread: bool,
    ) -> Result<Vec<Option<Box<T>>>, ReaderError>
    where
        T: Send,
        F: Fn(&[u8]) -> Result<T, AssetDataError> + Sync,
    {
        let to_asset = |ch| {
            inflate(&ch).and_then(|data| {
                // If the first u32 is 0 then the underlying data doesn't exist (is a None asset).
                match data.get(..4) {
                    Some(&[0, 0, 0, 0]) => Ok(None),
                    // If there are at least 4 bytes (Some) then [4..] will yield a 0-size slice.
                    Some(_) => Ok(Some(Box::new(deserializer(
                        data.get(4..).unwrap_or_else(|| unreachable!()),
                    )?))),
                    None => Err(ReaderError::AssetError(AssetDataError::MalformedData)),
                }
            })
        };

        if multithread {
            get_asset_refs(src)?
                .par_iter()
                .map(to_asset)
                .collect::<Result<Vec<_>, ReaderError>>()
        } else {
            get_asset_refs(src)?
                .iter()
                .map(to_asset)
                .collect::<Result<Vec<_>, ReaderError>>()
        }
    }

    // stuff to pass to asset deserializers
    let a_strict = strict;
    let a_version = game_ver;

    assert_ver!("extensions header", 700, exe.read_u32_le()?)?;
    let extension_count = exe.read_u32_le()? as usize;
    let mut extensions = Vec::with_capacity(extension_count);
    for _ in 0..extension_count {
        let ext = Extension::read(&mut exe, a_strict)?;
        log!(
            logger,
            "+ Added extension '{}' (files: {})",
            ext.name,
            ext.files.len()
        );
        extensions.push(ext);
    }

    // Rewrap data immutable.
    let prev_pos = exe.position();
    let mut exe = io::Cursor::new(exe.into_inner() as &[u8]);
    exe.set_position(prev_pos);

    // Triggers
    assert_ver!("triggers header", 800, exe.read_u32_le()?)?;
    let triggers = get_assets(
        &mut exe,
        |data| Trigger::deserialize(data, a_strict, a_version),
        multithread,
    )?;
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
        log!(
            logger,
            " + Added constant '{}' (expression: {})",
            name,
            expression
        );
        constants.push(Constant { name, expression });
    }

    // Sounds
    assert_ver!("sounds header", 800, exe.read_u32_le()?)?;
    let sounds = get_assets(
        &mut exe,
        |data| Sound::deserialize(data, a_strict, a_version),
        multithread,
    )?;
    if logger.is_some() {
        sounds.iter().flatten().for_each(|sound| {
            log!(logger, " + Added sound '{}' ({})", sound.name, sound.source);
        });
    }

    // Sprites
    assert_ver!("sprites header", 800, exe.read_u32_le()?)?;
    let sprites = get_assets(
        &mut exe,
        |data| Sprite::deserialize(data, a_strict, a_version),
        multithread,
    )?;
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
    let backgrounds = get_assets(
        &mut exe,
        |data| Background::deserialize(data, a_strict, a_version),
        multithread,
    )?;
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
    let paths = get_assets(
        &mut exe,
        |data| Path::deserialize(data, a_strict, a_version),
        multithread,
    )?;
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
    let scripts = get_assets(
        &mut exe,
        |data| Script::deserialize(data, a_strict, a_version),
        multithread,
    )?;
    if logger.is_some() {
        scripts.iter().flatten().for_each(|script| {
            log!(logger, " + Added script '{}'", script.name);
        });
    }

    // Fonts
    assert_ver!("fonts header", 800, exe.read_u32_le()?)?;
    let fonts = get_assets(
        &mut exe,
        |data| Font::deserialize(data, a_strict, a_version),
        multithread,
    )?;
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
    let timelines = get_assets(
        &mut exe,
        |data| Timeline::deserialize(data, a_strict, a_version),
        multithread,
    )?;
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
    let objects = get_assets(
        &mut exe,
        |data| Object::deserialize(data, a_strict, a_version),
        multithread,
    )?;
    if logger.is_some() {
        objects.iter().flatten().for_each(|object| {
            log!(
                logger,
                " + Added object {} ({}{}{}depth {})",
                object.name,
                if object.solid { "solid; " } else { "" },
                if object.visible { "visible; " } else { "" },
                if object.persistent {
                    "persistent; "
                } else {
                    ""
                },
                object.depth,
            );
        });
    }

    // Rooms
    assert_ver!("rooms header", 800, exe.read_u32_le()?)?;
    let rooms = get_assets(
        &mut exe,
        |data| Room::deserialize(data, a_strict, a_version),
        multithread,
    )?;
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
            inflate(chunk).and_then(|data| {
                IncludedFile::deserialize(data, a_strict, a_version).map_err(|e| e.into())
            })
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

    // Action library initialization code. These are GML strings which get run at game start, in order.
    assert_ver!(
        "action library initialization code header",
        500,
        exe.read_u32_le()?
    )?;
    let str_count = exe.read_u32_le()? as usize;
    let mut library_init_strings = Vec::with_capacity(str_count);
    for _ in 0..str_count {
        library_init_strings.push(exe.read_pas_string()?);
    }
    log!(
        logger,
        " + Read {} action library initialization strings",
        str_count
    );

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
        extensions,
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

        dx_dll,
        icon_data,
        ico_file_raw,
        version: game_ver,
        help_dialog,
        last_instance_id,
        last_tile_id,
        library_init_strings,
        room_order,

        settings,
        game_id,
        guid,
    })
}
