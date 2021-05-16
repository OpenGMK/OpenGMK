use crate::{
    game::{
        draw,
        external::{DefineInfo, External},
        includedfile::IncludedFile,
        model::Model,
        particle,
        pathfinding::PotentialStepSettings,
        RoomState,
        string::RCStr,
        surface::Surface,
        transition::UserTransition,
        Assets, Game, Replay, Version,
    },
    gml::{ds, rand::Random, Compiler},
    handleman::HandleList,
    input::InputManager,
    instance::DummyFieldHolder,
    math::Real,
};
use gmio::render::{BlendType, Fog, PrimitiveBuilder, SavedTexture, Scaling};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use shared::types::{Colour, ID};
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

/// Represents a savestate. Very similar to the Game struct, but without things which aren't serialized.
#[derive(Clone, Serialize, Deserialize)]
pub struct SaveState {
    pub compiler: Compiler,
    pub rand: Random,
    pub input_manager: InputManager,
    pub assets: Assets,
    pub event_holders: [IndexMap<u32, Rc<RefCell<Vec<ID>>>>; 12],
    pub custom_draw_objects: HashSet<ID>,

    pub background_colour: Colour,
    pub textures: Vec<Option<SavedTexture>>,
    pub alpha_blending: bool,
    pub blend_mode: (BlendType, BlendType),
    pub interpolate_pixels: bool,
    pub texture_repeat: bool,
    pub sprite_count: i32,
    pub vsync: bool,

    pub externals: Vec<Option<DefineInfo>>,
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
    pub using_3d: bool,
    pub depth: f32,
    pub depth_test: bool,
    pub write_depth: bool,
    pub culling: bool,
    pub perspective: bool,
    pub fog: Option<Fog>,
    pub gouraud: bool,
    pub surfaces: Vec<Option<Surface>>,
    pub surface_target: Option<i32>,
    pub model_matrix: [f32; 16],
    pub models: Vec<Option<Model>>,
    pub model_matrix_stack: Vec<[f32; 16]>,
    pub auto_draw: bool,
    pub circle_precision: i32,
    pub primitive_2d: PrimitiveBuilder,
    pub primitive_3d: PrimitiveBuilder,
    pub zbuf_trashed: bool,

    pub uninit_fields_are_zero: bool,
    pub uninit_args_are_zero: bool,

    pub potential_step_settings: PotentialStepSettings,

    pub transition_kind: i32,
    pub transition_steps: i32,
    pub cursor_sprite: i32,
    pub cursor_sprite_frame: u32,
    pub score: i32,
    pub score_capt: RCStr,
    pub score_capt_d: bool,
    pub has_set_show_score: bool,
    pub lives: i32,
    pub lives_capt: RCStr,
    pub lives_capt_d: bool,
    pub health: Real,
    pub health_capt: RCStr,
    pub health_capt_d: bool,
    pub error_occurred: bool,
    pub error_last: RCStr,

    pub game_id: i32,
    pub program_directory: RCStr,
    pub included_files: Vec<IncludedFile>,
    pub gm_version: Version,
    pub spoofed_time_nanos: Option<u128>,

    scaling: Scaling,
    unscaled_width: u32,
    unscaled_height: u32,
    window_width: u32,
    window_height: u32,

    replay: Replay,
    screenshot: Box<[u8]>,
    zbuffer: Box<[f32]>,
}

impl SaveState {
    pub fn from(game: &Game, replay: Replay) -> Self {
        let (window_width, window_height) = game.window.get_inner_size();
        let screenshot = game.renderer.get_pixels(0, 0, game.unscaled_width as _, game.unscaled_height as _);
        let zbuffer = game.renderer.dump_zbuffer();

        Self {
            compiler: game.compiler.clone(),
            rand: game.rand.clone(),
            input_manager: game.input_manager.clone(),
            assets: game.assets.clone(),
            event_holders: game.event_holders.clone(),
            custom_draw_objects: game.custom_draw_objects.clone(),
            background_colour: game.background_colour,
            textures: game.renderer.dump_dynamic_textures(),
            alpha_blending: game.renderer.get_alpha_blending(),
            blend_mode: game.renderer.get_blend_mode(),
            interpolate_pixels: game.renderer.get_pixel_interpolation(),
            texture_repeat: game.renderer.get_texture_repeat(),
            sprite_count: game.renderer.get_sprite_count(),
            vsync: game.renderer.get_vsync(),
            externals: game.externals.iter().map(|e| e.as_ref().map(|e| e.info.clone())).collect(),
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
            using_3d: game.renderer.get_3d(),
            depth: game.renderer.get_depth(),
            depth_test: game.renderer.get_depth_test(),
            write_depth: game.renderer.get_write_depth(),
            culling: game.renderer.get_culling(),
            perspective: game.renderer.get_perspective(),
            fog: game.renderer.get_fog(),
            gouraud: game.renderer.get_gouraud(),
            surfaces: game.surfaces.clone(),
            surface_target: game.surface_target,
            model_matrix: game.renderer.get_model_matrix(),
            models: game.models.clone(),
            model_matrix_stack: game.model_matrix_stack.clone(),
            auto_draw: game.auto_draw,
            circle_precision: game.renderer.get_circle_precision(),
            primitive_2d: game.renderer.get_primitive_2d(),
            primitive_3d: game.renderer.get_primitive_3d(),
            zbuf_trashed: game.renderer.get_zbuf_trashed(),
            uninit_fields_are_zero: game.uninit_fields_are_zero.clone(),
            uninit_args_are_zero: game.uninit_args_are_zero.clone(),
            potential_step_settings: game.potential_step_settings.clone(),
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
            replay,
            screenshot,
            zbuffer,
        }
    }

    pub fn load_into(self, game: &mut Game) -> Replay {
        if game.window.get_inner_size() != (self.window_width, self.window_height) {
            game.window.resize(self.window_width, self.window_height);
        }

        game.renderer.upload_dynamic_textures(&self.textures);

        game.renderer.draw_raw_frame(
            self.screenshot,
            self.zbuffer,
            self.unscaled_width as _,
            self.unscaled_height as _,
            self.window_width as _,
            self.window_height as _,
            self.scaling,
        );

        let surfaces = self.surfaces;
        if let Some(Some(surf)) = self.surface_target.and_then(|id| surfaces.get(id as usize)) {
            game.renderer.set_target(&surf.atlas_ref);
        } else {
            game.renderer.reset_target();
        }
        game.renderer.set_model_matrix(self.model_matrix);
        game.renderer.set_alpha_blending(self.alpha_blending);
        game.renderer.set_blend_mode(self.blend_mode.0, self.blend_mode.1);
        game.renderer.set_pixel_interpolation(self.interpolate_pixels);
        game.renderer.set_texture_repeat(self.texture_repeat);
        game.renderer.set_sprite_count(self.sprite_count);
        game.renderer.set_vsync(self.vsync);

        let mut externals = self.externals;
        // we're always gonna be recording if we're loading savestates so disable sound
        game.externals = externals
            .drain(..)
            .map(|i| {
                i.map(|i| {
                    External::new(i, true, match game.gm_version {
                        Version::GameMaker8_0 => game.encoding,
                        Version::GameMaker8_1 => encoding_rs::UTF_8,
                    })
                    .unwrap()
                })
            })
            .collect();

        game.surface_fix = self.surface_fix;

        game.compiler = self.compiler;
        game.rand = self.rand;
        game.input_manager = self.input_manager;
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
        game.renderer.set_3d(self.using_3d);
        game.renderer.set_depth(self.depth);
        game.renderer.set_depth_test(self.depth_test);
        game.renderer.set_write_depth(self.write_depth);
        game.renderer.set_culling(self.culling);
        game.renderer.set_perspective(self.perspective);
        game.renderer.set_fog(self.fog);
        game.renderer.set_gouraud(self.gouraud);
        game.surfaces = surfaces;
        game.surface_target = self.surface_target;
        game.models = self.models;
        game.model_matrix_stack = self.model_matrix_stack;
        game.auto_draw = self.auto_draw;
        game.renderer.set_circle_precision(self.circle_precision);
        game.renderer.set_primitive_2d(self.primitive_2d);
        game.renderer.set_primitive_3d(self.primitive_3d);
        game.renderer.set_zbuf_trashed(self.zbuf_trashed);
        game.uninit_fields_are_zero = self.uninit_fields_are_zero;
        game.uninit_args_are_zero = self.uninit_args_are_zero;
        game.potential_step_settings = self.potential_step_settings;
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
        game.scaling = self.scaling;
        game.unscaled_width = self.unscaled_width;
        game.unscaled_height = self.unscaled_height;
        self.replay
    }

    pub fn into_replay(self) -> Replay {
        self.replay
    }
}
