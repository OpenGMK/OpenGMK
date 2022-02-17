use byteorder::{LE, ReadBytesExt};
use crate::{asset::*, format, GameVersion, HelpDialog, rsrc, Settings};
use log::{error, info};
use std::{borrow::Cow, io::{self, Read, Seek}};

#[cfg(feature = "rayon")]
use crate::rayon::{iter::plumbing::*, prelude::*};

pub struct Gmk {
    data: Box<[u8]>,
    ico_file_raw: Option<Vec<u8>>,
    game_version: GameVersion,
    is_gmk: bool,

    settings_offset: usize,
    settings_len: usize,
    dll_name_offset: usize,
    dll_name_len: usize,
    dll_offset: usize,
    dll_len: usize,

    pro_flag: bool,
    game_id: u32,
    directplay_guid: [u32; 4],

    extensions: Vec<Extension>,
    triggers: AssetInfo,
    constants: Vec<Constant>,
    sounds: AssetInfo,
    sprites: AssetInfo,
    backgrounds: AssetInfo,
    paths: AssetInfo,
    scripts: AssetInfo,
    fonts: AssetInfo,
    timelines: AssetInfo,
    objects: AssetInfo,
    rooms: AssetInfo,
    included_files: AssetInfo,

    last_instance_id: i32,
    last_tile_id: i32,

    help_offset: usize,
    help_len: usize,
    lib_init_offset: usize,
    lib_init_count: u32,
    room_order_offset: usize,
    room_order_count: u32,
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
            sections.push(rsrc::PESection { virtual_size, virtual_address, _disk_size: disk_size, disk_address })
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

        // Garbage field - random bytes
        let garbage_dwords = exe.read_u32::<LE>()?;
        exe.seek(io::SeekFrom::Current(i64::from(garbage_dwords) * 4))?;

        // GM8 Pro flag, game ID
        let pro_flag: bool = exe.read_u32::<LE>()? != 0;
        let game_id = exe.read_u32::<LE>()?;
        let directplay_guid = [exe.read_u32::<LE>()?, exe.read_u32::<LE>()?, exe.read_u32::<LE>()?, exe.read_u32::<LE>()?];

        // Extensions
        if exe.read_u32::<LE>()? != 700 {
            return Err(io::Error::from(io::ErrorKind::InvalidData));
        }
        // We can't skip over these easily because they aren't compressed, so we decrypt and parse these in advance
        let extension_count = exe.read_u32::<LE>()? as usize;
        let mut extensions = Vec::with_capacity(extension_count);
        for _ in 0..extension_count {
            extensions.push(Extension::read(&mut exe, false)?);
        }

        // Triggers
        let triggers = skip_asset_block(&mut exe)?;

        // Constants
        if exe.read_u32::<LE>()? != 800 {
            return Err(io::Error::from(io::ErrorKind::InvalidData));
        }
        // Like Extensions, these aren't compressed, so it's easier to just do them in advance
        let constant_count = exe.read_u32::<LE>()? as usize;
        let mut constants = Vec::with_capacity(constant_count);
        for _ in 0..constant_count {
            constants.push(Constant::read(&mut exe)?);
        }

        // All main asset types..
        let sounds = skip_asset_block(&mut exe)?;
        let sprites = skip_asset_block(&mut exe)?;
        let backgrounds = skip_asset_block(&mut exe)?;
        let paths = skip_asset_block(&mut exe)?;
        let scripts = skip_asset_block(&mut exe)?;
        let fonts = skip_asset_block(&mut exe)?;
        let timelines = skip_asset_block(&mut exe)?;
        let objects = skip_asset_block(&mut exe)?;
        let rooms = skip_asset_block(&mut exe)?;

        let last_instance_id = exe.read_i32::<LE>()?;
        let last_tile_id = exe.read_i32::<LE>()?;

        let included_files = skip_asset_block(&mut exe)?;

        // Help dialog
        if exe.read_u32::<LE>()? != 800 {
            return Err(io::Error::from(io::ErrorKind::InvalidData));
        }
        let help_len = exe.read_u32::<LE>()? as usize;
        let help_offset = exe.position() as usize;
        exe.seek(io::SeekFrom::Current(help_len as i64))?;

        // Library initialization code
        if exe.read_u32::<LE>()? != 500 {
            return Err(io::Error::from(io::ErrorKind::InvalidData));
        }
        let lib_init_count = exe.read_u32::<LE>()?;
        let lib_init_offset = exe.position() as usize;
        for _ in 0..lib_init_count {
            let len = exe.read_u32::<LE>()?;
            exe.seek(io::SeekFrom::Current(len.into()))?;
        }

        // Room order
        if exe.read_u32::<LE>()? != 700 {
            return Err(io::Error::from(io::ErrorKind::InvalidData));
        }
        let room_order_count = exe.read_u32::<LE>()?;
        let room_order_offset = exe.position() as usize;
        exe.seek(io::SeekFrom::Current(i64::from(room_order_count * 4)))?; // verify amount of data left in file

        Ok(Self {
            data,
            ico_file_raw,
            game_version,
            is_gmk: false,
            settings_offset,
            settings_len,
            dll_name_offset,
            dll_name_len,
            dll_offset,
            dll_len,
            pro_flag,
            game_id,
            directplay_guid,
            extensions,
            triggers,
            constants,
            sounds,
            sprites,
            backgrounds,
            paths,
            scripts,
            fonts,
            timelines,
            objects,
            rooms,
            included_files,
            last_instance_id,
            last_tile_id,
            help_len,
            help_offset,
            lib_init_offset,
            lib_init_count,
            room_order_offset,
            room_order_count,
        })
    }

    /// Reports the GameVersion of this GMK.
    #[inline]
    pub fn version(&self) -> GameVersion {
        self.game_version
    }

    /// Returns a reference to the whole .ico file associated with this GMK, if any.
    #[inline]
    pub fn ico_file(&self) -> Option<&[u8]> {
        self.ico_file_raw.as_ref().map(|x| x.as_slice())
    }

    /// Returns the name and contents of this file's embedded DirectX DLL.
    /// The file is usually called D3DX8.dll and the contents do not usually change between games.
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

    /// Returns the settings header belonging to this file.
    ///
    /// Note that this data is compressed in the game file, and decompression is not done in advance, nor is it cached.
    /// As such, it would be ideal to store the result of this function rather than calling it more than once.
    pub fn settings(&self) -> io::Result<Settings> {
        fn read_data_maybe(data: &mut impl Read) -> io::Result<Option<Box<[u8]>>> {
            if data.read_u32::<LE>()? != 0 {
                let len = data.read_u32::<LE>()? as usize;
                let mut output = Vec::with_capacity(len);
                unsafe {
                    output.set_len(len);
                }
                data.read_exact(&mut output)?;
                Ok(Some(output.into_boxed_slice()))
            } else {
                Ok(None)
            }
        }

        unsafe {
            let slice = self.data.get_unchecked(self.settings_offset..(self.settings_offset + self.settings_len));
            let mut data = flate2::bufread::ZlibDecoder::new(slice);
            let fullscreen = data.read_u32::<LE>()? != 0;
            let interpolate_pixels = data.read_u32::<LE>()? != 0;
            let dont_draw_border = data.read_u32::<LE>()? != 0;
            let display_cursor = data.read_u32::<LE>()? != 0;
            let scaling = data.read_i32::<LE>()?;
            let allow_resize = data.read_u32::<LE>()? != 0;
            let window_on_top = data.read_u32::<LE>()? != 0;
            let clear_colour = data.read_u32::<LE>()?;
            let set_resolution = data.read_u32::<LE>()? != 0;
            let colour_depth = data.read_u32::<LE>()?;
            let resolution = data.read_u32::<LE>()?;
            let frequency = data.read_u32::<LE>()?;
            let dont_show_buttons = data.read_u32::<LE>()? != 0;
            let (vsync, force_cpu_render) = match (self.game_version, data.read_u32::<LE>()?) {
                (GameVersion::GameMaker8_0, x) => (x != 0, true), // see 8.1.141 changelog
                (GameVersion::GameMaker8_1, x) => ((x & 1) != 0, (x & (1 << 7)) != 0),
            };
            let disable_screensaver = data.read_u32::<LE>()? != 0;
            let f4_fullscreen_toggle = data.read_u32::<LE>()? != 0;
            let f1_help_menu = data.read_u32::<LE>()? != 0;
            let esc_close_game = data.read_u32::<LE>()? != 0;
            let f5_save_f6_load = data.read_u32::<LE>()? != 0;
            let f9_screenshot = data.read_u32::<LE>()? != 0;
            let treat_close_as_esc = data.read_u32::<LE>()? != 0;
            let priority = data.read_u32::<LE>()?;
            let freeze_on_lose_focus = data.read_u32::<LE>()? != 0;
            let loading_bar = data.read_u32::<LE>()?;
            let (backdata, frontdata) =
            if loading_bar != 0 { (read_data_maybe(&mut data)?, read_data_maybe(&mut data)?) } else { (None, None) };
            let custom_load_image = read_data_maybe(&mut data)?;
            let transparent = data.read_u32::<LE>()? != 0;
            let translucency = data.read_u32::<LE>()?;
            let scale_progress_bar = data.read_u32::<LE>()? != 0;
            let show_error_messages = data.read_u32::<LE>()? != 0;
            let log_errors = data.read_u32::<LE>()? != 0;
            let always_abort = data.read_u32::<LE>()? != 0;
            let (zero_uninitialized_vars, error_on_uninitialized_args) = match (self.game_version, data.read_u32::<LE>()?) {
                (GameVersion::GameMaker8_0, x) => (x != 0, false),
                (GameVersion::GameMaker8_1, x) => ((x & 1) != 0, (x & 2) != 0),
            };
            let swap_creation_events = match data.read_u32::<LE>() {
                Ok(_webgl) => data.read_u32::<LE>()? != 0,
                Err(_) => false,
            };

            Ok(Settings {
                fullscreen,
                scaling,
                interpolate_pixels,
                clear_colour,
                allow_resize,
                window_on_top,
                dont_draw_border,
                dont_show_buttons,
                display_cursor,
                freeze_on_lose_focus,
                disable_screensaver,
                force_cpu_render,
                set_resolution,
                colour_depth,
                resolution,
                frequency,
                vsync,
                esc_close_game,
                treat_close_as_esc,
                f1_help_menu,
                f4_fullscreen_toggle,
                f5_save_f6_load,
                f9_screenshot,
                priority,
                custom_load_image,
                transparent,
                translucency,
                loading_bar,
                backdata,
                frontdata,
                scale_progress_bar,
                show_error_messages,
                log_errors,
                always_abort,
                zero_uninitialized_vars,
                error_on_uninitialized_args,
                swap_creation_events,
            })
        }
    }

    /// Returns whether the pro flag is set for this game, i.e. whether GameMaker Pro features would be enabled.
    #[inline]
    pub fn pro(&self) -> bool {
        self.pro_flag
    }

    /// Returns the game ID for this file.
    #[inline]
    pub fn id(&self) -> u32 {
        self.game_id
    }

    /// Returns the DirectPlay Game GUID for this file.
    #[inline]
    pub fn directplay_guid(&self) -> [u32; 4] {
        self.directplay_guid
    }

    /// Returns an iterator over the Extensions found in this file.
    pub fn extensions(&self) -> impl Iterator<Item = &Extension> {
        self.extensions.iter()
    }

    /// Returns a parallel iterator over the Extensions found in this file.
    ///
    /// This method is provided for the sake of consistency with this struct's other methods. However,
    /// Extensions are collected in advance due to their gmk format making them difficult to ignore without parsing.
    /// Therefore, `par_extensions()` is essentially no different from `extensions().into_par_iter()`.
    #[cfg_attr(feature = "nightly-docs", doc(cfg(feature = "rayon")))]
    #[cfg_attr(not(feature = "nightly-docs"), cfg(feature = "rayon"))]
    pub fn par_extensions(&self) -> impl ParallelIterator<Item = &Extension> {
        self.extensions.par_iter()
    }

    /// Returns an iterator over the Triggers found in this file.
    pub fn triggers(&self) -> impl Iterator<Item = io::Result<Option<Trigger>>> + '_ {
        Parser::new(&self.data, self.triggers, self.is_gmk)
    }

    /// Returns a parallel iterator over the Triggers found in this file.
    #[cfg_attr(feature = "nightly-docs", doc(cfg(feature = "rayon")))]
    #[cfg_attr(not(feature = "nightly-docs"), cfg(feature = "rayon"))]
    pub fn par_triggers(&self) -> impl ParallelIterator<Item = io::Result<Option<Trigger>>> + '_ {
        ParallelParser::new(&self.data, self.triggers, self.is_gmk)
    }

    /// Returns an iterator over the Constants found in this file.
    #[inline]
    pub fn constants(&self) -> impl Iterator<Item = &Constant> {
        self.constants.iter()
    }

    /// Returns a parallel iterator over the Constants found in this file.
    ///
    /// This method is provided for the sake of consistency with this struct's other methods. However,
    /// Constants are collected in advance due to their gmk format making them difficult to ignore without parsing.
    /// Therefore, `par_constants()` is essentially no different from `constants().into_par_iter()`.
    #[cfg_attr(feature = "nightly-docs", doc(cfg(feature = "rayon")))]
    #[cfg_attr(not(feature = "nightly-docs"), cfg(feature = "rayon"))]
    pub fn par_constants(&self) -> impl ParallelIterator<Item = &Constant> {
        self.constants.par_iter()
    }

    /// Returns an iterator over the Sounds found in this file.
    pub fn sounds(&self) -> impl Iterator<Item = io::Result<Option<Sound>>> + '_ {
        Parser::new(&self.data, self.sounds, self.is_gmk)
    }

    /// Returns a parallel iterator over the Sounds found in this file.
    #[cfg_attr(feature = "nightly-docs", doc(cfg(feature = "rayon")))]
    #[cfg_attr(not(feature = "nightly-docs"), cfg(feature = "rayon"))]
    pub fn par_sounds(&self) -> impl ParallelIterator<Item = io::Result<Option<Sound>>> + '_ {
        ParallelParser::new(&self.data, self.sounds, self.is_gmk)
    }

    /// Returns an iterator over the Sprites found in this file.
    pub fn sprites(&self) -> impl Iterator<Item = io::Result<Option<Sprite>>> + '_ {
        Parser::new(&self.data, self.sprites, self.is_gmk)
    }

    /// Returns a parallel iterator over the Sprites found in this file.
    #[cfg_attr(feature = "nightly-docs", doc(cfg(feature = "rayon")))]
    #[cfg_attr(not(feature = "nightly-docs"), cfg(feature = "rayon"))]
    pub fn par_sprites(&self) -> impl ParallelIterator<Item = io::Result<Option<Sprite>>> + '_ {
        ParallelParser::new(&self.data, self.sprites, self.is_gmk)
    }

    /// Returns an iterator over the Backgrounds found in this file.
    pub fn backgrounds(&self) -> impl Iterator<Item = io::Result<Option<Background>>> + '_ {
        Parser::new(&self.data, self.backgrounds, self.is_gmk)
    }

    /// Returns a parallel iterator over the Backgrounds found in this file.
    #[cfg_attr(feature = "nightly-docs", doc(cfg(feature = "rayon")))]
    #[cfg_attr(not(feature = "nightly-docs"), cfg(feature = "rayon"))]
    pub fn par_backgrounds(&self) -> impl ParallelIterator<Item = io::Result<Option<Background>>> + '_ {
        ParallelParser::new(&self.data, self.backgrounds, self.is_gmk)
    }

    /// Returns an iterator over the Paths found in this file.
    pub fn paths(&self) -> impl Iterator<Item = io::Result<Option<Path>>> + '_ {
        Parser::new(&self.data, self.paths, self.is_gmk)
    }

    /// Returns a parallel iterator over the Paths found in this file.
    #[cfg_attr(feature = "nightly-docs", doc(cfg(feature = "rayon")))]
    #[cfg_attr(not(feature = "nightly-docs"), cfg(feature = "rayon"))]
    pub fn par_paths(&self) -> impl ParallelIterator<Item = io::Result<Option<Path>>> + '_ {
        ParallelParser::new(&self.data, self.paths, self.is_gmk)
    }

    /// Returns an iterator over the Scripts found in this file.
    pub fn scripts(&self) -> impl Iterator<Item = io::Result<Option<Script>>> + '_ {
        Parser::new(&self.data, self.scripts, self.is_gmk)
    }

    /// Returns a parallel iterator over the Scripts found in this file.
    #[cfg_attr(feature = "nightly-docs", doc(cfg(feature = "rayon")))]
    #[cfg_attr(not(feature = "nightly-docs"), cfg(feature = "rayon"))]
    pub fn par_scripts(&self) -> impl ParallelIterator<Item = io::Result<Option<Script>>> + '_ {
        ParallelParser::new(&self.data, self.scripts, self.is_gmk)
    }

    /// Returns an iterator over the Fonts found in this file.
    pub fn fonts(&self) -> impl Iterator<Item = io::Result<Option<Font>>> + '_ {
        Parser::new(&self.data, self.fonts, self.is_gmk)
    }

    /// Returns a parallel iterator over the Fonts found in this file.
    #[cfg_attr(feature = "nightly-docs", doc(cfg(feature = "rayon")))]
    #[cfg_attr(not(feature = "nightly-docs"), cfg(feature = "rayon"))]
    pub fn par_fonts(&self) -> impl ParallelIterator<Item = io::Result<Option<Font>>> + '_ {
        ParallelParser::new(&self.data, self.fonts, self.is_gmk)
    }

    /// Returns an iterator over the Timelines found in this file.
    pub fn timelines(&self) -> impl Iterator<Item = io::Result<Option<Timeline>>> + '_ {
        Parser::new(&self.data, self.timelines, self.is_gmk)
    }

    /// Returns a parallel iterator over the Timelines found in this file.
    #[cfg_attr(feature = "nightly-docs", doc(cfg(feature = "rayon")))]
    #[cfg_attr(not(feature = "nightly-docs"), cfg(feature = "rayon"))]
    pub fn par_timelines(&self) -> impl ParallelIterator<Item = io::Result<Option<Timeline>>> + '_ {
        ParallelParser::new(&self.data, self.timelines, self.is_gmk)
    }

    /// Returns an iterator over the Objects found in this file.
    pub fn objects(&self) -> impl Iterator<Item = io::Result<Option<Object>>> + '_ {
        Parser::new(&self.data, self.objects, self.is_gmk)
    }

    /// Returns a parallel iterator over the Objects found in this file.
    #[cfg_attr(feature = "nightly-docs", doc(cfg(feature = "rayon")))]
    #[cfg_attr(not(feature = "nightly-docs"), cfg(feature = "rayon"))]
    pub fn par_objects(&self) -> impl ParallelIterator<Item = io::Result<Option<Object>>> + '_ {
        ParallelParser::new(&self.data, self.objects, self.is_gmk)
    }

    /// Returns an iterator over the Rooms found in this file.
    pub fn rooms(&self) -> impl Iterator<Item = io::Result<Option<Room>>> + '_ {
        Parser::new(&self.data, self.rooms, self.is_gmk)
    }

    /// Returns a parallel iterator over the Rooms found in this file.
    #[cfg_attr(feature = "nightly-docs", doc(cfg(feature = "rayon")))]
    #[cfg_attr(not(feature = "nightly-docs"), cfg(feature = "rayon"))]
    pub fn par_rooms(&self) -> impl ParallelIterator<Item = io::Result<Option<Room>>> + '_ {
        ParallelParser::new(&self.data, self.rooms, self.is_gmk)
    }

    /// Returns an iterator over the Included Files found in this file.
    pub fn included_files(&self) -> impl Iterator<Item = io::Result<Option<IncludedFile>>> + '_ {
        Parser::new(&self.data, self.included_files, self.is_gmk)
    }

    /// Returns a parallel iterator over the Included Files found in this file.
    #[cfg_attr(feature = "nightly-docs", doc(cfg(feature = "rayon")))]
    #[cfg_attr(not(feature = "nightly-docs"), cfg(feature = "rayon"))]
    pub fn par_included_files(&self) -> impl ParallelIterator<Item = io::Result<Option<IncludedFile>>> + '_ {
        ParallelParser::new(&self.data, self.included_files, self.is_gmk)
    }

    /// Returns the last instance ID indicated by this file.
    ///
    /// In the editor, new instances placed in rooms will increment this number to generate their ID.
    /// In a game, this number will be incremented for a new ID when calling functions like instance_create().
    /// In a new GMK file this number defaults to 100000.
    #[inline]
    pub fn last_instance_id(&self) -> i32 {
        self.last_instance_id
    }

    /// Returns the last tile ID indicated by this file.
    ///
    /// In the editor, new tiles placed in rooms will increment this number to generate their ID.
    /// In a game, this number will be incremented for a new ID when calling functions like tile_add().
    /// In a new GMK file this number defaults to 10000000.
    #[inline]
    pub fn last_tile_id(&self) -> i32 {
        self.last_tile_id
    }

    /// Returns the Game Help Dialog belonging to this file.
    ///
    /// Note that this data is compressed in the game file, and decompression is not done in advance, nor is it cached.
    /// As such, it would be ideal to store the result of this function rather than calling it more than once.
    pub fn help_dialog(&self) -> io::Result<HelpDialog> {
        unsafe {
            let slice = self.data.get_unchecked(self.help_offset..(self.help_offset + self.help_len));
            let mut data = flate2::bufread::ZlibDecoder::new(slice);
            Ok(HelpDialog {
                bg_colour: data.read_u32::<LE>()?.into(),
                new_window: data.read_u32::<LE>()? != 0,
                caption: ByteString::read(&mut data)?,
                left: data.read_i32::<LE>()?,
                top: data.read_i32::<LE>()?,
                width: data.read_u32::<LE>()?,
                height: data.read_u32::<LE>()?,
                border: data.read_u32::<LE>()? != 0,
                resizable: data.read_u32::<LE>()? != 0,
                window_on_top: data.read_u32::<LE>()? != 0,
                freeze_game: data.read_u32::<LE>()? != 0,
                info: ByteString::read(&mut data)?,
            })
        }
    }

    /// Returns an iterator over the Library Initialization Strings found in this file.
    pub fn lib_init_strings(&self) -> LibInitStringParser {
        unsafe {
            let slice = self.data.get_unchecked(self.lib_init_offset..);
            LibInitStringParser::new(slice, self.lib_init_count)
        }
    }

    /// Returns an iterator over the Room IDs found in the Room Order in this file.
    pub fn room_order(&self) -> RoomOrderParser {
        unsafe {
            let slice = self.data.get_unchecked(self.room_order_offset..);
            RoomOrderParser::new(slice, self.room_order_count)
        }
    }
}

/// Using a Read object, skips over an asset block, returning the asset count and position of first asset.
/// Thisx will also parse the block's version header. An error will be returned if the version is not 800.
fn skip_asset_block(reader: &mut io::Cursor<&mut [u8]>) -> io::Result<AssetInfo> {
    if reader.read_u32::<LE>()? == 800 {
        let count = reader.read_u32::<LE>()?;
        let position = reader.position() as usize;
        for _ in 0..count {
            let len = reader.read_u32::<LE>()?;
            reader.seek(io::SeekFrom::Current(len.into()))?;
        }
        Ok(AssetInfo { count, position })
    } else {
        Err(io::Error::from(io::ErrorKind::InvalidData))
    }
}

/// Iterator over a given type of asset.
pub struct Parser<'a, A: Asset> {
    data: &'a [u8],
    count: u32,
    is_gmk: bool,
    _type: std::marker::PhantomData<A>,
}

impl<'a, A: Asset> Parser<'a, A> {
    fn new(data: &'a [u8], assets: AssetInfo, is_gmk: bool) -> Self {
        unsafe {
            Self {
                data: data.get_unchecked(assets.position..),
                count: assets.count,
                is_gmk,
                _type: std::marker::PhantomData,
            }
        }
    }
}

impl<A: Asset> Iterator for Parser<'_, A> {
    type Item = io::Result<Option<A>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count > 0 {
            unsafe {
                // TODO: this can't be Err, as the data has already been parsed successfully by `from_exe()`,
                // but there's no unchecked option for read_u32?
                let len = match self.data.read_u32::<LE>() {
                    Ok(l) => l,
                    Err(e) => return Some(Err(e)),
                };
                let cutoff = match usize::try_from(len) {
                    Ok(l) => l,
                    Err(_) => return Some(Err(io::Error::from(io::ErrorKind::InvalidInput))),
                };
                let mut t = io::BufReader::new(flate2::bufread::ZlibDecoder::new(self.data.get_unchecked(..cutoff)));
                let deserialize = if self.is_gmk { A::from_gmk } else { A::from_exe };
                let result = match t.read_u32::<LE>() {
                    Ok(0) => Ok(None),
                    Ok(_) => Some(deserialize(t)).transpose(),
                    Err(e) => Err(e),
                };
                self.count -= 1;
                self.data = self.data.get_unchecked(cutoff..);
                Some(result)
            }
        } else {
            None
        }
    }
}

/// Parallel iterator over a given type of asset.
#[cfg_attr(feature = "nightly-docs", doc(cfg(feature = "rayon")))]
#[cfg_attr(not(feature = "nightly-docs"), cfg(feature = "rayon"))]
pub struct ParallelParser<'a, A: Asset> {
    slices: Vec<&'a [u8]>,
    is_gmk: bool,
    _type: std::marker::PhantomData<A>,
}

#[cfg(feature = "rayon")]
impl<'a, A: Asset> ParallelParser<'a, A> {
    fn new(mut data: &'a [u8], assets: AssetInfo, is_gmk: bool) -> Self {
        unsafe {
            data = data.get_unchecked(assets.position..);
            let mut slices = Vec::with_capacity(assets.count as usize);
            for _ in 0..assets.count {
                let len = data.read_u32::<LE>().unwrap_unchecked();
                let (d1, d2) = data.split_at(len as usize);
                slices.push(d1);
                data = d2;
            }
            Self {
                slices,
                is_gmk,
                _type: std::marker::PhantomData,
            }
        }
    }
}

#[cfg(feature = "rayon")]
impl<'a, A: Asset + Send + Sync> ParallelIterator for ParallelParser<'a, A> {
    type Item = io::Result<Option<A>>;

    fn drive_unindexed<C: UnindexedConsumer<Self::Item>>(self, consumer: C) -> C::Result {
        self.slices.par_iter().map(|x| {
            let mut t = io::BufReader::new(flate2::bufread::ZlibDecoder::new(*x));
            let deserialize = if self.is_gmk { A::from_gmk } else { A::from_exe };
            match t.read_u32::<LE>() {
                Ok(0) => Ok(None),
                Ok(_) => Some(deserialize(t)).transpose(),
                Err(e) => Err(e),
            }
        }).drive_unindexed(consumer)
    }
}

#[derive(Clone, Copy)]
struct AssetInfo {
    count: u32,
    position: usize,
}

/// An iterator over Library Initialization Strings in a GMK. Created by [`Gmk::lib_init_strings`].
pub struct LibInitStringParser<'a> {
    data: &'a [u8],
    count: u32,
}

impl<'a> LibInitStringParser<'a> {
    fn new(data: &'a [u8], count: u32) -> Self {
        Self { data, count }
    }
}

impl<'a> Iterator for LibInitStringParser<'a> {
    type Item = ByteString;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count > 0 {
            self.count -= 1;
            Some(unsafe { ByteString::read(&mut self.data).unwrap_unchecked() })
        } else {
            None
        }
    }
}

/// An iterator over the room IDs in the Room Order in a GMK. Created by [`Gmk::room_order`].
pub struct RoomOrderParser<'a> {
    data: &'a [u8],
    count: u32,
}

impl<'a> RoomOrderParser<'a> {
    fn new(data: &'a [u8], count: u32) -> Self {
        Self { data, count }
    }
}

impl<'a> Iterator for RoomOrderParser<'a> {
    type Item = i32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count > 0 {
            self.count -= 1;
            unsafe { Some(self.data.read_i32::<LE>().unwrap_unchecked()) }
        } else {
            None
        }
    }
}

impl<'a, A: Asset> ExactSizeIterator for Parser<'a, A> { }
impl<'a, A: Asset> std::iter::FusedIterator for Parser<'a, A> { }

impl<'a> ExactSizeIterator for RoomOrderParser<'a> { }
impl<'a> std::iter::FusedIterator for RoomOrderParser<'a> { }
