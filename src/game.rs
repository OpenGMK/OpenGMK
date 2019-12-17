use crate::{atlas::AtlasBuilder, render::opengl::OpenGLRenderer};
use gm8exe::{rsrc::WindowsIcon, GameAssets};

use std::convert::identity;

/// Resolves icon closest to preferred_width and converts it from a WindowsIcon to proper RGBA pixels.
fn get_icon(icons: &[WindowsIcon], preferred_width: i32) -> Option<(Vec<u8>, u32, u32)> {
    fn closest<'a, I: Iterator<Item = &'a WindowsIcon>>(preferred_width: i32, i: I) -> Option<&'a WindowsIcon> {
        i.min_by(|a, b| {
            (a.width as i32 - preferred_width)
                .abs()
                .cmp(&(b.width as i32 - preferred_width).abs())
        })
    }

    fn icon_from_win32(raw: &[u8], width: usize) -> Option<(Vec<u8>, u32, u32)> {
        let mut rgba = Vec::with_capacity(raw.len());
        for chunk in raw.rchunks_exact(width * 4) {
            rgba.extend_from_slice(chunk);
            let vlen = rgba.len();
            crate::util::bgra2rgba(rgba.get_mut(vlen - (width * 4)..)?);
        }
        Some((rgba, width as u32, width as u32))
    }

    closest(
        preferred_width,
        icons.iter().filter(|i| i.original_bpp == 24 || i.original_bpp == 32),
    )
    .or_else(|| closest(preferred_width, icons.iter()))
    .and_then(|i| icon_from_win32(&i.bgra_data, i.width as usize))
}

pub fn launch(assets: GameAssets) {
    // If there are no rooms, you can't build a GM8 game. Fatal error.
    // We need a lot of the initialization info from the first room,
    // the window size, and title, etc. is based on it.
    let room1 = assets
        .room_order
        .first() // first index
        .map(|x| assets.rooms.get(*x as usize))
        .and_then(identity) // Option<Option<T>> -> Option<T>
        .and_then(|x| x.as_ref()) // Option<&Option<T>> -> Option<&T>
        .map(|r| r.as_ref()) // Option<&Box<T>> -> Option<&T>
        .unwrap();

    let icon = get_icon(&assets.icon_data, 32);

    // more renderers later?
    // let _renderer = OpenGLRenderer::new(
    //     &room1.caption,
    //     (room1.width, room1.height),
    //     icon,
    //     assets.settings.allow_resize,
    //     assets.settings.window_on_top,
    //     !assets.settings.dont_draw_border,
    //     assets.settings.fullscreen,
    //     assets.settings.vsync,
    // );

    let max_size = 69;

    // // multi-atlas builder/manager
    // let mut atlases = AtlasBuilder::new(max_size);

    // // image-ref to atl-ref map
    // let mut pixelrefs = Vec::new();

    // // background associations
    // let mut _bgrefs = Vec::new();

    // // sprite associations
    // let mut _spriterefs = Vec::new();

    // for sprite in assets.sprites.iter().flatten().map(|s| &**s) {
    //     for frame in &sprite.frames {
    //         let atl_ref = atlases.add(frame.width as _, frame.height as _);

    //         pixelrefs.push((&frame.data, atl_ref.clone()));
    //         _spriterefs.push((sprite, frame, atl_ref));
    //     }
    // }

    // for bg in assets.backgrounds.iter().flatten().map(|b| &**b) {
    //     if let Some(data) = &bg.data {
    //         let atl_ref = atlases.add(bg.width as _, bg.height as _);

    //         pixelrefs.push((data, atl_ref.clone()));
    //         _bgrefs.push((bg, atl_ref));
    //     }
    // }

    // not done - needs A to RGBA
    // for font in assets.fonts.iter().flatten().map(|f| &**f) {
    //     spriterefs.push((&font.pixel_map, atlases.add(font.map_width as _, font.map_height as _)));
    // }

    // let mut frames = atlases
    //     .into_frames()
    //     .iter()
    //     .map(|(maxx, maxy)| (vec![0u8; ((*maxx * *maxy) * 4) as usize], *maxx, *maxy))
    //     .collect::<Vec<_>>();
    // for (f, r) in pixelrefs {
    //     let maxx = frames[r.atlas_id as usize].1;
    //     let out_buf = &mut frames[r.atlas_id as usize].0;

    //     for (i, y) in ((r.y as usize)..(r.y as usize + r.h as usize)).enumerate() {
    //         let dst_len = (maxx as usize * y as usize * 4) + (r.x as usize * 4);
    //         let dst = &mut out_buf[dst_len..dst_len + (r.w as usize * 4)];
    //         let src = &f[(r.w as usize * 4) * i..((r.w as usize * 4) * (i + 1))];
    //         dst.copy_from_slice(src);
    //     }
    // }

    // `frames` contains the full atlases at this point --
}
