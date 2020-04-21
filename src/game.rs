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
    gml::{self, ev, file::FileManager, rand::Random, Compiler, Context},
    input::{self, InputManager},
    instance::{DummyFieldHolder, Instance, InstanceState},
    instancelist::{InstanceList, TileList},
    render::{opengl::OpenGLRenderer, Renderer, RendererOptions},
    tile,
    types::ID,
    view::View,
};
use gm8exe::{GameAssets, GameVersion};
use indexmap::IndexMap;
use std::{
    cell::RefCell,
    cmp::Ordering,
    collections::{HashMap, HashSet},
    hint::unreachable_unchecked,
    iter::repeat,
    rc::Rc,
};
use winit::{
    dpi::PhysicalSize,
    event::{ElementState, Event, KeyboardInput, MouseButton, MouseScrollDelta, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

/// Structure which contains all the components of a game.
pub struct Game {
    pub compiler: Compiler,
    pub file_manager: FileManager,
    pub instance_list: InstanceList,
    pub tile_list: TileList,
    pub rand: Random,
    pub renderer: Box<dyn Renderer>,
    pub input_manager: InputManager,
    pub assets: Assets,
    pub event_holders: [IndexMap<u32, Rc<RefCell<Vec<i32>>>>; 12],

    pub last_instance_id: ID,
    pub last_tile_id: ID,

    pub views_enabled: bool,
    pub views: Vec<View>,
    pub backgrounds: Vec<background::Background>,

    pub room_id: i32,
    pub room_width: i32,
    pub room_height: i32,
    pub room_order: Box<[i32]>,
    pub room_speed: u32,
    pub room_target: Option<ID>,

    pub globals: DummyFieldHolder,

    pub uninit_fields_are_zero: bool,
    pub uninit_args_are_zero: bool,

    pub transition_kind: i32,  // default 0
    pub transition_steps: i32, // default 80
    pub score: i32,            // default 0
    pub score_capt: Rc<str>,   // default "Score: "
    pub score_capt_d: bool,    // display in caption?
    pub lives: i32,            // default -1
    pub lives_capt: Rc<str>,   // default "Lives: "
    pub lives_capt_d: bool,    // display in caption?
    pub health: f64,           // default 100.0
    pub health_capt: Rc<str>,  // default "Health: "
    pub health_capt_d: bool,   // display in caption?

    pub game_id: i32,
    pub gm_version: GameVersion,

    // window caption
    pub caption: Rc<str>,
    pub caption_stale: bool,

    // winit windowing
    event_loop: Option<EventLoop<()>>,
    window: Window,
    // Width the window is supposed to have, assuming it hasn't been resized by the user
    unscaled_width: u32,
    // Height the window is supposed to have, assuming it hasn't been resized by the user
    unscaled_height: u32,
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
            game_id,
            backgrounds,
            constants,
            fonts,
            icon_data,
            last_instance_id,
            last_tile_id,
            objects,
            paths,
            room_order,
            rooms,
            scripts,
            settings,
            sounds,
            sprites,
            timelines,
            triggers,
            version,
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
        let room1_speed = room1.speed;

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
            global_clear_colour: settings.clear_colour.into(),
            resizable: settings.allow_resize,
            on_top: settings.window_on_top,
            decorations: !settings.dont_draw_border,
            fullscreen: settings.fullscreen,
            vsync: settings.vsync, // TODO: Overrideable
        };

        // TODO: fullscreening
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_inner_size(PhysicalSize::new(options.size.0, options.size.1))
            .with_visible(false)
            .build(&event_loop)?;

        let mut renderer = OpenGLRenderer::new(options, &window)?;

        renderer.swap_interval(0); // no vsync

        let mut atlases = AtlasBuilder::new(renderer.max_gpu_texture_size() as _);

        //println!("GPU Max Texture Size: {}", renderer.max_gpu_texture_size());

        let sprites = sprites
            .into_iter()
            .map(|o| {
                o.map(|b| {
                    let (w, h) = b.frames.first().map_or((0, 0), |f| (f.width, f.height));
                    let origin_x = b.origin_x;
                    let origin_y = b.origin_y;
                    Ok(Box::new(Sprite {
                        name: b.name.into(),
                        frames: b
                            .frames
                            .into_iter()
                            .map(|f| {
                                Ok(Frame {
                                    width: f.width,
                                    height: f.height,
                                    atlas_ref: atlases
                                        .texture(f.width as _, f.height as _, origin_x, origin_y, f.data)
                                        .ok_or(())?,
                                })
                            })
                            .collect::<Result<_, ()>>()?,
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
                    }))
                })
                .transpose()
            })
            .collect::<Result<Vec<_>, ()>>()
            .expect("failed to pack sprites");

        let backgrounds = backgrounds
            .into_iter()
            .map(|o| {
                o.map(|b| {
                    let width = b.width;
                    let height = b.height;
                    Ok(Box::new(Background {
                        name: b.name.into(),
                        width,
                        height,
                        atlas_ref: match b.data {
                            Some(data) => Some(atlases.texture(width as _, height as _, 0, 0, data).ok_or(())?),
                            None => None,
                        },
                    }))
                })
                .transpose()
            })
            .collect::<Result<Vec<_>, ()>>()
            .expect("failed to pack backgrounds");

        let fonts = fonts
            .into_iter()
            .map(|o| {
                o.map(|b| {
                    Ok(Box::new(Font {
                        name: b.name.into(),
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
                            .ok_or(())?,
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
                    }))
                })
                .transpose()
            })
            .collect::<Result<Vec<_>, ()>>()
            .expect("failed to pack fonts");

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
                        let mut events: [HashMap<u32, Rc<RefCell<Tree>>>; 12] = std::default::Default::default();
                        for ((i, map), input) in events.iter_mut().enumerate().zip(b.events.iter()) {
                            map.reserve(input.len());
                            for (sub, actions) in input {
                                map.insert(*sub, match Tree::from_list(actions, &mut compiler) {
                                    Ok(t) => Rc::new(RefCell::new(t)),
                                    Err(e) => {
                                        return Err(format!(
                                            "Compiler error in object {} event {},{}: {}",
                                            b.name, i, sub, e
                                        ))
                                    },
                                });
                            }
                        }
                        Ok(Box::new(Object {
                            name: b.name.into(),
                            solid: b.solid,
                            visible: b.visible,
                            persistent: b.persistent,
                            depth: b.depth,
                            sprite_index: b.sprite_index,
                            mask_index: b.mask_index,
                            parent_index: b.parent_index,
                            events,
                            identities: Rc::new(RefCell::new(HashSet::new())),
                            children: Rc::new(RefCell::new(HashSet::new())),
                        }))
                    })
                    .transpose()
                })
                .collect::<Result<Vec<_>, _>>()?;

            // Populate identity lists
            for (i, object) in objects.iter_mut().enumerate().filter_map(|(i, x)| x.as_mut().map(|x| (i, x))) {
                object.identities.borrow_mut().insert(i as _);
                object.children.borrow_mut().insert(i as _);
                let mut parent_index = object.parent_index;
                while parent_index >= 0 {
                    object.identities.borrow_mut().insert(parent_index);
                    if let Some(Some(parent)) = object_parents.get(parent_index as usize) {
                        parent_index = *parent;
                    } else {
                        return Err(format!(
                            "Invalid parent tree for object {}: non-existent object: {}",
                            object.name, parent_index
                        )
                        .into())
                    }
                }
            }
            for (i, mut parent_index) in
                object_parents.iter().enumerate().filter_map(|(i, x)| x.as_ref().map(|x| (i, *x)))
            {
                while parent_index >= 0 {
                    if let Some(Some(parent)) = objects.get_mut(parent_index as usize) {
                        parent.children.borrow_mut().insert(i as _);
                        parent_index = parent.parent_index;
                    } else {
                        return Err(format!(
                            "Invalid parent tree for object {}: non-existent object: {}",
                            i, parent_index
                        )
                        .into())
                    }
                }
            }

            objects
        };

        let timelines = timelines
            .into_iter()
            .map(|t| {
                t.map(|b| {
                    let mut moments: HashMap<u32, Rc<RefCell<Tree>>> = HashMap::with_capacity(b.moments.len());
                    for (moment, actions) in b.moments.iter() {
                        match Tree::from_list(actions, &mut compiler) {
                            Ok(t) => {
                                moments.insert(*moment, Rc::new(RefCell::new(t)));
                            },
                            Err(e) => {
                                return Err(format!("Compiler error in timeline {} moment {}: {}", b.name, moment, e))
                            },
                        };
                    }
                    Ok(Box::new(Timeline { name: b.name.into(), moments }))
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
                    Ok(Box::new(Script { name: b.name.into(), source: b.source.into(), compiled }))
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
                    let width = b.width;
                    let height = b.height;
                    Ok(Box::new(Room {
                        name: b.name.into(),
                        caption: b.caption.into(),
                        width,
                        height,
                        speed: b.speed,
                        persistent: b.persistent,
                        bg_colour: (b.bg_colour.r, b.bg_colour.g, b.bg_colour.b).into(),
                        clear_screen: b.clear_screen,
                        creation_code: creation_code,
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
                                xscale: if bg.stretch {
                                    if let Some(bg_asset) = backgrounds.get_asset(bg.source_bg) {
                                        f64::from(width) / f64::from(bg_asset.width)
                                    } else {
                                        f64::from(width)
                                    }
                                } else {
                                    1.0
                                },
                                yscale: if bg.stretch {
                                    if let Some(bg_asset) = backgrounds.get_asset(bg.source_bg) {
                                        f64::from(height) / f64::from(bg_asset.height)
                                    } else {
                                        f64::from(height)
                                    }
                                } else {
                                    1.0
                                },
                                blend: 0xFFFFFF,
                                alpha: 1.0,
                            })
                            .collect::<Vec<_>>()
                            .into(),
                        views_enabled: b.views_enabled,
                        views: b
                            .views
                            .into_iter()
                            .map(|v| View {
                                visible: v.visible,
                                source_x: v.source_x,
                                source_y: v.source_y,
                                source_w: v.source_w,
                                source_h: v.source_h,
                                port_x: v.port_x,
                                port_y: v.port_y,
                                port_w: v.port_w,
                                port_h: v.port_h,
                                angle: 0.0,
                                follow_target: v.following.target,
                                follow_hborder: v.following.hborder,
                                follow_vborder: v.following.vborder,
                                follow_hspeed: v.following.hspeed,
                                follow_vspeed: v.following.vspeed,
                            })
                            .collect::<Vec<_>>()
                            .into(),
                        instances: b
                            .instances
                            .into_iter()
                            .map(|i| {
                                Ok(room::Instance {
                                    x: i.x,
                                    y: i.y,
                                    object: i.object,
                                    id: i.id,
                                    creation: match compiler.compile(&i.creation_code) {
                                        Ok(c) => c,
                                        Err(e) => {
                                            return Err(format!(
                                                "Compiler error in creation code of instance {}: {}",
                                                i.id, e
                                            ))
                                        },
                                    },
                                })
                            })
                            .collect::<Result<Vec<_>, _>>()?
                            .into(),
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
                            .collect::<Vec<_>>()
                            .into(),
                    }))
                })
                .transpose()
            })
            .collect::<Result<Vec<_>, _>>()?;

        // Make event holder lists
        let mut event_holders: [IndexMap<u32, Rc<RefCell<Vec<i32>>>>; 12] = Default::default();
        for object in objects.iter().flatten() {
            for (holder_list, object_events) in event_holders.iter_mut().zip(object.events.iter()) {
                for (sub, _) in object_events.iter() {
                    let mut sub_list = holder_list.entry(*sub).or_insert(Default::default()).borrow_mut();
                    for object_id in object.children.borrow().iter() {
                        if !sub_list.contains(object_id) {
                            sub_list.push(*object_id);
                        }
                    }
                }
            }
        }

        // Swap collision events over to targets and their children etc...
        let collision_holders = &mut event_holders[ev::COLLISION];
        let mut i = 0;
        while let Some(key) = collision_holders.get_index(i).map(|(x, _)| *x) {
            if let Some(Some(object)) = objects.get(key as usize) {
                let list = collision_holders[&key].clone();
                let mut j = 0;
                while let Some(collider) = {
                    let a = list.borrow();
                    a.get(j).copied()
                } {
                    {
                        let mut sub_list =
                            collision_holders.entry(collider as _).or_insert(Default::default()).borrow_mut();
                        for child in object.children.borrow().iter() {
                            if !sub_list.contains(child) {
                                sub_list.push(*child);
                            }
                        }
                    }
                    for child in object.children.borrow().iter().copied() {
                        let mut sub_list =
                            collision_holders.entry(child as _).or_insert(Default::default()).borrow_mut();
                        if !sub_list.contains(&collider) {
                            sub_list.push(collider);
                        }
                    }
                    j += 1;
                }
            }
            i += 1;
        }
        for (sub, list) in collision_holders.iter() {
            list.borrow_mut().retain(|x| *x >= *sub as _);
        }
        event_holders[ev::COLLISION].retain(|_, x| !x.borrow_mut().is_empty());

        // Sort all the event holder lists into ascending order
        for map in event_holders.iter_mut() {
            map.sort_by(|x, _, y, _| x.cmp(y));
            for list in map.values_mut() {
                list.borrow_mut().sort();
            }
        }

        renderer.upload_atlases(atlases)?;

        let mut game = Self {
            compiler,
            file_manager: FileManager::new(),
            instance_list: InstanceList::new(),
            tile_list: TileList::new(),
            rand: Random::new(),
            renderer: Box::new(renderer),
            input_manager: InputManager::new(),
            assets: Assets { backgrounds, fonts, objects, rooms, scripts, sprites, timelines },
            event_holders,
            views_enabled: false,
            views: Vec::new(),
            backgrounds: Vec::new(),
            room_id: room1_id,
            room_width: room1_width as i32,
            room_height: room1_height as i32,
            room_order: room_order.into_boxed_slice(),
            room_speed: room1_speed,
            room_target: None,
            globals: DummyFieldHolder::new(),
            last_instance_id,
            last_tile_id,
            uninit_fields_are_zero: settings.zero_uninitialized_vars,
            uninit_args_are_zero: !settings.error_on_uninitialized_args,
            transition_kind: 0,
            transition_steps: 80,
            score: 0,
            score_capt: "Score: ".to_string().into(),
            lives: -1,
            lives_capt: "Lives: ".to_string().into(),
            health: 100.0,
            health_capt: "Health: ".to_string().into(),
            game_id: game_id as i32,
            gm_version: version,
            caption: "".to_string().into(),
            caption_stale: false,
            score_capt_d: false,
            lives_capt_d: false,
            health_capt_d: false,
            window,
            event_loop: Some(event_loop),

            // load_room sets this
            unscaled_width: 0,
            unscaled_height: 0,
        };

        game.load_room(room1_id)?;
        game.window.set_visible(true);

        Ok(game)
    }

    fn resize_window(&mut self, width: u32, height: u32) {
        // GameMaker only actually resizes the window if the expected (unscaled) size is changing.
        if self.unscaled_width != width || self.unscaled_height != height {
            self.unscaled_width = width;
            self.unscaled_height = height;
            self.window.set_inner_size(PhysicalSize::new(width, height));
        }
    }

    pub fn load_room(&mut self, room_id: i32) -> Result<(), Box<dyn std::error::Error>> {
        let room = if let Some(Some(room)) = self.assets.rooms.get(room_id as usize) {
            room.clone()
        } else {
            return Err(format!("Tried to load non-existent room with id {}", room_id).into())
        };

        // Run room end event for each instance
        let mut iter = self.instance_list.iter_by_insertion();
        while let Some(instance) = iter.next(&self.instance_list) {
            self.run_instance_event(ev::OTHER, 5, instance, instance)?;
        }

        // Delete non-persistent instances
        // TODO: back up remaining instances and put them at the END of insertion order after making new ones
        self.instance_list.remove_with(|instance| !instance.persistent.get());

        // Update renderer
        let (view_width, view_height) = {
            if !room.views_enabled {
                (room.width, room.height)
            } else {
                let xw = |view: &View| view.port_x + (view.port_w as i32);
                let yh = |view: &View| view.port_y + (view.port_h as i32);
                let x_max = room
                    .views
                    .iter()
                    .filter(|view| view.visible)
                    .max_by(|v1, v2| xw(v1).cmp(&xw(v2)))
                    .map(xw)
                    .unwrap_or(room.width as i32);
                let y_max = room
                    .views
                    .iter()
                    .filter(|view| view.visible)
                    .max_by(|v1, v2| yh(v1).cmp(&yh(v2)))
                    .map(yh)
                    .unwrap_or(room.height as i32);
                if x_max < 0 || y_max < 0 {
                    return Err(format!("Bad room width/height {},{} loading room {}", x_max, y_max, room_id).into())
                }
                (x_max as u32, y_max as u32)
            }
        };

        self.resize_window(view_width, view_height);
        self.renderer.set_background_colour(if room.clear_screen { Some(room.bg_colour) } else { None });

        // Update views, backgrounds
        // Using clear() followed by extend_from_slice() guarantees re-using vec capacity and avoids unnecessary allocs
        self.views_enabled = room.views_enabled;
        self.views.clear();
        self.views.extend_from_slice(&room.views);
        self.backgrounds.clear();
        self.backgrounds.extend_from_slice(&room.backgrounds);

        // Update some stored vars
        self.room_id = room_id;
        self.room_width = room.width as _;
        self.room_height = room.height as _;
        self.room_speed = room.speed;
        self.room_target = None;
        self.input_manager.clear_presses();

        // Load all tiles in new room
        for tile in room.tiles.iter() {
            self.tile_list.insert(*tile);
        }

        // Load all instances in new room, unless they already exist due to persistence
        for instance in room.instances.iter() {
            if self.instance_list.get_by_instid(instance.id).is_none() {
                // Get object
                let object = match self.assets.objects.get(instance.object as usize) {
                    Some(&Some(ref o)) => o.as_ref(),
                    _ => return Err(format!("Instance of invalid Object in room {}", room.name).into()),
                };

                // Add instance to list
                let handle = self.instance_list.insert(Instance::new(
                    instance.id as _,
                    f64::from(instance.x),
                    f64::from(instance.y),
                    instance.object,
                    object,
                ));

                // Run this instance's room creation code
                self.execute(&instance.creation, &mut Context {
                    this: handle,
                    other: handle,
                    event_action: 0,
                    relative: false,
                    event_type: 11, // GM8 does this for some reason
                    event_number: 0,
                    event_object: instance.object,
                    arguments: Default::default(),
                    argument_count: 0,
                    locals: Default::default(),
                    return_value: Default::default(),
                })?;

                // Run create event for this instance
                self.run_instance_event(ev::CREATE, 0, handle, handle)?;
            }
        }

        // TODO: run room's creation code (event_type = 11)

        // Run room start event for each instance
        let mut iter = self.instance_list.iter_by_insertion();
        while let Some(instance) = iter.next(&self.instance_list) {
            self.run_instance_event(ev::OTHER, 4, instance, instance)?;
        }

        if let Some(target) = self.room_target {
            // A room change has been requested during this room change, so let's recurse...
            self.load_room(target)
        } else {
            // Draw "frame 0" and then return
            self.draw()?;
            Ok(())
        }
    }

    /// Runs a frame loop and draws the screen. Exits immediately, without waiting for any FPS limitation.
    pub fn frame(&mut self) -> gml::Result<()> {
        // Update xprevious and yprevious for all instances
        let mut iter = self.instance_list.iter_by_insertion();
        while let Some(instance) = iter.next(&self.instance_list).and_then(|x| self.instance_list.get(x)) {
            instance.xprevious.set(instance.x.get());
            instance.yprevious.set(instance.y.get());
        }

        // Begin step event
        self.run_object_event(ev::STEP, 1, None)?;
        if self.room_target.is_some() {
            return Ok(())
        }

        // Step event
        self.run_object_event(ev::STEP, 0, None)?;
        if self.room_target.is_some() {
            return Ok(())
        }

        // Movement: apply friction, gravity, and hspeed/vspeed
        let mut iter = self.instance_list.iter_by_insertion();
        while let Some(instance) = iter.next(&self.instance_list).and_then(|i| self.instance_list.get(i)) {
            let friction = instance.friction.get();
            if friction != 0.0 {
                // "Subtract" friction from speed towards 0
                let speed = instance.speed.get();
                if speed >= 0.0 {
                    if friction > speed {
                        instance.set_speed(0.0);
                    } else {
                        instance.set_speed(speed - friction);
                    }
                } else {
                    if friction > -speed {
                        instance.set_speed(0.0);
                    } else {
                        instance.set_speed(speed + friction);
                    }
                }
            }

            let gravity = instance.gravity.get();
            if gravity != 0.0 {
                // Apply gravity in gravity_direction to hspeed and vspeed
                let gravity_direction = instance.gravity_direction.get().to_radians();
                instance.set_hvspeed(
                    instance.hspeed.get() + (gravity_direction.cos() * gravity),
                    instance.vspeed.get() - (gravity_direction.sin() * gravity),
                );
            }

            // Apply hspeed and vspeed to x and y
            let hspeed = instance.hspeed.get();
            let vspeed = instance.vspeed.get();
            if hspeed != 0.0 || vspeed != 0.0 {
                instance.x.set(instance.x.get() + hspeed);
                instance.y.set(instance.y.get() + vspeed);
                instance.bbox_is_stale.set(true);
            }
        }

        // End step event
        self.run_object_event(ev::STEP, 2, None)?;
        if self.room_target.is_some() {
            return Ok(())
        }

        // Clear out any deleted instances
        self.instance_list.remove_with(|instance| instance.state.get() == InstanceState::Deleted);

        // Draw everything, including running draw events, and return the result
        self.draw()
    }

    /// Draws all instances, tiles and backgrounds to the screen, taking all active views into account.
    /// Note that this function runs GML code associated with object draw events, so its usage must match GameMaker 8.
    fn draw(&mut self) -> gml::Result<()> {
        // Draw all views
        if self.views_enabled {
            // Iter views in a non-borrowing way
            let mut count = 0;
            while let Some(&view) = self.views.get(count) {
                if view.visible {
                    self.draw_view(
                        view.source_x,
                        view.source_y,
                        view.source_w as _,
                        view.source_h as _,
                        view.port_x,
                        view.port_y,
                        view.port_w as _,
                        view.port_h as _,
                        view.angle,
                    )?;
                }
                count += 1;
            }
        } else {
            self.draw_view(0, 0, self.room_width, self.room_height, 0, 0, self.room_width, self.room_height, 0.0)?;
        }

        // Tell renderer to finish the frame and start the next one
        let (width, height) = self.window.inner_size().into();
        self.renderer.finish(width, height);

        // Clear inputs for this frame
        self.input_manager.clear_presses();

        Ok(())
    }

    /// Draws everything in the scene using a given view rectangle
    fn draw_view(
        &mut self,
        src_x: i32,
        src_y: i32,
        src_w: i32,
        src_h: i32,
        port_x: i32,
        port_y: i32,
        port_w: i32,
        port_h: i32,
        angle: f64,
    ) -> gml::Result<()> {
        let (width, height) = self.window.inner_size().into();
        self.renderer.set_view(
            width,
            height,
            self.unscaled_width,
            self.unscaled_height,
            src_x,
            src_y,
            src_w,
            src_h,
            angle.to_radians(),
            port_x,
            port_y,
            port_w,
            port_h,
        );

        fn draw_instance(game: &mut Game, idx: usize) {
            let instance = game.instance_list.get(idx).unwrap_or_else(|| unsafe { unreachable_unchecked() });
            if let Some(Some(sprite)) = game.assets.sprites.get(instance.sprite_index.get() as usize) {
                let image_index = instance.image_index.get().floor() as i32 % sprite.frames.len() as i32;
                let atlas_ref = match sprite.frames.get(image_index as usize) {
                    Some(f1) => &f1.atlas_ref,
                    None => return, // sprite with 0 frames?
                };
                game.renderer.draw_sprite(
                    atlas_ref,
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

        fn draw_tile(game: &mut Game, idx: usize) {
            let tile = game.tile_list.get(idx).unwrap_or_else(|| unsafe { unreachable_unchecked() });
            if let Some(Some(background)) = game.assets.backgrounds.get(tile.background_index as usize) {
                if let Some(atlas) = &background.atlas_ref {
                    game.renderer.draw_sprite_partial(
                        atlas,
                        tile.tile_x as _,
                        tile.tile_y as _,
                        tile.width as _,
                        tile.height as _,
                        tile.x,
                        tile.y,
                        tile.xscale,
                        tile.yscale,
                        0.0,
                        tile.blend,
                        tile.alpha,
                    )
                }
            }
        }

        // draw backgrounds
        for background in self.backgrounds.iter().filter(|x| x.visible && !x.is_foreground) {
            if let Some(atlas_ref) =
                self.assets.backgrounds.get_asset(background.background_id).and_then(|x| x.atlas_ref.as_ref())
            {
                self.renderer.draw_sprite(
                    atlas_ref,
                    background.x_offset,
                    background.y_offset,
                    background.xscale,
                    background.yscale,
                    0.0,
                    background.blend,
                    background.alpha,
                );
            }
        }

        self.instance_list.draw_sort();
        let mut iter_inst = self.instance_list.iter_by_drawing();
        let mut iter_inst_v = iter_inst.next(&self.instance_list);
        self.tile_list.draw_sort();
        let mut iter_tile = self.tile_list.iter_by_drawing();
        let mut iter_tile_v = iter_tile.next(&self.tile_list);
        loop {
            match (iter_inst_v, iter_tile_v) {
                (Some(idx_inst), Some(idx_tile)) => {
                    let inst = self.instance_list.get(idx_inst).unwrap_or_else(|| unsafe { unreachable_unchecked() });
                    let tile = self.tile_list.get(idx_tile).unwrap_or_else(|| unsafe { unreachable_unchecked() });
                    match inst.depth.get().cmp(&tile.depth) {
                        Ordering::Greater | Ordering::Equal => {
                            draw_instance(self, idx_inst);
                            iter_inst_v = iter_inst.next(&self.instance_list);
                        },
                        Ordering::Less => {
                            draw_tile(self, idx_tile);
                            iter_tile_v = iter_tile.next(&self.tile_list);
                        },
                    }
                },
                (Some(idx_inst), None) => {
                    draw_instance(self, idx_inst);
                    while let Some(idx_inst) = iter_inst.next(&self.instance_list) {
                        draw_instance(self, idx_inst);
                    }
                    break
                },
                (None, Some(idx_tile)) => {
                    draw_tile(self, idx_tile);
                    while let Some(idx_tile) = iter_tile.next(&self.tile_list) {
                        draw_tile(self, idx_tile);
                    }
                    break
                },
                (None, None) => break,
            }
        }

        // draw foregrounds
        for background in self.backgrounds.iter().filter(|x| x.visible && x.is_foreground) {
            if let Some(atlas_ref) =
                self.assets.backgrounds.get_asset(background.background_id).and_then(|x| x.atlas_ref.as_ref())
            {
                self.renderer.draw_sprite(
                    atlas_ref,
                    background.x_offset,
                    background.y_offset,
                    background.xscale,
                    background.yscale,
                    0.0,
                    background.blend,
                    background.alpha,
                );
            }
        }

        Ok(())
    }

    /// Runs an event for all objects which hold the given event.
    /// If no "other" instance is provided, "self" will be used as "other". This is what GM8 tends to do.
    fn run_object_event(&mut self, event_id: usize, event_sub: u32, other: Option<usize>) -> gml::Result<()> {
        let holders = match self.event_holders.get(event_id).and_then(|x| x.get(&event_sub)) {
            Some(e) => e.clone(),
            None => return Ok(()),
        };
        let mut position = 0;
        while let Some(&object_id) = holders.borrow().get(position) {
            let mut iter = self.instance_list.iter_by_object(object_id);
            while let Some(instance) = iter.next(&self.instance_list) {
                self.run_instance_event(event_id, event_sub, instance, other.unwrap_or(instance))?;
            }
            position += 1;
        }
        Ok(())
    }

    /// Runs an event for a given instance. Does nothing if that instance doesn't have the specified event.
    pub fn run_instance_event(
        &mut self,
        event_id: usize,
        event_sub: u32,
        instance: usize,
        other: usize,
    ) -> gml::Result<()> {
        // Running instance events is not allowed if a room change is pending. This appears to be
        // how GM8 is implemented as well, given the related room creation bug and collision/solid bugs.
        if self.room_target.is_none() {
            let mut object_id =
                self.instance_list.get(instance).ok_or(gml::Error::InvalidInstanceHandle(instance))?.object_index.get();
            let event = loop {
                if object_id < 0 {
                    return Ok(())
                }
                if let Some(Some(object)) = self.assets.objects.get(object_id as usize) {
                    if let Some(event) = object.events.get(event_id).and_then(|x| x.get(&event_sub)) {
                        break event.clone()
                    } else {
                        object_id = object.parent_index;
                    }
                } else {
                    return Ok(())
                }
            };

            self.execute_tree(event, instance, other, event_id, event_sub as _, object_id)
        } else {
            Ok(())
        }
    }

    pub fn run(mut self) {
        use std::{
            thread,
            time::{Duration, Instant},
        };

        let event_loop = self.event_loop.take().unwrap();
        let mut now = std::time::Instant::now();
        event_loop.run(move |event, _, control_flow| {
            // ControlFlow::Poll continuously runs the event loop, even if the OS hasn't
            // dispatched any events. This is ideal for games and similar applications.
            *control_flow = ControlFlow::Poll;

            match event {
                Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                    println!("The close button was pressed; stopping");
                    *control_flow = ControlFlow::Exit
                },

                Event::WindowEvent {
                    event:
                        WindowEvent::KeyboardInput {
                            input: KeyboardInput { scancode, state: ElementState::Pressed, .. },
                            ..
                        },
                    ..
                } => {
                    // Keyboard key press with given scancode
                    self.input_manager.key_press(scancode);
                },

                Event::WindowEvent {
                    event:
                        WindowEvent::KeyboardInput {
                            input: KeyboardInput { scancode, state: ElementState::Released, .. },
                            ..
                        },
                    ..
                } => {
                    // Keyboard key release with given scancode
                    self.input_manager.key_release(scancode);
                },

                Event::WindowEvent { event: WindowEvent::CursorMoved { position, .. }, .. } => {
                    // Cursor movement within window
                    self.input_manager.set_mouse_pos(position.x, position.y);
                },

                Event::WindowEvent {
                    event: WindowEvent::MouseInput { button, state: ElementState::Pressed, .. },
                    ..
                } => match button {
                    // Mouse button press
                    MouseButton::Left => self.input_manager.mouse_press(input::MB_LEFT),
                    MouseButton::Right => self.input_manager.mouse_press(input::MB_RIGHT),
                    MouseButton::Middle => self.input_manager.mouse_press(input::MB_MIDDLE),
                    _ => (),
                },

                Event::WindowEvent {
                    event: WindowEvent::MouseInput { button, state: ElementState::Released, .. },
                    ..
                } => match button {
                    // Mouse button release
                    MouseButton::Left => self.input_manager.mouse_release(input::MB_LEFT),
                    MouseButton::Right => self.input_manager.mouse_release(input::MB_RIGHT),
                    MouseButton::Middle => self.input_manager.mouse_release(input::MB_MIDDLE),
                    _ => (),
                },

                Event::WindowEvent { event: WindowEvent::MouseWheel { delta, .. }, .. } => {
                    // Mouse wheel scrolled
                    // Note: we don't care if the scroll distance is in lines or pixels,
                    // because we only care whether it's positive (up) or negative (down)
                    let y = match delta {
                        MouseScrollDelta::LineDelta(_, y) => f64::from(y),
                        MouseScrollDelta::PixelDelta(p) => p.y,
                    };

                    if y > 0.0 {
                        self.input_manager.mouse_scroll_up();
                    } else if y < 0.0 {
                        self.input_manager.mouse_scroll_down();
                    }
                },

                Event::MainEventsCleared => {
                    self.window.request_redraw();
                },

                Event::RedrawRequested(_) => {
                    self.frame().unwrap();
                    if let Some(target) = self.room_target {
                        self.load_room(target).unwrap();
                    }

                    let diff = Instant::now().duration_since(now);
                    if let Some(slep) = Duration::new(0, 1_000_000_000u32 / self.room_speed).checked_sub(diff) {
                        thread::sleep(slep);
                    }
                },

                Event::RedrawEventsCleared => {
                    now = Instant::now();
                },

                _ => (),
            }
        });
    }
}

pub trait GetAsset<T> {
    fn get_asset(&self, index: ID) -> Option<&T>;
    fn get_asset_mut(&mut self, index: ID) -> Option<&mut T>;
}

impl<T> GetAsset<T> for Vec<Option<T>> {
    fn get_asset(&self, index: ID) -> Option<&T> {
        if index < 0 {
            None
        } else {
            match self.get(index as usize) {
                Some(Some(t)) => Some(t),
                _ => None,
            }
        }
    }

    fn get_asset_mut(&mut self, index: ID) -> Option<&mut T> {
        if index < 0 {
            None
        } else {
            match self.get_mut(index as usize) {
                Some(Some(t)) => Some(t),
                _ => None,
            }
        }
    }
}
