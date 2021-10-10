use crate::{
    game::{
        audio::AudioState, draw, external, includedfile::IncludedFile, model::Model, particle,
        pathfinding::PotentialStepSettings, surface::Surface, transition::UserTransition, Assets, Game, Replay,
        RoomState, Version,
    },
    gml::{self, ds, rand::Random, Compiler},
    handleman::HandleList,
    input::Input,
    instance::DummyFieldHolder,
    math::Real,
    render::{RendererState, SavedTexture, Scaling},
    types::{Colour, ID},
};
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use indexmap::IndexMap;
use lzzzz::lz4;
use serde::{Deserialize, Serialize};
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    fs::{File, OpenOptions},
    io::{self, Read, Write},
    path::PathBuf,
    rc::Rc,
};

/// Represents a savestate. Very similar to the Game struct, but without things which aren't serialized.
#[derive(Clone, Serialize, Deserialize)]
pub struct SaveState {
    pub compiler: Compiler,
    pub rand: Random,
    pub input: Input,
    pub assets: Assets,
    pub event_holders: [IndexMap<u32, Rc<RefCell<Vec<ID>>>>; 12],
    pub custom_draw_objects: HashSet<ID>,

    pub background_colour: Colour,
    pub textures: Vec<Option<SavedTexture>>,

    pub externals: external::ExternalState,
    pub surface_fix: bool,

    pub view_current: usize,

    pub last_instance_id: ID,
    pub last_tile_id: ID,

    pub particles: particle::Manager,

    pub room: RoomState,
    pub stored_rooms: Vec<RoomState>,
    pub room_order: Box<[i32]>,
    pub user_transitions: HashMap<i32, UserTransition>,

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

    pub draw_font_id: ID,
    pub draw_colour: Colour,
    pub draw_alpha: Real,
    pub draw_halign: draw::Halign,
    pub draw_valign: draw::Valign,
    pub surfaces: Vec<Option<Surface>>,
    pub surface_target: Option<i32>,
    pub models: Vec<Option<Model>>,
    pub model_matrix_stack: Vec<[f32; 16]>,
    pub auto_draw: bool,
    pub renderer_state: RendererState,

    pub uninit_fields_are_zero: bool,
    pub uninit_args_are_zero: bool,

    pub potential_step_settings: PotentialStepSettings,

    pub fps: u32,
    pub frame_counter: u32,
    pub transition_kind: i32,
    pub transition_steps: i32,
    pub cursor_sprite: i32,
    pub cursor_sprite_frame: u32,
    pub score: i32,
    pub score_capt: gml::String,
    pub score_capt_d: bool,
    pub has_set_show_score: bool,
    pub lives: i32,
    pub lives_capt: gml::String,
    pub lives_capt_d: bool,
    pub health: Real,
    pub health_capt: gml::String,
    pub health_capt_d: bool,
    pub error_occurred: bool,
    pub error_last: gml::String,

    pub game_id: i32,
    pub program_directory: gml::String,
    pub included_files: Vec<IncludedFile>,
    pub gm_version: Version,
    pub spoofed_time_nanos: Option<u128>,

    scaling: Scaling,
    unscaled_width: u32,
    unscaled_height: u32,
    window_width: u32,
    window_height: u32,

    audio_state: AudioState,

    replay: Replay,
    screenshot: Box<[u8]>,
    zbuffer: Box<[f32]>,
}

impl SaveState {
    /// Creates a new SaveState from the given components.
    pub fn from(game: &mut Game, replay: Replay, renderer_state: RendererState) -> Self {
        let (window_width, window_height) = game.renderer.stored_size();
        let screenshot = game.renderer.stored_pixels();
        let zbuffer = game.renderer.stored_zbuffer();

        Self {
            compiler: game.compiler.clone(),
            rand: game.rand.clone(),
            input: game.input.clone(),
            assets: game.assets.clone(),
            event_holders: game.event_holders.clone(),
            custom_draw_objects: game.custom_draw_objects.clone(),
            background_colour: game.background_colour,
            textures: game.renderer.dump_dynamic_textures(),
            externals: game.externals.save_state(),
            surface_fix: game.surface_fix.clone(),
            view_current: game.view_current,
            last_instance_id: game.last_instance_id.clone(),
            last_tile_id: game.last_tile_id.clone(),
            particles: game.particles.clone(),
            room: game.room.clone(),
            stored_rooms: game.stored_rooms.clone(),
            room_order: game.room_order.clone(),
            user_transitions: game.user_transitions.clone(),
            globals: game.globals.clone(),
            globalvars: game.globalvars.clone(),
            game_start: game.game_start.clone(),
            stacks: game.stacks.clone(),
            queues: game.queues.clone(),
            lists: game.lists.clone(),
            maps: game.maps.clone(),
            priority_queues: game.priority_queues.clone(),
            grids: game.grids.clone(),
            ds_precision: game.ds_precision.clone(),
            draw_font_id: game.draw_font_id.clone(),
            draw_colour: game.draw_colour.clone(),
            draw_alpha: game.draw_alpha.clone(),
            draw_halign: game.draw_halign.clone(),
            draw_valign: game.draw_valign.clone(),
            surfaces: game.surfaces.clone(),
            surface_target: game.surface_target,
            models: game.models.clone(),
            model_matrix_stack: game.model_matrix_stack.clone(),
            auto_draw: game.auto_draw,
            renderer_state,
            uninit_fields_are_zero: game.uninit_fields_are_zero.clone(),
            uninit_args_are_zero: game.uninit_args_are_zero.clone(),
            potential_step_settings: game.potential_step_settings.clone(),
            fps: game.fps,
            frame_counter: game.frame_counter,
            transition_kind: game.transition_kind.clone(),
            transition_steps: game.transition_steps.clone(),
            cursor_sprite: game.cursor_sprite.clone(),
            cursor_sprite_frame: game.cursor_sprite_frame.clone(),
            score: game.score.clone(),
            score_capt: game.score_capt.clone(),
            score_capt_d: game.score_capt_d.clone(),
            has_set_show_score: game.has_set_show_score.clone(),
            lives: game.lives.clone(),
            lives_capt: game.lives_capt.clone(),
            lives_capt_d: game.lives_capt_d.clone(),
            health: game.health.clone(),
            health_capt: game.health_capt.clone(),
            health_capt_d: game.health_capt_d.clone(),
            error_occurred: game.error_occurred,
            error_last: game.error_last.clone(),
            game_id: game.game_id.clone(),
            program_directory: game.program_directory.clone(),
            included_files: game.included_files.clone(),
            gm_version: game.gm_version.clone(),
            spoofed_time_nanos: game.spoofed_time_nanos,
            scaling: game.scaling,
            unscaled_width: game.unscaled_width,
            unscaled_height: game.unscaled_height,
            window_width,
            window_height,
            audio_state: game.audio.state(),
            replay,
            screenshot,
            zbuffer,
        }
    }

    /// Loads this SaveState into the given Game struct, returning the Replay and RendererState it contained.
    pub fn load_into(self, game: &mut Game) -> (Replay, RendererState) {
        game.renderer.upload_dynamic_textures(&self.textures);

        game.renderer.set_stored(self.screenshot, self.zbuffer, self.window_width, self.window_height);

        let surfaces = self.surfaces;
        if let Some(Some(surf)) = self.surface_target.and_then(|id| surfaces.get(id as usize)) {
            game.renderer.set_target(surf.atlas_ref);
        } else {
            game.renderer.reset_target();
        }

        game.externals.load_state(self.externals);

        game.surface_fix = self.surface_fix;

        game.compiler = self.compiler;
        game.rand = self.rand;
        game.input = self.input;
        game.assets = self.assets;
        game.event_holders = self.event_holders;
        game.custom_draw_objects = self.custom_draw_objects;
        game.background_colour = self.background_colour;
        game.last_instance_id = self.last_instance_id;
        game.last_tile_id = self.last_tile_id;
        game.view_current = self.view_current;
        game.particles = self.particles;
        game.room = self.room;
        game.stored_rooms = self.stored_rooms;
        game.room_order = self.room_order;
        game.user_transitions = self.user_transitions;
        game.globals = self.globals;
        game.globalvars = self.globalvars;
        game.game_start = self.game_start;
        game.stacks = self.stacks;
        game.queues = self.queues;
        game.lists = self.lists;
        game.maps = self.maps;
        game.priority_queues = self.priority_queues;
        game.grids = self.grids;
        game.ds_precision = self.ds_precision;
        game.draw_font_id = self.draw_font_id;
        game.draw_colour = self.draw_colour;
        game.draw_alpha = self.draw_alpha;
        game.draw_halign = self.draw_halign;
        game.draw_valign = self.draw_valign;
        game.surfaces = surfaces;
        game.surface_target = self.surface_target;
        game.models = self.models;
        game.model_matrix_stack = self.model_matrix_stack;
        game.auto_draw = self.auto_draw;
        game.uninit_fields_are_zero = self.uninit_fields_are_zero;
        game.uninit_args_are_zero = self.uninit_args_are_zero;
        game.potential_step_settings = self.potential_step_settings;
        game.fps = self.fps;
        game.frame_counter = self.frame_counter;
        game.transition_kind = self.transition_kind;
        game.transition_steps = self.transition_steps;
        game.cursor_sprite = self.cursor_sprite;
        game.cursor_sprite_frame = self.cursor_sprite_frame;
        game.score = self.score;
        game.score_capt = self.score_capt;
        game.score_capt_d = self.score_capt_d;
        game.has_set_show_score = self.has_set_show_score;
        game.lives = self.lives;
        game.lives_capt = self.lives_capt;
        game.lives_capt_d = self.lives_capt_d;
        game.health = self.health;
        game.health_capt = self.health_capt;
        game.health_capt_d = self.health_capt_d;
        game.error_occurred = self.error_occurred;
        game.error_last = self.error_last;
        game.game_id = self.game_id;
        game.program_directory = self.program_directory;
        game.included_files = self.included_files;
        game.gm_version = self.gm_version;
        game.spoofed_time_nanos = self.spoofed_time_nanos;
        game.audio.set_state(self.audio_state);
        game.scaling = self.scaling;
        game.unscaled_width = self.unscaled_width;
        game.unscaled_height = self.unscaled_height;
        (self.replay, self.renderer_state)
    }

    /// Consumes this SaveState and returns just its Replay component.
    pub fn into_replay(self) -> Replay {
        self.replay
    }

    /// Loads a SaveState from a file. The format will always match the one used by `save_to_file()`.
    pub fn from_file(path: &PathBuf, buffer: &mut Buffer) -> Result<Self, ReadError> {
        match File::open(path).map(|f| (f.metadata().map(|m| m.len() as usize + 1).unwrap_or(0), f)) {
            Ok((init_size, mut file)) => {
                buffer.lz4_buf.clear();
                buffer.lz4_buf.reserve(init_size);
                match file.read_to_end(&mut buffer.lz4_buf) {
                    Ok(_) => {
                        match (buffer.lz4_buf.as_slice().read_u64::<LE>().map(|x| x as usize), buffer.lz4_buf.get(8..))
                        {
                            (Ok(len), Some(block)) => {
                                buffer.bin_buf.clear();
                                buffer.bin_buf.reserve(len);
                                unsafe { buffer.bin_buf.set_len(len) };
                                match lz4::decompress(block, buffer.bin_buf.as_mut_slice()) {
                                    Ok(len) => {
                                        unsafe { buffer.bin_buf.set_len(len) };
                                        bincode::deserialize::<'_, SaveState>(buffer.bin_buf.as_slice())
                                            .map_err(ReadError::DeserializeErr)
                                    },
                                    Err(err) => Err(ReadError::DecompressErr(err)),
                                }
                            },
                            (Ok(_), None) => Err(ReadError::IOErr(io::Error::from(io::ErrorKind::UnexpectedEof))),
                            (Err(err), _) => Err(ReadError::IOErr(err)),
                        }
                    },
                    Err(err) => Err(ReadError::IOErr(err)),
                }
            },
            Err(err) => Err(ReadError::IOErr(err)),
        }
    }

    /// Saves a SaveState to a file. The SaveState object is formatted with Serde/bincode and compressed with lz4.
    /// A Buffer object is needed for the lz4 compression. Ideally, the same buffer should be re-used on each call.
    pub fn save_to_file(&self, path: &PathBuf, buffer: &mut Buffer) -> Result<(), WriteError> {
        buffer.bin_buf.clear();
        buffer.lz4_buf.clear();
        match bincode::serialize_into(&mut buffer.bin_buf, self) {
            Ok(()) => {
                match lz4::compress_to_vec(buffer.bin_buf.as_slice(), buffer.lz4_buf.as_mut(), lz4::ACC_LEVEL_DEFAULT) {
                    Ok(_length) => {
                        match OpenOptions::new().create(true).write(true).truncate(true).open(path).and_then(|mut f| {
                            f.write_u64::<LE>(buffer.bin_buf.len() as u64)
                                .and_then(|_| f.write_all(buffer.lz4_buf.as_slice()))
                        }) {
                            Ok(()) => Ok(()),
                            Err(e) => Err(WriteError::IOErr(e)),
                        }
                    },
                    Err(err) => Err(WriteError::CompressErr(err)),
                }
            },
            Err(err) => Err(WriteError::SerializeErr(err)),
        }
    }
}

pub struct Buffer {
    bin_buf: Vec<u8>,
    lz4_buf: Vec<u8>,
}

impl Buffer {
    pub fn new() -> Self {
        Self { bin_buf: Vec::new(), lz4_buf: Vec::new() }
    }
}

#[derive(Debug)]
pub enum ReadError {
    IOErr(io::Error),
    DecompressErr(lzzzz::Error),
    DeserializeErr(Box<bincode::ErrorKind>),
}

#[derive(Debug)]
pub enum WriteError {
    IOErr(io::Error),
    CompressErr(lzzzz::Error),
    SerializeErr(Box<bincode::ErrorKind>),
}
