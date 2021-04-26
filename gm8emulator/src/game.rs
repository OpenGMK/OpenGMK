pub mod background;
pub mod draw;
pub mod events;
pub mod external;
pub mod gm_save;
pub mod includedfile;
pub mod model;
pub mod movement;
pub mod particle;
pub mod pathfinding;
pub mod replay;
pub mod savestate;
pub mod string;
pub mod surface;
pub mod transition;
pub mod view;

pub use background::Background;
pub use replay::Replay;
pub use savestate::SaveState;
pub use view::View;

use crate::{
    action::Tree,
    asset::{
        self,
        font::{Character, Font},
        path::{self, Path},
        room::{self, Room},
        sprite::{Collider, Frame, Sprite},
        trigger::{self, Trigger},
        Object, Script, Timeline,
    },
    gml::{self, ds, ev, file, rand::Random, Compiler, Context},
    handleman::{HandleArray, HandleList},
    input::InputManager,
    instance::{DummyFieldHolder, Instance, InstanceState},
    instancelist::{InstanceList, TileList},
    math::Real,
    tile, util,
};
use encoding_rs::Encoding;
use gm8exe::asset::PascalString;
use gmio::{
    atlas::AtlasBuilder,
    render::{Renderer, RendererOptions, Scaling},
    window::{self, Window, WindowBuilder},
};
use includedfile::IncludedFile;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use shared::{
    input::MouseButton,
    message::{self, Message, MessageStream},
    types::{Colour, ID},
};
use std::{
    borrow::Cow,
    cell::{Cell, RefCell},
    collections::{BTreeMap, HashMap, HashSet, VecDeque},
    convert::TryFrom,
    fs::File,
    io::{BufReader, Write},
    net::{SocketAddr, TcpStream},
    path::PathBuf,
    rc::Rc,
    time::{Duration, Instant},
};
use string::RCStr;

/// Structure which contains all the components of a game.
pub struct Game {
    pub compiler: Compiler,
    pub text_files: HandleArray<file::TextHandle, 32>,
    pub binary_files: HandleArray<file::BinaryHandle, 32>,
    pub instance_list: InstanceList,
    pub tile_list: TileList,
    pub rand: Random,
    pub input_manager: InputManager,
    pub assets: Assets,
    pub event_holders: [IndexMap<u32, Rc<RefCell<Vec<ID>>>>; 12],
    pub custom_draw_objects: HashSet<ID>,

    pub renderer: Renderer,
    pub background_colour: Colour,
    pub room_colour: Colour,
    pub show_room_colour: bool,

    pub externals: Vec<Option<external::External>>,
    pub surface_fix: bool,

    pub last_instance_id: ID,
    pub last_tile_id: ID,

    pub views_enabled: bool,
    pub view_current: usize,
    pub views: Vec<View>,
    pub backgrounds: Vec<background::Background>,

    pub particles: particle::Manager,

    pub room_id: i32,
    pub room_width: i32,
    pub room_height: i32,
    pub room_order: Box<[i32]>,
    pub room_speed: u32,
    pub scene_change: Option<SceneChange>, // Queued scene change which has been requested by GML, if any
    pub user_transitions: HashMap<i32, transition::UserTransition>,

    pub constants: Vec<gml::Value>,
    pub globals: DummyFieldHolder,
    pub globalvars: HashSet<usize>,
    pub game_start: bool,

    pub stacks: HandleList<ds::Stack>,
    pub queues: HandleList<ds::Queue>,
    pub lists: HandleList<ds::List>,
    pub maps: HandleList<ds::Map>,
    pub priority_queues: HandleList<ds::Priority>,
    pub grids: HandleList<ds::Grid>,
    pub ds_precision: Real,

    pub default_font: Font,
    pub draw_font_id: ID,
    pub draw_colour: Colour,
    pub draw_alpha: Real,
    pub draw_halign: draw::Halign,
    pub draw_valign: draw::Valign,
    pub surfaces: Vec<Option<surface::Surface>>,
    pub surface_target: Option<i32>,
    pub models: Vec<Option<model::Model>>,
    pub model_matrix_stack: Vec<[f32; 16]>,
    pub auto_draw: bool,
    pub uninit_fields_are_zero: bool,
    pub uninit_args_are_zero: bool,

    pub potential_step_settings: pathfinding::PotentialStepSettings,

    pub fps: u32,                 // initially 0
    pub transition_kind: i32,     // default 0
    pub transition_steps: i32,    // default 80
    pub cursor_sprite: i32,       // default -1
    pub cursor_sprite_frame: u32, // default 0
    pub score: i32,               // default 0
    pub score_capt: RCStr,        // default "Score: "
    pub score_capt_d: bool,       // display in caption?
    pub has_set_show_score: bool, // if false, score displays if > 0
    pub lives: i32,               // default -1
    pub lives_capt: RCStr,        // default "Lives: "
    pub lives_capt_d: bool,       // display in caption?
    pub health: Real,             // default 100.0
    pub health_capt: RCStr,       // default "Health: "
    pub health_capt_d: bool,      // display in caption?

    pub error_occurred: bool,
    pub error_last: RCStr,

    pub game_id: i32,
    pub program_directory: RCStr,
    pub temp_directory: RCStr,
    pub included_files: Vec<IncludedFile>,
    pub gm_version: Version,
    pub open_ini: Option<(ini::Ini, RCStr)>, // keep the filename for writing
    pub open_file: Option<file::TextHandle>, // for legacy file functions from GM <= 5.1
    pub file_finder: Option<Box<dyn Iterator<Item = PathBuf>>>,
    pub spoofed_time_nanos: Option<u128>, // use this instead of real time if this is set
    pub parameters: Vec<String>,
    pub encoding: &'static Encoding,

    pub esc_close_game: bool,

    // window caption
    pub caption: RCStr,
    pub caption_stale: bool,

    pub play_type: PlayType,
    pub stored_events: VecDeque<replay::Event>,

    // winit windowing
    pub window: Window,
    pub window_border: bool,
    pub window_icons: bool,
    // Scaling type
    pub scaling: Scaling,
    // Width the window is supposed to have, assuming it hasn't been resized by the user
    pub unscaled_width: u32,
    // Height the window is supposed to have, assuming it hasn't been resized by the user
    pub unscaled_height: u32,
}

/// Enum indicating which GameMaker version a game was built with
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Version {
    GameMaker8_0,
    GameMaker8_1,
}

/// Enum indicating how this game is being played - normal, recording or replaying
#[derive(Clone, Debug, PartialEq)]
pub enum PlayType {
    Normal,
    Record,
    Replay,
}

/// Various different types of scene change which can be requested by GML
#[derive(Clone, Copy)]
pub enum SceneChange {
    Room(ID), // Go to the specified room
    Restart,  // Restart the game and go to the first room
    End,      // End the game
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Assets {
    pub backgrounds: Vec<Option<Box<asset::Background>>>,
    pub fonts: Vec<Option<Box<Font>>>,
    pub objects: Vec<Option<Box<Object>>>,
    pub paths: Vec<Option<Box<Path>>>,
    pub rooms: Vec<Option<Box<Room>>>,
    pub scripts: Vec<Option<Box<Script>>>,
    pub sprites: Vec<Option<Box<Sprite>>>,
    pub timelines: Vec<Option<Box<Timeline>>>,
    pub triggers: Vec<Option<Box<Trigger>>>,
    // todo
}

impl From<PascalString> for RCStr {
    fn from(s: PascalString) -> Self {
        s.0.as_ref().into()
    }
}

impl Game {
    pub fn launch(
        assets: gm8exe::GameAssets,
        file_path: PathBuf,
        game_arguments: Vec<String>,
        temp_dir: Option<PathBuf>,
        encoding: &'static Encoding,
        play_type: PlayType,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Parse file path
        let mut file_path2 = file_path.clone();
        file_path2.pop();
        // Game Maker doesn't change working directory on load but doing it anyway makes life easier
        std::env::set_current_dir(&file_path2)?;
        let mut param_string: &str = &file_path.to_string_lossy();
        let mut program_directory: &str = &file_path2.to_string_lossy();

        if cfg!(target_os = "windows") {
            param_string = param_string.trim_start_matches("\\\\?\\");
            program_directory = program_directory.trim_start_matches("\\\\?\\");
        }
        // TODO: store these as RCStr probably?
        println!("param_string: {}", param_string);
        println!("program_directory: {}", program_directory);

        // Improve framepacing on Windows
        #[cfg(target_os = "windows")]
        unsafe {
            winapi::um::timeapi::timeBeginPeriod(1);
        }

        // Destructure assets
        let gm8exe::GameAssets {
            game_id,
            backgrounds,
            constants,
            fonts,
            icon_data: _,
            included_files,
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

        let gm_version = match version {
            gm8exe::GameVersion::GameMaker8_0 => Version::GameMaker8_0,
            gm8exe::GameVersion::GameMaker8_1 => Version::GameMaker8_1,
        };

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
        let room1_colour = room1.bg_colour.as_decimal().into();
        let room1_show_colour = room1.clear_screen;

        let mut rand = Random::new();

        // manual decode to avoid errors
        let decode_str_maybe = |bytes: Vec<u8>| match gm_version {
            Version::GameMaker8_0 => {
                encoding.decode_without_bom_handling_and_without_replacement(&bytes).map(|x| x.into_owned())
            },
            Version::GameMaker8_1 => String::from_utf8(bytes).ok(),
        };

        let temp_directory = match temp_dir {
            Some(path) => path,
            None => {
                // read path from tempdir.txt or if that's not possible get std::env::temp_dir()
                let mut dir = if let Some(path) =
                    std::fs::read("tempdir.txt").ok().and_then(decode_str_maybe).map(|path| PathBuf::from(path))
                {
                    path
                } else {
                    std::env::temp_dir()
                };
                // closure to make a gm_ttt folder within a given path
                let mut make_temp_dir = |path: &mut PathBuf| {
                    let mut folder = "gm_ttt_".to_string();
                    folder += &rand.next_int(99999).to_string();
                    path.push(&folder);
                    while path.exists() {
                        path.pop();
                        folder.truncate(7); // length of "gm_ttt_"
                        folder += &rand.next_int(99999).to_string();
                        path.push(&folder);
                    }
                    std::fs::create_dir_all(path)
                };
                // try making folders
                if let Err(e) = make_temp_dir(&mut dir) {
                    eprintln!("Could not create temp folder in {:?}: {}", dir, e);
                    // GM8 would try C:\temp but let's skip that
                    match std::env::current_dir().map(|x| {
                        dir = x;
                        make_temp_dir(&mut dir)
                    }) {
                        Ok(_) => eprintln!("Using game directory instead."),
                        Err(e) => {
                            eprintln!("Could not use game directory either: {}", e);
                            eprintln!("Trying to run anyway. If this game uses the temp folder, it will likely crash.");
                            dir = PathBuf::new();
                        },
                    }
                }
                dir
            },
        };

        let included_files = included_files
            .into_iter()
            .map(|i| {
                use gm8exe::asset::included_file::ExportSetting;
                let export_settings = match i.export_settings {
                    ExportSetting::NoExport => includedfile::ExportSetting::NoExport,
                    ExportSetting::TempFolder => includedfile::ExportSetting::TempFolder,
                    ExportSetting::GameFolder => includedfile::ExportSetting::GameFolder,
                    ExportSetting::CustomFolder(dir) => match decode_str_maybe(dir.0.to_vec()) {
                        Some(s) => includedfile::ExportSetting::CustomFolder(s),
                        None => {
                            panic!("could not decode includedfile export directory {}", String::from_utf8_lossy(&dir.0))
                        },
                    },
                };
                let mut i = IncludedFile {
                    name: match decode_str_maybe(i.file_name.0.to_vec()) {
                        Some(s) => s,
                        None => {
                            panic!("could not decode includedfile name {}", String::from_utf8_lossy(&i.file_name.0))
                        },
                    },
                    data: i.embedded_data,
                    export_settings,
                    overwrite: i.overwrite_file,
                    free_after_export: i.free_memory,
                    remove_at_end: i.remove_at_end,
                };
                i.export(temp_directory.clone(), program_directory.to_string().into())?;
                Ok(i)
            })
            .collect::<Result<Vec<_>, std::io::Error>>()
            .expect("failed to extract included files");

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
                + triggers.iter().flatten().count(),
        );
        compiler.reserve_user_constants(constants.len());

        // Helper fn for registering asset names as constants
        fn register_all<T>(compiler: &mut Compiler, assets: &[Option<T>], get_name: fn(&T) -> &PascalString) {
            assets
                .iter()
                .enumerate()
                .filter_map(|(i, x)| x.as_ref().map(|x| (i, x)))
                .for_each(|(i, x)| compiler.register_constant(get_name(x).0.clone(), i as f64))
        }

        // Register all asset names
        // These are in order of asset precedence, please don't change the order
        register_all(&mut compiler, &objects, |x| &x.name);
        register_all(&mut compiler, &sprites, |x| &x.name);
        register_all(&mut compiler, &sounds, |x| &x.name);
        register_all(&mut compiler, &backgrounds, |x| &x.name);
        register_all(&mut compiler, &paths, |x| &x.name);
        register_all(&mut compiler, &fonts, |x| &x.name);
        register_all(&mut compiler, &timelines, |x| &x.name);
        register_all(&mut compiler, &scripts, |x| &x.name);
        register_all(&mut compiler, &rooms, |x| &x.name);
        register_all(&mut compiler, &triggers, |x| &x.constant_name);

        // Register scripts
        scripts
            .iter()
            .enumerate()
            .filter_map(|(i, x)| x.as_ref().map(|x| (i, x)))
            .for_each(|(i, x)| compiler.register_script(x.name.0.clone(), i));

        // Register user constants
        constants.iter().enumerate().for_each(|(i, x)| compiler.register_user_constant(x.name.0.clone(), i));

        // Set up a Renderer
        let options = RendererOptions {
            size: (room1_width, room1_height),
            vsync: settings.vsync, // TODO: Overrideable
            interpolate_pixels: settings.interpolate_pixels,
            normalize_normals: gm_version == Version::GameMaker8_1,
            zbuf_24: gm_version == Version::GameMaker8_1, // TODO: set to true if surface fix is found
        };

        let (width, height) = options.size;
        let window_border = !settings.dont_draw_border;
        let window_icons = !settings.dont_show_buttons;
        let wb = WindowBuilder::new().with_size(width, height).with_style(if play_type == PlayType::Record {
            window::Style::Regular
        } else {
            match (window_border, window_icons) {
                (true, true) => window::Style::Regular,
                (true, false) => window::Style::Undecorated,
                (false, _) => window::Style::Borderless,
            }
        });

        // TODO: specific flags here (make wb mutable)

        let window = wb.build().expect("oh no");
        let mut renderer = Renderer::new((), &options, &window, settings.clear_colour.into())?;

        let mut atlases = AtlasBuilder::new(renderer.max_texture_size() as _);

        let scaling = match settings.scaling {
            0 => Scaling::Full,
            n if n < 0 => Scaling::Aspect(f64::from(n) / 100.0),
            n => Scaling::Fixed(f64::from(n) / 100.0),
        };

        //println!("GPU Max Texture Size: {}", renderer.max_gpu_texture_size());

        let particle_shapes = particle::load_shapes(&mut atlases);

        let default_font = asset::font::load_default_font(&mut atlases)?;

        let sprites = sprites
            .into_iter()
            .map(|o| {
                o.map(|b| {
                    let (w, h) = b.frames.first().map_or((0, 0), |f| (f.width, f.height));
                    let origin_x = b.origin_x;
                    let origin_y = b.origin_y;
                    let bbox_left = b.colliders.iter().map(|x| x.bbox_left).min().unwrap_or(0);
                    let bbox_right = b.colliders.iter().map(|x| x.bbox_right).max().unwrap_or(0);
                    let bbox_top = b.colliders.iter().map(|x| x.bbox_top).min().unwrap_or(0);
                    let bbox_bottom = b.colliders.iter().map(|x| x.bbox_bottom).max().unwrap_or(0);
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
                        bbox_left,
                        bbox_right,
                        bbox_top,
                        bbox_bottom,
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
                    let mut tallest_char_height = 0;
                    let charset = match gm_version {
                        Version::GameMaker8_0 => 1, // DEFAULT_CHARSET
                        Version::GameMaker8_1 => b.charset,
                    };
                    let chars = b
                        .dmap
                        .chunks_exact(6)
                        .skip(b.range_start as usize)
                        .take(((b.range_end - b.range_start) + 1) as usize)
                        .map(|char_blob| {
                            if tallest_char_height < char_blob[3] {
                                tallest_char_height = char_blob[3];
                            }
                            let mut data: Vec<u8> = Vec::with_capacity((char_blob[2] * char_blob[3] * 4) as usize);
                            for y in 0..char_blob[3] {
                                for x in 0..char_blob[2] {
                                    data.push(0xFF);
                                    data.push(0xFF);
                                    data.push(0xFF);
                                    data.push(
                                        b.pixel_map[((y + char_blob[1]) * b.map_width + x + char_blob[0]) as usize],
                                    );
                                }
                            }
                            Ok(Character {
                                offset: char_blob[4] as _,
                                distance: char_blob[5] as _,
                                atlas_ref: atlases
                                    .texture(char_blob[2] as _, char_blob[3] as _, 0, 0, data.into_boxed_slice())
                                    .ok_or(())?,
                            })
                        })
                        .collect::<Result<Box<_>, ()>>()?;
                    Ok(Box::new(Font {
                        name: b.name.into(),
                        sys_name: b.sys_name.into(),
                        charset,
                        size: b.size,
                        bold: b.bold,
                        italic: b.italic,
                        first: b.range_start as _,
                        last: b.range_end as _,
                        tallest_char_height,
                        chars,
                        own_graphics: true,
                    }))
                })
                .transpose()
            })
            .collect::<Result<Vec<_>, ()>>()
            .expect("failed to pack fonts");

        let paths = paths
            .into_iter()
            .map(|t| {
                t.map(|b| {
                    let mut path = Path {
                        name: b.name.into(),
                        points: b
                            .points
                            .into_iter()
                            .map(|point| path::Point {
                                x: Real::from(point.x),
                                y: Real::from(point.y),
                                speed: Real::from(point.speed),
                            })
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

        // Code compiling starts here. The order in which things are compiled is important for
        // keeping savestates compatible. This isn't 100% accurate right now, but it's mostly right.

        let triggers = triggers
            .into_iter()
            .map(|t| {
                t.map(|b| {
                    let condition = match compiler.compile(&b.condition.0) {
                        Ok(s) => s,
                        Err(e) => return Err(format!("Compiler error in trigger {}: {}", b.name, e)),
                    };
                    Ok(Box::new(Trigger { name: b.name.into(), condition, moment: b.moment.into() }))
                })
                .transpose()
            })
            .collect::<Result<Vec<_>, _>>()?;

        let scripts = scripts
            .into_iter()
            .map(|t| {
                t.map(|b| {
                    let compiled = match compiler.compile(&b.source.0) {
                        Ok(s) => s,
                        Err(e) => return Err(format!("Compiler error in script {}: {}", b.name, e)),
                    };
                    Ok(Box::new(Script { name: b.name.into(), source: b.source.into(), compiled }))
                })
                .transpose()
            })
            .collect::<Result<Vec<_>, _>>()?;

        let timelines = timelines
            .into_iter()
            .map(|t| {
                t.map(|b| {
                    let mut moments: BTreeMap<i32, Rc<RefCell<Tree>>> = BTreeMap::new();
                    for (moment, actions) in b.moments.iter() {
                        match Tree::from_list(actions, &mut compiler) {
                            Ok(t) => {
                                moments.insert(*moment as i32, Rc::new(RefCell::new(t)));
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
                            children: Rc::new(RefCell::new(HashSet::new())),
                        }))
                    })
                    .transpose()
                })
                .collect::<Result<Vec<_>, _>>()?;

            // Populate identity lists
            for (i, object) in objects.iter_mut().enumerate().filter_map(|(i, x)| x.as_mut().map(|x| (i, x))) {
                object.children.borrow_mut().insert(i as _);
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

        let rooms = rooms
            .into_iter()
            .map(|t| {
                t.map(|b| {
                    let creation_code = compiler
                        .compile(&b.creation_code.0)
                        .map_err(|e| format!("Compiler error in room {} creation code: {}", b.name, e));
                    let width = b.width;
                    let height = b.height;
                    Box::new(Room {
                        name: b.name.into(),
                        caption: b.caption.into(),
                        width,
                        height,
                        speed: b.speed,
                        persistent: b.persistent,
                        bg_colour: (b.bg_colour.r, b.bg_colour.g, b.bg_colour.b).into(),
                        clear_screen: b.clear_screen,
                        creation_code,
                        backgrounds: b
                            .backgrounds
                            .into_iter()
                            .map(|bg| Background {
                                visible: bg.visible_on_start,
                                is_foreground: bg.is_foreground,
                                background_id: bg.source_bg,
                                x_offset: Real::from(bg.xoffset),
                                y_offset: Real::from(bg.yoffset),
                                tile_horizontal: bg.tile_horz,
                                tile_vertical: bg.tile_vert,
                                hspeed: Real::from(bg.hspeed),
                                vspeed: Real::from(bg.vspeed),
                                xscale: if bg.stretch {
                                    if let Some(bg_asset) = backgrounds.get_asset(bg.source_bg) {
                                        Real::from(width) / Real::from(bg_asset.width)
                                    } else {
                                        Real::from(width)
                                    }
                                } else {
                                    Real::from(1.0)
                                },
                                yscale: if bg.stretch {
                                    if let Some(bg_asset) = backgrounds.get_asset(bg.source_bg) {
                                        Real::from(height) / Real::from(bg_asset.height)
                                    } else {
                                        Real::from(height)
                                    }
                                } else {
                                    Real::from(1.0)
                                },
                                blend: 0xFFFFFF,
                                alpha: Real::from(1.0),
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
                                angle: Real::from(0.0),
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
                            .map(|i| room::Instance {
                                x: i.x,
                                y: i.y,
                                object: i.object,
                                id: i.id,
                                creation: compiler.compile(&i.creation_code.0).map_err(|e| {
                                    format!("Compiler error in creation code of instance {}: {}", i.id, e)
                                }),
                            })
                            .collect::<Vec<_>>()
                            .into(),
                        tiles: b
                            .tiles
                            .into_iter()
                            .map(|t| tile::Tile {
                                x: Cell::new(t.x.into()),
                                y: Cell::new(t.y.into()),
                                background_index: Cell::new(t.source_bg),
                                tile_x: Cell::new(t.tile_x as _),
                                tile_y: Cell::new(t.tile_y as _),
                                width: Cell::new(t.width as _),
                                height: Cell::new(t.height as _),
                                depth: Cell::new(t.depth.into()),
                                id: Cell::new(t.id),
                                alpha: Cell::new(1.0.into()),
                                blend: Cell::new(0xFFFFFF),
                                xscale: Cell::new(1.0.into()),
                                yscale: Cell::new(1.0.into()),
                                visible: Cell::new(true),
                            })
                            .collect::<Vec<_>>()
                            .into(),
                    })
                })
            })
            .collect::<Vec<_>>();

        // Make event holder lists
        let mut event_holders: [IndexMap<u32, Rc<RefCell<Vec<i32>>>>; 12] = Default::default();
        Self::fill_event_holders(&mut event_holders, &objects);

        // Make list of objects with custom draw events
        let custom_draw_objects =
            event_holders[ev::DRAW].iter().flat_map(|(_, x)| x.borrow().iter().copied().collect::<Vec<_>>()).collect();

        renderer.push_atlases(atlases)?;

        let mut game = Self {
            compiler,
            text_files: HandleArray::new(),
            binary_files: HandleArray::new(),
            instance_list: InstanceList::new(),
            tile_list: TileList::new(),
            rand,
            renderer: renderer,
            background_colour: settings.clear_colour.into(),
            externals: Vec::new(),
            surface_fix: false,
            room_colour: room1_colour,
            show_room_colour: room1_show_colour,
            input_manager: InputManager::new(),
            assets: Assets { backgrounds, fonts, objects, paths, rooms, scripts, sprites, timelines, triggers },
            event_holders,
            custom_draw_objects,
            views_enabled: false,
            view_current: 0,
            views: Vec::new(),
            backgrounds: Vec::new(),
            particles: particle::Manager::new(particle_shapes),
            room_id: room1_id,
            room_width: room1_width as i32,
            room_height: room1_height as i32,
            room_order: room_order.into_boxed_slice(),
            room_speed: room1_speed,
            scene_change: None,
            user_transitions: HashMap::new(),
            constants: Vec::with_capacity(constants.len()),
            globals: DummyFieldHolder::new(),
            globalvars: HashSet::new(),
            game_start: true,
            stacks: HandleList::new(),
            queues: HandleList::new(),
            lists: HandleList::new(),
            maps: HandleList::new(),
            priority_queues: HandleList::new(),
            grids: HandleList::new(),
            ds_precision: Real::from(0.00000001),
            default_font,
            draw_font_id: -1,
            draw_colour: Colour::new(0.0, 0.0, 0.0),
            draw_alpha: Real::from(1.0),
            draw_halign: draw::Halign::Left,
            draw_valign: draw::Valign::Top,
            surfaces: Vec::new(),
            surface_target: None,
            models: Vec::new(),
            model_matrix_stack: Vec::new(),
            auto_draw: true,
            last_instance_id,
            last_tile_id,
            uninit_fields_are_zero: settings.zero_uninitialized_vars,
            uninit_args_are_zero: !settings.error_on_uninitialized_args,
            potential_step_settings: Default::default(),
            transition_kind: 0,
            transition_steps: 80,
            cursor_sprite: -1,
            cursor_sprite_frame: 0,
            score: 0,
            score_capt: "Score: ".to_string().into(),
            lives: -1,
            lives_capt: "Lives: ".to_string().into(),
            health: Real::from(100.0),
            health_capt: "Health: ".to_string().into(),
            game_id: game_id as i32,
            program_directory: program_directory.into(),
            temp_directory: "".into(),
            included_files,
            gm_version,
            open_ini: None,
            open_file: None,
            file_finder: None,
            spoofed_time_nanos: None,
            fps: 0,
            parameters: game_arguments,
            encoding,
            esc_close_game: settings.esc_close_game,
            caption: "".to_string().into(),
            caption_stale: false,
            score_capt_d: true,
            has_set_show_score: false,
            lives_capt_d: false,
            health_capt_d: false,
            error_occurred: false,
            error_last: "".to_string().into(),
            window,
            window_border,
            window_icons,
            scaling,
            play_type,
            stored_events: VecDeque::new(),

            // load_room sets this
            unscaled_width: 0,
            unscaled_height: 0,
        };

        game.temp_directory = game.encode_str_maybe(temp_directory.to_str().unwrap()).unwrap().into_owned().into();

        // Evaluate constants
        for c in &constants {
            let expr = game.compiler.compile_expression(&c.expression.0)?;
            let dummy_instance = game
                .instance_list
                .insert_dummy(Instance::new_dummy(game.assets.objects.get_asset(0).map(|x| x.as_ref())));
            let value = game.eval(&expr, &mut Context {
                this: dummy_instance,
                other: dummy_instance,
                event_action: 0,
                relative: false,
                event_type: 0,
                event_number: 0,
                event_object: 0,
                arguments: Default::default(),
                argument_count: 0,
                locals: Default::default(),
                return_value: Default::default(),
            })?;
            game.constants.push(value);
            game.instance_list.remove_dummy(dummy_instance);
        }

        // Re-initialization after constants are done
        game.globals.fields.clear();
        game.globals.vars.clear();
        game.globalvars.clear();

        game.window.set_visible(true);

        Ok(game)
    }

    pub fn refresh_event_holders(&mut self) {
        // It might be better to not redo the entire holder list from scratch?

        // Clear holder lists
        for holder_list in self.event_holders.iter_mut() {
            holder_list.clear();
        }

        // Refill holder lists
        Self::fill_event_holders(&mut self.event_holders, &self.assets.objects);

        // Make list of objects with custom draw events
        self.custom_draw_objects = self.event_holders[ev::DRAW]
            .iter()
            .flat_map(|(_, x)| x.borrow().iter().copied().collect::<Vec<_>>())
            .collect();
    }

    fn fill_event_holders(
        event_holders: &mut [IndexMap<u32, Rc<RefCell<Vec<ID>>>>],
        objects: &Vec<Option<Box<Object>>>,
    ) {
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
    }

    fn resize_window(&mut self, width: u32, height: u32) {
        // GameMaker only actually resizes the window if the expected (unscaled) size is changing.
        if self.unscaled_width != width || self.unscaled_height != height {
            self.unscaled_width = width;
            self.unscaled_height = height;
            self.renderer.resize_framebuffer(width, height);
            let (width, height) = match self.scaling {
                Scaling::Fixed(scale) => ((f64::from(width) * scale) as u32, (f64::from(height) * scale) as u32),
                _ => (width, height),
            };
            self.window.resize(width, height);
        }
    }

    pub fn decode_str<'a>(&self, string: &'a [u8]) -> Cow<'a, str> {
        match self.gm_version {
            Version::GameMaker8_0 => self.encoding.decode_without_bom_handling(string).0,
            Version::GameMaker8_1 => String::from_utf8_lossy(string),
        }
    }

    pub fn encode_str_maybe<'a>(&self, utf8: &'a str) -> Option<Cow<'a, [u8]>> {
        match self.gm_version {
            Version::GameMaker8_0 => {
                let (encoded, _, is_bad) = self.encoding.encode(utf8);
                if is_bad { None } else { Some(encoded) }
            },
            Version::GameMaker8_1 => Some(Cow::from(utf8.as_bytes())),
        }
    }

    pub fn load_room(&mut self, room_id: i32) -> Result<(), Box<dyn std::error::Error>> {
        let room = if let Some(Some(room)) = self.assets.rooms.get(room_id as usize) {
            room.clone()
        } else {
            return Err(format!("Tried to load non-existent room with id {}", room_id).into())
        };

        // Update this early so the other events run
        self.scene_change = None;

        // Initialize room transition surface
        let transition_kind = self.transition_kind;
        let (trans_surf_old, trans_surf_new) = if self.get_transition(transition_kind).is_some() {
            let (width, height) = self.window.get_inner_size();
            let make_zbuf = self.gm_version == Version::GameMaker8_1 || self.surface_fix;
            let old_surf = surface::Surface {
                width,
                height,
                atlas_ref: self.renderer.create_surface(width as _, height as _, make_zbuf)?,
            };
            let new_surf = surface::Surface {
                width,
                height,
                atlas_ref: self.renderer.create_surface(width as _, height as _, make_zbuf)?,
            };
            self.renderer.set_target(&old_surf.atlas_ref);
            self.draw()?;
            self.renderer.set_target(&new_surf.atlas_ref);
            let old_surf_id = self.surfaces.len() as i32;
            self.surfaces.push(Some(old_surf));
            self.surfaces.push(Some(new_surf));
            (old_surf_id, old_surf_id + 1)
        } else {
            (-1, -1)
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
        self.room_colour = room.bg_colour;
        self.show_room_colour = room.clear_screen;

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
        self.caption = room.caption;
        self.input_manager.clear_presses();
        self.particles.effect_clear();
        self.cursor_sprite_frame = 0;

        // Load all tiles in new room
        for tile in room.tiles.iter() {
            self.tile_list.insert(tile.clone());
        }

        // Load all instances in new room, unless they already exist due to persistence
        let mut new_handles: Vec<(usize, &asset::room::Instance)> = Vec::new();
        for instance in room.instances.iter() {
            if self.instance_list.get_by_instid(instance.id).is_none() {
                // Get object
                let object = match self.assets.objects.get(instance.object as usize) {
                    Some(&Some(ref o)) => o.as_ref(),
                    _ => return Err(format!("Instance of invalid Object in room {}", room.name).into()),
                };

                // Add instance to list
                new_handles.push((
                    self.instance_list.insert(Instance::new(
                        instance.id as _,
                        Real::from(instance.x),
                        Real::from(instance.y),
                        instance.object,
                        object,
                    )),
                    instance,
                ));
            }
        }
        for (handle, instance) in &new_handles {
            if self.instance_list.get(*handle).is_active() {
                // Run this instance's room creation code
                self.execute(&instance.creation.clone()?, &mut Context {
                    this: *handle,
                    other: *handle,
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
                self.run_instance_event(ev::CREATE, 0, *handle, *handle, None)?;
            }
        }

        if self.game_start {
            // Run game start event for each instance
            let mut iter = self.instance_list.iter_by_insertion();
            while let Some(instance) = iter.next(&self.instance_list) {
                self.run_instance_event(ev::OTHER, 2, instance, instance, None)?;
            }
            self.game_start = false;
        }

        // Run room creation code
        let dummy_instance =
            self.instance_list.insert_dummy(Instance::new_dummy(self.assets.objects.get_asset(0).map(|x| x.as_ref())));
        self.execute(&room.creation_code?, &mut Context {
            this: dummy_instance,
            other: dummy_instance,
            event_action: 0,
            relative: false,
            event_type: 11,
            event_number: 0,
            event_object: 0,
            arguments: Default::default(),
            argument_count: 0,
            locals: Default::default(),
            return_value: Default::default(),
        })?;
        self.instance_list.remove_dummy(dummy_instance);

        // Run room start event for each instance
        let mut iter = self.instance_list.iter_by_insertion();
        while let Some(instance) = iter.next(&self.instance_list) {
            self.run_instance_event(ev::OTHER, 4, instance, instance, None)?;
        }

        if let Some(change) = self.scene_change {
            self.scene_change = None;
            // GM8 would have a memory leak here. We're not doing that.
            if let Some(surf) = self.surfaces.get_asset_mut(trans_surf_old) {
                self.renderer.delete_sprite(surf.atlas_ref);
                self.surfaces[trans_surf_old as usize] = None;
            }
            if let Some(surf) = self.surfaces.get_asset_mut(trans_surf_new) {
                self.renderer.delete_sprite(surf.atlas_ref);
                self.surfaces[trans_surf_new as usize] = None;
            }

            if let SceneChange::Room(target) = change {
                // A room change has been requested during this room change, so let's recurse...
                self.load_room(target)
            } else {
                // Natural game end or restart happened during room change, so just quit
                Ok(())
            }
        } else {
            // Draw "frame 0", perform transition if applicable, and then return
            if self.auto_draw {
                self.draw()?;
                if let Some(transition) = self.get_transition(transition_kind) {
                    let (width, height) = self.window.get_inner_size();
                    self.renderer.reset_target();
                    // Here, we see the limitations of GM8's vsync.
                    // Room transitions don't have a specific framerate, they just vsync. Unfortunately, this gets
                    // messy. Instead of telling the display driver to use vsync like a sane program, GM8 manually waits
                    // for vsync before drawing. It does this by calling WaitForVBlank(DDWAITVB_BLOCKBEGIN) from
                    // DirectDraw (the only use of ddraw in the entire runner). According to experimentation, this
                    // function will wait until the next vblank, unless a vblank just happened, in which case it returns
                    // instantly. There's usually enough processing between vblanks that this isn't a problem, but when
                    // using builtin room transitions, the processing is so lightweight it will skip frames. This means
                    // the builtin transitions will run too fast.
                    // This would be hell to emulate, so let's just standardize the framerate and call it a day.
                    // Most of the builtin transitions seem to run at around 120FPS in our tests, so let's go with that.
                    const FRAME_TIME: Duration = Duration::from_nanos(1_000_000_000u64 / 120);
                    let mut current_time = Instant::now();
                    let perspective = self.renderer.get_perspective();
                    for i in 0..self.transition_steps + 1 {
                        let progress = Real::from(i) / self.transition_steps.into();
                        if self.surface_fix {
                            self.renderer.set_perspective(false);
                            self.renderer.set_projection_ortho(
                                0.0,
                                0.0,
                                self.unscaled_width.into(),
                                self.unscaled_height.into(),
                                0.0,
                            );
                        }
                        transition(self, trans_surf_old, trans_surf_new, width as _, height as _, progress)?;
                        self.renderer.present(width, height, self.scaling);
                        let diff = current_time.elapsed();
                        if let Some(dur) = FRAME_TIME.checked_sub(diff) {
                            gml::datetime::sleep(dur);
                        }
                        current_time = Instant::now();
                    }
                    if self.surface_fix {
                        self.renderer.set_perspective(perspective);
                    }
                }
            }
            if let Some(surf) = self.surfaces.get_asset_mut(trans_surf_old) {
                self.renderer.delete_sprite(surf.atlas_ref);
                self.surfaces[trans_surf_old as usize] = None;
            }
            if let Some(surf) = self.surfaces.get_asset_mut(trans_surf_new) {
                self.renderer.delete_sprite(surf.atlas_ref);
                self.surfaces[trans_surf_new as usize] = None;
            }
            self.transition_kind = 0;
            Ok(())
        }
    }

    /// Restarts the game in the same half-baked way GM8 does, including running all relevant events.
    pub fn restart(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Room end, game end events
        self.run_game_end_events()?;

        // Clear some stored variables
        self.instance_list = InstanceList::new();
        self.globals = DummyFieldHolder::new();
        self.game_start = true;

        // Go to first room
        self.load_room(self.room_order.first().copied().ok_or("Empty room order during Game::restart()")?)
    }

    /// Runs a frame loop and draws the screen. Exits immediately, without waiting for any FPS limitation.
    pub fn frame(&mut self) -> gml::Result<()> {
        if self.esc_close_game && self.input_manager.key_get_lastkey() == 0x1b {
            self.scene_change = Some(SceneChange::End);
            return Ok(())
        }

        // Update xprevious and yprevious for all instances
        let mut iter = self.instance_list.iter_by_insertion();
        while let Some(instance) = iter.next(&self.instance_list).map(|x| self.instance_list.get(x)) {
            instance.xprevious.set(instance.x.get());
            instance.yprevious.set(instance.y.get());
            instance.path_positionprevious.set(instance.path_position.get());
        }

        // Begin step trigger events
        self.run_triggers(trigger::TriggerTime::BeginStep)?;
        if self.scene_change.is_some() {
            return Ok(())
        }

        // Begin step event
        self.run_object_event(ev::STEP, 1, None)?;
        if self.scene_change.is_some() {
            return Ok(())
        }

        // Advance timelines for all instances
        let mut iter = self.instance_list.iter_by_insertion();
        while let Some(handle) = iter.next(&self.instance_list) {
            let instance = self.instance_list.get(handle);
            let object_index = instance.object_index.get();
            if instance.timeline_running.get() {
                if let Some(timeline) = self.assets.timelines.get_asset(instance.timeline_index.get()) {
                    let moments = timeline.moments.clone();
                    let timeline_len = Real::from(*moments.borrow().keys().max().unwrap_or(&0));

                    if timeline_len > Real::from(0) {
                        let old_position = instance.timeline_position.get();
                        let new_position = old_position + instance.timeline_speed.get();

                        match instance.timeline_speed.get() {
                            x if x > Real::from(0) => {
                                if new_position > timeline_len && instance.timeline_loop.get() {
                                    instance.timeline_position.set(Real::from(0));
                                } else {
                                    instance.timeline_position.set(new_position)
                                }

                                for (_, tree) in moments
                                    .borrow()
                                    .iter()
                                    .filter(|(&x, _)| Real::from(x) >= old_position && Real::from(x) < new_position)
                                {
                                    self.execute_tree(tree.clone(), handle, handle, 0, 0, object_index)?;
                                }
                            },
                            x if x < Real::from(0) => {
                                if new_position < Real::from(0) && instance.timeline_loop.get() {
                                    instance.timeline_position.set(timeline_len);
                                } else {
                                    instance.timeline_position.set(new_position)
                                }

                                for (_, tree) in moments
                                    .borrow()
                                    .iter()
                                    .filter(|(&x, _)| Real::from(x) > new_position && Real::from(x) <= old_position)
                                    .rev()
                                {
                                    self.execute_tree(tree.clone(), handle, handle, 0, 0, object_index)?;
                                }
                            },
                            _ => {},
                        };
                    }
                }
            }
        }

        // Alarm events
        self.run_alarms()?;
        if self.scene_change.is_some() {
            return Ok(())
        }

        // Key events
        self.run_keyboard_events()?;
        if self.scene_change.is_some() {
            return Ok(())
        }

        self.run_mouse_events()?;
        if self.scene_change.is_some() {
            return Ok(())
        }

        // Key press events
        self.run_key_press_events()?;
        if self.scene_change.is_some() {
            return Ok(())
        }

        // Key release events
        self.run_key_release_events()?;
        if self.scene_change.is_some() {
            return Ok(())
        }

        // Step trigger events
        self.run_triggers(trigger::TriggerTime::Step)?;
        if self.scene_change.is_some() {
            return Ok(())
        }

        // Step event
        self.run_object_event(ev::STEP, 0, None)?;
        if self.scene_change.is_some() {
            return Ok(())
        }

        // Movement: apply friction, gravity, and hspeed/vspeed
        self.process_speeds();
        let mut iter = self.instance_list.iter_by_insertion();
        while let Some(handle) = iter.next(&self.instance_list) {
            if self.apply_speeds(handle) {
                self.run_instance_event(ev::OTHER, 8, handle, handle, None)?;
            }
        }

        // Outside room, intersect boundary, outside/intersect view
        self.run_bound_events()?;
        if self.scene_change.is_some() {
            return Ok(())
        }

        // Run collision events
        self.run_collisions()?;
        if self.scene_change.is_some() {
            return Ok(())
        }

        // End step trigger events
        self.run_triggers(trigger::TriggerTime::EndStep)?;
        if self.scene_change.is_some() {
            return Ok(())
        }

        // End step event
        self.run_object_event(ev::STEP, 2, None)?;
        if self.scene_change.is_some() {
            return Ok(())
        }

        self.particles.auto_update_systems(&mut self.rand);

        // Clear out any deleted instances
        self.instance_list.remove_with(|instance| instance.state.get() == InstanceState::Deleted);

        // Draw everything, including running draw events
        if self.auto_draw {
            self.draw()?;
        }

        // Move backgrounds
        for bg in self.backgrounds.iter_mut() {
            bg.x_offset += bg.hspeed;
            bg.y_offset += bg.vspeed;
        }

        // Advance sprite animations
        let mut iter = self.instance_list.iter_by_insertion();
        while let Some(handle) = iter.next(&self.instance_list) {
            let instance = self.instance_list.get(handle);
            let new_index = instance.image_index.get() + instance.image_speed.get();
            instance.image_index.set(new_index);
            if let Some(sprite) = self.assets.sprites.get_asset(instance.sprite_index.get()) {
                let frame_count = sprite.frames.len() as f64;
                if new_index.into_inner() >= frame_count {
                    instance.image_index.set(new_index - Real::from(frame_count));
                    self.run_instance_event(ev::OTHER, 7, handle, handle, None)?; // animation end event
                }
            }
        }
        self.cursor_sprite_frame += 1;

        // Clear inputs for this frame
        self.input_manager.clear_presses();

        Ok(())
    }

    pub fn process_window_events(&mut self) {
        use gmio::window::Event;

        match self.play_type {
            PlayType::Normal => {
                self.input_manager.mouse_update_previous();
                for event in self.window.process_events().copied() {
                    match event {
                        Event::KeyboardDown(key) => self.input_manager.key_press(key),
                        Event::KeyboardUp(key) => self.input_manager.key_release(key),
                        Event::MenuOption(_) => (),
                        Event::MouseMove(x, y) => self.input_manager.set_mouse_pos(x.into(), y.into()),
                        Event::MouseButtonDown(button) => self.input_manager.mouse_press(button),
                        Event::MouseButtonUp(button) => self.input_manager.mouse_release(button),
                        Event::MouseWheelUp => self.input_manager.mouse_scroll_up(),
                        Event::MouseWheelDown => self.input_manager.mouse_scroll_down(),
                        Event::Resize(w, h) => println!("user resize: width={}, height={}", w, h),
                    }
                }
            },
            _ => (),
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.load_room(self.room_id)?;

        let mut time_now = Instant::now();
        let mut time_last = time_now;
        let mut frame_counter = 0;
        loop {
            self.process_window_events();

            self.frame()?;
            match self.scene_change {
                Some(SceneChange::Room(id)) => self.load_room(id)?,
                Some(SceneChange::Restart) => self.restart()?,
                Some(SceneChange::End) => break Ok(self.run_game_end_events()?),
                None => (),
            }

            // exit if X pressed or game_end() invoked
            if self.window.close_requested() {
                break Ok(self.run_game_end_events()?)
            }

            // frame limiter
            let diff = Instant::now().duration_since(time_now);
            let duration = Duration::new(0, 1_000_000_000u32 / self.room_speed);
            if let Some(t) = self.spoofed_time_nanos.as_mut() {
                *t += duration.as_nanos();
                self.fps = self.room_speed.into();
            } else {
                // gm8 just ignores any leftover time after a second has passed, so we do the same
                if time_now.duration_since(time_last) >= Duration::from_secs(1) {
                    time_last = time_now;
                    self.fps = frame_counter;
                    frame_counter = 0;
                }
            }
            frame_counter += 1;

            if let Some(time) = duration.checked_sub(diff) {
                gml::datetime::sleep(time);
                time_now += duration;
            } else {
                time_now = Instant::now();
            }
        }
    }

    // Create a TAS for this game
    pub fn record(&mut self, project_path: PathBuf, tcp_port: u16) -> Result<(), Box<dyn std::error::Error>> {
        use gmio::window::Event;

        // Helper fn: Instance -> InstanceDetails
        fn instance_details(assets: &Assets, instance: &Instance) -> message::InstanceDetails {
            message::InstanceDetails {
                id: instance.id.get(),
                object_name: match assets.objects.get_asset(instance.object_index.get()) {
                    Some(obj) => obj.name.decode_utf8().into(),
                    None => "<deleted object>".into(),
                },
                x: instance.x.get().into(),
                y: instance.y.get().into(),
                speed: instance.speed.get().into(),
                direction: instance.direction.get().into(),
                timeline_info: if assets.timelines.get_asset(instance.timeline_index.get()).is_some() {
                    Some((
                        instance.timeline_index.get(),
                        instance.timeline_position.get().into(),
                        instance.timeline_speed.get().into(),
                    ))
                } else {
                    None
                },
                path_info: if assets.paths.get_asset(instance.path_index.get()).is_some() {
                    Some((
                        instance.path_index.get(),
                        instance.path_position.get().into(),
                        instance.path_speed.get().into(),
                    ))
                } else {
                    None
                },
                alarms: instance.alarms.borrow().clone(),
                bbox_top: instance.bbox_top.get(),
                bbox_left: instance.bbox_left.get(),
                bbox_right: instance.bbox_right.get(),
                bbox_bottom: instance.bbox_bottom.get(),
            }
        }

        let mut stream = TcpStream::connect(&SocketAddr::from(([127, 0, 0, 1], tcp_port)))?;
        stream.set_nonblocking(true)?;
        let mut read_buffer: Vec<u8> = Vec::new();

        let mut replay = Replay::new(self.spoofed_time_nanos.unwrap_or(0), self.rand.seed());

        // Wait for a Hello, then send an update
        loop {
            match stream.receive_message::<Message>(&mut read_buffer)? {
                Some(None) => std::thread::yield_now(),
                Some(Some(m)) => match m {
                    Message::Hello { keys_requested, mouse_buttons_requested, filename } => {
                        // Create or load savefile, depending if it exists
                        let mut path = project_path.clone();
                        std::fs::create_dir_all(&path)?;
                        path.push(&filename);
                        if path.exists() {
                            println!("Project '{}' exists, loading workspace", filename);
                            let state = bincode::deserialize_from::<_, SaveState>(BufReader::new(File::open(&path)?))?;
                            replay = state.load_into(self);
                        } else {
                            println!("Project '{}' doesn't exist, so loading game at entry point", filename);
                            self.load_room(self.room_id)?;
                            for ev in self.stored_events.iter() {
                                replay.startup_events.push(ev.clone());
                            }
                            self.stored_events.clear();

                            println!("Creating new workspace...");
                            let bytes = bincode::serialize(&SaveState::from(self, replay.clone()))?;
                            File::create(&path)?.write_all(&bytes)?;
                        }

                        // Send an update
                        stream.send_message(&message::Information::Update {
                            keys_held: keys_requested
                                .into_iter()
                                .filter(|x| self.input_manager.key_check((*x as u8).into()))
                                .collect(),
                            mouse_buttons_held: mouse_buttons_requested
                                .into_iter()
                                .filter(|x| self.input_manager.mouse_check(*x))
                                .collect(),
                            mouse_location: self.input_manager.mouse_get_location(),
                            frame_count: replay.frame_count(),
                            seed: self.rand.seed(),
                            instance: None,
                        })?;
                        break
                    },
                    m => return Err(format!("Waiting for greeting from server, but got {:?}", m).into()),
                },
                None => return Ok(()),
            }
        }

        let mut game_mousex = 0;
        let mut game_mousey = 0;
        let mut do_update_mouse = false;
        let mut frame_counter = 0;

        loop {
            match stream.receive_message::<Message>(&mut read_buffer)? {
                Some(None) => self.renderer.wait_vsync(),
                Some(Some(m)) => match m {
                    Message::Advance {
                        key_inputs,
                        mouse_inputs,
                        mouse_location,
                        keys_requested,
                        mouse_buttons_requested,
                        instance_requested,
                        new_seed,
                    } => {
                        // Create a frame...
                        let mut frame = replay.new_frame(self.room_speed);
                        frame.mouse_x = mouse_location.0;
                        frame.mouse_y = mouse_location.1;
                        frame.new_seed = new_seed;

                        if let Some(seed) = new_seed {
                            self.rand.set_seed(seed);
                        }

                        // Process inputs
                        for (key, press) in key_inputs.into_iter() {
                            if press {
                                self.input_manager.key_press(key);
                                frame.inputs.push(replay::Input::KeyPress(key));
                            } else {
                                self.input_manager.key_release(key);
                                frame.inputs.push(replay::Input::KeyRelease(key));
                            }
                        }
                        for (button, press) in mouse_inputs.into_iter() {
                            if press {
                                self.input_manager.mouse_press(button);
                                frame.inputs.push(replay::Input::MousePress(button));
                            } else {
                                self.input_manager.mouse_release(button);
                                frame.inputs.push(replay::Input::MouseRelease(button));
                            }
                        }
                        self.input_manager.mouse_update_previous();
                        self.input_manager.set_mouse_pos(mouse_location.0, mouse_location.1);

                        // Advance a frame
                        self.frame()?;
                        match self.scene_change {
                            Some(SceneChange::Room(id)) => self.load_room(id)?,
                            Some(SceneChange::Restart) => self.restart()?,
                            Some(SceneChange::End) => self.restart()?,
                            None => (),
                        }
                        for ev in self.stored_events.iter() {
                            frame.events.push(ev.clone());
                        }
                        self.stored_events.clear();

                        // Fake frame limiter stuff (don't actually frame-limit in record mode)
                        if let Some(t) = self.spoofed_time_nanos.as_mut() {
                            *t += Duration::new(0, 1_000_000_000u32 / self.room_speed).as_nanos();
                        }
            
                        if frame_counter == self.room_speed {
                            self.fps = self.room_speed;
                            frame_counter = 0;
                        }
                        frame_counter += 1;

                        // Send an update
                        stream.send_message(&message::Information::Update {
                            keys_held: keys_requested
                                .into_iter()
                                .filter(|x| self.input_manager.key_check((*x as u8).into()))
                                .collect(),
                            mouse_buttons_held: mouse_buttons_requested
                                .into_iter()
                                .filter(|x| self.input_manager.mouse_check(*x))
                                .collect(),
                            mouse_location: self.input_manager.mouse_get_location(),
                            frame_count: replay.frame_count(),
                            seed: self.rand.seed(),
                            instance: instance_requested.and_then(|x| self.instance_list.get_by_instid(x)).map(|x| {
                                let instance = self.instance_list.get(x);
                                instance.update_bbox(self.get_instance_mask_sprite(x));
                                instance_details(&self.assets, instance)
                            }),
                        })?
                    },

                    Message::SetUpdateMouse { update } => do_update_mouse = update,

                    Message::Save { filename } => {
                        // Save a savestate to a file
                        let mut path = project_path.clone();
                        std::fs::create_dir_all(&path)?;
                        path.push(filename);
                        let mut f = File::create(&path)?;
                        let bytes = bincode::serialize(&SaveState::from(self, replay.clone()))?;
                        f.write_all(&bytes)?;
                    },

                    Message::Load { filename, keys_requested, mouse_buttons_requested, instance_requested } => {
                        // Load savestate from a file
                        let mut path = project_path.clone();
                        path.push(filename);
                        let f = File::open(&path)?;
                        let state = bincode::deserialize_from::<_, SaveState>(BufReader::new(f))?;
                        replay = state.load_into(self);

                        // Send an update
                        stream.send_message(&message::Information::Update {
                            keys_held: keys_requested
                                .into_iter()
                                .filter(|x| self.input_manager.key_check((*x as u8).into()))
                                .collect(),
                            mouse_buttons_held: mouse_buttons_requested
                                .into_iter()
                                .filter(|x| self.input_manager.mouse_check(*x))
                                .collect(),
                            mouse_location: self.input_manager.mouse_get_location(),
                            frame_count: replay.frame_count(),
                            seed: self.rand.seed(),
                            instance: instance_requested.and_then(|x| self.instance_list.get_by_instid(x)).map(|x| {
                                let instance = self.instance_list.get(x);
                                instance.update_bbox(self.get_instance_mask_sprite(x));
                                instance_details(&self.assets, instance)
                            }),
                        })?;
                    },

                    m => break Err(format!("Unexpected message from server: {:?}", m).into()),
                },
                None => break Ok(()),
            }

            for event in self.window.process_events().copied() {
                match event {
                    Event::MouseMove(x, y) => {
                        if do_update_mouse {
                            stream.send_message(&message::Information::MousePosition { x, y })?;
                        }
                        game_mousex = x;
                        game_mousey = y;
                    },

                    Event::MouseButtonDown(MouseButton::Left) => {
                        stream.send_message(&message::Information::LeftClick { x: game_mousex, y: game_mousey })?;
                    },

                    Event::MouseButtonUp(MouseButton::Right) => {
                        let mut options: Vec<(String, usize)> = Vec::new();
                        let (x, y) = self.translate_screen_to_room(f64::from(game_mousex), f64::from(game_mousey));
                        let mut iter = self.instance_list.iter_by_drawing();
                        while let Some(handle) = iter.next(&self.instance_list) {
                            let instance = self.instance_list.get(handle);
                            instance.update_bbox(self.get_instance_mask_sprite(handle));
                            if x >= instance.bbox_left.get()
                                && x <= instance.bbox_right.get()
                                && y >= instance.bbox_top.get()
                                && y <= instance.bbox_bottom.get()
                            {
                                let id = instance.id.get();
                                let description = match self.assets.objects.get_asset(instance.object_index.get()) {
                                    Some(obj) => format!("{} ({})\0", obj.name, id.to_string()),
                                    None => format!("<deleted object> ({})\0", id.to_string()),
                                };
                                options.push((description, id as usize));
                            }
                        }
                        self.window.show_context_menu(&options);
                        break
                    },

                    Event::MenuOption(id) => {
                        if let Some(handle) = self.instance_list.get_by_instid(id as _) {
                            let instance = self.instance_list.get(handle);
                            instance.update_bbox(self.get_instance_mask_sprite(handle));
                            stream.send_message(message::Information::InstanceClicked {
                                details: instance_details(&self.assets, instance),
                            })?;
                            break
                        } else {
                            println!("Requested info for instance #{} [non-existent or deleted]", id);
                        }
                    },

                    Event::KeyboardDown(key) => {
                        stream.send_message(message::Information::KeyPressed { key })?;
                    },

                    _ => (),
                }
            }

            if self.window.close_requested() {
                break Ok(())
            }
        }
    }

    // Replays some recorded inputs to the game
    pub fn replay(mut self, replay: Replay) -> Result<(), Box<dyn std::error::Error>> {
        let mut frame_count: usize = 0;
        self.rand.set_seed(replay.start_seed);
        self.spoofed_time_nanos = Some(replay.start_time);
        let mut frame_counter = 0;

        for ev in replay.startup_events.iter() {
            self.stored_events.push_back(ev.clone());
        }
        self.load_room(self.room_id)?;

        let mut time_now = std::time::Instant::now();
        loop {
            self.window.process_events();
            self.input_manager.mouse_update_previous();
            if let Some(frame) = replay.get_frame(frame_count) {
                self.stored_events.clear();
                for ev in frame.events.iter() {
                    self.stored_events.push_back(ev.clone());
                }

                if let Some(seed) = frame.new_seed {
                    self.rand.set_seed(seed);
                }

                if let Some(time) = frame.new_time {
                    self.spoofed_time_nanos = Some(time);
                }

                self.input_manager.set_mouse_pos(frame.mouse_x, frame.mouse_y);
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
            match self.scene_change {
                Some(SceneChange::Room(id)) => self.load_room(id)?,
                Some(SceneChange::Restart) => self.restart()?,
                Some(SceneChange::End) => break Ok(self.run_game_end_events()?),
                None => (),
            }

            // exit if X pressed or game_end() invoked
            if self.window.close_requested() {
                break Ok(self.run_game_end_events()?)
            }

            // frame limiter
            let diff = Instant::now().duration_since(time_now);
            let duration = Duration::new(0, 1_000_000_000u32 / self.room_speed);
            if let Some(t) = self.spoofed_time_nanos.as_mut() {
                *t += duration.as_nanos();
            }

            if frame_counter == self.room_speed {
                self.fps = self.room_speed;
                frame_counter = 0;
            }
            frame_counter += 1;

            if let Some(time) = duration.checked_sub(diff) {
                gml::datetime::sleep(time);
                time_now += duration;
            } else {
                time_now = Instant::now();
            }

            frame_count += 1;
        }
    }

    // Gets the mouse position in room coordinates
    pub fn get_mouse_in_room(&self) -> (i32, i32) {
        let (x, y) = self.input_manager.mouse_get_location();
        self.translate_screen_to_room(x, y)
    }

    // Gets the previous mouse position in room coordinates
    pub fn get_mouse_previous_in_room(&self) -> (i32, i32) {
        let (x, y) = self.input_manager.mouse_get_previous_location();
        self.translate_screen_to_room(x, y)
    }

    // Translates screen coordinates to room coordinates
    pub fn translate_screen_to_room(&self, x: f64, y: f64) -> (i32, i32) {
        let x = x as i32;
        let y = y as i32;
        if self.views_enabled {
            match self.views.iter().rev().find(|view| view.visible && view.contains_point(x, y)) {
                Some(view) => view.transform_point(x, y),
                None => match self.views.iter().find(|view| view.visible) {
                    Some(view) => view.transform_point(x, y),
                    None => (x, y),
                },
            }
        } else {
            (x, y)
        }
    }

    // Checks for collision between two instances
    pub fn check_collision(&self, i1: usize, i2: usize) -> bool {
        // Don't check for collision with yourself
        if i1 == i2 {
            return false
        }
        // Get the sprite masks we're going to use and update instances' bbox vars
        let inst1 = self.instance_list.get(i1);
        let inst2 = self.instance_list.get(i2);
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
                sprite1
                    .colliders
                    .get((inst1.image_index.get().floor().round() % sprite1.colliders.len() as i32) as usize)
            } else {
                sprite1.colliders.first()
            } {
                Some(c) => c,
                None => return false,
            };

            let collider2 = match if sprite2.per_frame_colliders {
                sprite2
                    .colliders
                    .get((inst2.image_index.get().floor().round() % sprite2.colliders.len() as i32) as usize)
            } else {
                sprite2.colliders.first()
            } {
                Some(c) => c,
                None => return false,
            };

            // round x and y values, and get sin and cos of each angle...
            let x1 = inst1.x.get().round();
            let y1 = inst1.y.get().round();
            let x2 = inst2.x.get().round();
            let y2 = inst2.y.get().round();
            let angle1 = inst1.image_angle.get().to_radians();
            let sin1 = angle1.sin().into_inner();
            let cos1 = angle1.cos().into_inner();
            let angle2 = inst2.image_angle.get().to_radians();
            let sin2 = angle2.sin().into_inner();
            let cos2 = angle2.cos().into_inner();

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
                    let mut x = Real::from(intersect_x) - x1.into();
                    let mut y = Real::from(intersect_y) - y1.into();
                    util::rotate_around_center(x.as_mut_ref(), y.as_mut_ref(), sin1, cos1);
                    let x = (Real::from(sprite1.origin_x) + (x / inst1.image_xscale.get()).floor()).round();
                    let y = (Real::from(sprite1.origin_y) + (y / inst1.image_yscale.get()).floor()).round();

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
                        let mut x = Real::from(intersect_x) - x2.into();
                        let mut y = Real::from(intersect_y) - y2.into();
                        util::rotate_around_center(x.as_mut_ref(), y.as_mut_ref(), sin2, cos2);
                        let x = (Real::from(sprite2.origin_x) + (x / inst2.image_xscale.get()).floor()).round();
                        let y = (Real::from(sprite2.origin_y) + (y / inst2.image_yscale.get()).floor()).round();

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
    pub fn check_collision_point(&self, inst: usize, x: i32, y: i32, precise: bool) -> bool {
        // Get sprite mask, update bbox
        let inst = self.instance_list.get(inst);
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

        // Stop now if precise collision is disabled
        if !precise {
            return true
        }

        // Can't collide if no sprite or no associated collider
        if let Some(sprite) = sprite {
            // Get collider
            let collider = match if sprite.per_frame_colliders {
                sprite.colliders.get(inst.image_index.get().floor().into_inner() as usize % sprite.colliders.len())
            } else {
                sprite.colliders.first()
            } {
                Some(c) => c,
                None => return false,
            };

            // Transform point to be relative to collider
            let angle = inst.image_angle.get().to_radians();
            let mut x = Real::from(x) - inst.x.get();
            let mut y = Real::from(y) - inst.y.get();
            util::rotate_around_center(x.as_mut_ref(), y.as_mut_ref(), angle.sin().into(), angle.cos().into());
            let x = (Real::from(sprite.origin_x) + (x / inst.image_xscale.get())).round();
            let y = (Real::from(sprite.origin_y) + (y / inst.image_yscale.get())).round();

            // And finally, look up this point in the collider
            x >= collider.bbox_left as i32
                && y >= collider.bbox_top as i32
                && x <= collider.bbox_right as i32
                && y <= collider.bbox_bottom as i32
                && collider.data.get((y as usize * collider.width as usize) + x as usize).copied().unwrap_or(false)
        } else {
            false
        }
    }

    // Checks if an instance is colliding with a rectangle
    pub fn check_collision_rectangle(&self, inst: usize, x1: i32, y1: i32, x2: i32, y2: i32, precise: bool) -> bool {
        // Get sprite mask, update bbox
        let inst = self.instance_list.get(inst);
        let sprite = self
            .assets
            .sprites
            .get_asset(if inst.mask_index.get() < 0 { inst.sprite_index.get() } else { inst.mask_index.get() })
            .map(|x| x.as_ref());
        inst.update_bbox(sprite);

        let rect_left = x1.min(x2);
        let rect_top = y1.min(y2);
        let rect_right = x1.max(x2);
        let rect_bottom = y1.max(y2);

        // AABB with the rectangle
        if inst.bbox_right.get() < rect_left
            || rect_right < inst.bbox_left.get()
            || inst.bbox_bottom.get() < rect_top
            || rect_bottom < inst.bbox_top.get()
        {
            return false
        }

        // Stop now if precise collision is disabled
        if !precise {
            return true
        }

        // Can't collide if no sprite or no associated collider
        if let Some(sprite) = sprite {
            // Get collider
            let collider = match if sprite.per_frame_colliders {
                sprite.colliders.get(inst.image_index.get().floor().into_inner() as usize % sprite.colliders.len())
            } else {
                sprite.colliders.first()
            } {
                Some(c) => c,
                None => return false,
            };

            let inst_x = inst.x.get().round();
            let inst_y = inst.y.get().round();
            let angle = inst.image_angle.get().to_radians();
            let sin = angle.sin().into_inner();
            let cos = angle.cos().into_inner();

            // Get intersect rectangle
            let intersect_top = inst.bbox_top.get().max(rect_top);
            let intersect_bottom = inst.bbox_bottom.get().min(rect_bottom);
            let intersect_left = inst.bbox_left.get().max(rect_left);
            let intersect_right = inst.bbox_right.get().min(rect_right);

            // Go through each pixel in the intersect
            for intersect_y in intersect_top..=intersect_bottom {
                for intersect_x in intersect_left..=intersect_right {
                    // Transform point to be relative to collider
                    let mut x = Real::from(intersect_x) - inst_x.into();
                    let mut y = Real::from(intersect_y) - inst_y.into();
                    util::rotate_around_center(x.as_mut_ref(), y.as_mut_ref(), sin, cos);
                    let x = (Real::from(sprite.origin_x) + (x / inst.image_xscale.get()).floor()).round();
                    let y = (Real::from(sprite.origin_y) + (y / inst.image_yscale.get()).floor()).round();

                    // And finally, look up this point in the collider
                    if x >= collider.bbox_left as i32
                        && y >= collider.bbox_top as i32
                        && x <= collider.bbox_right as i32
                        && y <= collider.bbox_bottom as i32
                        && collider
                            .data
                            .get((y as usize * collider.width as usize) + x as usize)
                            .copied()
                            .unwrap_or(false)
                    {
                        return true
                    }
                }
            }

            false
        } else {
            false
        }
    }

    pub fn check_collision_ellipse(&self, inst: usize, x1: Real, y1: Real, x2: Real, y2: Real, precise: bool) -> bool {
        // Get sprite mask, update bbox
        let inst = self.instance_list.get(inst);
        let sprite = self
            .assets
            .sprites
            .get_asset(if inst.mask_index.get() < 0 { inst.sprite_index.get() } else { inst.mask_index.get() })
            .map(|x| x.as_ref());
        inst.update_bbox(sprite);

        let bbox_left: Real = inst.bbox_left.get().into();
        let bbox_right: Real = inst.bbox_right.get().into();
        let bbox_top: Real = inst.bbox_top.get().into();
        let bbox_bottom: Real = inst.bbox_bottom.get().into();

        let rect_left = x1.min(x2);
        let rect_right = x1.max(x2);
        let rect_top = y1.min(y2);
        let rect_bottom = y1.max(y2);

        // AABB with the rectangle
        if bbox_right + Real::from(1.0) <= rect_left
            || rect_right < bbox_left
            || bbox_bottom + Real::from(1.0) <= rect_top
            || rect_bottom < bbox_top
        {
            return false
        }

        let rect_left = rect_left.round();
        let rect_right = rect_right.round();
        let rect_top = rect_top.round();
        let rect_bottom = rect_bottom.round();

        let ellipse_xcenter = Real::from(rect_right + rect_left) / 2.into();
        let ellipse_ycenter = Real::from(rect_bottom + rect_top) / 2.into();
        let ellipse_xrad = Real::from(rect_right - rect_left) / 2.into();
        let ellipse_yrad = Real::from(rect_bottom - rect_top) / 2.into();

        let point_in_ellipse = |x: Real, y: Real| {
            let x_dist = (x - ellipse_xcenter) / ellipse_xrad;
            let y_dist = (y - ellipse_ycenter) / ellipse_yrad;
            x_dist * x_dist + y_dist * y_dist <= 1.into()
        };

        // The AABB passed, so if the ellipse's center isn't diagonally separated from the instance's bbox,
        // that means the leftmost or rightmost or whatever point of the circle is inside the bbox, so we're colliding.
        if (ellipse_xcenter < bbox_left || ellipse_xcenter > bbox_right)
            && (ellipse_ycenter < bbox_top || ellipse_ycenter > bbox_bottom)
        {
            // If this isn't the case, there can only be collision if the closest corner is inside the ellipse.
            if !point_in_ellipse(bbox_left.into(), bbox_top.into())
                && !point_in_ellipse(bbox_left.into(), bbox_bottom.into())
                && !point_in_ellipse(bbox_right.into(), bbox_top.into())
                && !point_in_ellipse(bbox_right.into(), bbox_bottom.into())
            {
                return false
            }
        }

        // Stop now if precise collision is disabled
        if !precise {
            return true
        }

        // Can't collide if no sprite or no associated collider
        if let Some(sprite) = sprite {
            // Get collider
            let collider = match if sprite.per_frame_colliders {
                sprite.colliders.get(inst.image_index.get().floor().into_inner() as usize % sprite.colliders.len())
            } else {
                sprite.colliders.first()
            } {
                Some(c) => c,
                None => return false,
            };

            // Round everything, as GM does
            let inst_x = inst.x.get().round();
            let inst_y = inst.y.get().round();
            let angle = inst.image_angle.get().to_radians();
            let sin = angle.sin().into_inner();
            let cos = angle.cos().into_inner();

            // Get intersect rectangle
            let intersect_top = inst.bbox_top.get().max(rect_top);
            let intersect_bottom = inst.bbox_bottom.get().min(rect_bottom);
            let intersect_left = inst.bbox_left.get().max(rect_left);
            let intersect_right = inst.bbox_right.get().min(rect_right);

            // Go through each pixel in the intersect
            for intersect_y in intersect_top..=intersect_bottom {
                for intersect_x in intersect_left..=intersect_right {
                    // Check if point is in ellipse
                    if point_in_ellipse(intersect_x.into(), intersect_y.into()) {
                        // Transform point to be relative to collider
                        let mut x = Real::from(intersect_x) - inst_x.into();
                        let mut y = Real::from(intersect_y) - inst_y.into();
                        util::rotate_around_center(x.as_mut_ref(), y.as_mut_ref(), sin, cos);
                        let x = (Real::from(sprite.origin_x) + (x / inst.image_xscale.get()).floor()).round();
                        let y = (Real::from(sprite.origin_y) + (y / inst.image_yscale.get()).floor()).round();

                        // And finally, look up this point in the collider
                        if x >= collider.bbox_left as i32
                            && y >= collider.bbox_top as i32
                            && x <= collider.bbox_right as i32
                            && y <= collider.bbox_bottom as i32
                            && collider
                                .data
                                .get((y as usize * collider.width as usize) + x as usize)
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

    pub fn check_collision_line(&self, inst: usize, x1: Real, y1: Real, x2: Real, y2: Real, precise: bool) -> bool {
        // Get sprite mask, update bbox
        let inst = self.instance_list.get(inst);
        let sprite = self
            .assets
            .sprites
            .get_asset(if inst.mask_index.get() < 0 { inst.sprite_index.get() } else { inst.mask_index.get() })
            .map(|x| x.as_ref());
        inst.update_bbox(sprite);

        let bbox_left: Real = inst.bbox_left.get().into();
        let bbox_right: Real = inst.bbox_right.get().into();
        let bbox_top: Real = inst.bbox_top.get().into();
        let bbox_bottom: Real = inst.bbox_bottom.get().into();

        let rect_left = x1.min(x2);
        let rect_right = x1.max(x2);
        let rect_top = y1.min(y2);
        let rect_bottom = y1.max(y2);

        // AABB with the rectangle
        if bbox_right + Real::from(1.0) <= rect_left
            || rect_right < bbox_left
            || bbox_bottom + Real::from(1.0) <= rect_top
            || rect_bottom < bbox_top
        {
            return false
        }

        // Truncate to the line horizontally
        let (mut x1, mut y1, mut x2, mut y2) = if x2 < x1 { (x2, y2, x1, y1) } else { (x1, y1, x2, y2) };
        if x1 < bbox_left {
            y1 = (y2 - y1) * (bbox_left - x1) / (x2 - x1) + y1;
            x1 = bbox_left;
        }
        if x2 > bbox_right + Real::from(1.0) {
            let new_x2 = bbox_right + Real::from(1.0);
            y2 = (y2 - y1) * (new_x2 - x2) / (x2 - x1) + y2;
            x2 = new_x2;
        }

        // Check for overlap
        if (bbox_top > y1 && bbox_top > y2)
            || (y1 >= bbox_bottom + Real::from(1.0) && y2 >= bbox_bottom + Real::from(1.0))
        {
            return false
        }

        // Stop now if precise collision is disabled
        if !precise {
            return true
        }

        // Can't collide if no sprite or no associated collider
        if let Some(sprite) = sprite {
            // Get collider
            let collider = match if sprite.per_frame_colliders {
                sprite.colliders.get(inst.image_index.get().floor().into_inner() as usize % sprite.colliders.len())
            } else {
                sprite.colliders.first()
            } {
                Some(c) => c,
                None => return false,
            };

            // Round everything, as GM does
            let inst_x = inst.x.get().round();
            let inst_y = inst.y.get().round();
            let angle = inst.image_angle.get().to_radians();
            let sin = angle.sin().into_inner();
            let cos = angle.cos().into_inner();

            let x1 = x1.round();
            let y1 = y1.round();
            let x2 = x2.round();
            let y2 = y2.round();

            // Set up the iterator
            let iter_vert = (x2 - x1).abs() < (y2 - y1).abs();
            let point_count = (if iter_vert { y2 - y1 } else { x2 - x1 }) + 1;
            // If iterating vertically, make sure we're going top to bottom
            let (x1, y1, x2, y2) = if iter_vert && y2 < y1 { (x2, y2, x1, y1) } else { (x1, y1, x2, y2) };
            // Helper function for getting points on the line
            let get_point = |i: i32| {
                // Avoid dividing by zero
                if point_count == 1 {
                    return (Real::from(x1), Real::from(y1))
                }
                if iter_vert {
                    let slope = Real::from(x2 - x1) / Real::from(y2 - y1);
                    (Real::from(x1) + Real::from(i) * slope, Real::from(y1 + i))
                } else {
                    let slope = Real::from(y2 - y1) / Real::from(x2 - x1);
                    (Real::from(x1 + i), Real::from(y1) + Real::from(i) * slope)
                }
            };

            for i in 0..point_count {
                let (mut x, mut y) = get_point(i);

                // Transform point to be relative to collider
                x -= inst_x.into();
                y -= inst_y.into();
                util::rotate_around_center(x.as_mut_ref(), y.as_mut_ref(), sin, cos);
                let x = (Real::from(sprite.origin_x) + (x / inst.image_xscale.get()).floor()).round();
                let y = (Real::from(sprite.origin_y) + (y / inst.image_yscale.get()).floor()).round();

                // And finally, look up this point in the collider
                if x >= collider.bbox_left as i32
                    && y >= collider.bbox_top as i32
                    && x <= collider.bbox_right as i32
                    && y <= collider.bbox_bottom as i32
                    && collider.data.get((y as usize * collider.width as usize) + x as usize).copied().unwrap_or(false)
                {
                    return true
                }
            }
            false
        } else {
            false
        }
    }

    // Checks if an instance is colliding with any solid, returning the solid if it is, otherwise None
    pub fn check_collision_solid(&self, inst: usize) -> Option<usize> {
        let mut iter = self.instance_list.iter_by_insertion();
        while let Some(target) = iter.next(&self.instance_list) {
            if self.instance_list.get(target).solid.get() {
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

    /// Finds an instance that matches the predicate.
    /// `object_id` can be -3 for `all`, an object ID, or an instance ID.
    /// The predicate should take an instance handle as an argument, and return true if it matches.
    pub fn find_instance_with(&self, object_id: i32, pred: impl Fn(usize) -> bool) -> Option<usize> {
        match object_id {
            gml::ALL => {
                let mut iter = self.instance_list.iter_by_insertion();
                loop {
                    match iter.next(&self.instance_list) {
                        Some(handle) => {
                            if pred(handle) {
                                break Some(handle)
                            }
                        },
                        None => break None,
                    }
                }
            },
            _ if object_id < 0 => None,
            object_id if object_id < 100000 => {
                if let Some(ids) = self.assets.objects.get_asset(object_id).map(|x| x.children.clone()) {
                    let mut iter = self.instance_list.iter_by_identity(ids);
                    loop {
                        match iter.next(&self.instance_list) {
                            Some(handle) => {
                                if pred(handle) {
                                    break Some(handle)
                                }
                            },
                            None => break None,
                        }
                    }
                } else {
                    None
                }
            },
            instance_id => {
                if let Some(handle) = self.instance_list.get_by_instid(instance_id) {
                    if self.instance_list.get(handle).is_active() && pred(handle) { Some(handle) } else { None }
                } else {
                    None
                }
            },
        }
    }
}

pub trait GetAsset<T> {
    fn get_asset(&self, index: ID) -> Option<&T>;
    fn get_asset_mut(&mut self, index: ID) -> Option<&mut T>;
}

impl<T> GetAsset<T> for Vec<Option<T>> {
    fn get_asset(&self, index: ID) -> Option<&T> {
        self.get(usize::try_from(index).ok()?)?.as_ref()
    }

    fn get_asset_mut(&mut self, index: ID) -> Option<&mut T> {
        self.get_mut(usize::try_from(index).ok()?)?.as_mut()
    }
}
