use crate::{
    game::{string::RCStr, Game, GetAsset},
    gml::{self, Context},
    instance::Instance,
    math::Real,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct UserTransition {
    pub script_name: RCStr,
}

fn wipe(game: &mut Game, surf_old: i32, surf_new: i32, width: i32, height: i32, progress: Real, horz: i32, vert: i32) {
    let surf_old = game.surfaces.get_asset(surf_old).unwrap();
    let surf_new = game.surfaces.get_asset(surf_new).unwrap();

    game.renderer.draw_sprite(&surf_old.atlas_ref, 0.0, 0.0, 1.0, 1.0, 0.0, 0xffffff, 1.0);

    let left = if horz < 0 { width - (progress * width.into()).round() } else { 0 };
    let right = if horz > 0 { (progress * width.into()).round() } else { width };
    let top = if vert < 0 { height - (progress * height.into()).round() } else { 0 };
    let bottom = if vert > 0 { (progress * height.into()).round() } else { height };
    game.renderer.draw_sprite_partial(
        &surf_new.atlas_ref,
        left,
        top,
        right - left,
        bottom - top,
        left.into(),
        top.into(),
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

    game.renderer.draw_sprite(&surf_old.atlas_ref, 0.0, 0.0, 1.0, 1.0, 0.0, 0xffffff, 1.0);

    let x = progress * width.into() * horz.into();
    let y = progress * height.into() * vert.into();

    game.renderer.draw_sprite(&surf_new.atlas_ref, x.into(), y.into(), 1.0, 1.0, 0.0, 0xffffff, 1.0);
}

impl Game {
    pub fn get_transition(
        &self,
        transition_id: i32,
    ) -> Option<Box<dyn Fn(&mut Game, i32, i32, i32, i32, Real) -> gml::Result<()>>> {
        // TODO custom transitions
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
                            .instance_list
                            .insert_dummy(Instance::new_dummy(game.assets.objects.get_asset(0).map(|x| x.as_ref())));
                        game.execute(&instructions, &mut Context {
                            this: dummy_instance,
                            other: dummy_instance,
                            event_action: 0,
                            relative: false,
                            event_type: 11,
                            event_number: 0,
                            event_object: 0,
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
                            locals: Default::default(),
                            return_value: Default::default(),
                        })?;
                        game.instance_list.remove_dummy(dummy_instance);
                    }
                } else {
                    match transition_id {
                        1 => wipe(game, surf_old, surf_new, width, height, progress, 1, 0),
                        2 => wipe(game, surf_old, surf_new, width, height, progress, -1, 0),
                        3 => wipe(game, surf_old, surf_new, width, height, progress, 0, 1),
                        4 => wipe(game, surf_old, surf_new, width, height, progress, 0, -1),
                        6 => slide(game, surf_old, surf_new, width, height, progress, 1, 0),
                        7 => slide(game, surf_old, surf_new, width, height, progress, -1, 0),
                        8 => slide(game, surf_old, surf_new, width, height, progress, 0, 1),
                        9 => slide(game, surf_old, surf_new, width, height, progress, 0, -1),
                        _ => (), // TODO
                    };
                }
                Ok(())
            }))
        } else {
            None
        }
    }
}
