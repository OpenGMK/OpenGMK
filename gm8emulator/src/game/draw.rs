use crate::{
    asset::font,
    game::{Game, GetAsset},
    gml,
    math::Real,
};
use serde::{Deserialize, Serialize};
use std::fmt::Write;

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum Halign {
    Left,
    Middle,
    Right,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum Valign {
    Top,
    Middle,
    Bottom,
}

struct LineIterator<'a> {
    text: &'a str,
    font: font::Font,
    max_width: Option<i32>,
    word_buf: String,
    word_width: i32,
}

impl<'a> LineIterator<'a> {
    fn next(&mut self, font: &font::Font) -> Option<(String, i32)> {
        if self.text.is_empty() {
            return None
        }
        let mut line = String::new();
        let mut line_width = 0;

        let mut iter = self.text.char_indices().peekable();
        while let Some((_, c)) = iter.next() {
            // First, process escape characters
            let c = match c {
                '#' | '\r' | '\n' => {
                    // '#' is a newline character, don't process it but start a new line instead
                    // Likewise CR, LF, and CRLF
                    if c == '\r' && iter.peek().map(|t| t.1) == Some('\n') {
                        // CRLF only counts as one line break so consume the LF
                        iter.next();
                    }
                    '\n'
                },
                '\\' if iter.peek().map(|t| t.1) == Some('#') => {
                    // '\#' is an escaped newline character, treat it as '#'
                    iter.next();
                    '#'
                },
                _ if font.get_char(u32::from(c)).is_some() => c, // Normal character
                _ => ' ',                                        // Character is not in the font, replace with space
            };
            // Next, insert the character into the word buffer
            match c {
                '\n' => {
                    // Newline
                    line.push_str(&self.word_buf);
                    line_width += self.word_width;
                    self.word_buf.clear();
                    self.word_width = 0;
                    break
                },
                _ => {
                    // Normal character
                    if let Some(character) = font.get_char(u32::from(c)) {
                        self.word_buf.push(c);
                        self.word_width += character.offset;
                    } else {
                        // Space when it isn't in the font
                        self.word_buf.push(' ');
                        if let Some(character) = font.get_char(self.font.first) {
                            self.word_width += character.offset;
                        }
                    }
                },
            };

            // Check if we're going over the max width
            if let Some(max_width) = self.max_width {
                if line_width + self.word_width > max_width && line_width != 0 {
                    break
                }
            }

            // Push new word if applicable
            if c == ' ' {
                line.push_str(&self.word_buf);
                line_width += self.word_width;
                self.word_buf.clear();
                self.word_width = 0;
            }
        }

        if let Some((pos, _)) = iter.peek() {
            self.text = &self.text[*pos..];
        } else {
            // Add the last word
            line.push_str(&self.word_buf);
            line_width += self.word_width;
            self.text = "";
        }

        Some((line, line_width))
    }
}

impl Game {
    /// Draws all instances, tiles and backgrounds to the screen, taking all active views into account.
    /// Note that this function runs GML code associated with object draw events, so its usage must match GameMaker 8.
    pub fn draw(&mut self) -> gml::Result<()> {
        // Update views that should be following objects
        if self.views_enabled {
            self.renderer.clear_view(self.background_colour, 1.0);
            for view in self.views.iter_mut().filter(|x| x.visible) {
                if let Some(obj) = self.assets.objects.get_asset(view.follow_target) {
                    if let Some(handle) =
                        self.instance_list.iter_by_identity(obj.children.clone()).next(&self.instance_list)
                    {
                        let inst = self.instance_list.get(handle);

                        let x = inst.x.get().round();
                        let y = inst.y.get().round();
                        if view.follow_hborder < (view.source_w / 2) as i32 {
                            let border_left = x - view.follow_hborder;
                            let border_right = x + view.follow_hborder;
                            if border_left < view.source_x {
                                if view.follow_hspeed < 0 {
                                    view.source_x = border_left;
                                } else {
                                    view.source_x -= (view.source_x - border_left).min(view.follow_hspeed);
                                }
                            } else if border_right > (view.source_x + view.source_w as i32) {
                                if view.follow_hspeed < 0 {
                                    view.source_x = border_right - view.source_w as i32;
                                } else {
                                    view.source_x +=
                                        (border_right - (view.source_x + view.source_w as i32)).min(view.follow_hspeed);
                                }
                            }
                        } else {
                            view.source_x = x - (view.source_w / 2) as i32;
                        }
                        view.source_x = view.source_x.max(0).min(self.room_width - view.source_w as i32);

                        if view.follow_vborder < (view.source_h / 2) as i32 {
                            let border_top = y - view.follow_vborder;
                            let border_bottom = y + view.follow_vborder;
                            if border_top < view.source_y {
                                if view.follow_vspeed < 0 {
                                    view.source_y = border_top;
                                } else {
                                    view.source_y -= (view.source_y - border_top).min(view.follow_vspeed);
                                }
                            } else if border_bottom > (view.source_y + view.source_h as i32) {
                                if view.follow_vspeed < 0 {
                                    view.source_y = border_bottom - view.source_h as i32;
                                } else {
                                    view.source_y += (border_bottom - (view.source_y + view.source_h as i32))
                                        .min(view.follow_vspeed);
                                }
                            }
                        } else {
                            view.source_y = y - (view.source_h / 2) as i32;
                        }
                        view.source_y = view.source_y.max(0).min(self.room_height - view.source_h as i32);
                    }
                }
            }
        }

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
                        view.angle.into(),
                    )?;
                }
                count += 1;
            }
            self.view_current = 0;
        } else {
            self.draw_view(0, 0, self.room_width, self.room_height, 0, 0, self.room_width, self.room_height, 0.0)?;
        }

        // Tell renderer to finish the frame
        let (width, height) = self.window.get_inner_size();
        self.renderer.present(width, height, self.scaling);

        // Reset viewport
        self.renderer.set_view(
            0,
            0,
            self.unscaled_width as _,
            self.unscaled_height as _,
            0.0,
            0,
            0,
            self.unscaled_width as _,
            self.unscaled_height as _,
        );

        // Apply room caption
        if self.score_capt_d || self.lives_capt_d {
            let mut caption = self.decode_str(self.caption.as_ref()).into_owned();
            // write!() on a String never panics
            if self.score_capt_d {
                write!(caption, " {}{}", self.decode_str(self.score_capt.as_ref()), self.score).unwrap();
            }
            if self.lives_capt_d {
                write!(caption, " {}{}", self.decode_str(self.lives_capt.as_ref()), self.lives).unwrap();
            }
            self.window.set_title(&caption);
        } else {
            self.window.set_title(self.decode_str(self.caption.as_ref()).as_ref());
        }

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
        self.renderer.set_view(src_x, src_y, src_w, src_h, angle, port_x, port_y, port_w, port_h);

        if self.show_room_colour {
            self.renderer.clear_view(self.room_colour, 1.0);
        } else {
            self.renderer.clear_zbuf();
        }

        fn draw_instance(game: &mut Game, idx: usize) -> gml::Result<()> {
            let instance = game.instance_list.get(idx);
            if instance.visible.get() {
                game.renderer.set_depth(instance.depth.get().into_inner() as f32);
                if game.custom_draw_objects.contains(&instance.object_index.get()) {
                    // Custom draw event
                    game.run_instance_event(gml::ev::DRAW, 0, idx, idx, None)
                } else {
                    // Default draw action
                    if let Some(Some(sprite)) = game.assets.sprites.get(instance.sprite_index.get() as usize) {
                        let image_index = (instance.image_index.get().floor().into_inner() as i32)
                            .rem_euclid(sprite.frames.len() as i32);
                        let atlas_ref = match sprite.frames.get(image_index as usize) {
                            Some(f1) => &f1.atlas_ref,
                            None => return Ok(()), // sprite with 0 frames?
                        };
                        game.renderer.draw_sprite(
                            atlas_ref,
                            instance.x.get().into(),
                            instance.y.get().into(),
                            instance.image_xscale.get().into(),
                            instance.image_yscale.get().into(),
                            instance.image_angle.get().into(),
                            instance.image_blend.get(),
                            instance.image_alpha.get().into(),
                        )
                    }
                    Ok(())
                }
            } else {
                Ok(())
            }
        }

        fn draw_tile(game: &mut Game, idx: usize) {
            let tile = game.tile_list.get(idx);
            if let Some(Some(background)) = game.assets.backgrounds.get(tile.background_index.get() as usize) {
                if let Some(atlas) = &background.atlas_ref {
                    game.renderer.set_depth(tile.depth.get().into_inner() as f32);
                    game.renderer.draw_sprite_partial(
                        atlas,
                        tile.tile_x.get().into(),
                        tile.tile_y.get().into(),
                        tile.width.get().into(),
                        tile.height.get().into(),
                        tile.x.get().into(),
                        tile.y.get().into(),
                        tile.xscale.get().into(),
                        tile.yscale.get().into(),
                        0.0,
                        tile.blend.get(),
                        tile.alpha.get().into(),
                    )
                }
            }
        }

        fn draw_part_syst(game: &mut Game, id: i32) {
            game.particles.draw_system(id, &mut game.renderer, &game.assets, true);
        }

        // draw backgrounds
        self.renderer.set_depth(12000.0);
        for background in self.backgrounds.iter().filter(|x| x.visible && !x.is_foreground) {
            if let Some(bg_asset) = self.assets.backgrounds.get_asset(background.background_id) {
                if let Some(atlas_ref) = bg_asset.atlas_ref.as_ref() {
                    self.renderer.draw_sprite_tiled(
                        atlas_ref,
                        background.x_offset.into(),
                        background.y_offset.into(),
                        background.xscale.into(),
                        background.yscale.into(),
                        background.blend,
                        background.alpha.into(),
                        if background.tile_horizontal { Some((src_x + src_w).into()) } else { None },
                        if background.tile_vertical { Some((src_y + src_h).into()) } else { None },
                    );
                }
            }
        }

        self.instance_list.draw_sort();
        let mut iter_inst = self.instance_list.iter_by_drawing();
        let mut iter_inst_v = iter_inst.next(&self.instance_list);
        self.tile_list.draw_sort();
        let mut iter_tile = self.tile_list.iter_by_drawing();
        let mut iter_tile_v = iter_tile.next(&self.tile_list);
        self.particles.draw_sort();
        let mut iter_part = self.particles.iter_by_drawing();
        let mut iter_part_v = iter_part.next(&self.particles);
        loop {
            match (iter_inst_v, iter_tile_v, iter_part_v) {
                (None, None, None) => break,
                (Some(idx_inst), None, None) => {
                    draw_instance(self, idx_inst)?;
                    while let Some(idx_inst) = iter_inst.next(&self.instance_list) {
                        draw_instance(self, idx_inst)?;
                    }
                    break
                },
                (None, Some(idx_tile), None) => {
                    draw_tile(self, idx_tile);
                    while let Some(idx_tile) = iter_tile.next(&self.tile_list) {
                        draw_tile(self, idx_tile);
                    }
                    break
                },
                (None, None, Some(idx_part)) => {
                    draw_part_syst(self, idx_part);
                    while let Some(idx_part) = iter_part.next(&self.particles) {
                        draw_part_syst(self, idx_part);
                    }
                    break
                },
                (idx_opt_inst, idx_opt_tile, idx_opt_part) => {
                    let inst_depth = idx_opt_inst.map(|h| self.instance_list.get(h).depth.get());
                    let tile_depth = idx_opt_tile.map(|h| self.tile_list.get(h).depth.get());
                    let part_depth = idx_opt_part.map(|h| self.particles.get_system(h).unwrap().depth);
                    if part_depth < inst_depth && part_depth < tile_depth {
                        if inst_depth < tile_depth {
                            draw_tile(self, idx_opt_tile.unwrap());
                            iter_tile_v = iter_tile.next(&self.tile_list);
                        } else {
                            draw_instance(self, idx_opt_inst.unwrap())?;
                            iter_inst_v = iter_inst.next(&self.instance_list);
                        }
                    } else {
                        draw_part_syst(self, idx_opt_part.unwrap());
                        iter_part_v = iter_part.next(&self.particles);
                    }
                },
            }
        }

        // draw foregrounds
        self.renderer.set_depth(-12000.0);
        for background in self.backgrounds.clone().iter().filter(|x| x.visible && x.is_foreground) {
            if let Some(bg_asset) = self.assets.backgrounds.get_asset(background.background_id) {
                if let Some(atlas_ref) = bg_asset.atlas_ref.as_ref() {
                    self.renderer.draw_sprite_tiled(
                        atlas_ref,
                        background.x_offset.into(),
                        background.y_offset.into(),
                        background.xscale.into(),
                        background.yscale.into(),
                        background.blend,
                        background.alpha.into(),
                        if background.tile_horizontal { Some((src_x + src_w).into()) } else { None },
                        if background.tile_vertical { Some((src_y + src_h).into()) } else { None },
                    );
                }
            }
        }

        self.renderer.set_depth(-13000.0);
        if let Some(sprite) = self.assets.sprites.get_asset(self.cursor_sprite) {
            let (x, y) = self.get_mouse_in_room();
            self.renderer.draw_sprite(
                &sprite.frames[self.cursor_sprite_frame as usize % sprite.frames.len()].atlas_ref,
                x.into(),
                y.into(),
                1.0,
                1.0,
                0.0,
                0xffffff,
                1.0,
            );
        }

        Ok(())
    }

    /// Splits the string into line-width pairs.
    fn split_string<'a>(&self, string: &'a str, max_width: Option<i32>) -> LineIterator<'a> {
        let font = self.draw_font.as_ref().unwrap();
        LineIterator { text: string, font: font.clone(), max_width, word_buf: String::new(), word_width: 0 }
    }

    /// Gets width and height of a string using the current draw_font.
    /// If line_height is None, a line height will be inferred from the font.
    /// If max_width is None, the string will not be given a maximum width.
    pub fn get_string_size(&self, string: &str, line_height: Option<i32>, max_width: Option<i32>) -> (i32, i32) {
        let font = self.draw_font.as_ref().unwrap();

        // Figure out what the height of a line is if one wasn't specified
        let line_height = match line_height {
            Some(h) => h,
            None => font.tallest_char_height as i32,
        };

        let mut width = 0;
        let mut line_count = 0;
        let mut iter = self.split_string(string, max_width);
        while let Some((_, current_w)) = iter.next(&font) {
            if width < current_w {
                width = current_w;
            }
            line_count += 1;
        }

        (width, line_count * line_height)
    }

    /// Draws a string to the screen at the given coordinates.
    /// If line_height is None, a line height will be inferred from the font.
    /// If max_width is None, the string will not be given a maximum width.
    pub fn draw_string(
        &mut self,
        x: Real,
        y: Real,
        string: &str,
        line_height: Option<i32>,
        max_width: Option<i32>,
        xscale: Real,
        yscale: Real,
        angle: Real,
    ) {
        let font = self.draw_font.as_ref().unwrap();

        let sin = angle.to_radians().sin();
        let cos = angle.to_radians().cos();

        // Figure out what the height of a line is if one wasn't specified
        let line_height = match line_height {
            Some(h) => h,
            None => font.tallest_char_height as i32,
        };

        let mut cursor_y = match self.draw_valign {
            Valign::Top => 0,
            Valign::Middle => -(self.get_string_size(string, Some(line_height), max_width).1 / 2),
            Valign::Bottom => -self.get_string_size(string, Some(line_height), max_width).1,
        };

        let mut iter = self.split_string(string, max_width);
        while let Some((line, width)) = iter.next(&font) {
            let mut cursor_x = match self.draw_halign {
                Halign::Left => 0,
                Halign::Middle => -(width as i32 / 2),
                Halign::Right => -width as i32,
            };

            for c in line.chars() {
                let character = match font.get_char(u32::from(c)) {
                    Some(character) => character,
                    None => {
                        // Space if it isn't in the font
                        if let Some(character) = font.get_char(font.first) {
                            cursor_x += character.offset as i32;
                        }
                        continue
                    },
                };

                let xdiff = Real::from(character.distance as i32 + cursor_x);
                let ydiff = Real::from(cursor_y);

                let (xdiff, ydiff) =
                    (xdiff * xscale * cos + ydiff * yscale * sin, ydiff * yscale * cos - xdiff * xscale * sin);

                self.renderer.draw_sprite(
                    &character.atlas_ref,
                    (x + xdiff).into(),
                    (y + ydiff).into(),
                    xscale.into(),
                    yscale.into(),
                    angle.into(),
                    u32::from(self.draw_colour) as i32,
                    self.draw_alpha.into(),
                );

                cursor_x += character.offset as i32;
            }

            cursor_y += line_height;
        }
    }
}
