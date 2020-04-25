use crate::{game::{Game, GetAsset}, gml};
use std::{cmp::Ordering, hint::unreachable_unchecked};

pub enum Halign {
    Left,
    Middle,
    Right,
}

pub enum Valign {
    Top,
    Middle,
    Bottom,
}

impl Game {
    /// Draws all instances, tiles and backgrounds to the screen, taking all active views into account.
    /// Note that this function runs GML code associated with object draw events, so its usage must match GameMaker 8.
    pub fn draw(&mut self) -> gml::Result<()> {
        // Draw all views
        if self.views_enabled {
            // Iter views in a non-borrowing way
            let mut count = 0;
            while let Some(&view) = self.views.get(count) {
                if view.visible {
                    self.view_current = count;
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
            self.view_current = 0;
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

        fn draw_instance(game: &mut Game, idx: usize) -> gml::Result<()> {
            let instance = game.instance_list.get(idx).unwrap_or_else(|| unsafe { unreachable_unchecked() });
            if game.custom_draw_objects.contains(&instance.object_index.get()) {
                // Custom draw event
                game.run_instance_event(gml::ev::DRAW, 0, idx, idx, None)
            } else {
                // Default draw action
                if let Some(Some(sprite)) = game.assets.sprites.get(instance.sprite_index.get() as usize) {
                    let image_index = instance.image_index.get().floor() as i32 % sprite.frames.len() as i32;
                    let atlas_ref = match sprite.frames.get(image_index as usize) {
                        Some(f1) => &f1.atlas_ref,
                        None => return Ok(()), // sprite with 0 frames?
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
                Ok(())
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
                            draw_instance(self, idx_inst)?;
                            iter_inst_v = iter_inst.next(&self.instance_list);
                        },
                        Ordering::Less => {
                            draw_tile(self, idx_tile);
                            iter_tile_v = iter_tile.next(&self.tile_list);
                        },
                    }
                },
                (Some(idx_inst), None) => {
                    draw_instance(self, idx_inst)?;
                    while let Some(idx_inst) = iter_inst.next(&self.instance_list) {
                        draw_instance(self, idx_inst)?;
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
}