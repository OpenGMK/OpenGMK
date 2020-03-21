use crate::{
    action::Tree,
    asset::{
        font::{Character, Font},
        room::{self, Room},
        sprite::{Collider, Frame, Sprite},
        Background, Object, Script, Timeline,
    },
    atlas::AtlasBuilder,
    background,
    gml::{rand::Random, Compiler},
    instance::Instance,
    instancelist::{InstanceList, TileList},
    render::{opengl::OpenGLRenderer, Renderer, RendererOptions},
    tile, view,
};
use gm8exe::GameAssets;
use std::{
    collections::{HashMap, HashSet},
    iter::repeat,
    rc::Rc,
    sync::mpsc::Receiver,
};

/// Structure which contains all the components of a game.
pub struct Game {
    pub compiler: Compiler,
    pub glfw: glfw::Glfw,
    pub glfw_events: Receiver<(f64, glfw::WindowEvent)>,
    pub instance_list: InstanceList,
    pub tile_list: TileList,
    pub rand: Random,
    pub renderer: Box<dyn Renderer>,
    pub assets: Assets,

    pub room_id: i32,
    pub room_width: i32,
    pub room_height: i32,
}

pub struct Assets {
    pub backgrounds: Vec<Option<Box<Background>>>,
    pub fonts: Vec<Option<Box<Font>>>,
    pub objects: Vec<Option<Box<Object>>>,
    pub rooms: Vec<Option<Box<Room>>>,
    pub scripts: Vec<Option<Box<Script>>>,
    pub sprites: Vec<Option<Box<Sprite>>>,
    pub timelines: Vec<Option<Box<Timeline>>>,
    // todo
}

impl Game {
    pub fn launch(assets: GameAssets) -> Result<Self, Box<dyn std::error::Error>> {
        // destructure assets
        let GameAssets {
            backgrounds,
            constants,
            fonts,
            icon_data,
            objects,
            paths,
            room_order,
            rooms,
            scripts,
            sounds,
            sprites,
            timelines,
            triggers,
            ..
        } = assets;

        // If there are no rooms, you can't build a GM8 game. Fatal error.
        // We need a lot of the initialization info from the first room,
        // the window size, and title, etc. is based on it.
        let room1_id = *room_order.first().ok_or("Room order is empty")?;
        let room1 = match rooms.get(room1_id as usize) {
            Some(Some(r)) => r,
            _ => return Err("First room does not exist".into()),
        };
        let room1_width = room1.width;
        let room1_height = room1.height;

        // Set up a GML compiler
        let mut compiler = Compiler::new();
        compiler.reserve_scripts(scripts.iter().flatten().count());
        compiler.reserve_constants(
            backgrounds.iter().flatten().count()
                + fonts.iter().flatten().count()
                + objects.iter().flatten().count()
                + paths.iter().flatten().count()
                + rooms.iter().flatten().count()
                + scripts.iter().flatten().count()
                + sounds.iter().flatten().count()
                + sprites.iter().flatten().count()
                + timelines.iter().flatten().count()
                + triggers.iter().flatten().count()
                + constants.len(),
        );
        backgrounds
            .iter()
            .enumerate()
            .filter_map(|(i, x)| x.as_ref().map(|x| (i, x)))
            .for_each(|(i, x)| compiler.register_constant(x.name.clone(), i as f64));
        fonts
            .iter()
            .enumerate()
            .filter_map(|(i, x)| x.as_ref().map(|x| (i, x)))
            .for_each(|(i, x)| compiler.register_constant(x.name.clone(), i as f64));
        objects
            .iter()
            .enumerate()
            .filter_map(|(i, x)| x.as_ref().map(|x| (i, x)))
            .for_each(|(i, x)| compiler.register_constant(x.name.clone(), i as f64));
        paths
            .iter()
            .enumerate()
            .filter_map(|(i, x)| x.as_ref().map(|x| (i, x)))
            .for_each(|(i, x)| compiler.register_constant(x.name.clone(), i as f64));
        rooms
            .iter()
            .enumerate()
            .filter_map(|(i, x)| x.as_ref().map(|x| (i, x)))
            .for_each(|(i, x)| compiler.register_constant(x.name.clone(), i as f64));
        scripts
            .iter()
            .enumerate()
            .filter_map(|(i, x)| x.as_ref().map(|x| (i, x)))
            .for_each(|(i, x)| compiler.register_constant(x.name.clone(), i as f64));
        sounds
            .iter()
            .enumerate()
            .filter_map(|(i, x)| x.as_ref().map(|x| (i, x)))
            .for_each(|(i, x)| compiler.register_constant(x.name.clone(), i as f64));
        sprites
            .iter()
            .enumerate()
            .filter_map(|(i, x)| x.as_ref().map(|x| (i, x)))
            .for_each(|(i, x)| compiler.register_constant(x.name.clone(), i as f64));
        timelines
            .iter()
            .enumerate()
            .filter_map(|(i, x)| x.as_ref().map(|x| (i, x)))
            .for_each(|(i, x)| compiler.register_constant(x.name.clone(), i as f64));
        triggers
            .iter()
            .enumerate()
            .filter_map(|(i, x)| x.as_ref().map(|x| (i, x)))
            .for_each(|(i, x)| compiler.register_constant(x.constant_name.clone(), i as f64));

        scripts
            .iter()
            .enumerate()
            .filter_map(|(i, x)| x.as_ref().map(|x| (i, x)))
            .for_each(|(i, x)| compiler.register_script(x.name.clone(), i));

        // Set up a Renderer
        let options = RendererOptions {
            title: &room1.caption,
            size: (room1_width, room1_height),
            icons: icon_data.into_iter().map(|x| (x.bgra_data, x.width, x.height)).collect(),
            global_clear_colour: assets.settings.clear_colour.into(),
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
        glfw.set_swap_interval(if assets.settings.vsync {
            glfw::SwapInterval::Sync(1)
        } else {
            glfw::SwapInterval::None
        });

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

        let backgrounds = backgrounds
            .into_iter()
            .map(|o| {
                o.map(|b| {
                    let width = b.width;
                    let height = b.height;
                    Box::new(Background {
                        name: b.name,
                        width,
                        height,
                        atlas_ref: b.data.map(|d| atlases.texture(width as _, height as _, 0, 0, d).unwrap()),
                    })
                })
            })
            .collect::<Vec<_>>();

        let fonts = fonts
            .into_iter()
            .map(|o| {
                o.map(|b| {
                    Box::new(Font {
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
                            .map(|x| Character {
                                x: x[0],
                                y: x[1],
                                width: x[2],
                                height: x[3],
                                offset: x[4],
                                distance: x[5],
                            })
                            .collect(),
                    })
                })
            })
            .collect::<Vec<_>>();

        let objects = {
            let mut object_parents: Vec<Option<i32>> = Vec::with_capacity(objects.len());
            let mut objects = objects
                .into_iter()
                .map(|o| {
                    object_parents.push(match &o {
                        Some(b) => Some(b.parent_index),
                        None => None,
                    });
                    o.map(|b| {
                        let mut events: [HashMap<u32, Tree>; 12] = std::default::Default::default();
                        for ((i, map), input) in events.iter_mut().enumerate().zip(b.events.iter()) {
                            map.reserve(input.len());
                            for (sub, actions) in input {
                                map.insert(*sub, match Tree::from_list(actions, &mut compiler) {
                                    Ok(t) => t,
                                    Err(e) => {
                                        return Err(format!(
                                            "Compiler error in object {} event {},{}: {}",
                                            b.name, i, sub, e
                                        ));
                                    },
                                });
                            }
                        }
                        Ok(Box::new(Object {
                            name: b.name,
                            solid: b.solid,
                            visible: b.visible,
                            persistent: b.persistent,
                            depth: b.depth,
                            sprite_index: b.sprite_index,
                            mask_index: b.mask_index,
                            parent_index: b.parent_index,
                            events,
                            identities: Rc::new(HashSet::new()),
                            children: Rc::new(HashSet::new()),
                        }))
                    })
                    .transpose()
                })
                .collect::<Result<Vec<_>, _>>()?;

            // Populate identity lists
            for (i, object) in objects.iter_mut().enumerate().filter_map(|(i, x)| x.as_mut().map(|x| (i, x))) {
                Rc::get_mut(&mut object.identities).unwrap().insert(i as _);
                Rc::get_mut(&mut object.children).unwrap().insert(i as _);
                let mut parent_index = object.parent_index;
                while parent_index >= 0 {
                    Rc::get_mut(&mut object.identities).unwrap().insert(parent_index);
                    if let Some(Some(parent)) = object_parents.get(parent_index as usize) {
                        parent_index = *parent;
                    } else {
                        return Err(format!(
                            "Invalid parent tree for object {}: non-existent object: {}",
                            object.name, parent_index
                        )
                        .into());
                    }
                }
            }
            for (i, mut parent_index) in
                object_parents.iter().enumerate().filter_map(|(i, x)| x.as_ref().map(|x| (i, *x)))
            {
                while parent_index >= 0 {
                    if let Some(Some(parent)) = objects.get_mut(parent_index as usize) {
                        Rc::get_mut(&mut parent.children).unwrap().insert(i as _);
                        parent_index = parent.parent_index;
                    } else {
                        return Err(format!(
                            "Invalid parent tree for object {}: non-existent object: {}",
                            i, parent_index
                        )
                        .into());
                    }
                }
            }

            objects
        };

        let timelines = timelines
            .into_iter()
            .map(|t| {
                t.map(|b| {
                    let mut moments: HashMap<u32, Tree> = HashMap::with_capacity(b.moments.len());
                    for (moment, actions) in b.moments.iter() {
                        match Tree::from_list(actions, &mut compiler) {
                            Ok(t) => {
                                moments.insert(*moment, t);
                            },
                            Err(e) => {
                                return Err(format!("Compiler error in timeline {} moment {}: {}", b.name, moment, e));
                            },
                        };
                    }
                    Ok(Box::new(Timeline { name: b.name, moments }))
                })
                .transpose()
            })
            .collect::<Result<Vec<_>, _>>()?;

        let scripts = scripts
            .into_iter()
            .map(|t| {
                t.map(|b| {
                    let compiled = match compiler.compile(&b.source) {
                        Ok(s) => s,
                        Err(e) => return Err(format!("Compiler error in script {}: {}", b.name, e)),
                    };
                    Ok(Box::new(Script { name: b.name, source: b.source, compiled }))
                })
                .transpose()
            })
            .collect::<Result<Vec<_>, _>>()?;

        let rooms = rooms
            .into_iter()
            .map(|t| {
                t.map(|b| {
                    let creation_code = match compiler.compile(&b.creation_code) {
                        Ok(c) => c,
                        Err(e) => return Err(format!("Compiler error in room {} creation code: {}", b.name, e)),
                    };
                    Ok(Box::new(Room {
                        name: b.name,
                        caption: b.caption,
                        width: b.width,
                        height: b.height,
                        speed: b.speed,
                        persistent: b.persistent,
                        bg_colour: (b.bg_colour.r, b.bg_colour.g, b.bg_colour.b).into(),
                        clear_screen: b.clear_screen,
                        creation_code,
                        backgrounds: b
                            .backgrounds
                            .into_iter()
                            .map(|bg| background::Background {
                                visible: bg.visible_on_start,
                                is_foreground: bg.is_foreground,
                                background_id: bg.source_bg,
                                x_offset: f64::from(bg.xoffset),
                                y_offset: f64::from(bg.yoffset),
                                tile_horizontal: bg.tile_horz,
                                tile_vertical: bg.tile_vert,
                                hspeed: f64::from(bg.hspeed),
                                vspeed: f64::from(bg.vspeed),
                                stretch: bg.stretch,
                            })
                            .collect(),
                        views_enabled: b.views_enabled,
                        views: b
                            .views
                            .into_iter()
                            .map(|v| view::View {
                                visible: v.visible,
                                source_x: v.source_x,
                                source_y: v.source_y,
                                source_w: v.source_w,
                                source_h: v.source_h,
                                port_x: v.port_x,
                                port_y: v.port_y,
                                port_w: v.port_w,
                                port_h: v.port_h,
                                follow_target: v.following.target,
                                follow_hborder: v.following.hborder,
                                follow_vborder: v.following.vborder,
                                follow_hspeed: v.following.hspeed,
                                follow_vspeed: v.following.vspeed,
                            })
                            .collect(),
                        instances: b
                            .instances
                            .into_iter()
                            .map(|i| {
                                Ok(room::Instance {
                                    x: i.x,
                                    y: i.y,
                                    object: i.object,
                                    id: i.id as usize,
                                    creation: match compiler.compile(&i.creation_code) {
                                        Ok(c) => c,
                                        Err(e) => {
                                            return Err(format!(
                                                "Compiler error in creation code of instance {}: {}",
                                                i.id, e
                                            ));
                                        },
                                    },
                                })
                            })
                            .collect::<Result<Vec<_>, _>>()?,
                        tiles: b
                            .tiles
                            .into_iter()
                            .map(|t| tile::Tile {
                                x: f64::from(t.x),
                                y: f64::from(t.y),
                                background_index: t.source_bg,
                                tile_x: t.tile_x,
                                tile_y: t.tile_y,
                                width: t.width,
                                height: t.height,
                                depth: t.depth,
                                id: t.id as usize,
                                alpha: 1.0,
                                blend: 0xFFFFFF,
                                xscale: 1.0,
                                yscale: 1.0,
                                visible: true,
                            })
                            .collect(),
                    }))
                })
                .transpose()
            })
            .collect::<Result<Vec<_>, _>>()?;

        renderer.upload_atlases(atlases)?;

        let mut game = Self {
            compiler,
            glfw,
            glfw_events: events,
            instance_list: InstanceList::new(),
            tile_list: TileList::new(),
            rand: Random::new(),
            renderer: Box::new(renderer),
            assets: Assets { backgrounds, fonts, objects, rooms, scripts, sprites, timelines },
            room_id: room1_id,
            room_width: room1_width as i32,
            room_height: room1_height as i32,
        };

        game.load_room(room1_id)?;

        // Important: show window
        game.renderer.show_window();

        Ok(game)
    }

    pub fn load_room(&mut self, room_id: i32) -> Result<(), Box<dyn std::error::Error>> {
        self.instance_list.remove_with(|instance| !instance.persistent.get());
        if let Some(Some(room)) = self.assets.rooms.get(room_id as usize) {
            for instance in room.instances.iter() {
                let object = match self.assets.objects.get(instance.object as usize) {
                    Some(&Some(ref o)) => o.as_ref(),
                    _ => return Err(format!("Instance of invalid Object in room {}", room.name).into()),
                };
                self.instance_list.insert(Instance::new(
                    instance.id as _,
                    f64::from(instance.x),
                    f64::from(instance.y),
                    instance.object,
                    object,
                ));
            }
            for tile in room.tiles.iter() {
                self.tile_list.insert(*tile);
            }
            self.renderer.set_background_colour(if room.clear_screen { Some(room.bg_colour) } else { None });
            Ok(())
        } else {
            Err(format!("Tried to load non-existent room with id {}", room_id).into())
        }
    }
}
