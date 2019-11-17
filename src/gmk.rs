use crate::zlib::ZlibWriter;
use gm8x::reader::Settings;
use gm8x::{asset, GameVersion};
use minio::WritePrimitives;
use std::io::{self, Write};
use std::u32;

pub trait WritePascalString: io::Write + minio::WritePrimitives {
    fn write_pas_string(&mut self, s: &str) -> io::Result<usize> {
        self.write_u32_le(s.len() as u32)
            .and_then(|x| self.write(s.as_bytes()).map(|y| y + x))
    }
}
impl<W> WritePascalString for W where W: io::Write {}

// Writes GMK file header
pub fn write_header<W>(
    writer: &mut W,
    version: GameVersion,
    game_id: u32,
    guid: [u32; 4],
) -> io::Result<usize>
where
    W: io::Write,
{
    let mut result = writer.write_u32_le(1234321)?;
    result += writer.write_u32_le(match version {
        GameVersion::GameMaker8_0 => 800,
        GameVersion::GameMaker8_1 => 810,
    })?;
    result += writer.write_u32_le(game_id)?;
    for n in &guid {
        result += writer.write_u32_le(*n)?;
    }
    Ok(result)
}

// Write a timestamp (currently writes 0, which correlates to 1899-01-01)
#[inline]
pub fn write_timestamp<W>(writer: &mut W) -> io::Result<usize>
where
    W: io::Write,
{
    writer.write_u64_le(0)
}

// Writes a settings block to GMK
pub fn write_settings<W>(
    writer: &mut W,
    settings: Settings,
    ico_file: Vec<u8>,
    version: GameVersion,
) -> io::Result<usize>
where
    W: io::Write,
{
    let mut result = writer.write_u32_le(800)?;
    let mut enc = ZlibWriter::new();
    enc.write_u32_le(settings.fullscreen as u32)?;
    enc.write_u32_le(settings.dont_draw_border as u32)?;
    enc.write_u32_le(settings.display_cursor as u32)?;
    enc.write_u32_le(settings.interpolate_pixels as u32)?;
    enc.write_i32_le(settings.scaling)?;
    enc.write_u32_le(settings.allow_resize as u32)?;
    enc.write_u32_le(settings.window_on_top as u32)?;
    enc.write_u32_le(settings.clear_colour)?;
    enc.write_u32_le(settings.set_resolution as u32)?;
    enc.write_u32_le(settings.colour_depth)?;
    enc.write_u32_le(settings.resolution)?;
    enc.write_u32_le(settings.frequency)?;
    enc.write_u32_le(settings.dont_show_buttons as u32)?;
    enc.write_u32_le(settings.vsync as u32)?;
    enc.write_u32_le(settings.disable_screensaver as u32)?;
    enc.write_u32_le(settings.f4_fullscreen_toggle as u32)?;
    enc.write_u32_le(settings.f1_help_menu as u32)?;
    enc.write_u32_le(settings.esc_close_game as u32)?;
    enc.write_u32_le(settings.f5_save_f6_load as u32)?;
    enc.write_u32_le(settings.f9_screenshot as u32)?;
    enc.write_u32_le(settings.treat_close_as_esc as u32)?;
    enc.write_u32_le(settings.priority)?;
    enc.write_u32_le(settings.freeze_on_lose_focus as u32)?;

    enc.write_u32_le(settings.loading_bar)?;
    if settings.loading_bar == 2 {
        // 2 = custom loading bar - otherwise don't write anything here

        match settings.backdata {
            Some(data) => {
                enc.write_u32_le(1)?;
                let mut backdata_enc = ZlibWriter::new();
                backdata_enc.write_all(&data)?;
                backdata_enc.finish(&mut enc)?;
            }
            None => {
                enc.write_u32_le(0)?;
            }
        }

        match settings.frontdata {
            Some(data) => {
                enc.write_u32_le(1)?;
                let mut frontdata_enc = ZlibWriter::new();
                frontdata_enc.write_all(&data)?;
                frontdata_enc.finish(&mut enc)?;
            }
            None => {
                enc.write_u32_le(0)?;
            }
        }
    }

    match settings.custom_load_image {
        Some(data) => {
            // In GMK format, the first bool is for whether there's a custom load image and the second is for
            // whether there's actually any data following it. There is only one bool in exe format, thus
            // we need to write two redundant "true"s here.
            enc.write_u32_le(1)?;
            enc.write_u32_le(1)?;
            let mut ci_enc = ZlibWriter::new();
            ci_enc.write_all(&data)?;
            ci_enc.finish(&mut enc)?;
        }
        None => {
            enc.write_u32_le(0)?;
        }
    }

    enc.write_u32_le(settings.transparent as u32)?;
    enc.write_u32_le(settings.translucency)?;
    enc.write_u32_le(settings.scale_progress_bar as u32)?;

    enc.write_u32_le(ico_file.len() as u32)?;
    enc.write_all(&ico_file)?;

    enc.write_u32_le(settings.show_error_messages as u32)?;
    enc.write_u32_le(settings.log_errors as u32)?;
    enc.write_u32_le(settings.always_abort as u32)?;
    match version {
        GameVersion::GameMaker8_0 => enc.write_u32_le(settings.zero_uninitalized_vars as u32)?,
        GameVersion::GameMaker8_1 => enc.write_u32_le(
            ((settings.error_on_uninitalized_args as u32) << 1)
                | (settings.zero_uninitalized_vars as u32),
        )?,
    };

    enc.write_pas_string("decompiler clan :police_car: :police_car: :police_car:")?; // author
    enc.write_pas_string("")?; // version string
    write_timestamp(&mut enc)?; // timestamp
    enc.write_pas_string("")?; // information

    // TODO: extract all this stuff from .rsrc in gm8x
    enc.write_u32_le(1)?; // major version
    enc.write_u32_le(0)?; // minor version
    enc.write_u32_le(0)?; // release version
    enc.write_u32_le(0)?; // build version
    enc.write_pas_string("")?; // company
    enc.write_pas_string("")?; // product
    enc.write_pas_string("")?; // copyright info
    enc.write_pas_string("")?; // description
    write_timestamp(&mut enc)?; // timestamp

    result += enc.finish(writer)?;

    Ok(result)
}

// Helper fn - takes a set of assets from an iterator and passes them to the write function for that asset
pub fn write_asset_list<W, T, F>(
    writer: &mut W,
    list: &[Option<Box<T>>],
    write_fn: F,
    gmk_version: u32,
    version: GameVersion,
) -> io::Result<usize>
where
    W: io::Write,
    F: Fn(&mut ZlibWriter, &T, GameVersion) -> io::Result<usize>,
{
    let mut result = writer.write_u32_le(gmk_version)?;
    result += writer.write_u32_le(list.len() as u32)?;
    for asset in list.iter() {
        let mut enc = ZlibWriter::new();
        match asset {
            Some(a) => {
                enc.write_u32_le(true as u32)?;
                write_fn(&mut enc, a.as_ref(), version)?;
            }
            None => {
                enc.write_u32_le(false as u32)?;
            }
        };
        result += enc.finish(writer)?;
    }
    Ok(result)
}

// Writes a trigger (uncompressed data)
pub fn write_trigger<W>(
    writer: &mut W,
    trigger: &asset::Trigger,
    _version: GameVersion,
) -> io::Result<usize>
where
    W: io::Write,
{
    let mut result = writer.write_u32_le(800)?;
    result += writer.write_pas_string(&trigger.name)?;
    result += writer.write_pas_string(&trigger.condition)?;
    result += writer.write_u32_le(trigger.moment as u32)?;
    result += writer.write_pas_string(&trigger.constant_name)?;
    Ok(result)
}

// Writes a list of constants
// This isn't compatible with write_asset_list because constants have a different, simpler format than most assets.
pub fn write_constants<W>(writer: &mut W, constants: &[asset::Constant]) -> io::Result<usize>
where
    W: io::Write,
{
    let mut result = writer.write_u32_le(800)?;
    result += writer.write_u32_le(constants.len() as u32)?;
    for constant in constants {
        result += writer.write_pas_string(&constant.name)?;
        result += writer.write_pas_string(&constant.expression)?;
    }
    result += write_timestamp(writer)?;
    Ok(result)
}

// Writes a Sound (uncompressed data)
pub fn write_sound<W>(
    writer: &mut W,
    sound: &asset::Sound,
    _version: GameVersion,
) -> io::Result<usize>
where
    W: io::Write,
{
    let mut result = writer.write_pas_string(&sound.name)?;
    result += write_timestamp(writer)?;
    result += writer.write_u32_le(800)?;
    result += writer.write_u32_le(sound.kind as u32)?;
    result += writer.write_pas_string(&sound.extension)?;
    result += writer.write_pas_string(&sound.source)?;
    match &sound.data {
        Some(data) => {
            result += writer.write_u32_le(true as u32)?;
            result += writer.write(data)?;
        }
        None => {
            result += writer.write_u32_le(false as u32)?;
        }
    }
    result += writer.write_u32_le(
        (sound.fx.chorus as u32)
            | (sound.fx.echo as u32) << 1
            | (sound.fx.flanger as u32) << 2
            | (sound.fx.gargle as u32) << 3
            | (sound.fx.reverb as u32) << 4,
    )?;
    result += writer.write_f64_le(sound.volume)?;
    result += writer.write_f64_le(sound.pan)?;
    result += writer.write_u32_le(sound.preload as u32)?;

    Ok(result)
}

// Writes a Sprite (uncompressed data)
pub fn write_sprite<W>(
    writer: &mut W,
    sprite: &asset::Sprite,
    _version: GameVersion,
) -> io::Result<usize>
where
    W: io::Write,
{
    let mut result = writer.write_pas_string(&sprite.name)?;
    result += write_timestamp(writer)?;
    result += writer.write_u32_le(800)?;
    result += writer.write_i32_le(sprite.origin_x)?;
    result += writer.write_i32_le(sprite.origin_y)?;
    result += writer.write_u32_le(sprite.frames.len() as u32)?;
    for (i, frame) in sprite.frames.iter().enumerate() {
        result += writer.write_u32_le(800)?;
        result += writer.write_u32_le(frame.width)?;
        result += writer.write_u32_le(frame.height)?;
        if frame.width * frame.height != 0 {
            let mut enc = ZlibWriter::new();
            enc.write_all(&frame.data)?;
            result += enc.finish(writer)?;
        }
        // TODO: calculate shape and alpha tolerance, bounding box type
        result += writer.write_u32_le(2)?; // shape
        result += writer.write_u32_le(0)?; // alpha tolerance
        result += writer.write_u32_le(sprite.per_frame_colliders as u32)?;
        result += writer.write_u32_le(2)?; // bounding box type - 2 = manual
        let collision_data = sprite
            .colliders
            .get(if sprite.per_frame_colliders { i } else { 0 })
            .unwrap();
        result += writer.write_u32_le(collision_data.bbox_left)?;
        result += writer.write_u32_le(collision_data.bbox_right)?;
        result += writer.write_u32_le(collision_data.bbox_bottom)?;
        result += writer.write_u32_le(collision_data.bbox_top)?;
    }
    Ok(result)
}
