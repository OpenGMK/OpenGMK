use crate::{
    game::{Game, GetAsset},
    gml::{self, Context},
    instance::Instance,
    math::Real,
};
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

#[derive(Clone, Serialize, Deserialize)]
pub struct UserTransition {
    pub script_name: gml::String,
}

fn wipe(game: &mut Game, surf_old: i32, surf_new: i32, width: i32, height: i32, progress: Real, horz: i32, vert: i32) {
    let surf_old = game.surfaces.get_asset(surf_old).unwrap();
    let surf_new = game.surfaces.get_asset(surf_new).unwrap();

    game.renderer.draw_sprite(surf_old.atlas_ref, 0.0, 0.0, 1.0, 1.0, 0.0, 0xffffff, 1.0);

    let left = if horz < 0 { width - (progress * width.into()).round().to_i32() } else { 0 };
    let right = if horz > 0 { (progress * width.into()).round().to_i32() } else { width };
    let top = if vert < 0 { height - (progress * height.into()).round().to_i32() } else { 0 };
    let bottom = if vert > 0 { (progress * height.into()).round().to_i32() } else { height };
    game.renderer.draw_sprite_partial(
        surf_new.atlas_ref,
        left.into(),
        top.into(),
        (right - left).into(),
        (bottom - top).into(),
        left.into(),
        top.into(),
        1.0,
        1.0,
        0.0,
        0xffffff,
        1.0,
    );
}

fn wipe_from_center(game: &mut Game, surf_old: i32, surf_new: i32, width: i32, height: i32, progress: Real) {
    let surf_old = game.surfaces.get_asset(surf_old).unwrap();
    let surf_new = game.surfaces.get_asset(surf_new).unwrap();

    game.renderer.draw_sprite(surf_old.atlas_ref, 0.0, 0.0, 1.0, 1.0, 0.0, 0xffffff, 1.0);

    let region_width = (progress * width.into()).round().to_i32();
    let region_height = (progress * height.into()).round().to_i32();

    let x = (width - region_width) / 2;
    let y = (height - region_height) / 2;

    game.renderer.draw_sprite_partial(
        surf_new.atlas_ref,
        x.into(),
        y.into(),
        region_width.into(),
        region_height.into(),
        x.into(),
        y.into(),
        1.0,
        1.0,
        0.0,
        0xffffff,
        1.0,
    );
}

fn slide(game: &mut Game, surf_old: i32, surf_new: i32, width: i32, height: i32, progress: Real, horz: i32, vert: i32) {
    let surf_old = game.surfaces.get_asset(surf_old).unwrap();
    let surf_new = game.surfaces.get_asset(surf_new).unwrap();

    game.renderer.draw_sprite(surf_old.atlas_ref, 0.0, 0.0, 1.0, 1.0, 0.0, 0xffffff, 1.0);

    let x = (progress - 1.into()) * width.into() * horz.into();
    let y = (progress - 1.into()) * height.into() * vert.into();

    game.renderer.draw_sprite(surf_new.atlas_ref, x.into(), y.into(), 1.0, 1.0, 0.0, 0xffffff, 1.0);
}

fn interlace(
    game: &mut Game,
    surf_old: i32,
    surf_new: i32,
    width: i32,
    height: i32,
    progress: i32,
    length: i32,
    horz: i32,
    vert: i32,
) {
    let half_length = length / 2;
    if progress > half_length && progress != length {
        wipe(
            game,
            surf_old,
            surf_new,
            width,
            height,
            Real::from(progress - half_length) / half_length.into(),
            horz,
            vert,
        );
    }

    let surf_old = game.surfaces.get_asset(surf_old).unwrap();
    let surf_new = game.surfaces.get_asset(surf_new).unwrap();

    if progress == length {
        game.renderer.draw_sprite(surf_new.atlas_ref, 0.0, 0.0, 1.0, 1.0, 0.0, 0xffffff, 1.0);
        return
    }

    if progress <= half_length {
        game.renderer.draw_sprite(surf_old.atlas_ref, 0.0, 0.0, 1.0, 1.0, 0.0, 0xffffff, 1.0);
    }

    if horz != 0 {
        let part_width = width / length;
        for i in 0..half_length + 1 {
            let can_draw = if horz < 0 { half_length - progress <= i } else { i <= progress };
            if can_draw {
                let x = i * 2 * part_width;
                game.renderer.draw_sprite_partial(
                    surf_new.atlas_ref,
                    x.into(),
                    0.0,
                    part_width.into(),
                    height.into(),
                    x.into(),
                    0.0,
                    1.0,
                    1.0,
                    0.0,
                    0xffffff,
                    1.0,
                );
            }
        }
    }
    if vert != 0 {
        let part_height = height / length;
        for i in 0..half_length + 1 {
            let can_draw = if vert < 0 { half_length - progress <= i } else { i <= progress };
            if can_draw {
                let y = i * 2 * part_height;
                game.renderer.draw_sprite_partial(
                    surf_new.atlas_ref,
                    0.0,
                    y.into(),
                    width.into(),
                    part_height.into(),
                    0.0,
                    y.into(),
                    1.0,
                    1.0,
                    0.0,
                    0xffffff,
                    1.0,
                );
            }
        }
    }
}

fn push(game: &mut Game, surf_old: i32, surf_new: i32, width: i32, height: i32, progress: Real, horz: i32, vert: i32) {
    let surf_old = game.surfaces.get_asset(surf_old).unwrap();
    let surf_new = game.surfaces.get_asset(surf_new).unwrap();

    let old_x = progress * (horz * width).into();
    let old_y = progress * (vert * height).into();
    game.renderer.draw_sprite(surf_old.atlas_ref, old_x.into(), old_y.into(), 1.0, 1.0, 0.0, 0xffffff, 1.0);

    let new_x = old_x - (horz * width).into();
    let new_y = old_y - (vert * height).into();
    game.renderer.draw_sprite(surf_new.atlas_ref, new_x.into(), new_y.into(), 1.0, 1.0, 0.0, 0xffffff, 1.0);
}

fn rotate(game: &mut Game, surf_old: i32, surf_new: i32, width: Real, height: Real, progress: Real, to_right: bool) {
    let surf_old = game.surfaces.get_asset(surf_old).unwrap();
    let surf_new = game.surfaces.get_asset(surf_new).unwrap();

    game.renderer.draw_sprite(surf_old.atlas_ref, 0.0, 0.0, 1.0, 1.0, 0.0, 0xffffff, 1.0);

    let angle = progress.sqrt() * (PI * 2.0).into() * if to_right { -1.0 } else { 1.0 }.into();
    let sin_angle = angle.sin();
    let cos_angle = angle.cos();

    game.renderer.draw_sprite(
        surf_new.atlas_ref,
        (-progress * (cos_angle * width + sin_angle * height) + width).into_inner() / 2.0,
        (progress * (sin_angle * width - cos_angle * height) + height).into_inner() / 2.0,
        progress.into(),
        progress.into(),
        angle.to_degrees().into(),
        0xffffff,
        1.0,
    );
}

fn fade_direct(game: &mut Game, surf_old: i32, surf_new: i32, progress: Real) {
    let surf_old = game.surfaces.get_asset(surf_old).unwrap();
    let surf_new = game.surfaces.get_asset(surf_new).unwrap();

    game.renderer.draw_sprite(surf_old.atlas_ref, 0.0, 0.0, 1.0, 1.0, 0.0, 0xffffff, 1.0);
    game.renderer.draw_sprite(surf_new.atlas_ref, 0.0, 0.0, 1.0, 1.0, 0.0, 0xffffff, progress.into());
}

fn fade_black(game: &mut Game, surf_old: i32, surf_new: i32, progress: Real) {
    let surf_old = game.surfaces.get_asset(surf_old).unwrap();
    let surf_new = game.surfaces.get_asset(surf_new).unwrap();

    game.renderer.clear_view(0.into(), 1.0.into());
    if progress < 0.5.into() {
        game.renderer.draw_sprite(
            surf_old.atlas_ref,
            0.0,
            0.0,
            1.0,
            1.0,
            0.0,
            0xffffff,
            1.0 - progress.into_inner() * 2.0,
        );
    } else {
        game.renderer.draw_sprite(
            surf_new.atlas_ref,
            0.0,
            0.0,
            1.0,
            1.0,
            0.0,
            0xffffff,
            progress.into_inner() * 2.0 - 1.0,
        );
    }
}

impl Game {
    pub fn get_transition(
        &self,
        transition_id: i32,
    ) -> Option<Box<dyn Fn(&mut Game, i32, i32, i32, i32, Real) -> gml::Result<()>>> {
        if (transition_id > 0 && transition_id < 22) || self.user_transitions.contains_key(&transition_id) {
            Some(Box::new(move |game: &mut Game, surf_old, surf_new, width, height, progress| {
                if let Some(transition) = game.user_transitions.get(&transition_id) {
                    if let Some(Some(script)) = game
                        .compiler
                        .get_script_id(transition.script_name.as_ref())
                        .and_then(|id| game.assets.scripts.get(id))
                    {
                        let instructions = script.compiled.clone();
                        let dummy_instance = game
                            .room
                            .instance_list
                            .insert_dummy(Instance::new_dummy(game.assets.objects.get_asset(0).map(|x| x.as_ref())));
                        game.execute(&instructions, &mut Context {
                            this: dummy_instance,
                            other: dummy_instance,
                            arguments: [
                                surf_old.into(),
                                surf_new.into(),
                                width.into(),
                                height.into(),
                                progress.into(),
                                Default::default(),
                                Default::default(),
                                Default::default(),
                                Default::default(),
                                Default::default(),
                                Default::default(),
                                Default::default(),
                                Default::default(),
                                Default::default(),
                                Default::default(),
                                Default::default(),
                            ],
                            argument_count: 5,
                            ..Default::default()
                        })?;
                        game.room.instance_list.remove_dummy(dummy_instance);
                    }
                } else {
                    match transition_id {
                        1 => wipe(game, surf_old, surf_new, width, height, progress, 1, 0),
                        2 => wipe(game, surf_old, surf_new, width, height, progress, -1, 0),
                        3 => wipe(game, surf_old, surf_new, width, height, progress, 0, 1),
                        4 => wipe(game, surf_old, surf_new, width, height, progress, 0, -1),
                        5 => wipe_from_center(game, surf_old, surf_new, width, height, progress),
                        6 => slide(game, surf_old, surf_new, width, height, progress, 1, 0),
                        7 => slide(game, surf_old, surf_new, width, height, progress, -1, 0),
                        8 => slide(game, surf_old, surf_new, width, height, progress, 0, 1),
                        9 => slide(game, surf_old, surf_new, width, height, progress, 0, -1),
                        10 => interlace(
                            game,
                            surf_old,
                            surf_new,
                            width,
                            height,
                            (progress * game.transition_steps.into()).round().to_i32(),
                            game.transition_steps,
                            1,
                            0,
                        ),
                        11 => interlace(
                            game,
                            surf_old,
                            surf_new,
                            width,
                            height,
                            (progress * game.transition_steps.into()).round().to_i32(),
                            game.transition_steps,
                            -1,
                            0,
                        ),
                        12 => interlace(
                            game,
                            surf_old,
                            surf_new,
                            width,
                            height,
                            (progress * game.transition_steps.into()).round().to_i32(),
                            game.transition_steps,
                            0,
                            1,
                        ),
                        13 => interlace(
                            game,
                            surf_old,
                            surf_new,
                            width,
                            height,
                            (progress * game.transition_steps.into()).round().to_i32(),
                            game.transition_steps,
                            0,
                            -1,
                        ),
                        14 => push(game, surf_old, surf_new, width, height, progress, 1, 0),
                        15 => push(game, surf_old, surf_new, width, height, progress, -1, 0),
                        16 => push(game, surf_old, surf_new, width, height, progress, 0, 1),
                        17 => push(game, surf_old, surf_new, width, height, progress, 0, -1),
                        18 => rotate(game, surf_old, surf_new, width.into(), height.into(), progress, false),
                        19 => rotate(game, surf_old, surf_new, width.into(), height.into(), progress, true),
                        20 => fade_direct(game, surf_old, surf_new, progress),
                        21 => fade_black(game, surf_old, surf_new, progress),
                        _ => (), // transition got deleted during transition?
                    };
                }
                Ok(())
            }))
        } else {
            None
        }
    }
}
