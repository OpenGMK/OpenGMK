use crate::{
    asset::font::Font,
    game::{
        background, draw,
        external::{DefineInfo, External},
        particle,
        string::RCStr,
        surface::Surface,
        view::View,
        Assets, Game, Replay, Version,
    },
    gml::{
        ds::{self, DataStructureManager},
        rand::Random,
        Compiler,
    },
    input::InputManager,
    instance::DummyFieldHolder,
    instancelist::{InstanceList, TileList},
    math::Real,
};
use gmio::render::{BlendType, SavedTexture};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use shared::types::{Colour, ID};
use std::{cell::RefCell, collections::HashSet, rc::Rc};

/// Represents a savestate. Very similar to the Game struct, but without things which aren't serialized.
#[derive(Clone, Serialize, Deserialize)]
pub struct SaveState {
    pub compiler: Compiler,
    pub instance_list: InstanceList,
    pub tile_list: TileList,
    pub rand: Random,
    pub input_manager: InputManager,
    pub assets: Assets,
    pub event_holders: [IndexMap<u32, Rc<RefCell<Vec<ID>>>>; 12],
    pub custom_draw_objects: HashSet<ID>,

    pub background_colour: Colour,
    pub room_colour: Colour,
    pub show_room_colour: bool,
    pub textures: Vec<Option<SavedTexture>>,
    pub blend_mode: (BlendType, BlendType),
    pub interpolate_pixels: bool,
    pub vsync: bool,

    pub externals: Vec<Option<DefineInfo>>,

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

    pub globals: DummyFieldHolder,
    pub globalvars: HashSet<usize>,
    pub game_start: bool,

    pub stacks: DataStructureManager<ds::Stack>,
    pub queues: DataStructureManager<ds::Queue>,
    pub lists: DataStructureManager<ds::List>,
    pub maps: DataStructureManager<ds::Map>,
    pub priority_queues: DataStructureManager<ds::Priority>,
    pub grids: DataStructureManager<ds::Grid>,
    pub ds_precision: Real,

    pub draw_font: Option<Font>,
    pub draw_font_id: ID,
    pub draw_colour: Colour,
    pub draw_alpha: Real,
    pub draw_halign: draw::Halign,
    pub draw_valign: draw::Valign,
    pub surfaces: Vec<Option<Surface>>,
    pub surface_target: Option<i32>,
    pub auto_draw: bool,

    pub uninit_fields_are_zero: bool,
    pub uninit_args_are_zero: bool,

    pub transition_kind: i32,
    pub transition_steps: i32,
    pub score: i32,
    pub score_capt: RCStr,
    pub score_capt_d: bool,
    pub lives: i32,
    pub lives_capt: RCStr,
    pub lives_capt_d: bool,
    pub health: Real,
    pub health_capt: RCStr,
    pub health_capt_d: bool,

    pub game_id: i32,
    pub program_directory: RCStr,
    pub gm_version: Version,
    pub spoofed_time_nanos: Option<u128>,
    pub caption: RCStr,
    pub caption_stale: bool,

    unscaled_width: u32,
    unscaled_height: u32,

    replay: Replay,
    screenshot: Box<[u8]>,
    screenshot_width: u32,
    screenshot_height: u32,
}

impl SaveState {
    pub fn from(game: &Game, replay: Replay) -> Self {
        let (width, height) = game.window.get_inner_size();
        let screenshot = game.renderer.get_pixels(0, 0, width as _, height as _);

        Self {
            compiler: game.compiler.clone(),
            instance_list: game.instance_list.clone(),
            tile_list: game.tile_list.clone(),
            rand: game.rand.clone(),
            input_manager: game.input_manager.clone(),
            assets: game.assets.clone(),
            event_holders: game.event_holders.clone(),
            custom_draw_objects: game.custom_draw_objects.clone(),
            background_colour: game.background_colour,
            room_colour: game.room_colour,
            show_room_colour: game.show_room_colour,
            textures: game.renderer.dump_dynamic_textures(),
            blend_mode: game.renderer.get_blend_mode(),
            interpolate_pixels: game.renderer.get_pixel_interpolation(),
            vsync: game.renderer.get_vsync(),
            externals: game.externals.iter().map(|e| e.as_ref().map(|e| e.info.clone())).collect(),
            last_instance_id: game.last_instance_id.clone(),
            last_tile_id: game.last_tile_id.clone(),
            views_enabled: game.views_enabled.clone(),
            view_current: game.view_current.clone(),
            views: game.views.clone(),
            backgrounds: game.backgrounds.clone(),
            particles: game.particles.clone(),
            room_id: game.room_id.clone(),
            room_width: game.room_width.clone(),
            room_height: game.room_height.clone(),
            room_order: game.room_order.clone(),
            room_speed: game.room_speed.clone(),
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
            draw_font: game.draw_font.clone(),
            draw_font_id: game.draw_font_id.clone(),
            draw_colour: game.draw_colour.clone(),
            draw_alpha: game.draw_alpha.clone(),
            draw_halign: game.draw_halign.clone(),
            draw_valign: game.draw_valign.clone(),
            surfaces: game.surfaces.clone(),
            surface_target: game.surface_target,
            auto_draw: game.auto_draw,
            uninit_fields_are_zero: game.uninit_fields_are_zero.clone(),
            uninit_args_are_zero: game.uninit_args_are_zero.clone(),
            transition_kind: game.transition_kind.clone(),
            transition_steps: game.transition_steps.clone(),
            score: game.score.clone(),
            score_capt: game.score_capt.clone(),
            score_capt_d: game.score_capt_d.clone(),
            lives: game.lives.clone(),
            lives_capt: game.lives_capt.clone(),
            lives_capt_d: game.lives_capt_d.clone(),
            health: game.health.clone(),
            health_capt: game.health_capt.clone(),
            health_capt_d: game.health_capt_d.clone(),
            game_id: game.game_id.clone(),
            program_directory: game.program_directory.clone(),
            gm_version: game.gm_version.clone(),
            spoofed_time_nanos: game.spoofed_time_nanos,
            caption: game.caption.clone(),
            caption_stale: game.caption_stale.clone(),
            unscaled_width: game.unscaled_width,
            unscaled_height: game.unscaled_height,
            replay,
            screenshot,
            screenshot_width: width,
            screenshot_height: height,
        }
    }

    pub fn load_into(self, game: &mut Game) -> Replay {
        game.window.resize(self.screenshot_width, self.screenshot_height);

        game.renderer.upload_dynamic_textures(&self.textures);

        if let Some(Some(surf)) = self.surface_target.and_then(|id| self.surfaces.get(id as usize)) {
            game.renderer.set_target(&surf.atlas_ref);
        } else {
            game.renderer.reset_target(
                self.screenshot_width as _,
                self.screenshot_height as _,
                self.unscaled_width as _,
                self.unscaled_height as _,
            );
        }

        game.renderer.draw_raw_frame(
            self.screenshot,
            self.screenshot_width as _,
            self.screenshot_height as _,
            game.background_colour,
        );
        game.renderer.set_blend_mode(self.blend_mode.0, self.blend_mode.1);
        game.renderer.set_pixel_interpolation(self.interpolate_pixels);
        game.renderer.set_vsync(self.vsync);

        let mut externals = self.externals;
        // we're always gonna be recording if we're loading savestates so disable sound
        game.externals = externals.drain(..).map(|i| i.map(|i| External::new(i, true).unwrap())).collect();

        game.compiler = self.compiler;
        game.instance_list = self.instance_list;
        game.tile_list = self.tile_list;
        game.rand = self.rand;
        game.input_manager = self.input_manager;
        game.assets = self.assets;
        game.event_holders = self.event_holders;
        game.custom_draw_objects = self.custom_draw_objects;
        game.background_colour = self.background_colour;
        game.room_colour = self.room_colour;
        game.show_room_colour = self.show_room_colour;
        game.last_instance_id = self.last_instance_id;
        game.last_tile_id = self.last_tile_id;
        game.views_enabled = self.views_enabled;
        game.view_current = self.view_current;
        game.views = self.views;
        game.backgrounds = self.backgrounds;
        game.particles = self.particles;
        game.room_id = self.room_id;
        game.room_width = self.room_width;
        game.room_height = self.room_height;
        game.room_order = self.room_order;
        game.room_speed = self.room_speed;
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
        game.draw_font = self.draw_font;
        game.draw_font_id = self.draw_font_id;
        game.draw_colour = self.draw_colour;
        game.draw_alpha = self.draw_alpha;
        game.draw_halign = self.draw_halign;
        game.draw_valign = self.draw_valign;
        game.surfaces = self.surfaces;
        game.surface_target = self.surface_target;
        game.auto_draw = self.auto_draw;
        game.uninit_fields_are_zero = self.uninit_fields_are_zero;
        game.uninit_args_are_zero = self.uninit_args_are_zero;
        game.transition_kind = self.transition_kind;
        game.transition_steps = self.transition_steps;
        game.score = self.score;
        game.score_capt = self.score_capt;
        game.score_capt_d = self.score_capt_d;
        game.lives = self.lives;
        game.lives_capt = self.lives_capt;
        game.lives_capt_d = self.lives_capt_d;
        game.health = self.health;
        game.health_capt = self.health_capt;
        game.health_capt_d = self.health_capt_d;
        game.game_id = self.game_id;
        game.program_directory = self.program_directory;
        game.gm_version = self.gm_version;
        game.spoofed_time_nanos = self.spoofed_time_nanos;
        game.caption = self.caption;
        game.caption_stale = self.caption_stale;
        game.unscaled_width = self.unscaled_width;
        game.unscaled_height = self.unscaled_height;
        self.replay
    }

    pub fn into_replay(self) -> Replay {
        self.replay
    }
}
