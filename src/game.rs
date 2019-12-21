use crate::{
    asset::Object,
    atlas::AtlasBuilder,
    instancelist::InstanceList,
    render::{
        opengl::{OpenGLRenderer, OpenGLRendererOptions},
        Renderer,
    },
};
use gm8exe::{rsrc::WindowsIcon, GameAssets, asset::Room};

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

    let options = OpenGLRendererOptions {
        title: &room1.caption,
        size: (room1.width, room1.height),
        icon: get_icon(&assets.icon_data, 32),
        resizable: assets.settings.allow_resize,
        on_top: assets.settings.window_on_top,
        decorations: !assets.settings.dont_draw_border,
        fullscreen: assets.settings.fullscreen,
        vsync: assets.settings.vsync, // TODO: Overrideable
    };

    let mut renderer = OpenGLRenderer::new(options).unwrap();
    let mut atlases = AtlasBuilder::new(renderer.max_gpu_texture_size() as _);

    for frame in assets
        .sprites
        .iter()
        .flatten()
        .map(|s| s.frames.iter())
        .flatten()
    {
        atlases
            .texture(frame.width as _, frame.height as _, frame.data)
            .unwrap();
    }

    // let (packers, _) = atlases.into_inner();
    // for (i, packer) in packers.iter().enumerate() {
    //     let (w,h) = packer.size();
    //     println!("packer #{} size: {}, {}", i, w, h);
    // }

    println!("GPU Max Texture Size: {}", renderer.max_gpu_texture_size());
    renderer.upload_atlases(atlases).unwrap();

    let objects = assets.objects.into_iter().map(|o| o.map(|b| Box::new(Object::from(*b)))).collect::<Vec<_>>();

    let mut instance_list = InstanceList::new();

    for instance in &room1.instances {

    }

    // renderer.dump_atlases(|i| std::path::PathBuf::from(format!("./atl{}.png", i))).unwrap();
}
