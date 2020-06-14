use crate::{collision, zlib::ZlibWriter};
use flate2::{write::ZlibEncoder, Compression};
use gm8exe::{
    asset::{self, includedfile::ExportSetting, PascalString},
    settings::{GameHelpDialog, Settings},
    GameAssets, GameVersion,
};
use minio::WritePrimitives;
use rayon::prelude::*;
use std::{io, u32};

pub trait WritePascalString: WriteBuffer + minio::WritePrimitives {
    fn write_pas_string(&mut self, s: &PascalString) -> io::Result<usize> {
        self.write_u32_le(s.0.len() as u32).and_then(|x| self.write_buffer(s.0.as_ref()).map(|y| y + x))
    }
}
impl<W> WritePascalString for W where W: io::Write {}

pub trait WriteBuffer: io::Write {
    fn write_buffer(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.write_all(buf)?;
        Ok(buf.len())
    }
}
impl<W> WriteBuffer for W where W: io::Write {}

// Writes GMK file header
pub fn write_header<W>(writer: &mut W, version: GameVersion, game_id: u32, guid: [u32; 4]) -> io::Result<usize>
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
    settings: &Settings,
    ico_file: &[u8],
    version: GameVersion,
) -> io::Result<usize>
where
    W: io::Write,
{
    let mut result = writer.write_u32_le(800)?;
    let mut enc = ZlibWriter::new();
    enc.write_u32_le(settings.fullscreen as u32)?;
    enc.write_u32_le(settings.interpolate_pixels as u32)?;
    enc.write_u32_le(settings.dont_draw_border as u32)?;
    enc.write_u32_le(settings.display_cursor as u32)?;
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

        match &settings.backdata {
            Some(data) => {
                enc.write_u32_le(1)?;
                let mut backdata_enc = ZlibWriter::new();
                backdata_enc.write_buffer(&data)?;
                backdata_enc.finish(&mut enc)?;
            },
            None => {
                enc.write_u32_le(0)?;
            },
        }

        match &settings.frontdata {
            Some(data) => {
                enc.write_u32_le(1)?;
                let mut frontdata_enc = ZlibWriter::new();
                frontdata_enc.write_buffer(&data)?;
                frontdata_enc.finish(&mut enc)?;
            },
            None => {
                enc.write_u32_le(0)?;
            },
        }
    }

    match &settings.custom_load_image {
        Some(data) => {
            // In GMK format, the first bool is for whether there's a custom load image and the second is for
            // whether there's actually any data following it. There is only one bool in exe format, thus
            // we need to write two redundant "true"s here.
            enc.write_u32_le(1)?;
            enc.write_u32_le(1)?;
            let mut ci_enc = ZlibWriter::new();
            ci_enc.write_buffer(&data)?;
            ci_enc.finish(&mut enc)?;
        },
        None => {
            enc.write_u32_le(0)?;
        },
    }

    enc.write_u32_le(settings.transparent as u32)?;
    enc.write_u32_le(settings.translucency)?;
    enc.write_u32_le(settings.scale_progress_bar as u32)?;

    enc.write_u32_le(ico_file.len() as u32)?;
    enc.write_buffer(ico_file)?;

    enc.write_u32_le(settings.show_error_messages as u32)?;
    enc.write_u32_le(settings.log_errors as u32)?;
    enc.write_u32_le(settings.always_abort as u32)?;
    match version {
        GameVersion::GameMaker8_0 => enc.write_u32_le(settings.zero_uninitialized_vars as u32)?,
        GameVersion::GameMaker8_1 => enc.write_u32_le(
            ((settings.error_on_uninitialized_args as u32) << 1) | (settings.zero_uninitialized_vars as u32),
        )?,
    };

    enc.write_pas_string(&"decompiler clan :police_car: :police_car: :police_car:".into())?; // author
    enc.write_pas_string(&"".into())?; // version string
    write_timestamp(&mut enc)?; // timestamp
    enc.write_pas_string(&"".into())?; // information

    // TODO: extract all this stuff from .rsrc in gm8x
    enc.write_u32_le(1)?; // major version
    enc.write_u32_le(0)?; // minor version
    enc.write_u32_le(0)?; // release version
    enc.write_u32_le(0)?; // build version
    enc.write_pas_string(&"".into())?; // company
    enc.write_pas_string(&"".into())?; // product
    enc.write_pas_string(&"".into())?; // copyright info
    enc.write_pas_string(&"".into())?; // description
    write_timestamp(&mut enc)?; // timestamp

    result += enc.finish(writer)?;

    Ok(result)
}

// Helper fn - takes a set of assets from an iterator and passes them to the write function for that asset
pub fn write_asset_list<W, T, F>(
    writer: &mut W,
    list: &[Option<Box<T>>],
    write_fn: F,
    version: GameVersion,
    multithread: bool,
) -> io::Result<usize>
where
    T: Send + Sync,
    W: io::Write,
    F: Fn(&mut ZlibEncoder<Vec<u8>>, &T, GameVersion) -> io::Result<usize> + Send + Sync,
{
    let mut result = writer.write_u32_le(800)?;
    result += writer.write_u32_le(list.len() as u32)?;

    if multithread {
        result += list
            .par_iter()
            .map(|asset| {
                let mut enc = ZlibEncoder::new(Vec::new(), Compression::default());
                match asset {
                    Some(asset) => {
                        enc.write_u32_le(true as u32)?;
                        write_fn(&mut enc, asset, version)?;
                    },
                    None => {
                        enc.write_u32_le(false as u32)?;
                    },
                }
                enc.finish()
            })
            .collect::<Result<Vec<_>, io::Error>>()?
            .into_iter()
            .fold(Ok(0usize), |res: io::Result<_>, enc| {
                let mut result = writer.write_u32_le(enc.len() as u32)?;
                result += writer.write_buffer(&enc)?;
                res.map(|r| r + result)
            })?;
    } else {
        for asset in list {
            let mut enc = ZlibEncoder::new(Vec::new(), Compression::default());
            match asset {
                Some(asset) => {
                    enc.write_u32_le(true as u32)?;
                    write_fn(&mut enc, asset, version)?;
                },
                None => {
                    enc.write_u32_le(false as u32)?;
                },
            }
            let buf = enc.finish()?;
            result += writer.write_u32_le(buf.len() as u32)?;
            result += writer.write_buffer(&buf)?;
        }
    }

    Ok(result)
}

// Writes a trigger (uncompressed data)
pub fn write_trigger<W>(writer: &mut W, trigger: &asset::Trigger, _version: GameVersion) -> io::Result<usize>
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
pub fn write_sound<W>(writer: &mut W, sound: &asset::Sound, _version: GameVersion) -> io::Result<usize>
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
            result += writer.write_u32_le(data.len() as u32)?;
            result += writer.write_buffer(data)?;
        },
        None => {
            result += writer.write_u32_le(false as u32)?;
        },
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
pub fn write_sprite<W>(writer: &mut W, sprite: &asset::Sprite, _version: GameVersion) -> io::Result<usize>
where
    W: io::Write,
{
    let gmk_collision = collision::resolve_map(sprite);
    let mut result = writer.write_pas_string(&sprite.name)?;
    result += write_timestamp(writer)?;
    result += writer.write_u32_le(800)?;
    result += writer.write_i32_le(sprite.origin_x)?;
    result += writer.write_i32_le(sprite.origin_y)?;
    result += writer.write_u32_le(sprite.frames.len() as u32)?;
    for frame in &sprite.frames {
        result += writer.write_u32_le(800)?;
        result += writer.write_u32_le(frame.width)?;
        result += writer.write_u32_le(frame.height)?;
        if frame.width * frame.height != 0 {
            result += writer.write_u32_le(frame.data.len() as u32)?;
            result += writer.write_buffer(&frame.data)?;
        }
    }

    if let Some(map) = gmk_collision {
        result += writer.write_u32_le(map.shape as u32)?; // shape - 0 = precise
        result += writer.write_u32_le(map.alpha_tolerance)?; // alpha tolerance
        result += writer.write_u32_le(sprite.per_frame_colliders as u32)?;
        result += writer.write_u32_le(2)?; // bounding box type - 2 = manual
        result += writer.write_u32_le(map.bbox_left)?; // bbox left
        result += writer.write_u32_le(map.bbox_right)?; // bbox right
        result += writer.write_u32_le(map.bbox_bottom)?; // bbox bottom
        result += writer.write_u32_le(map.bbox_top)?; // bbox top
    } else {
        if !sprite.frames.is_empty() {
            println!("WARNING: couldn't resolve collision for sprite {}", sprite.name);
        }
        // Defaults
        result += writer.write_u32_le(0)?; // shape - 0 = precise
        result += writer.write_u32_le(0)?; // alpha tolerance
        result += writer.write_u32_le(sprite.per_frame_colliders as u32)?;
        result += writer.write_u32_le(2)?; // bounding box type - 2 = manual
        result += writer.write_u32_le(31)?; // bbox left
        result += writer.write_u32_le(0)?; // bbox right
        result += writer.write_u32_le(0)?; // bbox bottom
        result += writer.write_u32_le(31)?; // bbox top
    }
    Ok(result)
}

// Writes a Background (uncompressed data)
pub fn write_background<W>(writer: &mut W, background: &asset::Background, _version: GameVersion) -> io::Result<usize>
where
    W: io::Write,
{
    let mut result = writer.write_pas_string(&background.name)?;
    result += write_timestamp(writer)?;
    result += writer.write_u32_le(710)?;

    // Tileset info isn't in exe - not sure if there's a consistent way to reverse it...
    result += writer.write_u32_le(false as u32)?; // is tileset
    result += writer.write_u32_le(16)?; // tile width
    result += writer.write_u32_le(16)?; // tile height
    result += writer.write_u32_le(0)?; // H offset
    result += writer.write_u32_le(0)?; // V offset
    result += writer.write_u32_le(0)?; // H sep
    result += writer.write_u32_le(0)?; // V sep

    result += writer.write_u32_le(800)?;
    result += writer.write_u32_le(background.width)?;
    result += writer.write_u32_le(background.height)?;
    if background.width * background.height != 0 {
        if let Some(data) = &background.data {
            result += writer.write_u32_le(data.len() as u32)?;
            result += writer.write_buffer(&data)?;
        } else {
            result += writer.write_u32_le(0)?;
        }
    }
    Ok(result)
}

// Writes a Path (uncompressed data)
pub fn write_path<W>(writer: &mut W, path: &asset::Path, _version: GameVersion) -> io::Result<usize>
where
    W: io::Write,
{
    let mut result = writer.write_pas_string(&path.name)?;
    result += write_timestamp(writer)?;
    result += writer.write_u32_le(530)?;
    result += writer.write_u32_le(path.connection as u32)?;
    result += writer.write_u32_le(path.closed as u32)?;
    result += writer.write_u32_le(path.precision)?;
    result += writer.write_i32_le(-1)?; // Room to show as background in Path editor
    result += writer.write_u32_le(16)?; // Snap X
    result += writer.write_u32_le(16)?; // Snap Y
    result += writer.write_u32_le(path.points.len() as u32)?;
    for point in &path.points {
        result += writer.write_f64_le(point.x)?;
        result += writer.write_f64_le(point.y)?;
        result += writer.write_f64_le(point.speed)?;
    }
    Ok(result)
}

// Writes a Script (uncompressed data)
pub fn write_script<W>(writer: &mut W, script: &asset::Script, _version: GameVersion) -> io::Result<usize>
where
    W: io::Write,
{
    let mut result = writer.write_pas_string(&script.name)?;
    result += write_timestamp(writer)?;
    result += writer.write_u32_le(800)?;
    result += writer.write_pas_string(&script.source)?;
    Ok(result)
}

// Writes a Font (uncompressed data)
pub fn write_font<W>(writer: &mut W, font: &asset::Font, version: GameVersion) -> io::Result<usize>
where
    W: io::Write,
{
    let mut result = writer.write_pas_string(&font.name)?;
    result += write_timestamp(writer)?;
    result += writer.write_u32_le(800)?;
    result += writer.write_pas_string(&font.sys_name)?;
    result += writer.write_u32_le(font.size)?;
    result += writer.write_u32_le(font.bold as u32)?;
    result += writer.write_u32_le(font.italic as u32)?;
    result += match version {
        GameVersion::GameMaker8_0 => writer.write_u32_le(font.range_start)?,
        GameVersion::GameMaker8_1 => writer.write_u32_le(
            ((font.charset & 0xFF) << 24) | ((font.aa_level & 0xFF) << 16) | (font.range_start & 0xFFFF),
        )?,
    };
    result += writer.write_u32_le(font.range_end)?;
    Ok(result)
}

// Writes a DnD Code Action
pub fn write_action<W>(writer: &mut W, action: &asset::etc::CodeAction) -> io::Result<usize>
where
    W: io::Write,
{
    let mut result = writer.write_u32_le(440)?;
    result += writer.write_u32_le(action.lib_id)?;
    result += writer.write_u32_le(action.id)?;
    result += writer.write_u32_le(action.action_kind)?;
    result += writer.write_u32_le(action.can_be_relative)?;
    result += writer.write_u32_le(action.is_condition as u32)?;
    result += writer.write_u32_le(action.applies_to_something as u32)?;
    result += writer.write_u32_le(action.execution_type)?;
    result += writer.write_pas_string(&action.fn_name)?;
    result += writer.write_pas_string(&action.fn_code)?;

    result += writer.write_u32_le(action.param_count as u32)?;
    result += writer.write_u32_le(8)?;
    for i in &action.param_types {
        result += writer.write_u32_le(*i)?;
    }
    result += writer.write_i32_le(action.applies_to)?;
    result += writer.write_u32_le(action.is_relative as u32)?;
    result += writer.write_u32_le(8)?;
    for i in &action.param_strings {
        result += writer.write_pas_string(i)?;
    }
    result += writer.write_u32_le(action.invert_condition as u32)?;
    Ok(result)
}

// Writes a Timeline (uncompressed data)
pub fn write_timeline<W>(writer: &mut W, timeline: &asset::Timeline, _version: GameVersion) -> io::Result<usize>
where
    W: io::Write,
{
    let mut result = writer.write_pas_string(&timeline.name)?;
    result += write_timestamp(writer)?;
    result += writer.write_u32_le(500)?;
    result += writer.write_u32_le(timeline.moments.len() as u32)?;
    for (moment, actions) in &timeline.moments {
        result += writer.write_u32_le(*moment)?;
        result += writer.write_u32_le(400)?;
        result += writer.write_u32_le(actions.len() as u32)?;
        for action in actions {
            result += write_action(writer, action)?;
        }
    }
    Ok(result)
}

// Writes an Object (uncompressed data)
pub fn write_object<W>(writer: &mut W, object: &asset::Object, _version: GameVersion) -> io::Result<usize>
where
    W: io::Write,
{
    let mut result = writer.write_pas_string(&object.name)?;
    result += write_timestamp(writer)?;
    result += writer.write_u32_le(430)?;
    result += writer.write_i32_le(object.sprite_index)?;
    result += writer.write_u32_le(object.solid as u32)?;
    result += writer.write_u32_le(object.visible as u32)?;
    result += writer.write_i32_le(object.depth)?;
    result += writer.write_u32_le(object.persistent as u32)?;
    result += writer.write_i32_le(object.parent_index)?;
    result += writer.write_i32_le(object.mask_index)?;
    result += writer.write_u32_le(if object.events.is_empty() { 0 } else { (object.events.len() - 1) as u32 })?;
    for ev_list in &object.events {
        for (sub, actions) in ev_list {
            result += writer.write_u32_le(*sub)?;
            result += writer.write_u32_le(400)?;
            result += writer.write_u32_le(actions.len() as u32)?;
            for action in actions.iter() {
                result += write_action(writer, action)?;
            }
        }
        result += writer.write_i32_le(-1)?;
    }
    Ok(result)
}

// Writes an Room (uncompressed data)
pub fn write_room<W>(writer: &mut W, room: &asset::Room, _version: GameVersion) -> io::Result<usize>
where
    W: io::Write,
{
    let mut result = writer.write_pas_string(&room.name)?;
    result += write_timestamp(writer)?;
    result += writer.write_u32_le(541)?;
    result += writer.write_pas_string(&room.caption)?;
    result += writer.write_u32_le(room.width)?;
    result += writer.write_u32_le(room.height)?;
    result += writer.write_u32_le(32)?; // snap X
    result += writer.write_u32_le(32)?; // snap X
    result += writer.write_u32_le(false as u32)?; // isometric grid
    result += writer.write_u32_le(room.speed)?;
    result += writer.write_u32_le(room.persistent as u32)?;
    result += writer.write_u32_le(room.bg_colour.into())?;
    result += writer.write_u32_le(room.clear_screen as u32)?;
    result += writer.write_pas_string(&room.creation_code)?;

    result += writer.write_u32_le(room.backgrounds.len() as u32)?;
    for background in &room.backgrounds {
        result += writer.write_u32_le(background.visible_on_start as u32)?;
        result += writer.write_u32_le(background.is_foreground as u32)?;
        result += writer.write_i32_le(background.source_bg)?;
        result += writer.write_i32_le(background.xoffset)?;
        result += writer.write_i32_le(background.yoffset)?;
        result += writer.write_u32_le(background.tile_horz as u32)?;
        result += writer.write_u32_le(background.tile_vert as u32)?;
        result += writer.write_i32_le(background.hspeed)?;
        result += writer.write_i32_le(background.vspeed)?;
        result += writer.write_u32_le(background.stretch as u32)?;
    }

    result += writer.write_u32_le(room.views_enabled as u32)?;
    result += writer.write_u32_le(room.views.len() as u32)?;
    for view in &room.views {
        result += writer.write_u32_le(view.visible as u32)?;
        result += writer.write_i32_le(view.source_x)?;
        result += writer.write_i32_le(view.source_y)?;
        result += writer.write_u32_le(view.source_w)?;
        result += writer.write_u32_le(view.source_h)?;
        result += writer.write_i32_le(view.port_x)?;
        result += writer.write_i32_le(view.port_y)?;
        result += writer.write_u32_le(view.port_w)?;
        result += writer.write_u32_le(view.port_h)?;
        result += writer.write_i32_le(view.following.hborder)?;
        result += writer.write_i32_le(view.following.vborder)?;
        result += writer.write_i32_le(view.following.hspeed)?;
        result += writer.write_i32_le(view.following.vspeed)?;
        result += writer.write_i32_le(view.following.target)?;
    }

    result += writer.write_u32_le(room.instances.len() as u32)?;
    for instance in &room.instances {
        result += writer.write_i32_le(instance.x)?;
        result += writer.write_i32_le(instance.y)?;
        result += writer.write_i32_le(instance.object)?;
        result += writer.write_i32_le(instance.id)?;
        result += writer.write_pas_string(&instance.creation_code)?;
        result += writer.write_u32_le(false as u32)?; // locked in editor
    }

    result += writer.write_u32_le(room.tiles.len() as u32)?;
    for tile in &room.tiles {
        result += writer.write_i32_le(tile.x)?;
        result += writer.write_i32_le(tile.y)?;
        result += writer.write_i32_le(tile.source_bg)?;
        result += writer.write_u32_le(tile.tile_x)?;
        result += writer.write_u32_le(tile.tile_y)?;
        result += writer.write_u32_le(tile.width)?;
        result += writer.write_u32_le(tile.height)?;
        result += writer.write_i32_le(tile.depth)?;
        result += writer.write_i32_le(tile.id)?;
        result += writer.write_u32_le(false as u32)?; // locked in editor
    }

    // All these settings are 0/false by default when creating a new room in the IDE.
    // Commented with Zach's names for the variables, I haven't verified what they do
    result += writer.write_u32_le(false as u32)?; // remember room editor info
    result += writer.write_u32_le(0)?; // editor width
    result += writer.write_u32_le(0)?; // editor height
    result += writer.write_u32_le(false as u32)?; // show grid
    result += writer.write_u32_le(false as u32)?; // show objects
    result += writer.write_u32_le(false as u32)?; // show tiles
    result += writer.write_u32_le(false as u32)?; // show backgrounds
    result += writer.write_u32_le(false as u32)?; // show foregrounds
    result += writer.write_u32_le(false as u32)?; // show views
    result += writer.write_u32_le(false as u32)?; // delete underlying objects
    result += writer.write_u32_le(false as u32)?; // delete underlying tiles
    result += writer.write_u32_le(0)?; // tab
    result += writer.write_u32_le(0)?; // x position scroll
    result += writer.write_u32_le(0)?; // y position scroll

    Ok(result)
}

// Write GMK's room editor metadata
pub fn write_room_editor_meta<W>(writer: &mut W, last_instance_id: i32, last_tile_id: i32) -> io::Result<usize>
where
    W: io::Write,
{
    let mut result = writer.write_i32_le(last_instance_id)?;
    result += writer.write_i32_le(last_tile_id)?;
    Ok(result)
}

// Write included files to gmk
// Note: not compatible with write_asset_list because included files can't not exist
pub fn write_included_files<W>(writer: &mut W, files: &[asset::IncludedFile]) -> io::Result<usize>
where
    W: io::Write,
{
    let mut result = writer.write_u32_le(800)?;
    result += writer.write_u32_le(files.len() as u32)?;
    for file in files {
        let mut enc = ZlibWriter::new();
        write_timestamp(&mut enc)?;
        enc.write_u32_le(800)?;
        enc.write_pas_string(&file.file_name)?;
        enc.write_pas_string(&file.source_path)?;
        enc.write_u32_le(file.data_exists as u32)?;
        enc.write_u32_le(file.source_length as u32)?;
        enc.write_u32_le(file.stored_in_gmk as u32)?;
        if let Some(data) = &file.embedded_data {
            enc.write_u32_le(data.len() as u32)?;
            enc.write_buffer(data)?;
        }
        match &file.export_settings {
            ExportSetting::NoExport => {
                enc.write_u32_le(0)?;
                enc.write_pas_string(&"".into())?;
            },
            ExportSetting::TempFolder => {
                enc.write_u32_le(1)?;
                enc.write_pas_string(&"".into())?;
            },
            ExportSetting::GameFolder => {
                enc.write_u32_le(2)?;
                enc.write_pas_string(&"".into())?;
            },
            ExportSetting::CustomFolder(f) => {
                enc.write_u32_le(3)?;
                enc.write_pas_string(f)?;
            },
        }
        enc.write_u32_le(file.overwrite_file as u32)?;
        enc.write_u32_le(file.free_memory as u32)?;
        enc.write_u32_le(file.remove_at_end as u32)?;
        result += enc.finish(writer)?;
    }
    Ok(result)
}

// Write extensions to GMK (names only)
pub fn write_extensions<W>(writer: &mut W, extensions: &[asset::Extension]) -> io::Result<usize>
where
    W: io::Write,
{
    let mut result = writer.write_u32_le(700)?;
    result += writer.write_u32_le(extensions.len() as u32)?;
    for ext in extensions {
        result += writer.write_pas_string(&ext.name)?;
    }
    Ok(result)
}

// Write game information (help dialog) block to GMK
pub fn write_game_information<W>(writer: &mut W, info: &GameHelpDialog) -> io::Result<usize>
where
    W: io::Write,
{
    let mut result = writer.write_u32_le(800)?;
    let mut enc = ZlibWriter::new();
    enc.write_u32_le(info.bg_color.into())?;
    enc.write_u32_le(info.new_window as u32)?;
    enc.write_pas_string(&info.caption)?;
    enc.write_i32_le(info.left)?;
    enc.write_i32_le(info.top)?;
    enc.write_u32_le(info.width)?;
    enc.write_u32_le(info.height)?;
    enc.write_u32_le(info.border as u32)?;
    enc.write_u32_le(info.resizable as u32)?;
    enc.write_u32_le(info.window_on_top as u32)?;
    enc.write_u32_le(info.freeze_game as u32)?;
    write_timestamp(&mut enc)?;
    enc.write_pas_string(&info.info)?;
    result += enc.finish(writer)?;
    Ok(result)
}

// Write library initialization code strings to GMK
pub fn write_library_init_code<W>(writer: &mut W, init_code: &[PascalString]) -> io::Result<usize>
where
    W: io::Write,
{
    let mut result = writer.write_u32_le(500)?;
    result += writer.write_u32_le(init_code.len() as u32)?;
    for string in init_code {
        result += writer.write_pas_string(&string)?;
    }
    Ok(result)
}

// Write room order to GMK
pub fn write_room_order<W>(writer: &mut W, room_order: &[i32]) -> io::Result<usize>
where
    W: io::Write,
{
    let mut result = writer.write_u32_le(700)?;
    result += writer.write_u32_le(room_order.len() as u32)?;
    for room in room_order {
        result += writer.write_i32_le(*room)?;
    }
    Ok(result)
}

// Write resource tree to GMK
pub fn write_resource_tree<W>(writer: &mut W, assets: &GameAssets) -> io::Result<usize>
where
    W: io::Write,
{
    fn write_rt_heading<W>(writer: &mut W, name: &str, index: u32, count: usize) -> io::Result<usize>
    where
        W: io::Write,
    {
        let mut result = writer.write_u32_le(1)?;
        result += writer.write_u32_le(index)?;
        result += writer.write_u32_le(0)?;
        result += writer.write_pas_string(&name.into())?;
        result += writer.write_u32_le(count as u32)?;
        Ok(result)
    }

    fn write_rt_asset<W>(writer: &mut W, name: &PascalString, group: u32, index: u32) -> io::Result<usize>
    where
        W: io::Write,
    {
        let mut result = writer.write_u32_le(3)?;
        result += writer.write_u32_le(group)?;
        result += writer.write_u32_le(index)?;
        result += writer.write_pas_string(name)?;
        result += writer.write_u32_le(0)?;
        Ok(result)
    }

    /// Counts how many assets in the iterator are assets that "exist" in the gamedata.
    fn count_existing<'a, I, T>(iter: I) -> usize
    where
        I: IntoIterator<Item = &'a Option<T>>,
        T: 'a,
    {
        iter.into_iter().filter(|opt| opt.is_some()).count()
    }

    /// Enumerates "existing" assets with their index in the entire iterator (asset ID).
    fn enumerate_existing<'a, I, T>(iter: I) -> impl Iterator<Item = (usize, &'a T)>
    where
        I: IntoIterator<Item = &'a Option<T>>,
        T: 'a,
    {
        iter.into_iter().enumerate().filter_map(|(i, opt)| opt.as_ref().map(|x| (i, x)))
    }

    let mut result = write_rt_heading(writer, "Sprites", 2, count_existing(&assets.sprites))?;
    for (i, sprite) in enumerate_existing(&assets.sprites) {
        result += write_rt_asset(writer, &sprite.name, 2, i as u32)?;
    }
    result += write_rt_heading(writer, "Sounds", 3, count_existing(&assets.sounds))?;
    for (i, sound) in enumerate_existing(&assets.sounds) {
        result += write_rt_asset(writer, &sound.name, 3, i as u32)?;
    }
    result += write_rt_heading(writer, "Backgrounds", 6, count_existing(&assets.backgrounds))?;
    for (i, background) in enumerate_existing(&assets.backgrounds) {
        result += write_rt_asset(writer, &background.name, 6, i as u32)?;
    }
    result += write_rt_heading(writer, "Paths", 8, count_existing(&assets.paths))?;
    for (i, path) in enumerate_existing(&assets.paths) {
        result += write_rt_asset(writer, &path.name, 8, i as u32)?;
    }
    result += write_rt_heading(writer, "Scripts", 7, count_existing(&assets.scripts))?;
    for (i, script) in enumerate_existing(&assets.scripts) {
        result += write_rt_asset(writer, &script.name, 7, i as u32)?;
    }
    result += write_rt_heading(writer, "Fonts", 9, count_existing(&assets.fonts))?;
    for (i, font) in enumerate_existing(&assets.fonts) {
        result += write_rt_asset(writer, &font.name, 9, i as u32)?;
    }
    result += write_rt_heading(writer, "Time Lines", 12, count_existing(&assets.timelines))?;
    for (i, timeline) in enumerate_existing(&assets.timelines) {
        result += write_rt_asset(writer, &timeline.name, 12, i as u32)?;
    }
    result += write_rt_heading(writer, "Objects", 1, count_existing(&assets.objects))?;
    for (i, object) in enumerate_existing(&assets.objects) {
        result += write_rt_asset(writer, &object.name, 1, i as u32)?;
    }
    result += write_rt_heading(writer, "Rooms", 4, count_existing(&assets.rooms))?;
    for room_id in &assets.room_order {
        if let Some(Some(room)) = assets.rooms.get(*room_id as usize) {
            result += write_rt_asset(writer, &room.name, 4, *room_id as u32)?;
        } else {
            println!("WARNING: non-existent room id {} referenced in Room Order; skipping it", *room_id);
        }
    }
    write_rt_asset(writer, &"Game Information".into(), 10, 0)?;
    write_rt_asset(writer, &"Global Game Settings".into(), 11, 0)?;
    write_rt_asset(writer, &"Extension Packages".into(), 13, 0)?;
    Ok(result)
}
