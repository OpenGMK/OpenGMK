use crate::{
    action::Tree,
    asset::{
        font::{Character, Font},
        sprite::{Collider, Frame, Sprite},
        Background, Object,
    },
    atlas::AtlasBuilder,
    gml::Compiler,
    instance::Instance,
    instancelist::InstanceList,
    render::{opengl::OpenGLRenderer, Renderer, RendererOptions},
};
use gm8exe::GameAssets;
use std::iter::repeat;

/// Resolves icon closest to preferred_width and converts it from a WindowsIcon to proper RGBA pixels.
/*
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
*/

pub fn launch(assets: GameAssets) -> Result<(), Box<dyn std::error::Error>> {
    // destructure assets
    let GameAssets { backgrounds, constants, fonts, icon_data, objects, paths, room_order, rooms, scripts, sounds, sprites, timelines, triggers, .. } = assets;

    // If there are no rooms, you can't build a GM8 game. Fatal error.
    // We need a lot of the initialization info from the first room,
    // the window size, and title, etc. is based on it.
    let room1 = room_order
        .first()
        .and_then(|i| rooms.get(*i as usize).and_then(|o| o.as_ref()))
        .ok_or("first room not present in gamedata")?;

    // Set up a GML compiler
    let mut compiler = Compiler::new();
    compiler.reserve_scripts(scripts.iter().flatten().count());
    compiler.reserve_constants(
        backgrounds.iter().flatten().count() +
        fonts.iter().flatten().count() +
        objects.iter().flatten().count() +
        paths.iter().flatten().count() +
        rooms.iter().flatten().count() +
        scripts.iter().flatten().count() +
        sounds.iter().flatten().count() +
        sprites.iter().flatten().count() +
        timelines.iter().flatten().count() +
        triggers.iter().flatten().count() + 
        constants.len()
    );
    backgrounds.iter().enumerate().filter_map(|(i, x)| x.as_ref().map(|x| (i, x))).for_each(|(i, x)| compiler.register_constant(x.name.clone(), i as f64));
    fonts.iter().enumerate().filter_map(|(i, x)| x.as_ref().map(|x| (i, x))).for_each(|(i, x)| compiler.register_constant(x.name.clone(), i as f64));
    objects.iter().enumerate().filter_map(|(i, x)| x.as_ref().map(|x| (i, x))).for_each(|(i, x)| compiler.register_constant(x.name.clone(), i as f64));
    paths.iter().enumerate().filter_map(|(i, x)| x.as_ref().map(|x| (i, x))).for_each(|(i, x)| compiler.register_constant(x.name.clone(), i as f64));
    rooms.iter().enumerate().filter_map(|(i, x)| x.as_ref().map(|x| (i, x))).for_each(|(i, x)| compiler.register_constant(x.name.clone(), i as f64));
    scripts.iter().enumerate().filter_map(|(i, x)| x.as_ref().map(|x| (i, x))).for_each(|(i, x)| compiler.register_constant(x.name.clone(), i as f64));
    sounds.iter().enumerate().filter_map(|(i, x)| x.as_ref().map(|x| (i, x))).for_each(|(i, x)| compiler.register_constant(x.name.clone(), i as f64));
    sprites.iter().enumerate().filter_map(|(i, x)| x.as_ref().map(|x| (i, x))).for_each(|(i, x)| compiler.register_constant(x.name.clone(), i as f64));
    timelines.iter().enumerate().filter_map(|(i, x)| x.as_ref().map(|x| (i, x))).for_each(|(i, x)| compiler.register_constant(x.name.clone(), i as f64));
    triggers.iter().enumerate().filter_map(|(i, x)| x.as_ref().map(|x| (i, x))).for_each(|(i, x)| compiler.register_constant(x.constant_name.clone(), i as f64));

    scripts.iter().enumerate().filter_map(|(i, x)| x.as_ref().map(|x| (i, x))).for_each(|(i, x)| compiler.register_script(x.name.clone(), i));

    // Set up a Renderer
    let options = RendererOptions {
        title: &room1.caption,
        size: (room1.width, room1.height),
        icons: icon_data.into_iter().map(|x| (x.bgra_data, x.width, x.height)).collect(),
        resizable: assets.settings.allow_resize,
        on_top: assets.settings.window_on_top,
        decorations: !assets.settings.dont_draw_border,
        fullscreen: assets.settings.fullscreen,
        vsync: assets.settings.vsync, // TODO: Overrideable
    };

    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).expect("Failed to init GLFW");
    glfw.window_hint(glfw::WindowHint::Visible(false));

    let (window, events) = glfw
        .create_window(
            options.size.0,
            options.size.1,
            options.title,
            if options.fullscreen {
                // TODO: not possible to do this safely with current glfw bindings - maybe unsafe it?
                unimplemented!()
            } else {
                glfw::WindowMode::Windowed
            },
        )
        .expect("Failed to create GLFW window");

    let mut renderer = OpenGLRenderer::new(options, window)?;

    // needs to be done after renderer sets context
    glfw.set_swap_interval(if assets.settings.vsync { glfw::SwapInterval::Sync(1) } else { glfw::SwapInterval::None });

    let mut atlases = AtlasBuilder::new(renderer.max_gpu_texture_size() as _);

    //println!("GPU Max Texture Size: {}", renderer.max_gpu_texture_size());

    let sprites = sprites
        .into_iter()
        .map(|o| {
            o.map(|b| {
                let (w, h) = b.frames.first().map_or((0, 0), |f| (f.width, f.height));
                let origin_x = b.origin_x;
                let origin_y = b.origin_y;
                Box::new(Sprite {
                    name: b.name,
                    frames: b
                        .frames
                        .into_iter()
                        .map(|f| Frame {
                            width: f.width,
                            height: f.height,
                            atlas_ref: atlases
                                .texture(f.width as _, f.height as _, origin_x, origin_y, f.data)
                                .unwrap(),
                        })
                        .collect(),
                    colliders: b
                        .colliders
                        .into_iter()
                        .map(|c| Collider {
                            width: c.width,
                            height: c.height,
                            bbox_left: c.bbox_left,
                            bbox_right: c.bbox_right,
                            bbox_top: c.bbox_top,
                            bbox_bottom: c.bbox_bottom,
                            data: c.data,
                        })
                        .collect(),
                    width: w,
                    height: h,
                    origin_x,
                    origin_y,
                    per_frame_colliders: b.per_frame_colliders,
                })
            })
        })
        .collect::<Vec<_>>();

    let _backgrounds = backgrounds
        .into_iter()
        .map(|o| {
            o.map(|b| {
                let width = b.width;
                let height = b.height;
                Background {
                    name: b.name,
                    width,
                    height,
                    atlas_ref: b.data.map(|d| atlases.texture(width as _, height as _, 0, 0, d).unwrap()),
                }
            })
        })
        .collect::<Vec<_>>();

    let _fonts = fonts
        .into_iter()
        .map(|o| {
            o.map(|b| Font {
                name: b.name,
                sys_name: b.sys_name,
                size: b.size,
                bold: b.bold,
                italic: b.italic,
                first: b.range_start,
                last: b.range_end,
                atlas_ref: atlases
                    .texture(
                        b.map_width as _,
                        b.map_height as _,
                        0,
                        0,
                        b.pixel_map
                            .into_iter()
                            .flat_map(|x| repeat(0xFF).take(3).chain(Some(*x)))
                            .collect::<Vec<u8>>()
                            .into_boxed_slice(),
                    )
                    .unwrap(),
                chars: b
                    .dmap
                    .chunks_exact(6)
                    .skip(b.range_start as usize)
                    .take(((b.range_end - b.range_start) + 1) as usize)
                    .map(|x| Character { x: x[0], y: x[1], width: x[2], height: x[3], offset: x[4], distance: x[5] })
                    .collect(),
            })
        })
        .collect::<Vec<_>>();

    let objects = objects
        .into_iter()
        .map(|o| {
            o.map(|b| {
                Box::new(Object {
                    name: b.name,
                    solid: b.solid,
                    visible: b.visible,
                    persistent: b.persistent,
                    depth: b.depth,
                    sprite_index: b.sprite_index,
                    mask_index: b.mask_index,
                })
            })
        })
        .collect::<Vec<_>>();

    renderer.upload_atlases(atlases)?;

    let mut instance_list = InstanceList::new();

    for instance in &room1.instances {
        let object = match objects.get(instance.object as usize) {
            Some(&Some(ref o)) => o.as_ref(),
            _ => return Err(format!("Instance of invalid Object in room {}", room1.name).into()),
        };
        instance_list.insert(Instance::new(
            instance.id as _,
            f64::from(instance.x),
            f64::from(instance.y),
            instance.object,
            object,
        ));
    }

    // renderer.dump_atlases(|i| std::path::PathBuf::from(format!("./atl{}.png", i)))?;

    // Important: show window
    renderer.show_window();

    while !renderer.should_close() {
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            println!("Got event {:?}", event);
            match event {
                glfw::WindowEvent::Key(glfw::Key::Escape, _, glfw::Action::Press, _) => {
                    renderer.set_should_close(true);
                    continue; // So no draw events are fired while the window should be closing
                },
                _ => {},
            }
        }

        renderer.set_view(
            0,
            0,
            room1.width as i32,
            room1.height as i32,
            0.0,
            0,
            0,
            room1.width as i32,
            room1.height as i32,
        );
        if room1.clear_screen {
            renderer.clear(
                room1.bg_colour.r as f32 / 255.0,
                room1.bg_colour.g as f32 / 255.0,
                room1.bg_colour.b as f32 / 255.0,
            );
        }
        for (_, instance) in instance_list.iter() {
            if let Some(Some(sprite)) = sprites.get(instance.sprite_index.get() as usize) {
                renderer.draw_sprite(
                    &sprite.frames.first().unwrap().atlas_ref,
                    instance.x.get(),
                    instance.y.get(),
                    instance.image_xscale.get(),
                    instance.image_yscale.get(),
                    instance.image_angle.get(),
                    instance.image_blend.get(),
                    instance.image_alpha.get(),
                )
            }
        }
        renderer.finish();
    }

    Ok(())
}
