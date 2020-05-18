pub mod background;
pub mod draw;
pub mod events;
pub mod movement;
pub mod view;
pub mod window;

pub use background::Background;
pub use view::View;
pub use window::Window;

use crate::{
    action::Tree,
    asset::{
        self,
        font::{Character, Font},
        path::{self, Path},
        room::{self, Room},
        sprite::{Collider, Frame, Sprite},
        Object, Script, Timeline,
    },
    atlas::AtlasBuilder,
    gml::{self, ev, file::FileManager, rand::Random, Compiler, Context},
    input::InputManager,
    instance::{DummyFieldHolder, Instance, InstanceState},
    instancelist::{InstanceList, TileList},
    render::{opengl::OpenGLRenderer, Renderer, RendererOptions},
    replay::{self, Replay},
    tile,
    types::{Colour, ID},
    util,
};
use gm8exe::{GameAssets, GameVersion};
use indexmap::IndexMap;
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    iter::repeat,
    path::PathBuf,
    rc::Rc,
    thread,
    time::{Duration, Instant},
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
    pub event_holders: [IndexMap<u32, Rc<RefCell<Vec<ID>>>>; 12],
    pub custom_draw_objects: HashSet<ID>,

    pub last_instance_id: ID,
    pub last_tile_id: ID,

    pub views_enabled: bool,
    pub view_current: usize,
    pub views: Vec<View>,
    pub backgrounds: Vec<background::Background>,

    pub room_id: i32,
    pub room_width: i32,
    pub room_height: i32,
    pub room_order: Box<[i32]>,
    pub room_speed: u32,
    pub room_target: Option<ID>,

    pub globals: DummyFieldHolder,
    pub game_start: bool,

    pub draw_font: Option<Font>, // TODO: make this not an option when we have a default font
    pub draw_font_id: ID,
    pub draw_colour: Colour,
    pub draw_alpha: f64,
    pub draw_halign: draw::Halign,
    pub draw_valign: draw::Valign,

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
    pub window: Window,
    // Width the window is supposed to have, assuming it hasn't been resized by the user
    unscaled_width: u32,
    // Height the window is supposed to have, assuming it hasn't been resized by the user
    unscaled_height: u32,
}

pub struct Assets {
    pub backgrounds: Vec<Option<Box<asset::Background>>>,
    pub fonts: Vec<Option<Box<Font>>>,
    pub objects: Vec<Option<Box<Object>>>,
    pub paths: Vec<Option<Box<Path>>>,
    pub rooms: Vec<Option<Box<Room>>>,
    pub scripts: Vec<Option<Box<Script>>>,
    pub sprites: Vec<Option<Box<Sprite>>>,
    pub timelines: Vec<Option<Box<Timeline>>>,
    // todo
}

impl Game {
    pub fn launch(assets: GameAssets, file_path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        // Parse file path
        let mut file_path2 = file_path.clone();
        file_path2.pop();
        std::env::set_current_dir(&file_path2)?;
        let mut param_string: &str = &file_path.to_string_lossy();
        let mut program_directory: &str = &file_path2.to_string_lossy();

        if cfg!(target_os = "windows") {
            param_string = param_string.trim_start_matches("\\\\?\\");
            program_directory = program_directory.trim_start_matches("\\\\?\\");
        }
        // TODO: store these as Rc<str> probably?
        println!("param_string: {}", param_string);
        println!("program_directory: {}", program_directory);

        // Destructure assets
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

        // Helper fn for registering asset names as constants
        fn register_all<T>(compiler: &mut Compiler, assets: &[Option<T>], get_name: fn(&T) -> String) {
            assets
                .iter()
                .enumerate()
                .filter_map(|(i, x)| x.as_ref().map(|x| (i, x)))
                .for_each(|(i, x)| compiler.register_constant(get_name(x), i as f64))
        }

        // Register all asset names
        // These are in order of asset precedence, please don't change the order
        register_all(&mut compiler, &objects, |x| x.name.clone());
        register_all(&mut compiler, &sprites, |x| x.name.clone());
        register_all(&mut compiler, &sounds, |x| x.name.clone());
        register_all(&mut compiler, &backgrounds, |x| x.name.clone());
        register_all(&mut compiler, &paths, |x| x.name.clone());
        register_all(&mut compiler, &fonts, |x| x.name.clone());
        register_all(&mut compiler, &timelines, |x| x.name.clone());
        register_all(&mut compiler, &scripts, |x| x.name.clone());
        register_all(&mut compiler, &rooms, |x| x.name.clone());
        register_all(&mut compiler, &triggers, |x| x.constant_name.clone());

        // Register scripts
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

        let (width, height) = options.size;
        let wb = window::WindowBuilder::new().with_size(width, height);

        // TODO: specific flags here (make wb mutable)

        let window = wb.build().expect("oh no");
        let mut renderer = OpenGLRenderer::new(options, &window)?;

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
                    Ok(Box::new(asset::Background {
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
                    let chars = b
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
                        .collect::<Rc<_>>();
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
                        tallest_char_height: chars.iter().map(|x| x.height).max().unwrap_or_default(),
                        chars,
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

        let paths = paths
            .into_iter()
            .map(|t| {
                t.map(|b| {
                    let mut path = Path {
                        name: b.name.into(),
                        points: b
                            .points
                            .into_iter()
                            .map(|point| path::Point { x: point.x, y: point.y, speed: point.speed })
                            .collect(),
                        control_nodes: Default::default(),
                        length: Default::default(),
                        curve: b.connection as u32 == 1,
                        closed: b.closed,
                        precision: b.precision.min(8) as _, // ghetto clamp
                        start: Default::default(),
                        end: Default::default(),
                    };
                    path.update();
                    Box::new(path)
                })
            })
            .collect();

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
                    Ok(Box::new(Timeline { name: b.name.into(), moments: Rc::new(RefCell::new(moments)) }))
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
                            .map(|bg| Background {
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

        // Make list of objects with custom draw events
        let custom_draw_objects =
            event_holders[ev::DRAW].iter().flat_map(|(_, x)| x.borrow().iter().copied().collect::<Vec<_>>()).collect();

        renderer.upload_atlases(atlases)?;

        let mut game = Self {
            compiler,
            file_manager: FileManager::new(),
            instance_list: InstanceList::new(),
            tile_list: TileList::new(),
            rand: Random::new(),
            renderer: Box::new(renderer),
            input_manager: InputManager::new(),
            assets: Assets { backgrounds, fonts, objects, paths, rooms, scripts, sprites, timelines },
            event_holders,
            custom_draw_objects,
            views_enabled: false,
            view_current: 0,
            views: Vec::new(),
            backgrounds: Vec::new(),
            room_id: room1_id,
            room_width: room1_width as i32,
            room_height: room1_height as i32,
            room_order: room_order.into_boxed_slice(),
            room_speed: room1_speed,
            room_target: None,
            globals: DummyFieldHolder::new(),
            game_start: true,
            draw_font: None,
            draw_font_id: -1,
            draw_colour: Colour::new(0.0, 0.0, 0.0),
            draw_alpha: 1.0,
            draw_halign: draw::Halign::Left,
            draw_valign: draw::Valign::Top,
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

            // load_room sets this
            unscaled_width: 0,
            unscaled_height: 0,
        };

        game.load_room(room1_id)?;
        game.window.set_visible(true);
        game.renderer.swap_interval(0); // no vsync

        Ok(game)
    }

    fn resize_window(&mut self, width: u32, height: u32) {
        // GameMaker only actually resizes the window if the expected (unscaled) size is changing.
        if self.unscaled_width != width || self.unscaled_height != height {
            self.unscaled_width = width;
            self.unscaled_height = height;
            self.window.resize(width, height);
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
            self.run_instance_event(ev::OTHER, 5, instance, instance, None)?;
        }

        // Delete non-persistent instances and all tiles
        // TODO: back up remaining instances and put them at the END of insertion order after making new ones
        self.instance_list.remove_with(|instance| !instance.persistent.get());
        self.tile_list.clear();

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
                self.run_instance_event(ev::CREATE, 0, handle, handle, None)?;
            }
        }

        if self.game_start {
            // Run game start event for each instance
            let mut iter = self.instance_list.iter_by_insertion();
            while let Some(instance) = iter.next(&self.instance_list) {
                self.run_instance_event(ev::OTHER, 4, instance, instance, None)?;
            }
            self.game_start = false;
        }

        // TODO: run room's creation code (event_type = 11)

        // Run room start event for each instance
        let mut iter = self.instance_list.iter_by_insertion();
        while let Some(instance) = iter.next(&self.instance_list) {
            self.run_instance_event(ev::OTHER, 4, instance, instance, None)?;
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
            instance.path_positionprevious.set(instance.path_position.get());
        }

        // Begin step event
        self.run_object_event(ev::STEP, 1, None)?;
        if self.room_target.is_some() {
            return Ok(())
        }

        // Advance timelines for all instances
        let mut iter = self.instance_list.iter_by_insertion();
        while let Some(handle) = iter.next(&self.instance_list) {
            let instance = self.instance_list.get(handle).unwrap();
            let object_index = instance.object_index.get();
            if instance.timeline_running.get() {
                if let Some(timeline) = self.assets.timelines.get_asset(instance.timeline_index.get()) {
                    let old_position = instance.timeline_position.get();
                    let new_position = old_position + instance.timeline_speed.get();
                    instance.timeline_position.set(new_position);

                    let moments = timeline.moments.clone();
                    for (moment, tree) in moments.borrow().iter() {
                        let f_moment = f64::from(*moment);
                        if f_moment >= old_position && f_moment < new_position {
                            self.execute_tree(tree.clone(), handle, handle, 0, 0, object_index)?;
                        }
                    }
                }
            }
        }

        // Alarm events
        self.run_alarms()?;
        if self.room_target.is_some() {
            return Ok(())
        }

        // Key events
        self.run_keyboard_events()?;
        if self.room_target.is_some() {
            return Ok(())
        }

        // TODO: Mouse events go here

        // Key press events
        self.run_key_press_events()?;
        if self.room_target.is_some() {
            return Ok(())
        }

        // Key release events
        self.run_key_release_events()?;
        if self.room_target.is_some() {
            return Ok(())
        }

        // Step event
        self.run_object_event(ev::STEP, 0, None)?;
        if self.room_target.is_some() {
            return Ok(())
        }

        // Movement: apply friction, gravity, and hspeed/vspeed
        self.process_movement();

        // Outside room, intersect boundary, outside/intersect view
        self.run_bound_events()?;
        if self.room_target.is_some() {
            return Ok(())
        }

        // Advance paths
        let mut iter = self.instance_list.iter_by_insertion();
        while let Some(handle) = iter.next(&self.instance_list) {
            let mut run_event = false;
            let instance = self.instance_list.get(handle).unwrap();
            if let Some(path) = self.assets.paths.get_asset(instance.path_index.get()) {
                // Calculate how much offset (0-1) we want to add to the instance's path position
                let offset = instance.path_speed.get() * (instance.path_pointspeed.get() / 100.0) / path.length;

                // Work out what the new position should be
                let new_position = instance.path_position.get() + offset;
                if (new_position <= 0.0 && instance.path_speed.get() < 0.0)
                    || (new_position >= 1.0 && instance.path_speed.get() > 0.0)
                {
                    // Path end
                    let (new_position, path_end_pos) = if instance.path_speed.get() < 0.0 {
                        (new_position.fract() + 1.0, 0.0)
                    } else {
                        (new_position.fract(), 1.0)
                    };
                    match instance.path_endaction.get() {
                        1 => {
                            // Continue from start
                            instance.path_position.set(new_position);
                        },
                        2 => {
                            // Continue from end
                            instance.path_position.set(new_position);
                            let point = path.get_point(path_end_pos);
                            instance.path_xstart.set(point.x);
                            instance.path_ystart.set(point.y);
                        },
                        3 => {
                            // Reverse
                            instance.path_position.set(1.0 - (new_position));
                            instance.path_speed.set(-instance.path_speed.get());
                        },
                        _ => {
                            // Stop
                            instance.path_position.set(path_end_pos);
                            instance.path_index.set(-1);
                        },
                    }

                    // Set flag to run path end event
                    run_event = true;
                } else {
                    // Normally update path_position
                    instance.path_position.set(new_position);
                }

                // Update the instance's actual position based on its new path_position
                let mut point = path.get_point(instance.path_position.get());
                point.x -= path.start.x;
                point.y -= path.start.y;
                point.x *= instance.path_scale.get();
                point.y *= instance.path_scale.get();
                let angle = instance.path_orientation.get().to_radians();
                util::rotate_around(&mut point.x, &mut point.y, 0.0, 0.0, angle.sin(), angle.cos());

                instance.x.set(point.x + instance.path_xstart.get());
                instance.y.set(point.y + instance.path_ystart.get());
                instance.path_pointspeed.set(point.speed);
                instance.bbox_is_stale.set(true);
            }

            // Run path end event
            if run_event {
                self.run_instance_event(ev::OTHER, 8, handle, handle, None)?;
            }
        }

        // Run collision events
        self.run_collisions()?;
        if self.room_target.is_some() {
            return Ok(())
        }

        // End step event
        self.run_object_event(ev::STEP, 2, None)?;
        if self.room_target.is_some() {
            return Ok(())
        }

        // Update views that should be following objects
        if self.views_enabled {
            for view in self.views.iter_mut().filter(|x| x.visible) {
                if let Some(obj) = self.assets.objects.get_asset(view.follow_target) {
                    if let Some(handle) =
                        self.instance_list.iter_by_identity(obj.children.clone()).next(&self.instance_list)
                    {
                        let inst = self.instance_list.get(handle).unwrap();

                        let x = util::ieee_round(inst.x.get());
                        let y = util::ieee_round(inst.y.get());
                        let border_left = x - view.follow_hborder;
                        let border_right = x + view.follow_hborder;
                        let border_top = y - view.follow_vborder;
                        let border_bottom = y + view.follow_vborder;

                        let will_move_left = border_left < view.source_x;
                        let will_move_right = border_right > (view.source_x + view.source_w as i32);
                        let will_move_up = border_top < view.source_y;
                        let will_move_down = border_bottom > (view.source_y + view.source_h as i32);

                        match (will_move_left, will_move_right) {
                            (true, false) => {
                                if view.follow_hspeed < 0 {
                                    view.source_x = border_left;
                                } else {
                                    view.source_x -= (view.source_x - border_left).min(view.follow_hspeed);
                                }
                            },
                            (false, true) => {
                                if view.follow_hspeed < 0 {
                                    view.source_x = border_right - view.source_w as i32;
                                } else {
                                    view.source_x +=
                                        (border_right - (view.source_x + view.source_w as i32)).min(view.follow_hspeed);
                                }
                            },
                            (true, true) => view.source_x = x - (view.source_w / 2) as i32,
                            (false, false) => (),
                        }
                        view.source_x = view.source_x.max(0).min(self.room_width - view.source_w as i32);

                        match (will_move_up, will_move_down) {
                            (true, false) => {
                                if view.follow_vspeed < 0 {
                                    view.source_y = border_top;
                                } else {
                                    view.source_y -= (view.source_y - border_top).min(view.follow_vspeed);
                                }
                            },
                            (false, true) => {
                                if view.follow_vspeed < 0 {
                                    view.source_y = border_bottom - view.source_h as i32;
                                } else {
                                    view.source_y += (border_bottom - (view.source_y + view.source_h as i32))
                                        .min(view.follow_vspeed);
                                }
                            },
                            (true, true) => view.source_y = y - (view.source_h / 2) as i32,
                            (false, false) => (),
                        }
                        view.source_y = view.source_y.max(0).min(self.room_height - view.source_h as i32);
                    }
                }
            }
        }

        // Clear out any deleted instances
        self.instance_list.remove_with(|instance| instance.state.get() == InstanceState::Deleted);

        // Draw everything, including running draw events
        self.draw()?;

        // Advance sprite animations
        let mut iter = self.instance_list.iter_by_insertion();
        while let Some(handle) = iter.next(&self.instance_list) {
            let instance = self.instance_list.get(handle).unwrap();
            let new_index = instance.image_index.get() + instance.image_speed.get();
            instance.image_index.set(new_index);
            if let Some(sprite) = self.assets.sprites.get_asset(instance.sprite_index.get()) {
                let frame_count = sprite.frames.len() as f64;
                if new_index >= frame_count {
                    instance.image_index.set(new_index - frame_count);
                    self.run_instance_event(ev::OTHER, 7, handle, handle, None)?; // animation end event
                }
            }
        }

        Ok(())
    }

    pub fn run(&mut self) -> gml::Result<()> {
        use window::Event;

        let mut time_now = Instant::now();
        loop {
            for event in self.window.process_events().copied() {
                match event {
                    Event::KeyboardDown(key) => self.input_manager.key_press(key),
                    Event::KeyboardUp(key) => self.input_manager.key_release(key),
                    Event::MouseMove(x, y) => self.input_manager.set_mouse_pos(x.into(), y.into()),
                    Event::MouseButtonDown(button) => self.input_manager.mouse_press(button),
                    Event::MouseButtonUp(button) => self.input_manager.mouse_release(button),
                    Event::MouseWheelUp => self.input_manager.mouse_scroll_up(),
                    Event::MouseWheelDown => self.input_manager.mouse_scroll_down(),
                    Event::Resize(w, h) => println!("user resize: width={}, height={}", w, h),
                }
            }

            self.frame()?;
            if let Some(target) = self.room_target {
                self.load_room(target).unwrap();
            }

            // exit if X pressed or game_end() invoked
            if self.window.close_requested() {
                break Ok(())
            }

            // frame limiter
            let diff = Instant::now().duration_since(time_now);
            let duration = Duration::new(0, 1_000_000_000u32 / self.room_speed);
            if let Some(time) = duration.checked_sub(diff) {
                thread::sleep(time);
                time_now += duration;
            } else {
                time_now = Instant::now();
            }
        }
    }

    // Replays some recorded inputs to the game
    pub fn replay(mut self, replay: Replay) -> gml::Result<()> {
        let mut frame_count: usize = 0;
        self.rand.set_seed(replay.start_seed);

        let mut time_now = std::time::Instant::now();
        loop {
            if let Some(frame) = replay.get_frame(frame_count) {
                self.input_manager.set_mouse_pos(f64::from(frame.mouse_x), f64::from(frame.mouse_y));
                for ev in frame.inputs.iter() {
                    match ev {
                        replay::Input::KeyPress(v) => self.input_manager.key_press(*v),
                        replay::Input::KeyRelease(v) => self.input_manager.key_release(*v),
                        replay::Input::MousePress(b) => self.input_manager.mouse_press(*b),
                        replay::Input::MouseRelease(b) => self.input_manager.mouse_release(*b),
                        replay::Input::MouseWheelUp => self.input_manager.mouse_scroll_up(),
                        replay::Input::MouseWheelDown => self.input_manager.mouse_scroll_down(),
                    }
                }
            }

            self.frame()?;
            if let Some(target) = self.room_target {
                self.load_room(target).unwrap();
            }

            // exit if X pressed or game_end() invoked
            if self.window.close_requested() {
                break Ok(())
            }

            // frame limiter
            let diff = Instant::now().duration_since(time_now);
            let duration = Duration::new(0, 1_000_000_000u32 / self.room_speed);
            if let Some(time) = duration.checked_sub(diff) {
                thread::sleep(time);
                time_now += duration;
            } else {
                time_now = Instant::now();
            }

            frame_count += 1;
        }
    }

    // Checks for collision between two instances
    pub fn check_collision(&self, i1: usize, i2: usize) -> bool {
        // Get the sprite masks we're going to use and update instances' bbox vars
        let inst1 = self.instance_list.get(i1).unwrap();
        let inst2 = self.instance_list.get(i2).unwrap();
        let sprite1 = self
            .assets
            .sprites
            .get_asset(if inst1.mask_index.get() < 0 { inst1.sprite_index.get() } else { inst1.mask_index.get() })
            .map(|x| x.as_ref());
        let sprite2 = self
            .assets
            .sprites
            .get_asset(if inst2.mask_index.get() < 0 { inst2.sprite_index.get() } else { inst2.mask_index.get() })
            .map(|x| x.as_ref());
        inst1.update_bbox(sprite1);
        inst2.update_bbox(sprite2);

        // First, an AABB. This is specifically matching how it's coded in GM8 runner.
        if inst1.bbox_right < inst2.bbox_left
            || inst2.bbox_right < inst1.bbox_left
            || inst1.bbox_bottom < inst2.bbox_top
            || inst2.bbox_bottom < inst1.bbox_top
        {
            return false
        }

        // AABB passed - now we do precise pixel checks in the intersection of the two rectangles.
        // Collision cannot be true if either instance does not have a sprite.
        if let (Some(sprite1), Some(sprite2)) = (sprite1, sprite2) {
            // Get the colliders we're going to be colliding with
            let collider1 = match if sprite1.per_frame_colliders {
                sprite1.colliders.get(inst1.image_index.get().floor() as usize % sprite1.colliders.len())
            } else {
                sprite1.colliders.first()
            } {
                Some(c) => c,
                None => return false,
            };

            let collider2 = match if sprite2.per_frame_colliders {
                sprite2.colliders.get(inst2.image_index.get().floor() as usize % sprite2.colliders.len())
            } else {
                sprite2.colliders.first()
            } {
                Some(c) => c,
                None => return false,
            };

            // round x and y values, and get sin and cos of each angle...
            let x1 = util::ieee_round(inst1.x.get());
            let y1 = util::ieee_round(inst1.y.get());
            let x2 = util::ieee_round(inst2.x.get());
            let y2 = util::ieee_round(inst2.y.get());
            let angle1 = inst1.image_angle.get().to_radians();
            let sin1 = angle1.sin();
            let cos1 = angle1.cos();
            let angle2 = inst2.image_angle.get().to_radians();
            let sin2 = angle2.sin();
            let cos2 = angle2.cos();

            // Get intersect rectangle
            let intersect_top = inst1.bbox_top.get().max(inst2.bbox_top.get());
            let intersect_bottom = inst1.bbox_bottom.get().min(inst2.bbox_bottom.get());
            let intersect_left = inst1.bbox_left.get().max(inst2.bbox_left.get());
            let intersect_right = inst1.bbox_right.get().min(inst2.bbox_right.get());

            // Go through each pixel in the intersect
            for intersect_y in intersect_top..=intersect_bottom {
                for intersect_x in intersect_left..=intersect_right {
                    // Cast the coordinates to doubles, rotate them around inst1, then scale them by inst1; then
                    // floor them, as GM8 does, to get integer coordinates on the collider relative to the instance.
                    let mut x = f64::from(intersect_x);
                    let mut y = f64::from(intersect_y);
                    util::rotate_around(&mut x, &mut y, x1.into(), y1.into(), sin1, cos1);
                    let x =
                        (f64::from(sprite1.origin_x) + ((x - f64::from(x1)) / inst1.image_xscale.get()).floor()) as i32;
                    let y =
                        (f64::from(sprite1.origin_y) + ((y - f64::from(y1)) / inst1.image_yscale.get()).floor()) as i32;

                    // Now look in the collider map to figure out if instance 1 is touching this pixel
                    if x >= collider1.bbox_left as i32
                        && y >= collider1.bbox_top as i32
                        && x <= collider1.bbox_right as i32
                        && y <= collider1.bbox_bottom as i32
                        && collider1
                            .data
                            .get((y as usize * collider1.width as usize) + x as usize)
                            .copied()
                            .unwrap_or(false)
                    {
                        // Do all the exact same stuff for inst2 now
                        let mut x = f64::from(intersect_x);
                        let mut y = f64::from(intersect_y);
                        util::rotate_around(&mut x, &mut y, x2.into(), y2.into(), sin2, cos2);
                        let x = (f64::from(sprite2.origin_x) + ((x - f64::from(x2)) / inst2.image_xscale.get()).floor())
                            as i32;
                        let y = (f64::from(sprite2.origin_y) + ((y - f64::from(y2)) / inst2.image_yscale.get()).floor())
                            as i32;

                        // And finally check if there was a hit here too. If so, we can return true immediately.
                        if x >= collider2.bbox_left as i32
                            && y >= collider2.bbox_top as i32
                            && x <= collider2.bbox_right as i32
                            && y <= collider2.bbox_bottom as i32
                            && collider2
                                .data
                                .get((y as usize * collider2.width as usize) + x as usize)
                                .copied()
                                .unwrap_or(false)
                        {
                            return true
                        }
                    }
                }
            }

            false
        } else {
            false
        }
    }

    // Checks if an instance is colliding with a point
    pub fn check_collision_point(&self, inst: usize, x: i32, y: i32) -> bool {
        // Get sprite mask, update bbox
        let inst = self.instance_list.get(inst).unwrap();
        let sprite = self
            .assets
            .sprites
            .get_asset(if inst.mask_index.get() < 0 { inst.sprite_index.get() } else { inst.mask_index.get() })
            .map(|x| x.as_ref());
        inst.update_bbox(sprite);

        // AABB with the point
        if inst.bbox_right.get() < x
            || x < inst.bbox_left.get()
            || inst.bbox_bottom.get() < y
            || y < inst.bbox_top.get()
        {
            return false
        }

        // Can't collide if no sprite or no associated collider
        if let Some(sprite) = sprite {
            // Get collider
            let collider = match if sprite.per_frame_colliders {
                sprite.colliders.get(inst.image_index.get().floor() as usize % sprite.colliders.len())
            } else {
                sprite.colliders.first()
            } {
                Some(c) => c,
                None => return false,
            };

            // Transform point to be relative to collider
            let angle = inst.image_angle.get().to_radians();
            let mut x = f64::from(x);
            let mut y = f64::from(y);
            util::rotate_around(&mut x, &mut y, inst.x.get(), inst.y.get(), angle.sin(), angle.cos());
            let x = util::ieee_round(f64::from(sprite.origin_x) + ((x - inst.x.get()) / inst.image_xscale.get()));
            let y = util::ieee_round(f64::from(sprite.origin_y) + ((y - inst.y.get()) / inst.image_yscale.get()));

            // And finally, check look up this point in the collider
            x >= collider.bbox_left as i32
                && y >= collider.bbox_top as i32
                && x <= collider.bbox_right as i32
                && y <= collider.bbox_bottom as i32
                && collider.data.get((y as usize * collider.width as usize) + x as usize).copied().unwrap_or(false)
        } else {
            false
        }
    }

    // Checks if an instance is colliding with any solid, returning the solid if it is, otherwise None
    pub fn check_collision_solid(&self, inst: usize) -> Option<usize> {
        let mut iter = self.instance_list.iter_by_insertion();
        while let Some(target) = iter.next(&self.instance_list) {
            if self.instance_list.get(target).map(|x| x.solid.get()).unwrap_or(false) {
                if self.check_collision(inst, target) {
                    return Some(target)
                }
            }
        }
        None
    }

    // Checks if an instance is colliding with any instance, returning the target if it is, otherwise None
    pub fn check_collision_any(&self, inst: usize) -> Option<usize> {
        let mut iter = self.instance_list.iter_by_insertion();
        while let Some(target) = iter.next(&self.instance_list) {
            if inst != target {
                if self.check_collision(inst, target) {
                    return Some(target)
                }
            }
        }
        None
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
