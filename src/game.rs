use crate::{atlas::AtlasBuilder, gl};

use glutin::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::ControlFlow,
    event_loop::EventLoop,
    window::{Fullscreen, Icon, WindowBuilder},
    {Api, ContextBuilder, GlProfile, GlRequest},
};
use gm8x::reader::{GameAssets, WindowsIcon};

use std::convert::identity;

pub fn icon_from_win32(raw: &[u8], width: usize) -> Option<Icon> {
    let mut rgba = Vec::with_capacity(raw.len());
    for chunk in raw.rchunks_exact(width * 4) {
        rgba.extend_from_slice(chunk);
        let vlen = rgba.len();
        crate::util::bgra2rgba(rgba.get_mut(vlen - (width * 4)..)?);
    }
    Icon::from_rgba(rgba, width as u32, width as u32).ok()
}

fn get_icon_via_w(icons: &Vec<WindowsIcon>, w: i32) -> Option<Icon> {
    fn closest<'a, I: Iterator<Item = &'a WindowsIcon>>(w: i32, i: I) -> Option<&'a WindowsIcon> {
        i.min_by(|a, b| (a.width as i32 - w).abs().cmp(&(b.width as i32 - w).abs()))
    }

    closest(w, icons.iter().filter(|i| i.original_bpp == 24 || i.original_bpp == 32))
        .or_else(|| closest(w, icons.iter()))
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

    // Set up glutin (winit)
    let event_loop = EventLoop::new();
    let window_builder = WindowBuilder::new()
        .with_title(&room1.caption)
        .with_window_icon(get_icon_via_w(&assets.icon_data, 32))
        .with_inner_size(LogicalSize::from((room1.width, room1.height)))
        .with_resizable(assets.settings.allow_resize)
        .with_always_on_top(assets.settings.window_on_top)
        .with_decorations(!assets.settings.dont_draw_border)
        .with_visible(false)
        .with_fullscreen(if assets.settings.fullscreen {
            Some(Fullscreen::Borderless(event_loop.primary_monitor()))
        } else {
            None
        });

    // Set up OpenGL 3.3 Core context
    let context = ContextBuilder::new()
        .with_gl(GlRequest::Specific(Api::OpenGl, (3, 3)))
        .with_gl_profile(GlProfile::Core)
        .with_hardware_acceleration(Some(true))
        // TODO: Maybe manual override?
        .with_vsync(assets.settings.vsync)
        // TODO: Maybe on release, when we're done - robustness 0 CHECKS
        .build_windowed(window_builder, &event_loop)
        .unwrap(); // TODO

    // Make context current
    let (ctx, window) = unsafe { context.make_current().unwrap().split() };

    // Load OpenGL
    gl::load_with(|s| ctx.get_proc_address(s) as *const _);

    // max texture size
    let max_size = unsafe {
        let mut val: gl::types::GLint = 0;
        gl::GetIntegerv(gl::MAX_TEXTURE_SIZE, &mut val as *mut _);
        val
    };

    // multi-atlas builder/manager
    let mut atlases = AtlasBuilder::new(max_size);

    // image-ref to atl-ref map
    let mut pixelrefs = Vec::new();

    // background associations
    let mut _bgrefs = Vec::new();

    // sprite associations
    let mut _spriterefs = Vec::new();

    for sprite in assets.sprites.iter().flatten().map(|s| &**s) {
        for frame in &sprite.frames {
            let atl_ref = atlases.add(frame.width as _, frame.height as _);

            pixelrefs.push((&frame.data, atl_ref.clone()));
            _spriterefs.push((sprite, frame, atl_ref));
        }
    }

    for bg in assets.backgrounds.iter().flatten().map(|b| &**b) {
        if let Some(data) = &bg.data {
            let atl_ref = atlases.add(bg.width as _, bg.height as _);

            pixelrefs.push((data, atl_ref.clone()));
            _bgrefs.push((bg, atl_ref));
        }
    }

    // not done - needs A to RGBA
    // for font in assets.fonts.iter().flatten().map(|f| &**f) {
    //     spriterefs.push((&font.pixel_map, atlases.add(font.map_width as _, font.map_height as _)));
    // }

    let mut frames = atlases
        .into_frames()
        .iter()
        .map(|(maxx, maxy)| (vec![0u8; ((*maxx * *maxy) * 4) as usize], *maxx, *maxy))
        .collect::<Vec<_>>();
    for (f, r) in pixelrefs {
        let maxx = frames[r.atlas_id as usize].1;
        let out_buf = &mut frames[r.atlas_id as usize].0;

        for (i, y) in ((r.y as usize)..(r.y as usize + r.h as usize)).enumerate() {
            let dst_len = (maxx as usize * y as usize * 4) + (r.x as usize * 4);
            let dst = &mut out_buf[dst_len..dst_len + (r.w as usize * 4)];
            let src = &f[(r.w as usize * 4) * i..((r.w as usize * 4) * (i + 1))];
            dst.copy_from_slice(src);
        }
    }

    // `frames` contains the full atlases at this point --

    window.set_visible(true);
    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            window_id,
        } if window_id == window.id() => {
            *control_flow = ControlFlow::Exit;
        }
        _ => *control_flow = ControlFlow::Wait,
    });
}
