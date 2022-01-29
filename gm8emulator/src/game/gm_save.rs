use crate::{
    game::{Background, Game, View},
    gml,
    instance::DummyFieldHolder,
    instancelist::{InstanceList, TileList},
    types::Colour,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// A save file for use with game_save() and game_load().
/// The manual explicitly recommends against using save files between sessions, so this may be acceptable.
#[derive(Serialize, Deserialize)]
pub struct GMSave {
    pub game_id: i32,
    room_id: i32,
    transition_kind: i32,
    score: i32,
    lives: i32,
    health: f64,
    cursor_sprite: i32,
    cursor_sprite_frame: u32,
    auto_draw: bool,
    globalvars: HashSet<usize>,
    globals: DummyFieldHolder,
    // TODO persistent stuff
    room: GMRoomSave,
    last_instance_id: i32,
    last_tile_id: i32,
}

#[derive(Serialize, Deserialize)]
struct GMRoomSave {
    caption: gml::String,
    width: i32,
    height: i32,
    room_speed: u32,
    persistent: bool,
    bgcol: Colour,
    show_bgcol: bool,
    show_windowcol: bool,
    // TODO room creation code
    backgrounds: Vec<Background>,
    views_enabled: bool,
    views: Vec<View>,
    instances: InstanceList,
    tiles: TileList,
}

impl GMSave {
    pub fn from_game(game: &Game) -> Self {
        Self {
            game_id: game.game_id,
            room_id: game.room.id,
            transition_kind: game.transition_kind,
            score: game.score,
            lives: game.lives,
            health: game.health.into(),
            cursor_sprite: game.cursor_sprite,
            cursor_sprite_frame: game.cursor_sprite_frame,
            auto_draw: game.auto_draw,
            globalvars: game.globalvars.clone(),
            globals: game.globals.clone(),
            room: GMRoomSave {
                caption: game.room.caption.clone(),
                width: game.room.width,
                height: game.room.height,
                room_speed: game.room.speed,
                persistent: game.room.persistent,
                bgcol: game.room.colour,
                show_bgcol: game.room.show_colour,
                show_windowcol: true, // TODO:
                backgrounds: game.room.backgrounds.clone(),
                views_enabled: game.room.views_enabled,
                views: game.room.views.clone(),
                instances: game.room.instance_list.clone(),
                tiles: game.room.tile_list.clone(),
            },
            last_instance_id: game.last_instance_id,
            last_tile_id: game.last_tile_id,
        }
    }

    pub fn into_game(self, game: &mut Game) -> Result<(), String> {
        if self.game_id != game.game_id {
            return Err("tried to load save file for different game".into())
        }

        game.room.id = self.room_id;
        game.transition_kind = self.transition_kind;
        game.score = self.score;
        game.lives = self.lives;
        game.health = self.health.into();
        game.cursor_sprite = self.cursor_sprite;
        game.cursor_sprite_frame = self.cursor_sprite_frame;
        game.auto_draw = self.auto_draw;
        game.globalvars = self.globalvars;
        game.globals = self.globals;
        game.room.caption = self.room.caption;
        game.room.width = self.room.width;
        game.room.height = self.room.height;
        game.room.speed = self.room.room_speed;
        game.room.colour = self.room.bgcol;
        game.room.show_colour = self.room.show_bgcol;
        game.room.backgrounds = self.room.backgrounds;
        game.room.views_enabled = self.room.views_enabled;
        game.room.views = self.room.views;
        game.room.instance_list = self.room.instances;
        game.room.tile_list = self.room.tiles;
        game.last_instance_id = self.last_instance_id;
        game.last_tile_id = self.last_tile_id;

        // Update renderer
        let (view_width, view_height) = {
            if !game.room.views_enabled {
                (game.room.width as u32, game.room.height as u32)
            } else {
                let xw = |view: &View| view.port_x + (view.port_w as i32);
                let yh = |view: &View| view.port_y + (view.port_h as i32);
                let x_max = game
                    .room
                    .views
                    .iter()
                    .filter(|view| view.visible)
                    .max_by(|v1, v2| xw(v1).cmp(&xw(v2)))
                    .map(xw)
                    .unwrap_or(game.room.width as i32);
                let y_max = game
                    .room
                    .views
                    .iter()
                    .filter(|view| view.visible)
                    .max_by(|v1, v2| yh(v1).cmp(&yh(v2)))
                    .map(yh)
                    .unwrap_or(game.room.height as i32);
                if x_max < 0 || y_max < 0 {
                    return Err(format!(
                        "Bad room width/height {},{} loading room {} from state",
                        x_max, y_max, game.room.id
                    )
                    .into())
                }
                (x_max as u32, y_max as u32)
            }
        };
        game.resize_window(view_width, view_height);

        Ok(())
    }
}
