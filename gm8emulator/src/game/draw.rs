use crate::{
    asset::{self, font, Font},
    game::{Game, GetAsset, PlayType, Version},
    gml,
    math::Real,
};
use serde::{Deserialize, Serialize};

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
    text: Vec<u8>,
    pos: usize,
    font: &'a font::Font,
    max_width: Option<i32>,
    word_buf: Vec<u8>,
    word_width: i32,
}

impl<'a> LineIterator<'a> {
    fn next(&mut self) -> Option<(Vec<u8>, i32)> {
        if self.pos >= self.text.len() {
            return None
        }
        let mut line = Vec::new();
        let mut line_width = 0;

        let mut iter = self.text[self.pos..].iter().copied().enumerate().peekable();
        while let Some((_, c)) = iter.next() {
            // First, process escape characters
            let c = match c {
                b'#' | b'\r' | b'\n' => {
                    // '#' is a newline character, don't process it but start a new line instead
                    // Likewise CR, LF, and CRLF
                    if c == b'\r' && iter.peek().map(|t| t.1) == Some(b'\n') {
                        // CRLF only counts as one line break so consume the LF
                        iter.next();
                    }
                    b'\n'
                },
                b'\\' if iter.peek().map(|t| t.1) == Some(b'#') => {
                    // '\#' is an escaped newline character, treat it as '#'
                    iter.next();
                    b'#'
                },
                _ if self.font.get_char(c).is_some() => c, // Normal character
                _ => b' ',                                 // Character is not in the font, replace with space
            };
            // Next, insert the character into the word buffer
            match c {
                b'\n' => {
                    // Newline
                    line.extend_from_slice(&self.word_buf);
                    line_width += self.word_width;
                    self.word_buf.clear();
                    self.word_width = 0;
                    break
                },
                _ => {
                    // Normal character
                    if let Some(character) = self.font.get_char(c) {
                        self.word_buf.push(c);
                        self.word_width += character.offset;
                    } else {
                        // Space when it isn't in the font
                        self.word_buf.push(b' ');
                        if let Some(character) = self.font.get_char(self.font.first) {
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
            if c == b' ' {
                line.extend_from_slice(&self.word_buf);
                line_width += self.word_width;
                self.word_buf.clear();
                self.word_width = 0;
            }
        }

        if let Some((pos, _)) = iter.peek() {
            self.pos += pos;
        } else {
            // Add the last word
            line.extend_from_slice(&self.word_buf);
            line_width += self.word_width;
            self.pos = self.text.len();
        }

        Some((line, line_width))
    }
}

impl Game {
    /// Draws all instances, tiles and backgrounds to the screen, taking all active views into account.
    /// Note that this function runs GML code associated with object draw events, so its usage must match GameMaker 8.
    pub fn draw(&mut self) -> gml::Result<()> {
        // Update views that should be following objects
        if self.room.views_enabled {
            self.renderer.clear_view(self.background_colour, 1.0);
            for view in self.room.views.iter_mut().filter(|x| x.visible) {
                if let Some(handle) = match view.follow_target {
                    obj_id if obj_id < 100000 => {
                        self.room.instance_list.iter_by_identity(view.follow_target).next(&self.room.instance_list)
                    },
                    inst_id => self.room.instance_list.get_by_instid(inst_id),
                } {
                    let inst = self.room.instance_list.get(handle);

                    let x = inst.x.get().round().to_i32();
                    let y = inst.y.get().round().to_i32();
                    if view.follow_hborder < view.source_w / 2 {
                        let border_left = x - view.follow_hborder;
                        let border_right = x + view.follow_hborder;
                        if border_left < view.source_x {
                            if view.follow_hspeed < 0 {
                                view.source_x = border_left;
                            } else {
                                view.source_x -= (view.source_x - border_left).min(view.follow_hspeed);
                            }
                        } else if border_right > view.source_x + view.source_w {
                            if view.follow_hspeed < 0 {
                                view.source_x = border_right - view.source_w;
                            } else {
                                view.source_x +=
                                    (border_right - (view.source_x + view.source_w)).min(view.follow_hspeed);
                            }
                        }
                    } else {
                        view.source_x = x - view.source_w / 2;
                    }
                    view.source_x = view.source_x.max(0).min(self.room.width - view.source_w);

                    if view.follow_vborder < view.source_h / 2 {
                        let border_top = y - view.follow_vborder;
                        let border_bottom = y + view.follow_vborder;
                        if border_top < view.source_y {
                            if view.follow_vspeed < 0 {
                                view.source_y = border_top;
                            } else {
                                view.source_y -= (view.source_y - border_top).min(view.follow_vspeed);
                            }
                        } else if border_bottom > view.source_y + view.source_h {
                            if view.follow_vspeed < 0 {
                                view.source_y = border_bottom - view.source_h;
                            } else {
                                view.source_y +=
                                    (border_bottom - (view.source_y + view.source_h)).min(view.follow_vspeed);
                            }
                        }
                    } else {
                        view.source_y = y - view.source_h / 2;
                    }
                    view.source_y = view.source_y.max(0).min(self.room.height - view.source_h);
                }
            }
        }

        // Draw all views
        if self.room.views_enabled {
            // Iter views in a non-borrowing way
            let mut count = 0;
            while let Some(&view) = self.room.views.get(count) {
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
            self.draw_view(0, 0, self.room.width, self.room.height, 0, 0, self.room.width, self.room.height, 0.0)?;
        }

        // Tell renderer to finish the frame
        if self.play_type != PlayType::Record {
            self.renderer.present(self.window_inner_size.0, self.window_inner_size.1, self.scaling);
        }

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
        let title = self.get_window_title();
        if self.play_type != PlayType::Record {
            self.window.set_title(title.as_ref());
        }

        Ok(())
    }

    pub fn draw_instance_default(&mut self, idx: usize) -> gml::Result<()> {
        let instance = self.room.instance_list.get(idx);
        if let Some(sprite) = self.assets.sprites.get_asset(instance.sprite_index.get()) {
            if let Some(atlas_ref) = sprite.get_atlas_ref(instance.image_index.get().floor().to_i32()) {
                self.renderer.draw_sprite(
                    atlas_ref,
                    instance.x.get().into(),
                    instance.y.get().into(),
                    instance.image_xscale.get().into(),
                    instance.image_yscale.get().into(),
                    instance.image_angle.get().into(),
                    instance.image_blend.get(),
                    instance.image_alpha.get().into(),
                );
            }
            Ok(())
        } else {
            Err(gml::Error::NonexistentAsset(asset::Type::Sprite, instance.sprite_index.get()))
        }
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

        if self.room.show_colour {
            self.renderer.clear_view(self.room.colour, 1.0);
        } else {
            self.renderer.clear_zbuf();
        }

        fn draw_instance(game: &mut Game, idx: usize) -> gml::Result<()> {
            let instance = game.room.instance_list.get(idx);
            if instance.visible.get() {
                game.renderer.set_depth(instance.depth.get().into_inner() as f32);
                if game.custom_draw_objects.contains(&instance.object_index.get()) {
                    // Custom draw event
                    game.run_instance_event(gml::ev::DRAW, 0, idx, idx, None)
                } else {
                    // Default draw action
                    let _ = game.draw_instance_default(idx);
                    Ok(())
                }
            } else {
                Ok(())
            }
        }

        fn draw_tile(game: &mut Game, idx: usize) {
            let tile = game.room.tile_list.get(idx);
            if tile.visible.get() {
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
        }

        fn draw_part_syst(game: &mut Game, id: i32) {
            game.particles.draw_system(id, &mut game.renderer, &game.assets, true);
        }

        // draw backgrounds
        self.renderer.set_depth(12000.0);
        for background in self.room.backgrounds.iter().filter(|x| x.visible && !x.is_foreground) {
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

        self.room.instance_list.draw_sort();
        let mut iter_inst = self.room.instance_list.iter_by_drawing();
        let mut iter_inst_v = iter_inst.next(&self.room.instance_list);
        self.room.tile_list.draw_sort();
        let mut iter_tile = self.room.tile_list.iter_by_drawing();
        let mut iter_tile_v = iter_tile.next(&self.room.tile_list);
        self.particles.draw_sort();
        let mut iter_part = self.particles.iter_by_drawing();
        let mut iter_part_v = iter_part.next(&self.particles);
        loop {
            match (iter_inst_v, iter_tile_v, iter_part_v) {
                (None, None, None) => break,
                (Some(idx_inst), None, None) => {
                    draw_instance(self, idx_inst)?;
                    while let Some(idx_inst) = iter_inst.next(&self.room.instance_list) {
                        draw_instance(self, idx_inst)?;
                    }
                    break
                },
                (None, Some(idx_tile), None) => {
                    draw_tile(self, idx_tile);
                    while let Some(idx_tile) = iter_tile.next(&self.room.tile_list) {
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
                    let inst_depth = idx_opt_inst.map(|h| self.room.instance_list.get(h).depth.get());
                    let tile_depth = idx_opt_tile.map(|h| self.room.tile_list.get(h).depth.get());
                    let part_depth = idx_opt_part.map(|h| self.particles.get_system(h).unwrap().depth);
                    if part_depth < inst_depth && part_depth < tile_depth {
                        if inst_depth < tile_depth {
                            draw_tile(self, idx_opt_tile.unwrap());
                            iter_tile_v = iter_tile.next(&self.room.tile_list);
                        } else {
                            draw_instance(self, idx_opt_inst.unwrap())?;
                            iter_inst_v = iter_inst.next(&self.room.instance_list);
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
        for background in self.room.backgrounds.clone().iter().filter(|x| x.visible && x.is_foreground) {
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
            if let Some(atlas_ref) =
                sprite.get_atlas_ref((self.cursor_sprite_frame % sprite.frames.len() as u32) as i32)
            {
                self.renderer.draw_sprite(atlas_ref, x.into(), y.into(), 1.0, 1.0, 0.0, 0xffffff, 1.0);
            }
        }

        Ok(())
    }

    /// Splits the string into line-width pairs.
    fn split_string<'a>(&self, string: gml::String, max_width: Option<i32>, font: &'a Font) -> LineIterator<'a> {
        let encoded_text = match self.gm_version {
            Version::GameMaker8_0 => string.as_ref().to_vec(),
            Version::GameMaker8_1 => {
                let encoding = font.get_encoding(self.encoding);
                let string = string.decode_utf8();
                let (encoded_text, _, bad) = encoding.encode(string.as_ref());
                if bad {
                    // semi-custom decoder because bad characters must be replaced with ? instead of html codes
                    // and unfortunately encoding_rs doesn't have a nicer way of handling this
                    let mut encoded_text = Vec::with_capacity(encoded_text.as_ref().len());
                    let mut iter = string.char_indices().peekable();
                    while let Some((pos, _)) = iter.next() {
                        let slice = iter.peek().map(|(p, _)| &string[pos..*p]).unwrap_or(&string[pos..]);
                        let (encoded_char, _, bad) = encoding.encode(slice);
                        if bad {
                            encoded_text.push(b'?');
                        } else {
                            encoded_text.extend_from_slice(encoded_char.as_ref());
                        }
                    }
                    encoded_text
                } else {
                    encoded_text.into_owned()
                }
            },
        };
        LineIterator { text: encoded_text, pos: 0, font, max_width, word_buf: Vec::new(), word_width: 0 }
    }

    /// Gets width and height of a string using the current draw_font.
    /// If line_height is None, a line height will be inferred from the font.
    /// If max_width is None, the string will not be given a maximum width.
    pub fn get_string_size(&self, string: gml::String, line_height: Option<i32>, max_width: Option<i32>) -> (i32, i32) {
        let font = self.assets.fonts.get_asset(self.draw_font_id).map(|x| x.as_ref()).unwrap_or(&self.default_font);

        // Figure out what the height of a line is if one wasn't specified
        let line_height = match line_height {
            Some(h) => h,
            None => font.tallest_char_height as i32,
        };

        let mut width = 0;
        let mut line_count = 0;
        let mut iter = self.split_string(string, max_width, font);
        while let Some((_, current_w)) = iter.next() {
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
        string: gml::String,
        line_height: Option<i32>,
        max_width: Option<i32>,
        xscale: Real,
        yscale: Real,
        angle: Real,
        colours: Option<(i32, i32, i32, i32)>,
        alpha: Real,
    ) {
        let font = self.assets.fonts.get_asset(self.draw_font_id).map(|x| x.as_ref()).unwrap_or(&self.default_font);

        let sin = angle.to_radians().sin();
        let cos = angle.to_radians().cos();

        // Figure out what the height of a line is if one wasn't specified
        let line_height = match line_height {
            Some(h) => h,
            None => font.tallest_char_height as i32,
        };

        let mut cursor_y = match self.draw_valign {
            Valign::Top => 0,
            Valign::Middle => -(self.get_string_size(string.clone(), Some(line_height), max_width).1 / 2),
            Valign::Bottom => -self.get_string_size(string.clone(), Some(line_height), max_width).1,
        };

        fn lerp_col(c1: i32, c2: i32, ratio: f64) -> i32 {
            ((f64::from(c1 & 0xff) * (1.0 - ratio) + f64::from(c2 & 0xff) * ratio) as i32 & 0xff)
                + ((f64::from(c1 & 0xff00) * (1.0 - ratio) + f64::from(c2 & 0xff00) * ratio) as i32 & 0xff00)
                + ((f64::from(c1 & 0xff0000) * (1.0 - ratio) + f64::from(c2 & 0xff0000) * ratio) as i32 & 0xff0000)
        }

        let mut iter = self.split_string(string, max_width, font);
        while let Some((line, width)) = iter.next() {
            let left_offset = match self.draw_halign {
                Halign::Left => 0,
                Halign::Middle => -(width as i32 / 2),
                Halign::Right => -width as i32,
            };
            let mut cursor_x = left_offset;

            for c in line.iter().copied() {
                let character = match font.get_char(c) {
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

                match colours {
                    Some((c1, c2, c3, c4)) => self.renderer.draw_sprite_colour(
                        &character.atlas_ref,
                        (x + xdiff).into(),
                        (y + ydiff).into(),
                        xscale.into(),
                        yscale.into(),
                        angle.into(),
                        lerp_col(c1, c2, f64::from(cursor_x - left_offset) / f64::from(width)),
                        lerp_col(c1, c2, f64::from(cursor_x - left_offset + character.offset) / f64::from(width)),
                        lerp_col(c4, c3, f64::from(cursor_x - left_offset + character.offset) / f64::from(width)),
                        lerp_col(c4, c3, f64::from(cursor_x - left_offset) / f64::from(width)),
                        alpha.into(),
                    ),
                    None => self.renderer.draw_sprite(
                        &character.atlas_ref,
                        (x + xdiff).into(),
                        (y + ydiff).into(),
                        xscale.into(),
                        yscale.into(),
                        angle.into(),
                        u32::from(self.draw_colour) as i32,
                        alpha.into(),
                    ),
                }

                cursor_x += character.offset as i32;
            }

            cursor_y += line_height;
        }
    }
}
