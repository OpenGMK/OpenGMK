use crate::{collision, zlib::ZlibWriter};
use byteorder::{WriteBytesExt, LE};
use flate2::{write::ZlibEncoder, Compression};
use gm8exe::{
    asset::{self, included_file::ExportSetting, PascalString, WritePascalString},
    settings::{GameHelpDialog, Settings},
    GameAssets, GameVersion,
};
use rayon::prelude::*;
use std::{io, u32};

pub trait WriteBuffer: io::Write {
    fn write_buffer(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.write_all(buf)?;
        Ok(buf.len())
    }
}
impl<W> WriteBuffer for W where W: io::Write {}

// Writes GMK file header
pub fn write_header<W>(writer: &mut W, version: GameVersion, game_id: u32, guid: [u32; 4]) -> io::Result<()>
where
    W: io::Write,
{
    writer.write_u32::<LE>(1234321)?;
    writer.write_u32::<LE>(match version {
        GameVersion::GameMaker8_0 => 800,
        GameVersion::GameMaker8_1 => 810,
    })?;
    writer.write_u32::<LE>(game_id)?;
    for n in &guid {
        writer.write_u32::<LE>(*n)?;
    }
    Ok(())
}

// Write a timestamp (currently writes 0, which correlates to 1899-01-01)
#[inline]
pub fn write_timestamp<W>(writer: &mut W) -> io::Result<()>
where
    W: io::Write,
{
    writer.write_u64::<LE>(0)
}

// Writes a settings block to GMK
pub fn write_settings<W>(
    writer: &mut W,
    settings: &Settings,
    ico_file: Option<Vec<u8>>,
    version: GameVersion,
) -> io::Result<()>
where
    W: io::Write,
{
    writer.write_u32::<LE>(800)?;
    let mut enc = ZlibWriter::new();
    enc.write_u32::<LE>(settings.fullscreen as u32)?;
    enc.write_u32::<LE>(settings.interpolate_pixels as u32)?;
    enc.write_u32::<LE>(settings.dont_draw_border as u32)?;
    enc.write_u32::<LE>(settings.display_cursor as u32)?;
    enc.write_i32::<LE>(settings.scaling)?;
    enc.write_u32::<LE>(settings.allow_resize as u32)?;
    enc.write_u32::<LE>(settings.window_on_top as u32)?;
    enc.write_u32::<LE>(settings.clear_colour)?;
    enc.write_u32::<LE>(settings.set_resolution as u32)?;
    enc.write_u32::<LE>(settings.colour_depth)?;
    enc.write_u32::<LE>(settings.resolution)?;
    enc.write_u32::<LE>(settings.frequency)?;
    enc.write_u32::<LE>(settings.dont_show_buttons as u32)?;
    match version {
        GameVersion::GameMaker8_0 => enc.write_u32::<LE>(settings.vsync as u32)?,
        GameVersion::GameMaker8_1 => {
            enc.write_u32::<LE>(((settings.force_cpu_render as u32) << 7) | (settings.vsync as u32))?
        },
    };
    enc.write_u32::<LE>(settings.disable_screensaver as u32)?;
    enc.write_u32::<LE>(settings.f4_fullscreen_toggle as u32)?;
    enc.write_u32::<LE>(settings.f1_help_menu as u32)?;
    enc.write_u32::<LE>(settings.esc_close_game as u32)?;
    enc.write_u32::<LE>(settings.f5_save_f6_load as u32)?;
    enc.write_u32::<LE>(settings.f9_screenshot as u32)?;
    enc.write_u32::<LE>(settings.treat_close_as_esc as u32)?;
    enc.write_u32::<LE>(settings.priority)?;
    enc.write_u32::<LE>(settings.freeze_on_lose_focus as u32)?;

    enc.write_u32::<LE>(settings.loading_bar)?;
    if settings.loading_bar == 2 {
        // 2 = custom loading bar - otherwise don't write anything here

        match &settings.backdata {
            Some(data) => {
                enc.write_u32::<LE>(1)?;
                let mut backdata_enc = ZlibWriter::new();
                backdata_enc.write_buffer(&data)?;
                backdata_enc.finish(&mut enc)?;
            },
            None => {
                enc.write_u32::<LE>(0)?;
            },
        }

        match &settings.frontdata {
            Some(data) => {
                enc.write_u32::<LE>(1)?;
                let mut frontdata_enc = ZlibWriter::new();
                frontdata_enc.write_buffer(&data)?;
                frontdata_enc.finish(&mut enc)?;
            },
            None => {
                enc.write_u32::<LE>(0)?;
            },
        }
    }

    match &settings.custom_load_image {
        Some(data) => {
            // In GMK format, the first bool is for whether there's a custom load image and the second is for
            // whether there's actually any data following it. There is only one bool in exe format, thus
            // we need to write two redundant "true"s here.
            enc.write_u32::<LE>(1)?;
            enc.write_u32::<LE>(1)?;
            let mut ci_enc = ZlibWriter::new();
            ci_enc.write_buffer(&data)?;
            ci_enc.finish(&mut enc)?;
        },
        None => {
            enc.write_u32::<LE>(0)?;
        },
    }

    enc.write_u32::<LE>(settings.transparent as u32)?;
    enc.write_u32::<LE>(settings.translucency)?;
    enc.write_u32::<LE>(settings.scale_progress_bar as u32)?;

    if let Some(ico) = ico_file {
        enc.write_u32::<LE>(ico.len() as u32)?;
        enc.write_buffer(&ico)?;
    } else {
        enc.write_u32::<LE>(0)?;
    }

    enc.write_u32::<LE>(settings.show_error_messages as u32)?;
    enc.write_u32::<LE>(settings.log_errors as u32)?;
    enc.write_u32::<LE>(settings.always_abort as u32)?;
    match version {
        GameVersion::GameMaker8_0 => enc.write_u32::<LE>(settings.zero_uninitialized_vars as u32)?,
        GameVersion::GameMaker8_1 => enc.write_u32::<LE>(
            ((settings.error_on_uninitialized_args as u32) << 1) | (settings.zero_uninitialized_vars as u32),
        )?,
    };

    enc.write_pas_string(&"decompiler clan :police_car: :police_car: :police_car:".into())?; // author
    enc.write_pas_string(&"".into())?; // version string
    write_timestamp(&mut enc)?; // timestamp
    enc.write_pas_string(&"".into())?; // information

    // TODO: extract all this stuff from .rsrc in gm8x
    enc.write_u32::<LE>(1)?; // major version
    enc.write_u32::<LE>(0)?; // minor version
    enc.write_u32::<LE>(0)?; // release version
    enc.write_u32::<LE>(0)?; // build version
    enc.write_pas_string(&"".into())?; // company
    enc.write_pas_string(&"".into())?; // product
    enc.write_pas_string(&"".into())?; // copyright info
    enc.write_pas_string(&"".into())?; // description
    write_timestamp(&mut enc)?; // timestamp

    enc.finish(writer)?;

    Ok(())
}

// Helper fn - takes a set of assets from an iterator and passes them to the write function for that asset
pub fn write_asset_list<W, T, F>(
    writer: &mut W,
    list: &[Option<Box<T>>],
    write_fn: F,
    version: GameVersion,
    multithread: bool,
) -> io::Result<()>
where
    T: Send + Sync,
    W: io::Write,
    F: Fn(&mut ZlibEncoder<Vec<u8>>, &T, GameVersion) -> io::Result<()> + Send + Sync,
{
    writer.write_u32::<LE>(800)?;
    writer.write_u32::<LE>(list.len() as u32)?;

    if multithread {
        list.par_iter()
            .map(|asset| {
                let mut enc = ZlibEncoder::new(Vec::new(), Compression::default());
                match asset {
                    Some(asset) => {
                        enc.write_u32::<LE>(true as u32)?;
                        write_fn(&mut enc, asset, version)?;
                    },
                    None => {
                        enc.write_u32::<LE>(false as u32)?;
                    },
                }
                enc.finish()
            })
            .collect::<Result<Vec<_>, io::Error>>()?
            .into_iter()
            .try_fold((), |_, enc| {
                writer.write_u32::<LE>(enc.len().try_into().unwrap())?;
                writer.write_buffer(&enc)?;
                Ok(())
            })
    } else {
        for asset in list {
            let mut enc = ZlibEncoder::new(Vec::new(), Compression::default());
            match asset {
                Some(asset) => {
                    enc.write_u32::<LE>(true as u32)?;
                    write_fn(&mut enc, asset, version)?;
                },
                None => {
                    enc.write_u32::<LE>(false as u32)?;
                },
            }
            let buf = enc.finish()?;
            writer.write_u32::<LE>(buf.len() as u32)?;
            writer.write_buffer(&buf)?;
        }
        Ok(())
    }
}

// Writes a trigger (uncompressed data)
pub fn write_trigger<W>(writer: &mut W, trigger: &asset::Trigger, _version: GameVersion) -> io::Result<()>
where
    W: io::Write,
{
    writer.write_u32::<LE>(800)?;
    writer.write_pas_string(&trigger.name)?;
    writer.write_pas_string(&trigger.condition)?;
    writer.write_u32::<LE>(trigger.moment as u32)?;
    writer.write_pas_string(&trigger.constant_name)?;
    Ok(())
}

// Writes a list of constants
// This isn't compatible with write_asset_list because constants have a different, simpler format than most assets.
pub fn write_constants<W>(writer: &mut W, constants: &[asset::Constant]) -> io::Result<()>
where
    W: io::Write,
{
    writer.write_u32::<LE>(800)?;
    writer.write_u32::<LE>(constants.len() as u32)?;
    for constant in constants {
        writer.write_pas_string(&constant.name)?;
        writer.write_pas_string(&constant.expression)?;
    }
    write_timestamp(writer)?;
    Ok(())
}

// Writes a Sound (uncompressed data)
pub fn write_sound<W>(writer: &mut W, sound: &asset::Sound, _version: GameVersion) -> io::Result<()>
where
    W: io::Write,
{
    writer.write_pas_string(&sound.name)?;
    write_timestamp(writer)?;
    writer.write_u32::<LE>(800)?;
    writer.write_u32::<LE>(sound.kind as u32)?;
    writer.write_pas_string(&sound.extension)?;
    writer.write_pas_string(&sound.source)?;
    match &sound.data {
        Some(data) => {
            writer.write_u32::<LE>(true as u32)?;
            writer.write_u32::<LE>(data.len() as u32)?;
            writer.write_buffer(data)?;
        },
        None => {
            writer.write_u32::<LE>(false as u32)?;
        },
    }
    writer.write_u32::<LE>(
        (sound.fx.chorus as u32)
            | (sound.fx.echo as u32) << 1
            | (sound.fx.flanger as u32) << 2
            | (sound.fx.gargle as u32) << 3
            | (sound.fx.reverb as u32) << 4,
    )?;
    writer.write_f64::<LE>(sound.volume)?;
    writer.write_f64::<LE>(sound.pan)?;
    writer.write_u32::<LE>(sound.preload as u32)?;

    Ok(())
}

// Writes a Sprite (uncompressed data)
pub fn write_sprite<W>(writer: &mut W, sprite: &asset::Sprite, _version: GameVersion) -> io::Result<()>
where
    W: io::Write,
{
    let gmk_collision = collision::resolve_map(sprite);
    writer.write_pas_string(&sprite.name)?;
    write_timestamp(writer)?;
    writer.write_u32::<LE>(800)?;
    writer.write_i32::<LE>(sprite.origin_x)?;
    writer.write_i32::<LE>(sprite.origin_y)?;
    writer.write_u32::<LE>(sprite.frames.len() as u32)?;
    for frame in &sprite.frames {
        writer.write_u32::<LE>(800)?;
        writer.write_u32::<LE>(frame.width)?;
        writer.write_u32::<LE>(frame.height)?;
        if frame.width * frame.height != 0 {
            writer.write_u32::<LE>(frame.data.len() as u32)?;
            writer.write_buffer(&frame.data)?;
        }
    }

    if let Some(map) = gmk_collision {
        writer.write_u32::<LE>(map.shape as u32)?; // shape - 0 = precise
        writer.write_u32::<LE>(map.alpha_tolerance)?; // alpha tolerance
        writer.write_u32::<LE>(sprite.per_frame_colliders as u32)?;
        writer.write_u32::<LE>(2)?; // bounding box type - 2 = manual
        writer.write_u32::<LE>(map.bbox_left)?; // bbox left
        writer.write_u32::<LE>(map.bbox_right)?; // bbox right
        writer.write_u32::<LE>(map.bbox_bottom)?; // bbox bottom
        writer.write_u32::<LE>(map.bbox_top)?; // bbox top
    } else {
        if !sprite.frames.is_empty() {
            println!("WARNING: couldn't resolve collision for sprite {}", sprite.name);
        }
        // Defaults
        writer.write_u32::<LE>(0)?; // shape - 0 = precise
        writer.write_u32::<LE>(0)?; // alpha tolerance
        writer.write_u32::<LE>(sprite.per_frame_colliders as u32)?;
        writer.write_u32::<LE>(2)?; // bounding box type - 2 = manual
        writer.write_u32::<LE>(31)?; // bbox left
        writer.write_u32::<LE>(0)?; // bbox right
        writer.write_u32::<LE>(0)?; // bbox bottom
        writer.write_u32::<LE>(31)?; // bbox top
    }
    Ok(())
}

// Writes a Background (uncompressed data)
pub fn write_background<W>(writer: &mut W, background: &asset::Background, _version: GameVersion) -> io::Result<()>
where
    W: io::Write,
{
    writer.write_pas_string(&background.name)?;
    write_timestamp(writer)?;
    writer.write_u32::<LE>(710)?;

    // Tileset info isn't in exe - not sure if there's a consistent way to reverse it...
    writer.write_u32::<LE>(false as u32)?; // is tileset
    writer.write_u32::<LE>(16)?; // tile width
    writer.write_u32::<LE>(16)?; // tile height
    writer.write_u32::<LE>(0)?; // H offset
    writer.write_u32::<LE>(0)?; // V offset
    writer.write_u32::<LE>(0)?; // H sep
    writer.write_u32::<LE>(0)?; // V sep

    writer.write_u32::<LE>(800)?;
    writer.write_u32::<LE>(background.width)?;
    writer.write_u32::<LE>(background.height)?;
    if background.width * background.height != 0 {
        if let Some(data) = &background.data {
            writer.write_u32::<LE>(data.len() as u32)?;
            writer.write_buffer(&data)?;
        } else {
            writer.write_u32::<LE>(0)?;
        }
    }
    Ok(())
}

// Writes a Path (uncompressed data)
pub fn write_path<W>(writer: &mut W, path: &asset::Path, _version: GameVersion) -> io::Result<()>
where
    W: io::Write,
{
    writer.write_pas_string(&path.name)?;
    write_timestamp(writer)?;
    writer.write_u32::<LE>(530)?;
    writer.write_u32::<LE>(path.connection as u32)?;
    writer.write_u32::<LE>(path.closed as u32)?;
    writer.write_u32::<LE>(path.precision)?;
    writer.write_i32::<LE>(-1)?; // Room to show as background in Path editor
    writer.write_u32::<LE>(16)?; // Snap X
    writer.write_u32::<LE>(16)?; // Snap Y
    writer.write_u32::<LE>(path.points.len() as u32)?;
    for point in &path.points {
        writer.write_f64::<LE>(point.x)?;
        writer.write_f64::<LE>(point.y)?;
        writer.write_f64::<LE>(point.speed)?;
    }
    Ok(())
}

// Writes a Script (uncompressed data)
pub fn write_script<W>(writer: &mut W, script: &asset::Script, _version: GameVersion) -> io::Result<()>
where
    W: io::Write,
{
    writer.write_pas_string(&script.name)?;
    write_timestamp(writer)?;
    writer.write_u32::<LE>(800)?;
    writer.write_pas_string(&script.source)?;
    Ok(())
}

// Writes a Font (uncompressed data)
pub fn write_font<W>(writer: &mut W, font: &asset::Font, version: GameVersion) -> io::Result<()>
where
    W: io::Write,
{
    writer.write_pas_string(&font.name)?;
    write_timestamp(writer)?;
    writer.write_u32::<LE>(800)?;
    writer.write_pas_string(&font.sys_name)?;
    writer.write_u32::<LE>(font.size)?;
    writer.write_u32::<LE>(font.bold as u32)?;
    writer.write_u32::<LE>(font.italic as u32)?;
    match version {
        GameVersion::GameMaker8_0 => writer.write_u32::<LE>(font.range_start)?,
        GameVersion::GameMaker8_1 => writer.write_u32::<LE>(
            ((font.aa_level & 0xFF) << 24) | ((font.charset & 0xFF) << 16) | (font.range_start & 0xFFFF),
        )?,
    };
    writer.write_u32::<LE>(font.range_end)?;
    Ok(())
}

// Writes a DnD Code Action
pub fn write_action<W>(writer: &mut W, action: &asset::CodeAction) -> io::Result<()>
where
    W: io::Write,
{
    writer.write_u32::<LE>(440)?;
    writer.write_u32::<LE>(action.lib_id)?;
    writer.write_u32::<LE>(action.id)?;
    writer.write_u32::<LE>(action.action_kind)?;
    writer.write_u32::<LE>(action.can_be_relative)?;
    writer.write_u32::<LE>(action.is_condition as u32)?;
    writer.write_u32::<LE>(action.applies_to_something as u32)?;
    writer.write_u32::<LE>(action.execution_type)?;
    writer.write_pas_string(&action.fn_name)?;
    writer.write_pas_string(&action.fn_code)?;

    writer.write_u32::<LE>(action.param_count as u32)?;
    writer.write_u32::<LE>(8)?;
    for i in &action.param_types {
        writer.write_u32::<LE>(*i)?;
    }
    writer.write_i32::<LE>(action.applies_to)?;
    writer.write_u32::<LE>(action.is_relative as u32)?;
    writer.write_u32::<LE>(8)?;
    for i in &action.param_strings {
        writer.write_pas_string(i)?;
    }
    writer.write_u32::<LE>(action.invert_condition as u32)?;
    Ok(())
}

// Writes a Timeline (uncompressed data)
pub fn write_timeline<W>(writer: &mut W, timeline: &asset::Timeline, _version: GameVersion) -> io::Result<()>
where
    W: io::Write,
{
    writer.write_pas_string(&timeline.name)?;
    write_timestamp(writer)?;
    writer.write_u32::<LE>(500)?;
    writer.write_u32::<LE>(timeline.moments.len() as u32)?;
    for (moment, actions) in &timeline.moments {
        writer.write_u32::<LE>(*moment)?;
        writer.write_u32::<LE>(400)?;
        writer.write_u32::<LE>(actions.len() as u32)?;
        for action in actions {
            write_action(writer, action)?;
        }
    }
    Ok(())
}

// Writes an Object (uncompressed data)
pub fn write_object<W>(writer: &mut W, object: &asset::Object, _version: GameVersion) -> io::Result<()>
where
    W: io::Write,
{
    writer.write_pas_string(&object.name)?;
    write_timestamp(writer)?;
    writer.write_u32::<LE>(430)?;
    writer.write_i32::<LE>(object.sprite_index)?;
    writer.write_u32::<LE>(object.solid as u32)?;
    writer.write_u32::<LE>(object.visible as u32)?;
    writer.write_i32::<LE>(object.depth)?;
    writer.write_u32::<LE>(object.persistent as u32)?;
    writer.write_i32::<LE>(object.parent_index)?;
    writer.write_i32::<LE>(object.mask_index)?;
    writer.write_u32::<LE>(if object.events.is_empty() { 0 } else { (object.events.len() - 1) as u32 })?;
    for ev_list in &object.events {
        for (sub, actions) in ev_list {
            writer.write_u32::<LE>(*sub)?;
            writer.write_u32::<LE>(400)?;
            writer.write_u32::<LE>(actions.len() as u32)?;
            for action in actions.iter() {
                write_action(writer, action)?;
            }
        }
        writer.write_i32::<LE>(-1)?;
    }
    Ok(())
}

// Writes an Room (uncompressed data)
pub fn write_room<W>(writer: &mut W, room: &asset::Room, _: GameVersion) -> io::Result<()>
where
    W: io::Write,
{
    writer.write_pas_string(&room.name)?;
    write_timestamp(writer)?;
    writer.write_u32::<LE>(541)?;
    writer.write_pas_string(&room.caption)?;
    writer.write_u32::<LE>(room.width)?;
    writer.write_u32::<LE>(room.height)?;
    writer.write_u32::<LE>(32)?; // snap X
    writer.write_u32::<LE>(32)?; // snap X
    writer.write_u32::<LE>(false as u32)?; // isometric grid
    writer.write_u32::<LE>(room.speed)?;
    writer.write_u32::<LE>(room.persistent as u32)?;
    writer.write_u32::<LE>(room.bg_colour.into())?;
    writer.write_u32::<LE>(room.clear_screen as u32)?;

    let mut compat = String::new();
    if room.uses_810_features {
        for tile in &room.tiles {
            compat += &format!(
                "{}{}",
                if tile.xscale != 1.0 || tile.yscale != 1.0 { format!("tile_set_scale({},{},{});\r\n", tile.id, tile.xscale, tile.yscale) } else { String::new() },
                if tile.blend != u32::MAX { format!("tile_set_blend({},{});\r\n", tile.id, tile.blend) } else { String::new() },
            );
        }
    }
    if compat.len() == 0 {
        writer.write_pas_string(&room.creation_code)?;
    } else {
        compat = format!("/* gm8.2 compat */\r\n{}/****************/\r\n\r\n{}", compat, room.creation_code);
        writer.write_pas_string(&PascalString::from(compat.as_str()))?;
    }

    writer.write_u32::<LE>(room.backgrounds.len() as u32)?;
    for background in &room.backgrounds {
        writer.write_u32::<LE>(background.visible_on_start as u32)?;
        writer.write_u32::<LE>(background.is_foreground as u32)?;
        writer.write_i32::<LE>(background.source_bg)?;
        writer.write_i32::<LE>(background.xoffset)?;
        writer.write_i32::<LE>(background.yoffset)?;
        writer.write_u32::<LE>(background.tile_horz as u32)?;
        writer.write_u32::<LE>(background.tile_vert as u32)?;
        writer.write_i32::<LE>(background.hspeed)?;
        writer.write_i32::<LE>(background.vspeed)?;
        writer.write_u32::<LE>(background.stretch as u32)?;
    }

    writer.write_u32::<LE>(room.views_enabled as u32)?;
    writer.write_u32::<LE>(room.views.len() as u32)?;
    for view in &room.views {
        writer.write_u32::<LE>(view.visible as u32)?;
        writer.write_i32::<LE>(view.source_x)?;
        writer.write_i32::<LE>(view.source_y)?;
        writer.write_u32::<LE>(view.source_w)?;
        writer.write_u32::<LE>(view.source_h)?;
        writer.write_i32::<LE>(view.port_x)?;
        writer.write_i32::<LE>(view.port_y)?;
        writer.write_u32::<LE>(view.port_w)?;
        writer.write_u32::<LE>(view.port_h)?;
        writer.write_i32::<LE>(view.following.hborder)?;
        writer.write_i32::<LE>(view.following.vborder)?;
        writer.write_i32::<LE>(view.following.hspeed)?;
        writer.write_i32::<LE>(view.following.vspeed)?;
        writer.write_i32::<LE>(view.following.target)?;
    }

    writer.write_u32::<LE>(room.instances.len() as u32)?;
    for instance in &room.instances {
        writer.write_i32::<LE>(instance.x)?;
        writer.write_i32::<LE>(instance.y)?;
        writer.write_i32::<LE>(instance.object)?;
        writer.write_i32::<LE>(instance.id)?;
        if room.uses_810_features {
            let do_write_xscale = instance.xscale != 1.0;
            let do_write_yscale = instance.yscale != 1.0;
            let do_write_blend = instance.blend != u32::MAX;
            let do_write_angle = room.uses_811_features && instance.angle != 0.0;
            if do_write_xscale || do_write_yscale || do_write_blend || do_write_angle {
                let creation_code: String = format!(
                    "/* gm8.2 compat */\r\n{}{}{}{}/****************/\r\n\r\n{}",
                    if do_write_xscale { format!("image_xscale={};\r\n", instance.xscale) } else { String::new() },
                    if do_write_yscale { format!("image_yscale={};\r\n", instance.yscale) } else { String::new() },
                    if do_write_blend { format!("image_blend={};\r\n", instance.blend) } else { String::new() },
                    if do_write_angle { format!("image_angle={};\r\n", instance.angle) } else { String::new() },
                    instance.creation_code,
                );
                writer.write_pas_string(&PascalString::from(creation_code.as_str()))?;
            } else {
                writer.write_pas_string(&instance.creation_code)?;
            }
        } else {
            writer.write_pas_string(&instance.creation_code)?;
        }
        writer.write_u32::<LE>(false as u32)?; // locked in editor
    }

    writer.write_u32::<LE>(room.tiles.len() as u32)?;
    for tile in &room.tiles {
        writer.write_i32::<LE>(tile.x)?;
        writer.write_i32::<LE>(tile.y)?;
        writer.write_i32::<LE>(tile.source_bg)?;
        writer.write_u32::<LE>(tile.tile_x)?;
        writer.write_u32::<LE>(tile.tile_y)?;
        writer.write_u32::<LE>(tile.width)?;
        writer.write_u32::<LE>(tile.height)?;
        writer.write_i32::<LE>(tile.depth)?;
        writer.write_i32::<LE>(tile.id)?;
        writer.write_u32::<LE>(false as u32)?; // locked in editor
    }

    // All these settings are 0/false by default when creating a new room in the IDE.
    // Commented with Zach's names for the variables, I haven't verified what they do
    writer.write_u32::<LE>(false as u32)?; // remember room editor info
    writer.write_u32::<LE>(0)?; // editor width
    writer.write_u32::<LE>(0)?; // editor height
    writer.write_u32::<LE>(false as u32)?; // show grid
    writer.write_u32::<LE>(false as u32)?; // show objects
    writer.write_u32::<LE>(false as u32)?; // show tiles
    writer.write_u32::<LE>(false as u32)?; // show backgrounds
    writer.write_u32::<LE>(false as u32)?; // show foregrounds
    writer.write_u32::<LE>(false as u32)?; // show views
    writer.write_u32::<LE>(false as u32)?; // delete underlying objects
    writer.write_u32::<LE>(false as u32)?; // delete underlying tiles
    writer.write_u32::<LE>(0)?; // tab
    writer.write_u32::<LE>(0)?; // x position scroll
    writer.write_u32::<LE>(0)?; // y position scroll

    Ok(())
}

// Write GMK's room editor metadata
pub fn write_room_editor_meta<W>(writer: &mut W, last_instance_id: i32, last_tile_id: i32) -> io::Result<()>
where
    W: io::Write,
{
    writer.write_i32::<LE>(last_instance_id)?;
    writer.write_i32::<LE>(last_tile_id)?;
    Ok(())
}

// Write included files to gmk
// Note: not compatible with write_asset_list because included files can't not exist
pub fn write_included_files<W>(writer: &mut W, files: &[asset::IncludedFile]) -> io::Result<()>
where
    W: io::Write,
{
    writer.write_u32::<LE>(800)?;
    writer.write_u32::<LE>(files.len() as u32)?;
    for file in files {
        let mut enc = ZlibWriter::new();
        write_timestamp(&mut enc)?;
        enc.write_u32::<LE>(800)?;
        enc.write_pas_string(&file.file_name)?;
        enc.write_pas_string(&file.source_path)?;
        enc.write_u32::<LE>(file.data_exists as u32)?;
        enc.write_u32::<LE>(file.source_length as u32)?;
        enc.write_u32::<LE>(file.stored_in_gmk as u32)?;
        if let Some(data) = &file.embedded_data {
            enc.write_u32::<LE>(data.len() as u32)?;
            enc.write_buffer(data)?;
        }
        match &file.export_settings {
            ExportSetting::NoExport => {
                enc.write_u32::<LE>(0)?;
                enc.write_pas_string(&"".into())?;
            },
            ExportSetting::TempFolder => {
                enc.write_u32::<LE>(1)?;
                enc.write_pas_string(&"".into())?;
            },
            ExportSetting::GameFolder => {
                enc.write_u32::<LE>(2)?;
                enc.write_pas_string(&"".into())?;
            },
            ExportSetting::CustomFolder(f) => {
                enc.write_u32::<LE>(3)?;
                enc.write_pas_string(f)?;
            },
        }
        enc.write_u32::<LE>(file.overwrite_file as u32)?;
        enc.write_u32::<LE>(file.free_memory as u32)?;
        enc.write_u32::<LE>(file.remove_at_end as u32)?;
        enc.finish(&mut *writer)?;
    }
    Ok(())
}

// Write extensions to GMK (names only)
pub fn write_extensions<W>(writer: &mut W, extensions: &[asset::Extension]) -> io::Result<()>
where
    W: io::Write,
{
    writer.write_u32::<LE>(700)?;
    writer.write_u32::<LE>(extensions.len() as u32)?;
    for ext in extensions {
        writer.write_pas_string(&ext.name)?;
    }
    Ok(())
}

// Write game information (help dialog) block to GMK
pub fn write_game_information<W>(writer: &mut W, info: &GameHelpDialog) -> io::Result<()>
where
    W: io::Write,
{
    writer.write_u32::<LE>(800)?; // TODO: why is this hardcoded?? come on adam
    // maybe others are too ?
    let mut enc = ZlibWriter::new();
    enc.write_u32::<LE>(info.bg_colour.into())?;
    enc.write_u32::<LE>(info.new_window as u32)?;
    enc.write_pas_string(&info.caption)?;
    enc.write_i32::<LE>(info.left)?;
    enc.write_i32::<LE>(info.top)?;
    enc.write_u32::<LE>(info.width)?;
    enc.write_u32::<LE>(info.height)?;
    enc.write_u32::<LE>(info.border as u32)?;
    enc.write_u32::<LE>(info.resizable as u32)?;
    enc.write_u32::<LE>(info.window_on_top as u32)?;
    enc.write_u32::<LE>(info.freeze_game as u32)?;
    write_timestamp(&mut enc)?;
    enc.write_pas_string(&info.info)?;
    enc.finish(writer)?;
    Ok(())
}

// Write library initialization code strings to GMK
pub fn write_library_init_code<W>(writer: &mut W, init_code: &[PascalString]) -> io::Result<()>
where
    W: io::Write,
{
    writer.write_u32::<LE>(500)?;
    writer.write_u32::<LE>(init_code.len() as u32)?;
    for string in init_code {
        writer.write_pas_string(&string)?;
    }
    Ok(())
}

// Write room order to GMK
pub fn write_room_order<W>(writer: &mut W, room_order: &[i32]) -> io::Result<()>
where
    W: io::Write,
{
    writer.write_u32::<LE>(700)?;
    writer.write_u32::<LE>(room_order.len() as u32)?;
    for room in room_order {
        writer.write_i32::<LE>(*room)?;
    }
    Ok(())
}

// Write resource tree to GMK
pub fn write_resource_tree<W>(writer: &mut W, assets: &GameAssets) -> io::Result<()>
where
    W: io::Write,
{
    fn write_rt_heading<W>(writer: &mut W, name: &str, index: u32, count: usize) -> io::Result<()>
    where
        W: io::Write,
    {
        writer.write_u32::<LE>(1)?;
        writer.write_u32::<LE>(index)?;
        writer.write_u32::<LE>(0)?;
        writer.write_pas_string(&name.into())?;
        writer.write_u32::<LE>(count as u32)?;
        Ok(())
    }

    fn write_rt_asset<W>(writer: &mut W, name: &PascalString, group: u32, index: u32) -> io::Result<()>
    where
        W: io::Write,
    {
        writer.write_u32::<LE>(3)?;
        writer.write_u32::<LE>(group)?;
        writer.write_u32::<LE>(index)?;
        writer.write_pas_string(name)?;
        writer.write_u32::<LE>(0)?;
        Ok(())
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

    write_rt_heading(writer, "Sprites", 2, count_existing(&assets.sprites))?;
    for (i, sprite) in enumerate_existing(&assets.sprites) {
        write_rt_asset(writer, &sprite.name, 2, i as u32)?;
    }
    write_rt_heading(writer, "Sounds", 3, count_existing(&assets.sounds))?;
    for (i, sound) in enumerate_existing(&assets.sounds) {
        write_rt_asset(writer, &sound.name, 3, i as u32)?;
    }
    write_rt_heading(writer, "Backgrounds", 6, count_existing(&assets.backgrounds))?;
    for (i, background) in enumerate_existing(&assets.backgrounds) {
        write_rt_asset(writer, &background.name, 6, i as u32)?;
    }
    write_rt_heading(writer, "Paths", 8, count_existing(&assets.paths))?;
    for (i, path) in enumerate_existing(&assets.paths) {
        write_rt_asset(writer, &path.name, 8, i as u32)?;
    }
    write_rt_heading(writer, "Scripts", 7, count_existing(&assets.scripts))?;
    for (i, script) in enumerate_existing(&assets.scripts) {
        write_rt_asset(writer, &script.name, 7, i as u32)?;
    }
    write_rt_heading(writer, "Fonts", 9, count_existing(&assets.fonts))?;
    for (i, font) in enumerate_existing(&assets.fonts) {
        write_rt_asset(writer, &font.name, 9, i as u32)?;
    }
    write_rt_heading(writer, "Time Lines", 12, count_existing(&assets.timelines))?;
    for (i, timeline) in enumerate_existing(&assets.timelines) {
        write_rt_asset(writer, &timeline.name, 12, i as u32)?;
    }
    write_rt_heading(writer, "Objects", 1, count_existing(&assets.objects))?;
    for (i, object) in enumerate_existing(&assets.objects) {
        write_rt_asset(writer, &object.name, 1, i as u32)?;
    }
    write_rt_heading(writer, "Rooms", 4, count_existing(&assets.rooms))?;
    for room_id in &assets.room_order {
        if let Some(Some(room)) = assets.rooms.get(*room_id as usize) {
            write_rt_asset(writer, &room.name, 4, *room_id as u32)?;
        } else {
            println!("WARNING: non-existent room id {} referenced in Room Order; skipping it", *room_id);
        }
    }
    write_rt_asset(writer, &"Game Information".into(), 10, 0)?;
    write_rt_asset(writer, &"Global Game Settings".into(), 11, 0)?;
    write_rt_asset(writer, &"Extension Packages".into(), 13, 0)?;
    Ok(())
}
