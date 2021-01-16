use crate::{
    game::{string::RCStr, Background, Game, View},
    instance::DummyFieldHolder,
    instancelist::{InstanceList, TileList},
};
use serde::{Deserialize, Serialize};
use shared::types::Colour;
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
    caption: RCStr,
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
            room_id: game.room_id,
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
                caption: game.caption.clone(),
                width: game.room_width,
                height: game.room_height,
                room_speed: game.room_speed,
                persistent: false, // TODO
                bgcol: game.room_colour,
                show_bgcol: game.show_room_colour,
                show_windowcol: true, // TODO
                backgrounds: game.backgrounds.clone(),
                views_enabled: game.views_enabled,
                views: game.views.clone(),
                instances: game.instance_list.clone(),
                tiles: game.tile_list.clone(),
            },
            last_instance_id: game.last_instance_id,
            last_tile_id: game.last_tile_id,
        }
    }

    pub fn into_game(self, game: &mut Game) -> Result<(), String> {
        if self.game_id != game.game_id {
            return Err("tried to load save file for different game".into())
        }

        game.room_id = self.room_id;
        game.transition_kind = self.transition_kind;
        game.score = self.score;
        game.lives = self.lives;
        game.health = self.health.into();
        game.cursor_sprite = self.cursor_sprite;
        game.cursor_sprite_frame = self.cursor_sprite_frame;
        game.auto_draw = self.auto_draw;
        game.globalvars = self.globalvars;
        game.globals = self.globals;
        game.caption = self.room.caption;
        game.room_width = self.room.width;
        game.room_height = self.room.height;
        game.room_speed = self.room.room_speed;
        game.room_colour = self.room.bgcol;
        game.show_room_colour = self.room.show_bgcol;
        game.backgrounds = self.room.backgrounds;
        game.views_enabled = self.room.views_enabled;
        game.views = self.room.views;
        game.instance_list = self.room.instances;
        game.tile_list = self.room.tiles;
        game.last_instance_id = self.last_instance_id;
        game.last_tile_id = self.last_tile_id;

        // Update renderer
        let (view_width, view_height) = {
            if !game.views_enabled {
                (game.room_width as u32, game.room_height as u32)
            } else {
                let xw = |view: &View| view.port_x + (view.port_w as i32);
                let yh = |view: &View| view.port_y + (view.port_h as i32);
                let x_max = game
                    .views
                    .iter()
                    .filter(|view| view.visible)
                    .max_by(|v1, v2| xw(v1).cmp(&xw(v2)))
                    .map(xw)
                    .unwrap_or(game.room_width as i32);
                let y_max = game
                    .views
                    .iter()
                    .filter(|view| view.visible)
                    .max_by(|v1, v2| yh(v1).cmp(&yh(v2)))
                    .map(yh)
                    .unwrap_or(game.room_height as i32);
                if x_max < 0 || y_max < 0 {
                    return Err(format!(
                        "Bad room width/height {},{} loading room {} from state",
                        x_max, y_max, game.room_id
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
