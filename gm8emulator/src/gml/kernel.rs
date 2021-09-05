// This file was auto-generated based on a function table dump

#![allow(unused_macros)]

use crate::{
    action, asset,
    game::{
        draw, external, gm_save::GMSave, model, particle, pathfinding, replay, surface::Surface,
        transition::UserTransition, view::View, Game, GetAsset, PlayType, SceneChange, Version,
    },
    gml::{
        self,
        datetime::{self, DateTime},
        ds, file,
        mappings::{self, constants as gml_consts},
        network, Context, Value,
    },
    handleman::HandleManager,
    input::MouseButton,
    instance::{Field, Instance, InstanceState},
    math::Real,
    render::{BlendType, Fog, Light, Renderer, Scaling},
    tile::Tile,
};
use image::RgbaImage;
use ramen::window::Cursor;
use std::{
    convert::TryFrom,
    io::{Read, Write},
    process::Command,
};

macro_rules! _arg_into {
    (any, $v: expr) => {{ Ok($v.clone()) }};
    (bool, $v: expr) => {{ Ok($v.is_truthy()) }};
    (int, $v: expr) => {{ Ok(<Value as Into<i32>>::into($v.clone())) }};
    (real, $v: expr) => {{ Ok(<Value as Into<Real>>::into($v.clone())) }};
    (string, $v: expr) => {{ Ok(String::from_utf8_lossy(<&Value as Into<&[u8]>>::into($v))) }};
    (bytes, $v: expr) => {{ Ok(<Value as Into<gml::String>>::into($v.clone())) }};
}

macro_rules! _count_rep {
    () => { 0usize };
    ($x: ident $($ys: ident)*) => { 1usize + _count_rep!($($ys)*) };
}

include!(concat!(env!("OUT_DIR"), "/_apply_args.macro.rs"));

/// Helper macro to validate input arguments from a GML function.
macro_rules! expect_args {
    ($args: expr, [$($x: ident),*]) => {{
        (|| -> gml::Result<_> {
            let argc = _count_rep!($($x)*);
            if $args.len() != argc {
                Err(gml::runtime::Error::WrongArgumentCount(argc, $args.len()))
            } else {
                _apply_args!($args, $($x)*)
            }
        })()
    }};
    ($args: expr, [$($x: ident,)*]) => { expect_args!($args, $($x),*) };
}

#[rustfmt::skip]
fn rgb_to_hsv(colour: i32) -> (i32, i32, i32) {
    let (r, g, b) = (Real::from(0xFF & colour), Real::from(0xFF & (colour >> 8)), Real::from(0xFF & (colour >> 16)));

    let (min, max) = (r.min(g).min(b), r.max(g).max(b));

    let v = max.round().to_i32();
    let (h, s);

    if max == min {
        s = 0; // achromatic
        h = 0; // actually undefined, so value can be any
    } else {
        let x60 = Real::from(60);
        let angle360 = Real::from(360);
        let range255 = Real::from(255);

        let diff = max - min;
        s = ((diff / max) * range255).round().to_i32();

        h = ((((if max == g {
            x60 * ((b - r) / diff) + Real::from(120)
        } else if max == b {
            x60 * ((r - g) / diff) + Real::from(240)
        } else if max == r {
            x60 * ((g - b) / diff) + angle360
        } else {
            unsafe { std::hint::unreachable_unchecked() }
        }) % angle360) / angle360) * range255).round().to_i32();
    }

    (h, s, v)
}

impl Game {
    pub fn display_get_width(&self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 0
        unimplemented!("Called unimplemented kernel function display_get_width")
    }

    pub fn display_get_height(&self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 0
        unimplemented!("Called unimplemented kernel function display_get_height")
    }

    pub fn display_get_colordepth(&self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 0
        unimplemented!("Called unimplemented kernel function display_get_colordepth")
    }

    pub fn display_get_frequency(&self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 0
        unimplemented!("Called unimplemented kernel function display_get_frequency")
    }

    pub fn display_set_size(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 2
        unimplemented!("Called unimplemented kernel function display_set_size")
    }

    pub fn display_set_colordepth(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        unimplemented!("Called unimplemented kernel function display_set_colordepth")
    }

    pub fn display_set_frequency(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        unimplemented!("Called unimplemented kernel function display_set_frequency")
    }

    pub fn display_set_all(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 4
        unimplemented!("Called unimplemented kernel function display_set_all")
    }

    pub fn display_test_all(&self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 4
        unimplemented!("Called unimplemented kernel function display_test_all")
    }

    pub fn display_reset(&mut self, _args: &[Value]) -> gml::Result<Value> {
        unimplemented!("Called unimplemented kernel function display_reset")
    }

    pub fn display_mouse_get_x(&self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 0
        unimplemented!("Called unimplemented kernel function display_mouse_get_x")
    }

    pub fn display_mouse_get_y(&self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 0
        unimplemented!("Called unimplemented kernel function display_mouse_get_y")
    }

    pub fn display_mouse_set(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 2
        unimplemented!("Called unimplemented kernel function display_mouse_set")
    }

    pub fn window_set_visible(&mut self, args: &[Value]) -> gml::Result<Value> {
        let visible = expect_args!(args, [bool])?;
        self.window.set_visible(visible);
        Ok(Default::default())
    }

    // NB: This function is constant because window's visibility state is tracked.
    pub fn window_get_visible(&self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        Ok(self.window_visible.into())
    }

    pub fn window_set_fullscreen(&mut self, args: &[Value]) -> gml::Result<Value> {
        let _full = expect_args!(args, [bool])?;
        // TODO
        Ok(Default::default())
    }

    pub fn window_get_fullscreen(&self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        // TODO
        Ok(false.into())
    }

    pub fn window_set_showborder(&mut self, args: &[Value]) -> gml::Result<Value> {
        let show_border = expect_args!(args, [bool])?;
        if show_border != self.window_border {
            self.window_border = show_border;
            if self.play_type != PlayType::Record {
                // TODO: Borderless
                unimplemented!()
            }
        }
        Ok(Default::default())
    }

    pub fn window_get_showborder(&self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        Ok(self.window_border.into())
    }

    pub fn window_set_showicons(&mut self, args: &[Value]) -> gml::Result<Value> {
        let show_icons = expect_args!(args, [bool])?;
        if show_icons != self.window_icons {
            self.window_icons = show_icons;
            if self.play_type != PlayType::Record {
                self.window.set_controls({
                    if self.window_icons { Some(ramen::window::Controls::enabled()) } else { None }
                })
            }
        }
        Ok(Default::default())
    }

    pub fn window_get_showicons(&self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        Ok(self.window_icons.into())
    }

    pub fn window_set_stayontop(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        unimplemented!("Called unimplemented kernel function window_set_stayontop")
    }

    pub fn window_get_stayontop(&self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 0
        unimplemented!("Called unimplemented kernel function window_get_stayontop")
    }

    pub fn window_set_sizeable(&mut self, args: &[Value]) -> gml::Result<Value> {
        let sizeable = expect_args!(args, [bool])?;
        if sizeable != self.window_sizeable {
            self.window_sizeable = sizeable;
            if self.play_type != PlayType::Record {
                self.window.set_resizable(self.window_sizeable);
            }
        }
        Ok(Default::default())
    }

    pub fn window_get_sizeable(&self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        Ok(self.window_sizeable.into())
    }

    pub fn window_set_caption(&mut self, args: &[Value]) -> gml::Result<Value> {
        let caption = expect_args!(args, [string])?;
        if self.play_type == PlayType::Record {
            self.window.set_title(caption.as_ref());
        }
        self.window_caption = caption.into_owned();
        Ok(Default::default())
    }

    // NB: This function is constant because caption gets updated on every frame.
    pub fn window_get_caption(&self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        Ok(self.window_caption.clone().into())
    }

    pub fn window_set_cursor(&mut self, args: &[Value]) -> gml::Result<Value> {
        let mut code = expect_args!(args, [int])?;
        let cursor = match code {
            // TODO: maybe add more of these to ramen but wtf
            x if x == gml_consts::CR_DEFAULT as i32 => Cursor::Arrow,
            x if x == gml_consts::CR_ARROW as i32 => Cursor::Arrow,
            x if x == gml_consts::CR_CROSS as i32 => Cursor::Cross,
            x if x == gml_consts::CR_BEAM as i32 => Cursor::IBeam,
            x if x == gml_consts::CR_SIZE_NESW as i32 => Cursor::ResizeNESW,
            x if x == gml_consts::CR_SIZE_NS as i32 => Cursor::ResizeNS,
            x if x == gml_consts::CR_SIZE_NWSE as i32 => Cursor::ResizeNWSE,
            x if x == gml_consts::CR_SIZE_WE as i32 => Cursor::ResizeWE,
            x if x == gml_consts::CR_UPARROW as i32 => Cursor::Arrow, // ???
            x if x == gml_consts::CR_HOURGLASS as i32 => Cursor::Wait,
            x if x == gml_consts::CR_DRAG as i32 => Cursor::Arrow, // ???
            x if x == gml_consts::CR_NODROP as i32 => Cursor::Unavailable, // ???
            x if x == gml_consts::CR_HSPLIT as i32 => Cursor::ResizeWE,
            x if x == gml_consts::CR_VSPLIT as i32 => Cursor::ResizeNS,
            x if x == gml_consts::CR_MULTIDRAG as i32 => Cursor::Arrow, // ???
            x if x == gml_consts::CR_SQLWAIT as i32 => Cursor::Wait,    // ???
            x if x == gml_consts::CR_NO as i32 => Cursor::Unavailable,
            x if x == gml_consts::CR_APPSTART as i32 => Cursor::Progress, // ???
            x if x == gml_consts::CR_HELP as i32 => Cursor::Help,
            x if x == gml_consts::CR_HANDPOINT as i32 => Cursor::Hand,
            x if x == gml_consts::CR_SIZE_ALL as i32 => Cursor::ResizeAll,
            _ => {
                code = gml_consts::CR_NONE as i32;
                Cursor::Blank
            },
        };
        if self.play_type == PlayType::Normal {
            self.window.set_cursor(cursor);
        }
        self.window_cursor_gml = code;
        Ok(Default::default())
    }

    pub fn window_get_cursor(&self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        Ok(self.window_cursor_gml.into())
    }

    pub fn window_set_color(&mut self, args: &[Value]) -> gml::Result<Value> {
        let col = expect_args!(args, [int])?;
        self.background_colour = (col as u32).into();
        Ok(Default::default())
    }

    pub fn window_get_color(&self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        Ok(u32::from(self.background_colour).into())
    }

    pub fn window_set_position(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (x, y) = expect_args!(args, [int, int])?;
        self.window_offset_spoof = (x, y);
        Ok(Default::default())
    }

    pub fn window_set_size(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (width, height) = expect_args!(args, [int, int])?;
        if width > 0 && height > 0 {
            self.window_inner_size = (width as u32, height as u32);
            self.window.execute(|window| {
                use ramen::monitor::Size;
                if window.is_dpi_logical() {
                    unimplemented!();
                } else {
                    window.set_inner_size(Size::Physical(width as u32, height as u32));
                }
            });
        }
        Ok(Default::default())
    }

    pub fn window_set_rectangle(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 4
        unimplemented!("Called unimplemented kernel function window_set_rectangle")
    }

    pub fn window_center(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // TODO: make this do something!
        Ok(Default::default())
    }

    pub fn window_default(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 0
        unimplemented!("Called unimplemented kernel function window_default")
    }

    pub fn window_get_x(&self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        Ok(self.window_offset_spoof.0.into())
    }

    pub fn window_get_y(&self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        Ok(self.window_offset_spoof.1.into())
    }

    pub fn window_get_width(&self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        Ok(self.window_inner_size.0.into())
    }

    pub fn window_get_height(&self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        Ok(self.window_inner_size.1.into())
    }

    pub fn window_set_region_size(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 3
        // unscaled_width and unscaled_height will need to be separated into framebuffer size
        // and window region size for this to work
        // probably keep the framebuffer size on the renderer and make a getter?
        unimplemented!("Called unimplemented kernel function window_set_region_size")
    }

    pub fn window_get_region_width(&self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        Ok(self.unscaled_width.into())
    }

    pub fn window_get_region_height(&self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        Ok(self.unscaled_height.into())
    }

    pub fn window_set_region_scale(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (scaling, shrink_window) = expect_args!(args, [real, bool])?;
        let scaling = match scaling {
            n if n == 0.into() => Scaling::Full,
            n if n < 0.into() => Scaling::Aspect(n.into_inner()),
            n => Scaling::Fixed(n.into_inner()),
        };
        self.scaling = scaling;
        if let Scaling::Fixed(n) = scaling {
            let (region_w, region_h) =
                ((self.unscaled_width as f64 * n) as u32, (self.unscaled_height as f64 * n) as u32);
            let (width, height) = if shrink_window {
                let (window_w, window_h) = self.window_inner_size;
                (region_w.max(window_w), region_h.max(window_h))
            } else {
                (region_w, region_h)
            };
            self.window_inner_size = (width, height);
            self.window.set_inner_size(ramen::monitor::Size::Physical(width, height));
        }
        Ok(Default::default())
    }

    pub fn window_get_region_scale(&self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        Ok(match self.scaling {
            Scaling::Fixed(n) => n,
            Scaling::Aspect(n) => n,
            Scaling::Full => 0.0,
        }
        .into())
    }

    pub fn window_mouse_get_x(&self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        Ok(self.input.mouse_x().into())
    }

    pub fn window_mouse_get_y(&self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        Ok(self.input.mouse_y().into())
    }

    pub fn window_mouse_set(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 2
        unimplemented!("Called unimplemented kernel function window_mouse_set")
    }

    pub fn window_view_mouse_get_x(&self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        unimplemented!("Called unimplemented kernel function window_view_mouse_get_x")
    }

    pub fn window_view_mouse_get_y(&self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        unimplemented!("Called unimplemented kernel function window_view_mouse_get_y")
    }

    pub fn window_view_mouse_set(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 3
        unimplemented!("Called unimplemented kernel function window_view_mouse_set")
    }

    pub fn window_views_mouse_get_x(&self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 0
        unimplemented!("Called unimplemented kernel function window_views_mouse_get_x")
    }

    pub fn window_views_mouse_get_y(&self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 0
        unimplemented!("Called unimplemented kernel function window_views_mouse_get_y")
    }

    pub fn window_views_mouse_set(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 2
        unimplemented!("Called unimplemented kernel function window_views_mouse_set")
    }

    pub fn set_synchronization(&mut self, args: &[Value]) -> gml::Result<Value> {
        let synchro = expect_args!(args, [bool])?;
        self.renderer.set_vsync(synchro);
        Ok(Default::default())
    }

    pub fn set_automatic_draw(&mut self, args: &[Value]) -> gml::Result<Value> {
        let auto_draw = expect_args!(args, [bool])?;
        self.auto_draw = auto_draw;
        Ok(Default::default())
    }

    pub fn screen_redraw(&mut self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        self.draw()?;
        Ok(Default::default())
    }

    pub fn screen_refresh(&mut self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        let (width, height) = self.window_inner_size;
        if self.play_type != PlayType::Record {
            self.renderer.present(width, height, self.scaling);
        }
        Ok(Default::default())
    }

    pub fn screen_wait_vsync(&self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        self.renderer.wait_vsync();
        Ok(Default::default())
    }

    pub fn screen_save(&mut self, args: &[Value]) -> gml::Result<Value> {
        let fname = expect_args!(args, [string])?;
        self.renderer.flush_queue();
        let (width, height) = (self.unscaled_width, self.unscaled_height);
        let rgba = self.renderer.get_pixels(0, 0, width as _, height as _);
        let mut image = RgbaImage::from_vec(width, height, rgba.into()).unwrap();
        asset::sprite::process_image(&mut image, false, false, true);
        match file::save_image(fname.as_ref(), image) {
            Ok(()) => Ok(Default::default()),
            Err(e) => Err(gml::Error::FunctionError("screen_save".into(), e.to_string())),
        }
    }

    pub fn screen_save_part(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (fname, x, y, w, h) = expect_args!(args, [string, int, int, int, int])?;
        let x = x.max(0);
        let y = y.max(0);
        let w = w.min(self.unscaled_width as i32 - x);
        let h = h.min(self.unscaled_height as i32 - y);
        self.renderer.flush_queue();
        let rgba = self.renderer.get_pixels(x, y, w, h);
        let mut image = RgbaImage::from_vec(w as _, h as _, rgba.into()).unwrap();
        asset::sprite::process_image(&mut image, false, false, true);
        match file::save_image(fname.as_ref(), image) {
            Ok(()) => Ok(Default::default()),
            Err(e) => Err(gml::Error::FunctionError("screen_save_part".into(), e.to_string())),
        }
    }

    pub fn draw_getpixel(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (x, y) = expect_args!(args, [int, int])?;
        self.renderer.flush_queue();
        let data = self.renderer.get_pixels(x, y, 1, 1);
        Ok(u32::from_le_bytes([data[0], data[1], data[2], 0]).into())
    }

    pub fn draw_set_color(&mut self, args: &[Value]) -> gml::Result<Value> {
        let col = expect_args!(args, [int])?;
        self.draw_colour = (col as u32).into();
        Ok(Default::default())
    }

    pub fn draw_set_alpha(&mut self, args: &[Value]) -> gml::Result<Value> {
        self.draw_alpha = expect_args!(args, [real])?;
        Ok(Default::default())
    }

    pub fn draw_get_color(&self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        Ok(u32::from(self.draw_colour).into())
    }

    pub fn draw_get_alpha(&self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        Ok(self.draw_alpha.into())
    }

    pub fn make_color_rgb(args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [int, int, int]).map(|(r, g, b)| r | (g << 8) | (b << 16)).map(Value::from)
    }

    pub fn make_color_hsv(args: &[Value]) -> gml::Result<Value> {
        let (h, s, v) = expect_args!(args, [real, real, real])?;
        let h = h * Real::from(360.0) / Real::from(255.0);
        let s = s / Real::from(255.0);
        let v = v / Real::from(255.0);
        let chroma = v * s;
        let hprime = (h / Real::from(60.0)).rem_euclid(Real::from(6.0));
        let x = chroma * (Real::from(1.0) - (hprime.rem_euclid(Real::from(2.0)) - Real::from(1.0)).abs());
        let m = v - chroma;

        let (r, g, b) = match hprime.floor().into_inner() as i32 {
            0 => (chroma, x, Real::from(0.0)),
            1 => (x, chroma, Real::from(0.0)),
            2 => (Real::from(0.0), chroma, x),
            3 => (Real::from(0.0), x, chroma),
            4 => (x, Real::from(0.0), chroma),
            5 => (chroma, Real::from(0.0), x),
            _ => (Real::from(0.0), Real::from(0.0), Real::from(0.0)),
        };

        let out_r = ((r + m) * Real::from(255.0)).round().to_i32();
        let out_g = ((g + m) * Real::from(255.0)).round().to_i32();
        let out_b = ((b + m) * Real::from(255.0)).round().to_i32();
        Ok((out_r | (out_g << 8) | (out_b << 16)).into())
    }

    pub fn color_get_red(args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [int]).map(|c| 0xFF & c).map(Value::from)
    }

    pub fn color_get_green(args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [int]).map(|c| 0xFF & (c >> 8)).map(Value::from)
    }

    pub fn color_get_blue(args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [int]).map(|c| 0xFF & (c >> 16)).map(Value::from)
    }

    pub fn color_get_hue(args: &[Value]) -> gml::Result<Value> {
        let c = expect_args!(args, [int])?;
        let (h, _, _) = rgb_to_hsv(c);
        Ok(h.into())
    }

    pub fn color_get_saturation(args: &[Value]) -> gml::Result<Value> {
        let c = expect_args!(args, [int])?;
        let (_, s, _) = rgb_to_hsv(c);
        Ok(s.into())
    }

    pub fn color_get_value(args: &[Value]) -> gml::Result<Value> {
        let c = expect_args!(args, [int])?;
        let (_, _, v) = rgb_to_hsv(c);
        Ok(v.into())
    }

    pub fn merge_color(args: &[Value]) -> gml::Result<Value> {
        let (c1, c2, amount) = expect_args!(args, [int, int, real])?;
        let r = Real::from(c1 & 255) * (Real::from(1) - amount) + Real::from(c2 & 255) * amount;
        let g = Real::from((c1 >> 8) & 255) * (Real::from(1) - amount) + Real::from((c2 >> 8) & 255) * amount;
        let b = Real::from((c1 >> 16) & 255) * (Real::from(1) - amount) + Real::from((c2 >> 16) & 255) * amount;
        Ok(Value::from(
            (r.round().to_i32() & 255) + ((g.round().to_i32() & 255) << 8) + ((b.round().to_i32() & 255) << 16),
        ))
    }

    pub fn draw_set_blend_mode(&mut self, args: &[Value]) -> gml::Result<Value> {
        let mode = expect_args!(args, [int])?;
        let (src, dest) = match mode {
            1 => (BlendType::SrcAlpha, BlendType::One),          // bm_add
            2 => (BlendType::SrcAlpha, BlendType::InvSrcColour), // bm_subtract
            3 => (BlendType::Zero, BlendType::InvSrcColour),     // bm_max
            _ => (BlendType::SrcAlpha, BlendType::InvSrcAlpha),  // bm_normal
        };
        self.renderer.set_blend_mode(src, dest);
        Ok(Default::default())
    }

    pub fn draw_set_blend_mode_ext(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (src, dest) = expect_args!(args, [int, int])?;
        let int_to_blend_type = |i| match i {
            2 => BlendType::One,
            3 => BlendType::SrcColour,
            4 => BlendType::InvSrcColour,
            5 => BlendType::SrcAlpha,
            6 => BlendType::InvSrcAlpha,
            7 => BlendType::DestAlpha,
            8 => BlendType::InvDestAlpha,
            9 => BlendType::DestColour,
            10 => BlendType::InvDestColour,
            11 => BlendType::SrcAlphaSaturate,
            _ => BlendType::Zero,
        };
        let src = int_to_blend_type(src);
        let dest = int_to_blend_type(dest);
        self.renderer.set_blend_mode(src, dest);
        Ok(Default::default())
    }

    pub fn draw_clear(&mut self, args: &[Value]) -> gml::Result<Value> {
        let col = expect_args!(args, [int])?;
        if self.gm_version == Version::GameMaker8_0 && !self.surface_fix {
            self.renderer.clear_view_no_zbuf((col as u32).into(), 1.0);
        } else {
            self.renderer.clear_view((col as u32).into(), 1.0);
        }
        Ok(Default::default())
    }

    pub fn draw_clear_alpha(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (col, alpha) = expect_args!(args, [int, real])?;
        if self.gm_version == Version::GameMaker8_0 && !self.surface_fix {
            self.renderer.clear_view_no_zbuf((col as u32).into(), alpha.into());
        } else {
            self.renderer.clear_view((col as u32).into(), alpha.into());
        }
        self.renderer.clear_view((col as u32).into(), alpha.into());
        Ok(Default::default())
    }

    pub fn draw_point(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (x, y) = expect_args!(args, [real, real])?;
        self.renderer.draw_point(x.into(), y.into(), u32::from(self.draw_colour) as _, self.draw_alpha.into());
        Ok(Default::default())
    }

    pub fn draw_line(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (x1, y1, x2, y2) = expect_args!(args, [real, real, real, real])?;
        self.renderer.draw_line(
            x1.into(),
            y1.into(),
            x2.into(),
            y2.into(),
            None,
            u32::from(self.draw_colour) as _,
            u32::from(self.draw_colour) as _,
            self.draw_alpha.into(),
        );
        Ok(Default::default())
    }

    pub fn draw_line_width(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (x1, y1, x2, y2, w) = expect_args!(args, [real, real, real, real, real])?;
        self.renderer.draw_line(
            x1.into(),
            y1.into(),
            x2.into(),
            y2.into(),
            Some(w.into()),
            u32::from(self.draw_colour) as _,
            u32::from(self.draw_colour) as _,
            self.draw_alpha.into(),
        );
        Ok(Default::default())
    }

    pub fn draw_rectangle(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (x1, y1, x2, y2, outline) = expect_args!(args, [real, real, real, real, bool])?;
        if outline {
            self.renderer.draw_rectangle_outline(
                x1.into(),
                y1.into(),
                x2.into(),
                y2.into(),
                u32::from(self.draw_colour) as _,
                self.draw_alpha.into(),
            );
        } else {
            self.renderer.draw_rectangle(
                x1.into(),
                y1.into(),
                x2.into(),
                y2.into(),
                u32::from(self.draw_colour) as _,
                self.draw_alpha.into(),
            );
        }
        Ok(Default::default())
    }

    pub fn draw_roundrect(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (x1, y1, x2, y2, outline) = expect_args!(args, [real, real, real, real, bool])?;
        self.renderer.draw_roundrect(
            x1.into(),
            y1.into(),
            x2.into(),
            y2.into(),
            u32::from(self.draw_colour) as _,
            u32::from(self.draw_colour) as _,
            self.draw_alpha.into(),
            outline,
        );
        Ok(Default::default())
    }

    pub fn draw_triangle(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (x1, y1, x2, y2, x3, y3, outline) = expect_args!(args, [real, real, real, real, real, real, bool])?;
        self.renderer.draw_triangle(
            x1.into(),
            y1.into(),
            x2.into(),
            y2.into(),
            x3.into(),
            y3.into(),
            u32::from(self.draw_colour) as _,
            u32::from(self.draw_colour) as _,
            u32::from(self.draw_colour) as _,
            self.draw_alpha.into(),
            outline,
        );
        Ok(Default::default())
    }

    pub fn draw_circle(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (x, y, radius, outline) = expect_args!(args, [real, real, real, bool])?;
        self.renderer.draw_ellipse(
            x.into(),
            y.into(),
            radius.into(),
            radius.into(),
            u32::from(self.draw_colour) as _,
            u32::from(self.draw_colour) as _,
            self.draw_alpha.into(),
            outline,
        );
        Ok(Default::default())
    }

    pub fn draw_ellipse(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (x1, y1, x2, y2, outline) = expect_args!(args, [real, real, real, real, bool])?;
        let xcenter = (x1 + x2) / 2.into();
        let ycenter = (y1 + y2) / 2.into();
        let rad_x = (xcenter - x1).abs();
        let rad_y = (ycenter - y1).abs();
        self.renderer.draw_ellipse(
            xcenter.into(),
            ycenter.into(),
            rad_x.into(),
            rad_y.into(),
            u32::from(self.draw_colour) as _,
            u32::from(self.draw_colour) as _,
            self.draw_alpha.into(),
            outline,
        );
        Ok(Default::default())
    }

    pub fn draw_arrow(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (x1, y1, x2, y2, size) = expect_args!(args, [real, real, real, real, real])?;
        let (x1, y1, x2, y2) = (x1.into_inner(), y1.into_inner(), x2.into_inner(), y2.into_inner());
        let length = (x2 - x1).hypot(y2 - y1);
        if length != 0.0 {
            self.renderer.draw_line(
                x1,
                y1,
                x2,
                y2,
                None,
                u32::from(self.draw_colour) as _,
                u32::from(self.draw_colour) as _,
                self.draw_alpha.into(),
            );
            let size = size.into_inner().min(length);
            let x_offset = (x2 - x1) * size / length;
            let y_offset = (y2 - y1) * size / length;
            self.renderer.draw_triangle(
                (x2 - x_offset) - y_offset / 3.0,
                (y2 - y_offset) + x_offset / 3.0,
                x2,
                y2,
                (x2 - x_offset) + y_offset / 3.0,
                (y2 - y_offset) - x_offset / 3.0,
                u32::from(self.draw_colour) as _,
                u32::from(self.draw_colour) as _,
                u32::from(self.draw_colour) as _,
                self.draw_alpha.into(),
                false,
            );
        }
        Ok(Default::default())
    }

    pub fn draw_button(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (x1, y1, x2, y2, up) = expect_args!(args, [real, real, real, real, bool])?;
        let (top_col, bottom_col) = if up { (0xffffff, 0x808080) } else { (0x808080, 0xffffff) };
        self.renderer.draw_triangle(
            x1.into(),
            y1.into(),
            x2.into(),
            y1.into(),
            x1.into(),
            y2.into(),
            top_col,
            top_col,
            top_col,
            self.draw_alpha.into(),
            false,
        );
        self.renderer.draw_triangle(
            x1.into(),
            y2.into(),
            x2.into(),
            y1.into(),
            x2.into(),
            y2.into(),
            bottom_col,
            bottom_col,
            bottom_col,
            self.draw_alpha.into(),
            false,
        );
        self.renderer.draw_rectangle(
            x1.into_inner() + 2.0,
            y1.into_inner() + 2.0,
            x2.into_inner() - 2.0,
            y2.into_inner() - 2.0,
            u32::from(self.draw_colour) as i32,
            self.draw_alpha.into(),
        );
        Ok(Default::default())
    }

    pub fn draw_healthbar(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (x1, y1, x2, y2, amount, backcol, mincol, maxcol, direction, showback, showborder) =
            expect_args!(args, [real, real, real, real, real, int, int, int, int, bool, bool])?;
        let health_ratio = f64::from(amount / Real::from(100.0));
        let lerp = |min, max| (health_ratio * f64::from(max) + (1.0 - health_ratio) * f64::from(min)) as u8 as i32;
        let bar_colour = lerp(mincol & 0xff, maxcol & 0xff)
            | (lerp((mincol >> 8) & 0xff, (maxcol >> 8) & 0xff) << 8)
            | (lerp((mincol >> 16) & 0xff, (maxcol >> 16) & 0xff) << 16);
        let (x1, y1, x2, y2) = (x1.into_inner(), y1.into_inner(), x2.into_inner(), y2.into_inner());
        if showback {
            self.renderer.draw_rectangle(x1, y1, x2, y2, backcol, self.draw_alpha.into());
            if showborder {
                self.renderer.draw_rectangle_outline(x1, y1, x2, y2, 0, self.draw_alpha.into());
            }
        }
        let (x1, y1, x2, y2) = match direction {
            1 => (x2 - (x2 - x1) * health_ratio, y1, x2, y2),
            2 => (x1, y1, x2, y1 + (y2 - y1) * health_ratio),
            3 => (x1, y2 - (y2 - y1) * health_ratio, x2, y2),
            _ => (x1, y1, x1 + (x2 - x1) * health_ratio, y2),
        };
        self.renderer.draw_rectangle(x1, y1, x2, y2, bar_colour, self.draw_alpha.into());
        if showborder {
            self.renderer.draw_rectangle_outline(x1, y1, x2, y2, 0, self.draw_alpha.into());
        }
        Ok(Default::default())
    }

    pub fn draw_path(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (path, x, y, absolute) = expect_args!(args, [int, real, real, bool])?;
        let path = self.assets.paths.get_asset(path).ok_or(gml::Error::NonexistentAsset(asset::Type::Path, path))?;
        let (x_offset, y_offset) = if absolute { (0.into(), 0.into()) } else { (x - path.start.x, y - path.start.y) };

        for (node1, node2) in path.control_nodes.windows(2).map(|x| (x[0], x[1])) {
            self.renderer.draw_line(
                f64::from(node1.point.x + x_offset),
                f64::from(node1.point.y + y_offset),
                f64::from(node2.point.x + x_offset),
                f64::from(node2.point.y + y_offset),
                None,
                u32::from(self.draw_colour) as _,
                u32::from(self.draw_colour) as _,
                self.draw_alpha.into(),
            );
        }

        Ok(Default::default())
    }

    pub fn draw_point_color(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (x, y, col) = expect_args!(args, [real, real, int])?;
        self.renderer.draw_point(x.into(), y.into(), col, self.draw_alpha.into());
        Ok(Default::default())
    }

    pub fn draw_line_color(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (x1, y1, x2, y2, c1, c2) = expect_args!(args, [real, real, real, real, int, int])?;
        self.renderer.draw_line(x1.into(), y1.into(), x2.into(), y2.into(), None, c1, c2, self.draw_alpha.into());
        Ok(Default::default())
    }

    pub fn draw_line_width_color(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (x1, y1, x2, y2, w, c1, c2) = expect_args!(args, [real, real, real, real, real, int, int])?;
        self.renderer.draw_line(
            x1.into(),
            y1.into(),
            x2.into(),
            y2.into(),
            Some(w.into()),
            c1,
            c2,
            self.draw_alpha.into(),
        );
        Ok(Default::default())
    }

    pub fn draw_rectangle_color(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (x1, y1, x2, y2, c1, c2, c3, c4, outline) =
            expect_args!(args, [real, real, real, real, int, int, int, int, bool])?;
        self.renderer.draw_rectangle_gradient(
            x1.into(),
            y1.into(),
            x2.into(),
            y2.into(),
            c1,
            c2,
            c3,
            c4,
            self.draw_alpha.into(),
            outline,
        );
        Ok(Default::default())
    }

    pub fn draw_roundrect_color(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (x1, y1, x2, y2, col1, col2, outline) = expect_args!(args, [real, real, real, real, int, int, bool])?;
        self.renderer.draw_roundrect(
            x1.into(),
            y1.into(),
            x2.into(),
            y2.into(),
            col1,
            col2,
            self.draw_alpha.into(),
            outline,
        );
        Ok(Default::default())
    }

    pub fn draw_triangle_color(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (x1, y1, x2, y2, x3, y3, c1, c2, c3, outline) =
            expect_args!(args, [real, real, real, real, real, real, int, int, int, bool])?;
        self.renderer.draw_triangle(
            x1.into(),
            y1.into(),
            x2.into(),
            y2.into(),
            x3.into(),
            y3.into(),
            c1,
            c2,
            c3,
            self.draw_alpha.into(),
            outline,
        );
        Ok(Default::default())
    }

    pub fn draw_circle_color(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (x, y, radius, col1, col2, outline) = expect_args!(args, [real, real, real, int, int, bool])?;
        self.renderer.draw_ellipse(
            x.into(),
            y.into(),
            radius.into(),
            radius.into(),
            col1,
            col2,
            self.draw_alpha.into(),
            outline,
        );
        Ok(Default::default())
    }

    pub fn draw_ellipse_color(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (x1, y1, x2, y2, col1, col2, outline) = expect_args!(args, [real, real, real, real, int, int, bool])?;
        let xcenter = (x1 + x2) / 2.into();
        let ycenter = (y1 + y2) / 2.into();
        let rad_x = (xcenter - x1).abs();
        let rad_y = (ycenter - y1).abs();
        self.renderer.draw_ellipse(
            xcenter.into(),
            ycenter.into(),
            rad_x.into(),
            rad_y.into(),
            col1,
            col2,
            self.draw_alpha.into(),
            outline,
        );
        Ok(Default::default())
    }

    pub fn draw_set_circle_precision(&mut self, args: &[Value]) -> gml::Result<Value> {
        let prec = expect_args!(args, [int])?;
        self.renderer.set_circle_precision(prec);
        Ok(Default::default())
    }

    pub fn draw_primitive_begin(&mut self, args: &[Value]) -> gml::Result<Value> {
        let kind = expect_args!(args, [int])?;
        self.renderer.reset_primitive_2d(kind.into(), None);
        Ok(Default::default())
    }

    pub fn draw_primitive_begin_texture(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (kind, texture) = expect_args!(args, [int, int])?;
        self.renderer.reset_primitive_2d(kind.into(), self.renderer.get_texture_from_id(texture as _).copied());
        Ok(Default::default())
    }

    pub fn draw_primitive_end(&mut self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        self.renderer.draw_primitive_2d();
        Ok(Default::default())
    }

    pub fn draw_vertex(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (x, y) = expect_args!(args, [real, real])?;
        self.renderer.vertex_2d(x.into(), y.into(), 0.0, 0.0, u32::from(self.draw_colour) as _, self.draw_alpha.into());
        Ok(Default::default())
    }

    pub fn draw_vertex_color(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (x, y, col, alpha) = expect_args!(args, [real, real, int, real])?;
        self.renderer.vertex_2d(x.into(), y.into(), 0.0, 0.0, col, alpha.into());
        Ok(Default::default())
    }

    pub fn draw_vertex_texture(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (x, y, xtex, ytex) = expect_args!(args, [real, real, real, real])?;
        self.renderer.vertex_2d(
            x.into(),
            y.into(),
            xtex.into(),
            ytex.into(),
            u32::from(self.draw_colour) as _,
            self.draw_alpha.into(),
        );
        Ok(Default::default())
    }

    pub fn draw_vertex_texture_color(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (x, y, xtex, ytex, col, alpha) = expect_args!(args, [real, real, real, real, int, real])?;
        self.renderer.vertex_2d(x.into(), y.into(), xtex.into(), ytex.into(), col, alpha.into());
        Ok(Default::default())
    }

    pub fn sprite_get_texture(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (sprite_index, image_index) = expect_args!(args, [int, int])?;
        if let Some(sprite) = self.assets.sprites.get_asset(sprite_index) {
            if let Some(atlas_ref) = sprite.get_atlas_ref(image_index) {
                return Ok(self.renderer.get_texture_id(atlas_ref).into())
            }
            Ok((-1).into())
        } else {
            Err(gml::Error::NonexistentAsset(asset::Type::Sprite, sprite_index))
        }
    }

    pub fn background_get_texture(&mut self, args: &[Value]) -> gml::Result<Value> {
        let bg_index = expect_args!(args, [int])?;
        if let Some(background) = self.assets.backgrounds.get_asset(bg_index) {
            if let Some(atlas_ref) = &background.atlas_ref {
                Ok(self.renderer.get_texture_id(atlas_ref).into())
            } else {
                Ok((-1).into())
            }
        } else {
            Err(gml::Error::NonexistentAsset(asset::Type::Background, bg_index))
        }
    }

    pub fn texture_exists(&self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        unimplemented!("Called unimplemented kernel function texture_exists")
    }

    pub fn texture_set_interpolation(&mut self, args: &[Value]) -> gml::Result<Value> {
        let lerping = expect_args!(args, [bool])?;
        self.renderer.set_pixel_interpolation(lerping);
        Ok(Default::default())
    }

    pub fn texture_set_blending(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        unimplemented!("Called unimplemented kernel function texture_set_blending")
    }

    pub fn texture_set_repeat(&mut self, args: &[Value]) -> gml::Result<Value> {
        let repeat = expect_args!(args, [bool])?;
        self.renderer.set_texture_repeat(repeat);
        Ok(Default::default())
    }

    pub fn texture_get_width(args: &[Value]) -> gml::Result<Value> {
        let _texid = expect_args!(args, [int])?;
        Ok(1.into()) // we don't pad textures to power-of-2
    }

    pub fn texture_get_height(args: &[Value]) -> gml::Result<Value> {
        let _texid = expect_args!(args, [int])?;
        Ok(1.into()) // see texture_get_width
    }

    pub fn texture_preload(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        unimplemented!("Called unimplemented kernel function texture_preload")
    }

    pub fn texture_set_priority(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 2
        unimplemented!("Called unimplemented kernel function texture_set_priority")
    }

    pub fn draw_set_font(&mut self, args: &[Value]) -> gml::Result<Value> {
        let font_id = expect_args!(args, [int])?;
        if self.assets.fonts.get_asset(font_id).is_some() {
            self.draw_font_id = font_id;
        } else {
            self.draw_font_id = -1;
        }
        Ok(Default::default())
    }

    pub fn draw_set_halign(&mut self, args: &[Value]) -> gml::Result<Value> {
        self.draw_halign = match expect_args!(args, [int])? {
            1 => draw::Halign::Middle,
            2 => draw::Halign::Right,
            0 | _ => draw::Halign::Left,
        };
        Ok(Default::default())
    }

    pub fn draw_set_valign(&mut self, args: &[Value]) -> gml::Result<Value> {
        self.draw_valign = match expect_args!(args, [int])? {
            0 => draw::Valign::Top,
            1 => draw::Valign::Middle,
            2 | _ => draw::Valign::Bottom,
        };
        Ok(Default::default())
    }

    pub fn string_width(&self, args: &[Value]) -> gml::Result<Value> {
        let string = expect_args!(args, [bytes])?;
        let (width, _) = self.get_string_size(string, None, None);
        Ok(width.into())
    }

    pub fn string_height(&self, args: &[Value]) -> gml::Result<Value> {
        let string = expect_args!(args, [bytes])?;
        let (_, height) = self.get_string_size(string, None, None);
        Ok(height.into())
    }

    pub fn string_width_ext(&self, args: &[Value]) -> gml::Result<Value> {
        let (string, line_height, max_width) = expect_args!(args, [bytes, int, int])?;
        let (width, _) = self.get_string_size(
            string,
            if line_height < 0 { None } else { Some(line_height as _) },
            if max_width < 0 { None } else { Some(max_width as _) },
        );
        Ok(width.into())
    }

    pub fn string_height_ext(&self, args: &[Value]) -> gml::Result<Value> {
        let (string, line_height, max_width) = expect_args!(args, [bytes, int, int])?;
        let (_, height) = self.get_string_size(
            string,
            if line_height < 0 { None } else { Some(line_height as _) },
            if max_width < 0 { None } else { Some(max_width as _) },
        );
        Ok(height.into())
    }

    pub fn draw_text(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (x, y, text) = expect_args!(args, [real, real, any])?;
        self.draw_string(x, y, text.repr(), None, None, 1.into(), 1.into(), 0.into(), None, self.draw_alpha.into());
        Ok(Default::default())
    }

    pub fn draw_text_ext(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (x, y, text, line_height, max_width) = expect_args!(args, [real, real, any, int, int])?;
        let line_height = if line_height < 0 { None } else { Some(line_height as _) };
        let max_width = if max_width < 0 { None } else { Some(max_width as _) };

        self.draw_string(
            x,
            y,
            text.repr(),
            line_height,
            max_width,
            1.into(),
            1.into(),
            0.into(),
            None,
            self.draw_alpha.into(),
        );
        Ok(Default::default())
    }

    pub fn draw_text_transformed(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (x, y, text, xscale, yscale, angle) = expect_args!(args, [real, real, any, real, real, real])?;
        self.draw_string(x, y, text.repr(), None, None, xscale, yscale, angle, None, self.draw_alpha.into());
        Ok(Default::default())
    }

    pub fn draw_text_ext_transformed(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (x, y, text, line_height, max_width, xscale, yscale, angle) =
            expect_args!(args, [real, real, any, int, int, real, real, real])?;
        let line_height = if line_height < 0 { None } else { Some(line_height as _) };
        let max_width = if max_width < 0 { None } else { Some(max_width as _) };

        self.draw_string(
            x,
            y,
            text.repr(),
            line_height,
            max_width,
            xscale,
            yscale,
            angle,
            None,
            self.draw_alpha.into(),
        );
        Ok(Default::default())
    }

    pub fn draw_text_color(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (x, y, text, col1, col2, col3, col4, alpha) =
            expect_args!(args, [real, real, any, int, int, int, int, real])?;
        self.draw_string(
            x,
            y,
            text.repr(),
            None,
            None,
            1.into(),
            1.into(),
            0.into(),
            Some((col1, col2, col3, col4)),
            alpha,
        );
        Ok(Default::default())
    }

    pub fn draw_text_transformed_color(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (x, y, text, xscale, yscale, angle, col1, col2, col3, col4, alpha) =
            expect_args!(args, [real, real, any, real, real, real, int, int, int, int, real])?;
        self.draw_string(x, y, text.repr(), None, None, xscale, yscale, angle, Some((col1, col2, col3, col4)), alpha);
        Ok(Default::default())
    }

    pub fn draw_text_ext_color(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (x, y, text, line_height, max_width, col1, col2, col3, col4, alpha) =
            expect_args!(args, [real, real, any, int, int, int, int, int, int, real])?;
        let line_height = if line_height < 0 { None } else { Some(line_height as _) };
        let max_width = if max_width < 0 { None } else { Some(max_width as _) };

        self.draw_string(
            x,
            y,
            text.repr(),
            line_height,
            max_width,
            1.into(),
            1.into(),
            0.into(),
            Some((col1, col2, col3, col4)),
            alpha,
        );
        Ok(Default::default())
    }

    pub fn draw_text_ext_transformed_color(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (x, y, text, line_height, max_width, xscale, yscale, angle, col1, col2, col3, col4, alpha) =
            expect_args!(args, [real, real, any, int, int, real, real, real, int, int, int, int, real])?;
        let line_height = if line_height < 0 { None } else { Some(line_height as _) };
        let max_width = if max_width < 0 { None } else { Some(max_width as _) };

        self.draw_string(
            x,
            y,
            text.repr(),
            line_height,
            max_width,
            xscale,
            yscale,
            angle,
            Some((col1, col2, col3, col4)),
            alpha,
        );
        Ok(Default::default())
    }

    pub fn draw_self(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        self.draw_instance_default(context.this)?;
        Ok(Default::default())
    }

    pub fn draw_sprite(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (sprite_index, image_index, x, y) = expect_args!(args, [int, int, real, real])?;
        if let Some(sprite) = self.assets.sprites.get_asset(sprite_index) {
            let image_index = if image_index < 0 {
                self.room.instance_list.get(context.this).image_index.get().floor().to_i32()
            } else {
                image_index
            };
            if let Some(atlas_ref) = sprite.get_atlas_ref(image_index) {
                self.renderer.draw_sprite(atlas_ref, x.into(), y.into(), 1.0, 1.0, 0.0, 0xffffff, 1.0);
            }
            Ok(Default::default())
        } else {
            Err(gml::Error::NonexistentAsset(asset::Type::Sprite, sprite_index))
        }
    }

    pub fn draw_sprite_pos(&mut self, _context: &mut Context, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 11
        unimplemented!("Called unimplemented kernel function draw_sprite_pos")
    }

    pub fn draw_sprite_ext(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (sprite_index, image_index, x, y, xscale, yscale, angle, colour, alpha) =
            expect_args!(args, [int, int, real, real, real, real, real, int, real])?;
        if let Some(sprite) = self.assets.sprites.get_asset(sprite_index) {
            let image_index = if image_index < 0 {
                self.room.instance_list.get(context.this).image_index.get().floor().to_i32()
            } else {
                image_index
            };
            if let Some(atlas_ref) = sprite.get_atlas_ref(image_index) {
                self.renderer.draw_sprite(
                    atlas_ref,
                    x.into(),
                    y.into(),
                    xscale.into(),
                    yscale.into(),
                    angle.into(),
                    colour,
                    alpha.into(),
                );
            }
            Ok(Default::default())
        } else {
            Err(gml::Error::NonexistentAsset(asset::Type::Sprite, sprite_index))
        }
    }

    pub fn draw_sprite_stretched(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (sprite_index, image_index, x, y, w, h) = expect_args!(args, [any, any, any, any, any, any])?;
        let instance = self.room.instance_list.get(context.this);
        let args = [
            sprite_index,
            image_index,
            x,
            y,
            w,
            h,
            instance.image_blend.get().into(),
            instance.image_alpha.get().into(),
        ];
        self.draw_sprite_stretched_ext(context, &args)
    }

    pub fn draw_sprite_stretched_ext(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (sprite_index, image_index, x, y, w, h, colour, alpha) =
            expect_args!(args, [int, int, real, real, real, real, int, real])?;
        if let Some(sprite) = self.assets.sprites.get_asset(sprite_index) {
            let image_index = if image_index < 0 {
                self.room.instance_list.get(context.this).image_index.get().floor().to_i32()
            } else {
                image_index
            };
            if let Some(atlas_ref) = sprite.get_atlas_ref(image_index) {
                self.renderer.draw_sprite(
                    atlas_ref,
                    x.into(),
                    y.into(),
                    (w / sprite.width.into()).into(),
                    (h / sprite.height.into()).into(),
                    0.0,
                    colour,
                    alpha.into(),
                );
            }
            Ok(Default::default())
        } else {
            Err(gml::Error::NonexistentAsset(asset::Type::Sprite, sprite_index))
        }
    }

    pub fn draw_sprite_part(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (sprite_index, image_index, left, top, width, height, x, y) =
            expect_args!(args, [any, any, any, any, any, any, any, any])?;
        self.draw_sprite_part_ext(context, &[
            sprite_index,
            image_index,
            left,
            top,
            width,
            height,
            x,
            y,
            1.into(),
            1.into(),
            0xFFFFFF.into(),
            1.into(),
        ])
    }

    pub fn draw_sprite_part_ext(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (sprite_index, image_index, left, top, width, height, x, y, xscale, yscale, colour, alpha) =
            expect_args!(args, [int, int, real, real, real, real, real, real, real, real, int, real])?;
        if let Some(sprite) = self.assets.sprites.get_asset(sprite_index) {
            let image_index = if image_index < 0 {
                self.room.instance_list.get(context.this).image_index.get().floor().to_i32()
            } else {
                image_index
            };
            if let Some(atlas_ref) = sprite.get_atlas_ref(image_index) {
                self.renderer.draw_sprite_partial(
                    atlas_ref,
                    left.into(),
                    top.into(),
                    width.into(),
                    height.into(),
                    x.into(),
                    y.into(),
                    xscale.into(),
                    yscale.into(),
                    0.0,
                    colour,
                    alpha.into(),
                );
            }
            Ok(Default::default())
        } else {
            Err(gml::Error::NonexistentAsset(asset::Type::Sprite, sprite_index))
        }
    }

    pub fn draw_sprite_general(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (
            sprite_index,
            image_index,
            left,
            top,
            width,
            height,
            x,
            y,
            xscale,
            yscale,
            angle,
            col1,
            col2,
            col3,
            col4,
            alpha,
        ) = expect_args!(args, [
            int, int, real, real, real, real, real, real, real, real, real, int, int, int, int, real
        ])?;
        if let Some(sprite) = self.assets.sprites.get_asset(sprite_index) {
            let image_index = if image_index < 0 {
                self.room.instance_list.get(context.this).image_index.get().floor().to_i32()
            } else {
                image_index
            };
            if let Some(atlas_ref) = sprite.get_atlas_ref(image_index) {
                self.renderer.draw_sprite_general(
                    atlas_ref,
                    left.into(),
                    top.into(),
                    width.into(),
                    height.into(),
                    x.into(),
                    y.into(),
                    xscale.into(),
                    yscale.into(),
                    angle.into(),
                    col1,
                    col2,
                    col3,
                    col4,
                    alpha.into(),
                    false,
                );
            }
            Ok(Default::default())
        } else {
            Err(gml::Error::NonexistentAsset(asset::Type::Sprite, sprite_index))
        }
    }

    pub fn draw_sprite_tiled(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (sprite_index, image_index, x, y) = expect_args!(args, [any, any, any, any])?;
        self.draw_sprite_tiled_ext(context, &[
            sprite_index,
            image_index,
            x,
            y,
            1.into(),
            1.into(),
            0xffffff.into(),
            1.into(),
        ])
    }

    pub fn draw_sprite_tiled_ext(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (sprite_index, image_index, x, y, xscale, yscale, colour, alpha) =
            expect_args!(args, [int, int, real, real, real, real, int, real])?;
        if let Some(sprite) = self.assets.sprites.get_asset(sprite_index) {
            let image_index = if image_index < 0 {
                self.room.instance_list.get(context.this).image_index.get().floor().to_i32()
            } else {
                image_index
            };
            if let Some(atlas_ref) = sprite.get_atlas_ref(image_index) {
                self.renderer.draw_sprite_tiled(
                    atlas_ref,
                    x.into(),
                    y.into(),
                    xscale.into(),
                    yscale.into(),
                    colour,
                    alpha.into(),
                    Some(self.room.width.into()),
                    Some(self.room.height.into()),
                );
            }
            Ok(Default::default())
        } else {
            Err(gml::Error::NonexistentAsset(asset::Type::Sprite, sprite_index))
        }
    }

    pub fn draw_background(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (bg_index, x, y) = expect_args!(args, [any, any, any])?;
        self.draw_background_ext(&[bg_index, x, y, 1.into(), 1.into(), 0.into(), 0xFFFFFF.into(), 1.into()])
    }

    pub fn draw_background_ext(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (bg_index, x, y, xscale, yscale, angle, colour, alpha) =
            expect_args!(args, [int, real, real, real, real, real, int, real])?;
        if let Some(background) = self.assets.backgrounds.get_asset(bg_index) {
            if let Some(atlas_ref) = &background.atlas_ref {
                self.renderer.draw_sprite(
                    atlas_ref,
                    x.into(),
                    y.into(),
                    xscale.into(),
                    yscale.into(),
                    angle.into(),
                    colour,
                    alpha.into(),
                );
            }
            Ok(Default::default())
        } else {
            Err(gml::Error::NonexistentAsset(asset::Type::Background, bg_index))
        }
    }

    pub fn draw_background_stretched(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (bg_index, x, y, w, h) = expect_args!(args, [any, any, any, any, any])?;
        self.draw_background_stretched_ext(&[bg_index, x, y, w, h, 0xffffff.into(), 1.0.into()])
    }

    pub fn draw_background_stretched_ext(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (bg_index, x, y, w, h, colour, alpha) = expect_args!(args, [int, real, real, real, real, int, real])?;
        if let Some(background) = self.assets.backgrounds.get_asset(bg_index) {
            if let Some(atlas_ref) = &background.atlas_ref {
                self.renderer.draw_sprite(
                    atlas_ref,
                    x.into(),
                    y.into(),
                    (w / background.width.into()).into(),
                    (h / background.height.into()).into(),
                    0.0,
                    colour,
                    alpha.into(),
                );
            }
            Ok(Default::default())
        } else {
            Err(gml::Error::NonexistentAsset(asset::Type::Background, bg_index))
        }
    }

    pub fn draw_background_part(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (bg_index, left, top, width, height, x, y) = expect_args!(args, [any, any, any, any, any, any, any])?;

        self.draw_background_part_ext(&[
            bg_index,
            left,
            top,
            width,
            height,
            x,
            y,
            1.into(),
            1.into(),
            0xFFFFFF.into(),
            1.into(),
        ])
    }

    pub fn draw_background_part_ext(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (bg_index, left, top, width, height, x, y, xscale, yscale, colour, alpha) =
            expect_args!(args, [int, real, real, real, real, real, real, real, real, int, real])?;
        if let Some(background) = self.assets.backgrounds.get_asset(bg_index) {
            if let Some(atlas_ref) = &background.atlas_ref {
                self.renderer.draw_sprite_partial(
                    atlas_ref,
                    left.into(),
                    top.into(),
                    width.into(),
                    height.into(),
                    x.into(),
                    y.into(),
                    xscale.into(),
                    yscale.into(),
                    0.0,
                    colour,
                    alpha.into(),
                );
            }
            Ok(Default::default())
        } else {
            Err(gml::Error::NonexistentAsset(asset::Type::Background, bg_index))
        }
    }

    pub fn draw_background_general(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (bg_index, left, top, width, height, x, y, xscale, yscale, angle, col1, col2, col3, col4, alpha) =
            expect_args!(args, [int, real, real, real, real, real, real, real, real, real, int, int, int, int, real])?;
        if let Some(background) = self.assets.backgrounds.get_asset(bg_index) {
            if let Some(atlas_ref) = &background.atlas_ref {
                self.renderer.draw_sprite_general(
                    atlas_ref,
                    left.into(),
                    top.into(),
                    width.into(),
                    height.into(),
                    x.into(),
                    y.into(),
                    xscale.into(),
                    yscale.into(),
                    angle.into(),
                    col1,
                    col2,
                    col3,
                    col4,
                    alpha.into(),
                    false,
                );
            }
            Ok(Default::default())
        } else {
            Err(gml::Error::NonexistentAsset(asset::Type::Background, bg_index))
        }
    }

    pub fn draw_background_tiled(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (bg_index, x, y) = expect_args!(args, [any, any, any])?;
        self.draw_background_tiled_ext(&[bg_index, x, y, 1.into(), 1.into(), 0xFFFFFF.into(), 1.into()])
    }

    pub fn draw_background_tiled_ext(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (bg_index, x, y, xscale, yscale, colour, alpha) =
            expect_args!(args, [int, real, real, real, real, int, real])?;
        if let Some(background) = self.assets.backgrounds.get_asset(bg_index) {
            if let Some(atlas_ref) = &background.atlas_ref {
                self.renderer.draw_sprite_tiled(
                    atlas_ref,
                    x.into(),
                    y.into(),
                    xscale.into(),
                    yscale.into(),
                    colour,
                    alpha.into(),
                    Some(self.room.width.into()),
                    Some(self.room.height.into()),
                );
            }
            Ok(Default::default())
        } else {
            Err(gml::Error::NonexistentAsset(asset::Type::Background, bg_index))
        }
    }

    pub fn tile_get_x(&self, args: &[Value]) -> gml::Result<Value> {
        let tile_id = expect_args!(args, [int])?;
        if let Some(handle) = self.room.tile_list.get_by_tileid(tile_id) {
            Ok(self.room.tile_list.get(handle).x.get().into())
        } else {
            Err(gml::Error::FunctionError("tile_get_x".into(), format!("Tile with ID {} does not exist.", tile_id)))
        }
    }

    pub fn tile_get_y(&self, args: &[Value]) -> gml::Result<Value> {
        let tile_id = expect_args!(args, [int])?;
        if let Some(handle) = self.room.tile_list.get_by_tileid(tile_id) {
            Ok(self.room.tile_list.get(handle).y.get().into())
        } else {
            Err(gml::Error::FunctionError("tile_get_y".into(), format!("Tile with ID {} does not exist.", tile_id)))
        }
    }

    pub fn tile_get_left(&self, args: &[Value]) -> gml::Result<Value> {
        let tile_id = expect_args!(args, [int])?;
        if let Some(handle) = self.room.tile_list.get_by_tileid(tile_id) {
            Ok(self.room.tile_list.get(handle).tile_x.get().into())
        } else {
            Err(gml::Error::FunctionError("tile_get_left".into(), format!("Tile with ID {} does not exist.", tile_id)))
        }
    }

    pub fn tile_get_top(&self, args: &[Value]) -> gml::Result<Value> {
        let tile_id = expect_args!(args, [int])?;
        if let Some(handle) = self.room.tile_list.get_by_tileid(tile_id) {
            Ok(self.room.tile_list.get(handle).tile_y.get().into())
        } else {
            Err(gml::Error::FunctionError("tile_get_top".into(), format!("Tile with ID {} does not exist.", tile_id)))
        }
    }

    pub fn tile_get_width(&self, args: &[Value]) -> gml::Result<Value> {
        let tile_id = expect_args!(args, [int])?;
        if let Some(handle) = self.room.tile_list.get_by_tileid(tile_id) {
            Ok(self.room.tile_list.get(handle).width.get().into())
        } else {
            Err(gml::Error::FunctionError("tile_get_width".into(), format!("Tile with ID {} does not exist.", tile_id)))
        }
    }

    pub fn tile_get_height(&self, args: &[Value]) -> gml::Result<Value> {
        let tile_id = expect_args!(args, [int])?;
        if let Some(handle) = self.room.tile_list.get_by_tileid(tile_id) {
            Ok(self.room.tile_list.get(handle).height.get().into())
        } else {
            Err(gml::Error::FunctionError(
                "tile_get_height".into(),
                format!("Tile with ID {} does not exist.", tile_id),
            ))
        }
    }

    pub fn tile_get_depth(&self, args: &[Value]) -> gml::Result<Value> {
        let tile_id = expect_args!(args, [int])?;
        if let Some(handle) = self.room.tile_list.get_by_tileid(tile_id) {
            Ok(self.room.tile_list.get(handle).depth.get().into())
        } else {
            Err(gml::Error::FunctionError("tile_get_depth".into(), format!("Tile with ID {} does not exist.", tile_id)))
        }
    }

    pub fn tile_get_visible(&self, args: &[Value]) -> gml::Result<Value> {
        let tile_id = expect_args!(args, [int])?;
        if let Some(handle) = self.room.tile_list.get_by_tileid(tile_id) {
            Ok(self.room.tile_list.get(handle).visible.get().into())
        } else {
            Err(gml::Error::FunctionError(
                "tile_get_visible".into(),
                format!("Tile with ID {} does not exist.", tile_id),
            ))
        }
    }

    pub fn tile_get_xscale(&self, args: &[Value]) -> gml::Result<Value> {
        let tile_id = expect_args!(args, [int])?;
        if let Some(handle) = self.room.tile_list.get_by_tileid(tile_id) {
            Ok(self.room.tile_list.get(handle).xscale.get().into())
        } else {
            Err(gml::Error::FunctionError(
                "tile_get_xscale".into(),
                format!("Tile with ID {} does not exist.", tile_id),
            ))
        }
    }

    pub fn tile_get_yscale(&self, args: &[Value]) -> gml::Result<Value> {
        let tile_id = expect_args!(args, [int])?;
        if let Some(handle) = self.room.tile_list.get_by_tileid(tile_id) {
            Ok(self.room.tile_list.get(handle).yscale.get().into())
        } else {
            Err(gml::Error::FunctionError(
                "tile_get_yscale".into(),
                format!("Tile with ID {} does not exist.", tile_id),
            ))
        }
    }

    pub fn tile_get_blend(&self, args: &[Value]) -> gml::Result<Value> {
        let tile_id = expect_args!(args, [int])?;
        if let Some(handle) = self.room.tile_list.get_by_tileid(tile_id) {
            Ok(self.room.tile_list.get(handle).blend.get().into())
        } else {
            Err(gml::Error::FunctionError("tile_get_blend".into(), format!("Tile with ID {} does not exist.", tile_id)))
        }
    }

    pub fn tile_get_alpha(&self, args: &[Value]) -> gml::Result<Value> {
        let tile_id = expect_args!(args, [int])?;
        if let Some(handle) = self.room.tile_list.get_by_tileid(tile_id) {
            Ok(self.room.tile_list.get(handle).alpha.get().into())
        } else {
            Err(gml::Error::FunctionError("tile_get_alpha".into(), format!("Tile with ID {} does not exist.", tile_id)))
        }
    }

    pub fn tile_get_background(&self, args: &[Value]) -> gml::Result<Value> {
        let tile_id = expect_args!(args, [int])?;
        if let Some(handle) = self.room.tile_list.get_by_tileid(tile_id) {
            Ok(self.room.tile_list.get(handle).background_index.get().into())
        } else {
            Err(gml::Error::FunctionError(
                "tile_get_background".into(),
                format!("Tile with ID {} does not exist.", tile_id),
            ))
        }
    }

    pub fn tile_set_visible(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (tile_id, visible) = expect_args!(args, [int, bool])?;
        if let Some(handle) = self.room.tile_list.get_by_tileid(tile_id) {
            self.room.tile_list.get(handle).visible.set(visible);
            Ok(Default::default())
        } else {
            Err(gml::Error::FunctionError(
                "tile_set_visible".into(),
                format!("Tile with ID {} does not exist.", tile_id),
            ))
        }
    }

    pub fn tile_set_background(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (tile_id, bg_index) = expect_args!(args, [int, int])?;
        if let Some(handle) = self.room.tile_list.get_by_tileid(tile_id) {
            self.room.tile_list.get(handle).background_index.set(bg_index);
            Ok(Default::default())
        } else {
            Err(gml::Error::FunctionError(
                "tile_set_background".into(),
                format!("Tile with ID {} does not exist.", tile_id),
            ))
        }
    }

    pub fn tile_set_region(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (tile_id, left, top, width, height) = expect_args!(args, [int, int, int, int, int])?;
        if let Some(handle) = self.room.tile_list.get_by_tileid(tile_id) {
            let tile = self.room.tile_list.get(handle);
            tile.tile_x.set(left);
            tile.tile_y.set(top);
            tile.width.set(width);
            tile.height.set(height);
            Ok(Default::default())
        } else {
            Err(gml::Error::FunctionError(
                "tile_set_region".into(),
                format!("Tile with ID {} does not exist.", tile_id),
            ))
        }
    }

    pub fn tile_set_position(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (tile_id, x, y) = expect_args!(args, [int, real, real])?;
        if let Some(handle) = self.room.tile_list.get_by_tileid(tile_id) {
            let tile = self.room.tile_list.get(handle);
            tile.x.set(x);
            tile.y.set(y);
            Ok(Default::default())
        } else {
            Err(gml::Error::FunctionError(
                "tile_set_position".into(),
                format!("Tile with ID {} does not exist.", tile_id),
            ))
        }
    }

    pub fn tile_set_depth(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (tile_id, depth) = expect_args!(args, [int, real])?;
        if let Some(handle) = self.room.tile_list.get_by_tileid(tile_id) {
            self.room.tile_list.get(handle).depth.set(depth);
            Ok(Default::default())
        } else {
            Err(gml::Error::FunctionError("tile_set_depth".into(), format!("Tile with ID {} does not exist.", tile_id)))
        }
    }

    pub fn tile_set_scale(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (tile_id, xscale, yscale) = expect_args!(args, [int, real, real])?;
        if let Some(handle) = self.room.tile_list.get_by_tileid(tile_id) {
            let tile = self.room.tile_list.get(handle);
            tile.xscale.set(xscale);
            tile.yscale.set(yscale);
            Ok(Default::default())
        } else {
            Err(gml::Error::FunctionError("tile_set_scale".into(), format!("Tile with ID {} does not exist.", tile_id)))
        }
    }

    pub fn tile_set_blend(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (tile_id, blend) = expect_args!(args, [int, int])?;
        if let Some(handle) = self.room.tile_list.get_by_tileid(tile_id) {
            self.room.tile_list.get(handle).blend.set(blend);
            Ok(Default::default())
        } else {
            Err(gml::Error::FunctionError("tile_set_blend".into(), format!("Tile with ID {} does not exist.", tile_id)))
        }
    }

    pub fn tile_set_alpha(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (tile_id, alpha) = expect_args!(args, [int, real])?;
        if let Some(handle) = self.room.tile_list.get_by_tileid(tile_id) {
            self.room.tile_list.get(handle).alpha.set(alpha);
            Ok(Default::default())
        } else {
            Err(gml::Error::FunctionError("tile_set_alpha".into(), format!("Tile with ID {} does not exist.", tile_id)))
        }
    }

    pub fn tile_add(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (background_index, tile_x, tile_y, width, height, x, y, depth) =
            expect_args!(args, [int, int, int, int, int, real, real, real])?;
        self.last_tile_id += 1;
        self.room.tile_list.insert(Tile {
            x: x.into(),
            y: y.into(),
            background_index: background_index.into(),
            tile_x: tile_x.into(),
            tile_y: tile_y.into(),
            width: width.into(),
            height: height.into(),
            depth: depth.into(),
            id: self.last_tile_id.into(),
            alpha: Real::from(1.0).into(),
            blend: 0xffffff.into(),
            xscale: Real::from(1.0).into(),
            yscale: Real::from(1.0).into(),
            visible: true.into(),
        });
        Ok(self.last_tile_id.into())
    }

    pub fn tile_find(&self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 3
        unimplemented!("Called unimplemented kernel function tile_find")
    }

    pub fn tile_exists(&self, args: &[Value]) -> gml::Result<Value> {
        let tile_id = expect_args!(args, [int])?;
        Ok(self.room.tile_list.get_by_tileid(tile_id).is_some().into())
    }

    pub fn tile_delete(&mut self, args: &[Value]) -> gml::Result<Value> {
        let tile_id = expect_args!(args, [int])?;
        if let Some(handle) = self.room.tile_list.get_by_tileid(tile_id) {
            self.room.tile_list.remove(handle);
            Ok(Default::default())
        } else {
            Err(gml::Error::FunctionError("tile_delete".into(), format!("Tile with ID {} does not exist.", tile_id)))
        }
    }

    pub fn tile_delete_at(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 3
        unimplemented!("Called unimplemented kernel function tile_delete_at")
    }

    pub fn tile_layer_hide(&mut self, args: &[Value]) -> gml::Result<Value> {
        let depth = expect_args!(args, [real])?;
        let mut iter_tile = self.room.tile_list.iter_by_drawing();
        while let Some(handle) = iter_tile.next(&self.room.tile_list) {
            let tile = self.room.tile_list.get(handle);
            if tile.depth.get() == depth {
                tile.visible.set(false);
            }
        }
        Ok(Default::default())
    }

    pub fn tile_layer_show(&mut self, args: &[Value]) -> gml::Result<Value> {
        let depth = expect_args!(args, [real])?;
        let mut iter_tile = self.room.tile_list.iter_by_drawing();
        while let Some(handle) = iter_tile.next(&self.room.tile_list) {
            let tile = self.room.tile_list.get(handle);
            if tile.depth.get() == depth {
                tile.visible.set(true);
            }
        }
        Ok(Default::default())
    }

    pub fn tile_layer_delete(&mut self, args: &[Value]) -> gml::Result<Value> {
        let depth = expect_args!(args, [real])?;
        self.room.tile_list.remove_with(|t| t.depth.get() == depth);
        Ok(Default::default())
    }

    pub fn tile_layer_shift(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (depth, x, y) = expect_args!(args, [real, real, real])?;
        let mut iter_tile = self.room.tile_list.iter_by_drawing();
        while let Some(handle) = iter_tile.next(&self.room.tile_list) {
            let tile = self.room.tile_list.get(handle);
            if tile.depth.get() == depth {
                tile.x.set(tile.x.get() + x);
                tile.y.set(tile.y.get() + y);
            }
        }
        Ok(Default::default())
    }

    pub fn tile_layer_find(&self, args: &[Value]) -> gml::Result<Value> {
        let (depth, x, y) = expect_args!(args, [real, real, real])?;
        let use_scaling = self.gm_version == Version::GameMaker8_1; // 8.1 bugfix
        let mut iter_tile = self.room.tile_list.iter_by_drawing();
        while let Some(handle) = iter_tile.next(&self.room.tile_list) {
            let tile = self.room.tile_list.get(handle);
            if tile.depth.get() == depth
                && x >= tile.x.get()
                && x < tile.x.get() + if use_scaling { tile.xscale.get() } else { 0.into() } * tile.width.get().into()
                && y >= tile.y.get()
                && y < tile.y.get() + if use_scaling { tile.yscale.get() } else { 0.into() } * tile.height.get().into()
            {
                return Ok(tile.id.get().into())
            }
        }
        Ok((-1).into())
    }

    pub fn tile_layer_delete_at(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (depth, x, y) = expect_args!(args, [real, real, real])?;
        let use_scaling = self.gm_version == Version::GameMaker8_1; // 8.1 bugfix
        self.room.tile_list.remove_with(|tile| {
            tile.depth.get() == depth
                && x >= tile.x.get()
                && x < tile.x.get() + if use_scaling { tile.xscale.get() } else { 0.into() } * tile.width.get().into()
                && y >= tile.y.get()
                && y < tile.y.get() + if use_scaling { tile.yscale.get() } else { 0.into() } * tile.height.get().into()
        });
        Ok(Default::default())
    }

    pub fn tile_layer_depth(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (old_depth, new_depth) = expect_args!(args, [real, real])?;
        let mut iter_tile = self.room.tile_list.iter_by_drawing();
        while let Some(handle) = iter_tile.next(&self.room.tile_list) {
            let tile = self.room.tile_list.get(handle);
            if tile.depth.get() == old_depth {
                tile.depth.set(new_depth);
            }
        }
        Ok(Default::default())
    }

    pub fn surface_create(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (w, h) = expect_args!(args, [int, int])?;
        let make_zbuf = self.gm_version == Version::GameMaker8_1 || self.surface_fix;
        let surf = Surface {
            width: w as _,
            height: h as _,
            atlas_ref: match self.renderer.create_surface(w, h, make_zbuf) {
                Ok(atl_ref) => atl_ref,
                Err(e) => return Err(gml::Error::FunctionError("surface_create".into(), e.into())),
            },
        };
        if let Some(id) = self.surfaces.iter().position(|x| x.is_none()) {
            self.surfaces[id] = Some(surf);
            Ok(id.into())
        } else {
            self.surfaces.push(Some(surf));
            Ok((self.surfaces.len() - 1).into())
        }
    }

    pub fn surface_create_ext(args: &[Value]) -> gml::Result<Value> {
        // Special GM 8.1 function, has effect only in the original HTML5 runner.
        expect_args!(args, [any, any])?;
        Ok((-1).into())
    }

    pub fn surface_free(&mut self, args: &[Value]) -> gml::Result<Value> {
        let surf_id = expect_args!(args, [int])?;
        if self.surface_target == Some(surf_id) {
            self.surface_reset_target(&[])?;
        }
        if let Some(surf) = self.surfaces.get_asset(surf_id) {
            self.renderer.delete_sprite(surf.atlas_ref);
            self.surfaces[surf_id as usize] = None;
        }
        Ok(Default::default())
    }

    pub fn surface_exists(&self, args: &[Value]) -> gml::Result<Value> {
        let surf_id = expect_args!(args, [int])?;
        Ok(self.surfaces.get_asset(surf_id).is_some().into())
    }

    pub fn surface_get_width(&self, args: &[Value]) -> gml::Result<Value> {
        let surf_id = expect_args!(args, [int])?;
        if let Some(surf) = self.surfaces.get_asset(surf_id) { Ok(surf.width.into()) } else { Ok((-1).into()) }
    }

    pub fn surface_get_height(&self, args: &[Value]) -> gml::Result<Value> {
        let surf_id = expect_args!(args, [int])?;
        if let Some(surf) = self.surfaces.get_asset(surf_id) { Ok(surf.height.into()) } else { Ok((-1).into()) }
    }

    pub fn surface_get_texture(&mut self, args: &[Value]) -> gml::Result<Value> {
        let surf_id = expect_args!(args, [int])?;
        if let Some(surf) = self.surfaces.get_asset(surf_id) {
            Ok(self.renderer.get_texture_id(&surf.atlas_ref).into())
        } else {
            Ok((-1).into())
        }
    }

    pub fn surface_set_target(&mut self, args: &[Value]) -> gml::Result<Value> {
        let surf_id = expect_args!(args, [int])?;
        if let Some(surf) = self.surfaces.get_asset(surf_id) {
            self.renderer.set_target(&surf.atlas_ref);
            self.surface_target = Some(surf_id);
            if self.surface_fix && self.room.views_enabled {
                let view = &self.room.views[self.view_current];
                // would probably be good to make this its own method in the renderer
                if self.renderer.get_3d() && self.renderer.get_perspective() {
                    self.renderer.set_projection_perspective(
                        view.source_x.into(),
                        view.source_y.into(),
                        surf.width.into(),
                        surf.height.into(),
                        view.angle.into(),
                    );
                } else {
                    self.renderer.set_projection_ortho(
                        view.source_x.into(),
                        view.source_y.into(),
                        surf.width.into(),
                        surf.height.into(),
                        view.angle.into(),
                    );
                }
            }
        }
        Ok(Default::default())
    }

    pub fn surface_reset_target(&mut self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        // reset viewport to top left of room because lol
        self.renderer.reset_target();
        self.surface_target = None;
        if self.gm_version == Version::GameMaker8_0 {
            self.renderer.set_zbuf_trashed(!self.surface_fix);
        }
        if self.surface_fix && self.room.views_enabled {
            let view = &self.room.views[self.view_current];
            self.renderer.set_view(
                view.source_x,
                view.source_y,
                view.source_w,
                view.source_h,
                view.angle.into(),
                view.port_x,
                view.port_y,
                view.port_w as _,
                view.port_h as _,
            );
        }
        Ok(Default::default())
    }

    pub fn draw_surface(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (surf_id, x, y) = expect_args!(args, [any, any, any])?;
        self.draw_surface_ext(&[surf_id, x, y, 1.into(), 1.into(), 0.into(), 0xffffff.into(), 1.into()])
    }

    pub fn draw_surface_ext(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (surf_id, x, y, xscale, yscale, rot, colour, alpha) =
            expect_args!(args, [int, real, real, real, real, real, int, real])?;
        if let Some(surf) = self.surfaces.get_asset(surf_id) {
            self.renderer.draw_sprite(
                &surf.atlas_ref,
                x.into(),
                y.into(),
                xscale.into(),
                yscale.into(),
                rot.into(),
                colour,
                alpha.into(),
            );
        }
        Ok(Default::default())
    }

    pub fn draw_surface_stretched(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (surf_id, x, y, w, h) = expect_args!(args, [int, any, any, real, real])?;
        if let Some(surf) = self.surfaces.get_asset(surf_id) {
            let xscale = w / surf.width.into();
            let yscale = h / surf.height.into();
            self.draw_surface_ext(&[
                surf_id.into(),
                x,
                y,
                xscale.into(),
                yscale.into(),
                0.into(),
                0xffffff.into(),
                1.into(),
            ])
        } else {
            Ok(Default::default())
        }
    }

    pub fn draw_surface_stretched_ext(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (surf_id, x, y, w, h, colour, alpha) = expect_args!(args, [int, any, any, real, real, any, any])?;
        if let Some(surf) = self.surfaces.get_asset(surf_id) {
            let xscale = w / surf.width.into();
            let yscale = h / surf.height.into();
            self.draw_surface_ext(&[surf_id.into(), x, y, xscale.into(), yscale.into(), 0.into(), colour, alpha])
        } else {
            Ok(Default::default())
        }
    }

    pub fn draw_surface_part(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (surf_id, l, t, w, h, x, y) = expect_args!(args, [any, any, any, any, any, any, any])?;
        self.draw_surface_part_ext(&[surf_id, l, t, w, h, x, y, 1.into(), 1.into(), 0xffffff.into(), 1.into()])
    }

    pub fn draw_surface_part_ext(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (surf_id, l, t, w, h, x, y, xscale, yscale, colour, alpha) =
            expect_args!(args, [int, real, real, real, real, real, real, real, real, int, real])?;
        if let Some(surf) = self.surfaces.get_asset(surf_id) {
            self.renderer.draw_sprite_partial(
                &surf.atlas_ref,
                l.into(),
                t.into(),
                w.into(),
                h.into(),
                x.into(),
                y.into(),
                xscale.into(),
                yscale.into(),
                0.0,
                colour,
                alpha.into(),
            );
        }
        Ok(Default::default())
    }

    pub fn draw_surface_general(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (surf_id, l, t, w, h, x, y, xscale, yscale, angle, col1, col2, col3, col4, alpha) =
            expect_args!(args, [int, real, real, real, real, real, real, real, real, real, int, int, int, int, real])?;
        if let Some(surf) = self.surfaces.get_asset(surf_id) {
            self.renderer.draw_sprite_general(
                &surf.atlas_ref,
                l.into(),
                t.into(),
                w.into(),
                h.into(),
                x.into(),
                y.into(),
                xscale.into(),
                yscale.into(),
                angle.into(),
                col1,
                col2,
                col3,
                col4,
                alpha.into(),
                false,
            );
        }
        Ok(Default::default())
    }

    pub fn draw_surface_tiled(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (surf_id, x, y) = expect_args!(args, [any, any, any])?;
        self.draw_surface_tiled_ext(&[surf_id, x, y, 1.into(), 1.into(), 0xFFFFFF.into(), 1.into()])
    }

    pub fn draw_surface_tiled_ext(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (surf_id, x, y, xscale, yscale, colour, alpha) =
            expect_args!(args, [int, real, real, real, real, int, real])?;
        if let Some(surf) = self.surfaces.get_asset(surf_id) {
            self.renderer.draw_sprite_tiled(
                &surf.atlas_ref,
                x.into(),
                y.into(),
                xscale.into(),
                yscale.into(),
                colour,
                alpha.into(),
                Some(self.room.width.into()),
                Some(self.room.height.into()),
            );
        }
        Ok(Default::default())
    }

    pub fn surface_save(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (surf_id, fname) = expect_args!(args, [int, string])?;
        if Some(surf_id) == self.surface_target {
            self.renderer.flush_queue();
        }
        if let Some(surf) = self.surfaces.get_asset(surf_id) {
            let mut image =
                RgbaImage::from_vec(surf.width, surf.height, self.renderer.dump_sprite(&surf.atlas_ref).into())
                    .unwrap();
            asset::sprite::process_image(&mut image, false, false, true);
            match file::save_image(fname.as_ref(), image) {
                Ok(()) => Ok(Default::default()),
                Err(e) => Err(gml::Error::FunctionError("surface_save".into(), e.to_string())),
            }
        } else {
            Ok(Default::default())
        }
    }

    pub fn surface_save_part(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (surf_id, fname, x, y, w, h) = expect_args!(args, [int, string, int, int, int, int])?;
        if Some(surf_id) == self.surface_target {
            self.renderer.flush_queue();
        }
        if let Some(surf) = self.surfaces.get_asset(surf_id) {
            let x = x.max(0);
            let y = y.max(0);
            let w = w.min(surf.width as i32 - x);
            let h = h.min(surf.height as i32 - y);
            let mut image =
                RgbaImage::from_vec(w as _, h as _, self.renderer.dump_sprite_part(&surf.atlas_ref, x, y, w, h).into())
                    .unwrap();
            asset::sprite::process_image(&mut image, false, false, true);
            match file::save_image(fname.as_ref(), image) {
                Ok(()) => Ok(Default::default()),
                Err(e) => Err(gml::Error::FunctionError("surface_save_part".into(), e.to_string())),
            }
        } else {
            Ok(Default::default())
        }
    }

    pub fn surface_getpixel(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 3
        unimplemented!("Called unimplemented kernel function surface_getpixel")
    }

    pub fn surface_copy(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (dest_id, x, y, src_id) = expect_args!(args, [int, int, int, int])?;
        if let (Some(src), Some(dst)) = (self.surfaces.get_asset(src_id), self.surfaces.get_asset(dest_id)) {
            self.renderer.copy_surface(&dst.atlas_ref, x, y, &src.atlas_ref, 0, 0, src.width as _, src.height as _);
        }
        Ok(Default::default())
    }

    pub fn surface_copy_part(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (dest_id, dest_x, dest_y, src_id, src_x, src_y, width, height) =
            expect_args!(args, [int, int, int, int, int, int, int, int])?;
        if let (Some(src), Some(dst)) = (self.surfaces.get_asset(src_id), self.surfaces.get_asset(dest_id)) {
            self.renderer.copy_surface(&dst.atlas_ref, dest_x, dest_y, &src.atlas_ref, src_x, src_y, width, height);
        }
        Ok(Default::default())
    }

    pub fn action_path_old(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 3
        unimplemented!("Called unimplemented kernel function action_path_old")
    }

    pub fn action_set_sprite(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (sprite, scale) = expect_args!(args, [int, real])?;
        let instance = self.room.instance_list.get(context.this);
        instance.sprite_index.set(sprite);
        instance.image_xscale.set(scale);
        instance.image_yscale.set(scale);
        Ok(Default::default())
    }

    pub fn action_draw_font(&mut self, _context: &mut Context, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        unimplemented!("Called unimplemented kernel function action_draw_font")
    }

    pub fn action_draw_font_old(&mut self, _context: &mut Context, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 6
        unimplemented!("Called unimplemented kernel function action_draw_font_old")
    }

    pub fn action_fill_color(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        unimplemented!("Called unimplemented kernel function action_fill_color")
    }

    pub fn action_line_color(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        unimplemented!("Called unimplemented kernel function action_line_color")
    }

    pub fn action_highscore(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 0
        unimplemented!("Called unimplemented kernel function action_highscore")
    }

    pub fn action_move(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (dir_string, speed) = expect_args!(args, [bytes, real])?;
        let instance = self.room.instance_list.get(context.this);
        // dir_string is typically something like "000000100" indicating which of the 9 direction buttons were pressed.
        let bytes = dir_string.as_ref();
        if bytes.len() != 9 {
            return Err(gml::Error::FunctionError(
                "action_move".into(),
                format!("Invalid argument to action_move: {}", dir_string),
            ))
        }

        // Only invoke RNG if at least one of the options is checked, otherwise don't do anything
        if bytes.contains(&49) {
            // Call irandom until it lands on a byte that's '1' rather than '0'
            let offset = loop {
                let index = self.rand.next_int(8) as usize;
                if bytes[index] == 49 {
                    // '1'
                    break index
                }
            };

            // Handle each case separately
            let (speed, direction): (Real, Real) = match offset {
                0 => (speed, Real::from(225.0)),
                1 => (speed, Real::from(270.0)),
                2 => (speed, Real::from(315.0)),
                3 => (speed, Real::from(180.0)),
                4 => (Real::from(0.0), Real::from(0.0)),
                5 => (speed, Real::from(0.0)),
                6 => (speed, Real::from(135.0)),
                7 => (speed, Real::from(90.0)),
                8 => (speed, Real::from(45.0)),
                _ => unreachable!(),
            };
            if context.relative {
                instance.set_hspeed(direction.to_radians().cos() * speed + instance.hspeed.get());
                instance.set_vspeed(-direction.to_radians().sin() * speed + instance.vspeed.get());
            } else {
                instance.set_speed_direction(speed, direction);
            }
        }

        Ok(Default::default())
    }

    pub fn action_set_motion(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (direction, speed) = expect_args!(args, [real, real])?;
        let instance = self.room.instance_list.get(context.this);
        if context.relative {
            instance.set_hspeed(direction.to_radians().cos() * speed + instance.hspeed.get());
            instance.set_vspeed(-direction.to_radians().sin() * speed + instance.vspeed.get());
        } else {
            instance.set_speed_direction(speed, direction);
        }
        Ok(Default::default())
    }

    pub fn action_set_hspeed(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [real]).map(|x| {
            let instance = self.room.instance_list.get(context.this);
            if context.relative {
                instance.set_hspeed(x + instance.hspeed.get());
            } else {
                instance.set_hspeed(x);
            }
            Ok(Default::default())
        })?
    }

    pub fn action_set_vspeed(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [real]).map(|x| {
            let instance = self.room.instance_list.get(context.this);
            if context.relative {
                instance.set_vspeed(x + instance.vspeed.get());
            } else {
                instance.set_vspeed(x);
            }
            Ok(Default::default())
        })?
    }

    pub fn action_set_gravity(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [real, real]).map(|(direction, gravity)| {
            let instance = self.room.instance_list.get(context.this);
            if context.relative {
                instance.gravity.set(gravity + instance.gravity.get());
                instance.gravity_direction.set(direction + instance.gravity.get());
            } else {
                instance.gravity.set(gravity);
                instance.gravity_direction.set(direction);
            }
        })?;
        Ok(Default::default())
    }

    pub fn action_set_friction(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [real]).map(|x| {
            let instance = self.room.instance_list.get(context.this);
            if context.relative {
                instance.friction.set(x + instance.friction.get());
            } else {
                instance.friction.set(x);
            }
            Ok(Default::default())
        })?
    }

    pub fn action_move_point(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (x, y, speed) = expect_args!(args, [real, real, real])?;
        let instance = self.room.instance_list.get(context.this);
        let speed = if context.relative { instance.speed.get() + speed } else { speed };
        let direction = (instance.y.get() - y).arctan2(x - instance.x.get()).to_degrees();
        instance.set_speed_direction(speed, direction);
        Ok(Default::default())
    }

    pub fn action_move_to(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (x, y) = expect_args!(args, [real, real])?;
        let instance = self.room.instance_list.get(context.this);
        let (x, y) = if context.relative { (instance.x.get() + x, instance.y.get() + y) } else { (x, y) };
        instance.x.set(x);
        instance.y.set(y);
        instance.bbox_is_stale.set(true);
        Ok(Default::default())
    }

    pub fn action_move_start(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        let instance = self.room.instance_list.get(context.this);
        instance.x.set(instance.xstart.get());
        instance.y.set(instance.ystart.get());
        instance.bbox_is_stale.set(true);
        Ok(Default::default())
    }

    pub fn action_wrap(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (horizontal, vertical) = match expect_args!(args, [int])? {
            0 => (true, false),
            1 => (false, true),
            _ => (true, true),
        };

        let instance = self.room.instance_list.get(context.this);
        // Get sprite width/height, as these are used to decide how far to wrap
        let (w, h) = if let Some(Some(sprite)) = self.assets.sprites.get(instance.sprite_index.get() as usize) {
            (
                Real::from(sprite.width) * instance.image_xscale.get(),
                Real::from(sprite.height) * instance.image_yscale.get(),
            )
        } else {
            (Real::from(0.0), Real::from(0.0))
        };

        if horizontal {
            let room_width = Real::from(self.room.width);
            if instance.hspeed.get() > Real::from(0.0) && instance.x.get() > room_width {
                // Wrap x right-to-left
                instance.x.set(instance.x.get() - (room_width + w));
            }
            if instance.hspeed.get() < Real::from(0.0) && instance.x.get() < Real::from(0.0) {
                // Wrap x left-to-right
                instance.x.set(instance.x.get() + (room_width + w));
            }
        }
        if vertical {
            let room_height = Real::from(self.room.height);
            if instance.vspeed.get() > Real::from(0.0) && instance.y.get() > room_height {
                // Wrap y bottom-to-top
                instance.y.set(instance.y.get() - (room_height + h));
            }
            if instance.vspeed.get() < Real::from(0.0) && instance.y.get() < Real::from(0.0) {
                // Wrap y top-to-bottom
                instance.y.set(instance.y.get() + (room_height + h));
            }
        }
        Ok(Default::default())
    }

    pub fn action_reverse_xdir(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        let instance = self.room.instance_list.get(context.this);
        instance.set_hspeed(-instance.hspeed.get());
        Ok(Default::default())
    }

    pub fn action_reverse_ydir(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        let instance = self.room.instance_list.get(context.this);
        instance.set_vspeed(-instance.vspeed.get());
        Ok(Default::default())
    }

    pub fn action_move_contact(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (direction, max_distance, kind) = expect_args!(args, [any, any, int])?;
        if kind == 0 {
            self.move_contact_solid(context, &[direction, max_distance])
        } else {
            self.move_contact_all(context, &[direction, max_distance])
        }
    }

    pub fn action_bounce(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (advanced, kind) = expect_args!(args, [any, int])?;
        if kind == 0 {
            self.move_bounce_solid(context, &[advanced])
        } else {
            self.move_bounce_all(context, &[advanced])
        }
    }

    pub fn action_path_position(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let position = expect_args!(args, [real])?;
        let instance = self.room.instance_list.get(context.this);
        if context.relative {
            instance.path_position.set(position + instance.path_position.get());
        } else {
            instance.path_position.set(position);
        }
        Ok(Default::default())
    }

    pub fn action_path_speed(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let speed = expect_args!(args, [real])?;
        let instance = self.room.instance_list.get(context.this);
        if context.relative {
            instance.path_speed.set(speed + instance.path_speed.get());
        } else {
            instance.path_speed.set(speed);
        }
        Ok(Default::default())
    }

    pub fn action_linear_step(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (x, y, step_size, checkall) = expect_args!(args, [real, real, real, bool])?;
        let instance = self.room.instance_list.get(context.this);
        let (x, y) = if context.relative { (instance.x.get() + x, instance.y.get() + y) } else { (x, y) };
        Ok(pathfinding::linear_step(x, y, step_size, instance, || {
            if checkall {
                self.check_collision_any(context.this).is_some()
            } else {
                self.check_collision_solid(context.this).is_some()
            }
        })
        .into())
    }

    pub fn action_potential_step(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (x, y, step_size, checkall) = expect_args!(args, [real, real, real, bool])?;
        let instance = self.room.instance_list.get(context.this);
        let (x, y) = if context.relative { (instance.x.get() + x, instance.y.get() + y) } else { (x, y) };
        Ok(pathfinding::potential_step(x, y, step_size, &self.potential_step_settings, instance, || {
            if checkall {
                self.check_collision_any(context.this).is_some()
            } else {
                self.check_collision_solid(context.this).is_some()
            }
        })
        .into())
    }

    pub fn action_create_object(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (object_id, x, y) = expect_args!(args, [int, real, real])?;
        if let Some(Some(object)) = self.assets.objects.get(object_id as usize) {
            let (relative_x, relative_y) = if context.relative {
                let instance = self.room.instance_list.get(context.this);
                (instance.x.get(), instance.y.get())
            } else {
                (Real::from(0.0), Real::from(0.0))
            };
            self.last_instance_id += 1;
            let instance = self.room.instance_list.insert(Instance::new(
                self.last_instance_id,
                x + relative_x,
                y + relative_y,
                object_id,
                object,
            ));
            self.run_instance_event(gml::ev::CREATE, 0, instance, instance, None)?;
            Ok(Default::default())
        } else {
            Err(gml::Error::FunctionError("action_create_object".into(), format!("Invalid object ID: {}", object_id)))
        }
    }

    pub fn action_create_object_motion(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (object_id, x, y, speed, direction) = expect_args!(args, [int, real, real, real, real])?;
        if let Some(Some(object)) = self.assets.objects.get(object_id as usize) {
            let (relative_x, relative_y) = if context.relative {
                let instance = self.room.instance_list.get(context.this);
                (instance.x.get(), instance.y.get())
            } else {
                (Real::from(0.0), Real::from(0.0))
            };
            self.last_instance_id += 1;
            let instance = self.room.instance_list.insert(Instance::new(
                self.last_instance_id,
                x + relative_x,
                y + relative_y,
                object_id,
                object,
            ));
            self.room.instance_list.get(instance).set_speed_direction(speed, direction);
            self.run_instance_event(gml::ev::CREATE, 0, instance, instance, None)?;
            Ok(Default::default())
        } else {
            Err(gml::Error::FunctionError(
                "action_create_object_motion".into(),
                format!("Invalid object ID: {}", object_id),
            ))
        }
    }

    pub fn action_create_object_random(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (obj1, obj2, obj3, obj4, x, y) = expect_args!(args, [int, int, int, int, real, real])?;
        let (x, y) = if context.relative {
            let instance = self.room.instance_list.get(context.this);
            ((instance.x.get() + x.into()).into(), (instance.y.get() + y.into()).into())
        } else {
            (x, y)
        };
        let object_ids = [obj1, obj2, obj3, obj4];
        if object_ids.iter().any(|&id| self.assets.objects.get_asset(id).is_some()) {
            let (object_id, object) = loop {
                let i = self.rand.next_int(3) as usize;
                if let Some(object) = self.assets.objects.get_asset(object_ids[i]) {
                    break (object_ids[i], object)
                }
            };
            self.last_instance_id += 1;
            let id = self.last_instance_id;
            let instance = self.room.instance_list.insert(Instance::new(id, x, y, object_id, object));
            self.run_instance_event(gml::ev::CREATE, 0, instance, instance, None)?;
        }
        Ok(Default::default())
    }

    pub fn action_kill_position(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (x, y) = expect_args!(args, [any, any])?;
        let (x, y) = if context.relative {
            let instance = self.room.instance_list.get(context.this);
            ((instance.x.get() + x.into()).into(), (instance.y.get() + y.into()).into())
        } else {
            (x, y)
        };
        self.position_destroy(&[x, y])
    }

    pub fn action_sprite_set(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (sprite_id, image_index, image_speed) = expect_args!(args, [int, real, real])?;
        let instance = self.room.instance_list.get(context.this);
        instance.bbox_is_stale.set(true);
        instance.sprite_index.set(sprite_id);
        instance.image_index.set(image_index);
        instance.image_speed.set(image_speed);
        Ok(Default::default())
    }

    pub fn action_sprite_transform(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (mut xsc, mut ysc, ang, mirroring) = expect_args!(args, [real, real, real, int])?;
        let instance = self.room.instance_list.get(context.this);
        let (hmirr, vmirr) = match mirroring {
            1 => (true, false),
            2 => (false, true),
            3 => (true, true),
            0 | _ => (false, false),
        };
        if hmirr {
            xsc = -xsc;
        }
        if vmirr {
            ysc = -ysc;
        }
        instance.image_xscale.set(xsc);
        instance.image_yscale.set(ysc);
        instance.image_angle.set(ang);
        Ok(Default::default())
    }

    pub fn action_sprite_color(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (col, alpha) = expect_args!(args, [int, real])?;
        let instance = self.room.instance_list.get(context.this);
        instance.image_blend.set(col);
        instance.image_alpha.set(alpha);
        Ok(Default::default())
    }

    pub fn action_sound(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (sound_id, do_loop) = expect_args!(args, [any, bool])?;
        if do_loop { self.sound_loop(&[sound_id]) } else { self.sound_play(&[sound_id]) }
    }

    pub fn action_if_sound(&self, args: &[Value]) -> gml::Result<Value> {
        self.sound_isplaying(args)
    }

    pub fn action_another_room(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (room_id, transition_id) = expect_args!(args, [any, int])?;
        self.transition_kind = transition_id;
        self.room_goto(&[room_id])
    }

    pub fn action_current_room(&mut self, args: &[Value]) -> gml::Result<Value> {
        self.transition_kind = expect_args!(args, [int])?;
        self.room_restart(&[])
    }

    pub fn action_previous_room(&mut self, args: &[Value]) -> gml::Result<Value> {
        self.transition_kind = expect_args!(args, [int])?;
        self.room_goto_previous(&[])
    }

    pub fn action_next_room(&mut self, args: &[Value]) -> gml::Result<Value> {
        self.transition_kind = expect_args!(args, [int])?;
        self.room_goto_next(&[])
    }

    pub fn action_if_previous_room(&self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        match self.room_order.first() {
            Some(&room_id) => Ok((room_id != self.room.id).into()),
            None => Err(gml::Error::EndOfRoomOrder),
        }
    }

    pub fn action_if_next_room(&self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        match self.room_order.last() {
            Some(&room_id) => Ok((room_id != self.room.id).into()),
            None => Err(gml::Error::EndOfRoomOrder),
        }
    }

    pub fn action_set_alarm(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (time, alarm) = expect_args!(args, [int, int])?;
        let mut alarms = self.room.instance_list.get(context.this).alarms.borrow_mut();
        let time = if context.relative { time + alarms.get(&(alarm as u32)).copied().unwrap_or(-1) } else { time };
        alarms.insert(alarm as u32, time);
        Ok(Default::default())
    }

    pub fn action_sleep(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (millis, redraw) = expect_args!(args, [any, bool])?;
        if redraw {
            self.screen_redraw(&[])?;
        }
        self.sleep(&[millis])
    }

    pub fn action_set_timeline(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (index, position) = expect_args!(args, [int, real])?;
        let instance = self.room.instance_list.get(context.this);
        instance.timeline_index.set(index);
        instance.timeline_position.set(position);
        instance.timeline_running.set(true);
        Ok(Default::default())
    }

    pub fn action_timeline_set(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (index, position, start_option, loop_option) = expect_args!(args, [int, real, int, int])?;
        let instance = self.room.instance_list.get(context.this);
        instance.timeline_index.set(index);
        instance.timeline_position.set(position);
        instance.timeline_running.set(start_option == 0);
        instance.timeline_loop.set(loop_option == 1);
        Ok(Default::default())
    }

    pub fn action_timeline_start(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        self.room.instance_list.get(context.this).timeline_running.set(true);
        Ok(Default::default())
    }

    pub fn action_timeline_pause(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        self.room.instance_list.get(context.this).timeline_running.set(false);
        Ok(Default::default())
    }

    pub fn action_timeline_stop(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        let instance = self.room.instance_list.get(context.this);
        instance.timeline_position.set(Real::from(0.0));
        instance.timeline_running.set(false);
        Ok(Default::default())
    }

    pub fn action_set_timeline_position(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let position = expect_args!(args, [real])?;
        let instance = self.room.instance_list.get(context.this);
        if context.relative {
            instance.timeline_position.set(instance.timeline_position.get() + position);
        } else {
            instance.timeline_position.set(position);
        }
        Ok(Default::default())
    }

    pub fn action_set_timeline_speed(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let speed = expect_args!(args, [real])?;
        let instance = self.room.instance_list.get(context.this);
        if context.relative {
            instance.timeline_speed.set(instance.timeline_speed.get() + speed);
        } else {
            instance.timeline_speed.set(speed);
        }
        Ok(Default::default())
    }

    pub fn action_message(&mut self, args: &[Value]) -> gml::Result<Value> {
        self.show_message(&[match expect_args!(args, [any])? {
            v @ Value::Str(_) => v,
            _ => "".into(),
        }])
    }

    pub fn action_splash_text(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        unimplemented!("Called unimplemented kernel function action_splash_text")
    }

    pub fn action_splash_image(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        unimplemented!("Called unimplemented kernel function action_splash_image")
    }

    pub fn action_splash_web(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 2
        unimplemented!("Called unimplemented kernel function action_splash_web")
    }

    pub fn action_splash_settings(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 5
        unimplemented!("Called unimplemented kernel function action_splash_settings")
    }

    pub fn action_replace_sprite(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 3
        unimplemented!("Called unimplemented kernel function action_replace_sprite")
    }

    pub fn action_replace_sound(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 2
        unimplemented!("Called unimplemented kernel function action_replace_sound")
    }

    pub fn action_replace_background(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 2
        unimplemented!("Called unimplemented kernel function action_replace_background")
    }

    pub fn action_if_empty(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        Ok((!self.action_if_collision(context, args)?.is_truthy()).into())
    }

    pub fn action_if_collision(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (mut x, mut y, collision) = expect_args!(args, [real, real, bool])?;
        if context.relative {
            let instance = self.room.instance_list.get(context.this);
            x += instance.x.get();
            y += instance.y.get();
        }
        Ok((!if collision {
            self.place_empty(context, &[x.into(), y.into()])
        } else {
            self.place_free(context, &[x.into(), y.into()])
        }?
        .is_truthy())
        .into())
    }

    pub fn action_if(args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [any]).map(|x| x.clone())
    }

    pub fn action_if_number(&self, args: &[Value]) -> gml::Result<Value> {
        let (object_id, number, comparator) = expect_args!(args, [int, int, int])?;
        let count = self.room.instance_list.count(object_id) as i32;
        let cond = match comparator {
            1 => count < number,
            2 => count > number,
            0 | _ => count == number,
        };
        Ok(cond.into())
    }

    pub fn action_if_object(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (object, mut x, mut y) = expect_args!(args, [any, real, real])?;
        if context.relative {
            let instance = self.room.instance_list.get(context.this);
            x += instance.x.get();
            y += instance.y.get();
        }
        self.place_meeting(context, &[x.into(), y.into(), object])
    }

    pub fn action_if_question(&mut self, args: &[Value]) -> gml::Result<Value> {
        self.show_question(&[match expect_args!(args, [any])? {
            v @ Value::Str(_) => v,
            _ => "".into(),
        }])
    }

    pub fn action_if_dice(&mut self, args: &[Value]) -> gml::Result<Value> {
        let bound = expect_args!(args, [real])?;
        Ok((self.rand.next(bound.into()) < 1.0).into())
    }

    pub fn action_if_mouse(&mut self, args: &[Value]) -> gml::Result<Value> {
        let button = expect_args!(args, [int])?;
        let mb = match button {
            1 => MouseButton::Left as i8,
            2 => MouseButton::Right as i8,
            3 => MouseButton::Middle as i8,
            _ => return Ok((self.input.mouse_button() == 0).into()), // "no"
        };
        Ok((self.input.mouse_check_button(mb) || self.input.mouse_check_button_released(mb)).into())
    }

    pub fn action_if_aligned(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (xsnap, ysnap) = expect_args!(args, [real, real])?;
        let instance = self.room.instance_list.get(context.this);
        Ok((((xsnap <= 0.into()) || (instance.x.get() % xsnap == 0.into()))
            && ((ysnap <= 0.into()) || (instance.y.get() % ysnap == 0.into())))
        .into())
    }

    pub fn action_execute_script(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (script_id, arg1, arg2, arg3, arg4, arg5) = expect_args!(args, [int, any, any, any, any, any])?;
        if let Some(script) = self.assets.scripts.get_asset(script_id) {
            let instructions = script.compiled.clone();

            let mut new_context = Context::copy_with_args(
                context,
                [
                    arg1,
                    arg2,
                    arg3,
                    arg4,
                    arg5,
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
                5,
            );
            self.execute(&instructions, &mut new_context)?;
            Ok(new_context.return_value)
        } else {
            Err(gml::Error::NonexistentAsset(asset::Type::Script, script_id))
        }
    }

    pub fn action_if_variable(args: &[Value]) -> gml::Result<Value> {
        use std::cmp::Ordering;
        let (lhs, rhs, comparator) = expect_args!(args, [any, any, int])?;
        let desired = match comparator {
            1 => Ordering::Less,
            2 => Ordering::Greater,
            0 | _ => Ordering::Equal,
        };
        Ok(match (lhs, rhs) {
            (Value::Real(lhs), Value::Real(rhs)) => lhs.partial_cmp(&rhs) == Some(desired),
            (Value::Str(lhs), Value::Str(rhs)) => lhs.cmp(&rhs) == desired,
            (lhs, rhs) => {
                return Err(gml::Error::FunctionError(
                    "action_if_variable".to_string(),
                    format!(
                        "invalid operands {} and {} to {:?} operator ({} {2:?} {})",
                        lhs.ty_str(),
                        rhs.ty_str(),
                        desired,
                        lhs,
                        rhs
                    ),
                ))
            },
        }
        .into())
    }

    pub fn action_draw_variable(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (variable, mut x, mut y) = expect_args!(args, [any, real, real])?;
        if context.relative {
            let instance = self.room.instance_list.get(context.this);
            x += instance.x.get();
            y += instance.y.get();
        }
        self.draw_text(&[x.into(), y.into(), variable])
    }

    pub fn action_set_score(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let score = expect_args!(args, [int])?;
        if context.relative {
            self.score += score;
        } else {
            self.score = score;
        }
        Ok(Default::default())
    }

    pub fn action_if_score(&self, args: &[Value]) -> gml::Result<Value> {
        let (value, method) = expect_args!(args, [real, int])?;

        Ok(match method {
            1 => Real::from(self.score) < value,
            2 => Real::from(self.score) > value,
            0 | _ => Real::from(self.score) == value,
        }
        .into())
    }

    pub fn action_draw_score(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (mut x, mut y, caption) = expect_args!(args, [real, real, bytes])?;
        if context.relative {
            let instance = self.room.instance_list.get(context.this);
            x += instance.x.get();
            y += instance.y.get();
        }
        self.draw_text(&[x.into(), y.into(), format!("{}{}", caption, self.score).into()])
    }

    pub fn action_highscore_show(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 5
        unimplemented!("Called unimplemented kernel function action_highscore_show")
    }

    pub fn action_set_life(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let lives = expect_args!(args, [int])?;
        if context.relative {
            self.lives += lives;
        } else {
            self.lives = lives;
        }
        Ok(Default::default())
    }

    pub fn action_if_life(&self, args: &[Value]) -> gml::Result<Value> {
        let (value, method) = expect_args!(args, [real, int])?;

        Ok(match method {
            1 => Real::from(self.lives) < value,
            2 => Real::from(self.lives) > value,
            0 | _ => Real::from(self.lives) == value,
        }
        .into())
    }

    pub fn action_draw_life(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (mut x, mut y, caption) = expect_args!(args, [real, real, bytes])?;
        if context.relative {
            let instance = self.room.instance_list.get(context.this);
            x += instance.x.get();
            y += instance.y.get();
        }
        self.draw_text(&[x.into(), y.into(), format!("{}{}", caption, self.lives).into()])
    }

    pub fn action_draw_life_images(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (mut x, mut y, sprite_index) = expect_args!(args, [real, real, int])?;
        if context.relative {
            let inst = self.room.instance_list.get(context.this);
            x += inst.x.get();
            y += inst.y.get();
        }
        if let Some(sprite) = self.assets.sprites.get_asset(sprite_index) {
            if let Some(atlas_ref) = sprite.get_atlas_ref(0) {
                for _ in 0..self.lives {
                    self.renderer.draw_sprite(atlas_ref, x.into(), y.into(), 1.0, 1.0, 0.0, 0xFFFFFF, 1.0);
                    x += sprite.width.into();
                }
            }
        }
        Ok(Default::default())
    }

    pub fn action_set_health(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let health = expect_args!(args, [real])?;
        let old_health = self.health;
        if context.relative {
            self.health += health;
        } else {
            self.health = health;
        }
        if old_health > 0.into() && self.health <= 0.into() {
            self.run_other_event(9)?;
        }
        Ok(Default::default())
    }

    pub fn action_if_health(&self, args: &[Value]) -> gml::Result<Value> {
        let (value, method) = expect_args!(args, [real, int])?;

        Ok(match method {
            1 => self.health < value,
            2 => self.health > value,
            0 | _ => self.health == value,
        }
        .into())
    }

    pub fn action_draw_health(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (mut x1, mut y1, mut x2, mut y2, back_col, col) = expect_args!(args, [real, real, real, real, int, int])?;

        if context.relative {
            let instance = self.room.instance_list.get(context.this);
            let x = instance.x.get();
            let y = instance.y.get();
            x1 += x;
            x2 += x;
            y1 += y;
            y2 += y;
        }

        let health_ratio = f64::from(self.health / Real::from(100.0));

        use mappings::constants;
        let bar_colour = match col {
            0 => {
                // green to red (actually c_lime to c_red)
                if health_ratio > 0.5 {
                    i32::from_le_bytes([((1.0 - health_ratio) * (2.0 * 255.0)) as u8, 255, 0, 0])
                } else {
                    i32::from_le_bytes([255, (health_ratio * (2.0 * 255.0)) as u8, 0, 0])
                }
            },
            1 => {
                // white to black
                let value = (health_ratio * 255.0) as u8;
                i32::from_le_bytes([value, value, value, 0])
            },
            2 => constants::C_BLACK as i32,
            3 => constants::C_GRAY as i32,
            4 => constants::C_SILVER as i32,
            5 => constants::C_WHITE as i32,
            6 => constants::C_MAROON as i32,
            7 => constants::C_GREEN as i32,
            8 => constants::C_OLIVE as i32,
            9 => constants::C_NAVY as i32,
            10 => constants::C_PURPLE as i32,
            11 => constants::C_TEAL as i32,
            12 => constants::C_RED as i32,
            13 => constants::C_LIME as i32,
            14 => constants::C_YELLOW as i32,
            15 => constants::C_BLUE as i32,
            16 => constants::C_FUCHSIA as i32,
            17 => constants::C_AQUA as i32,
            _ => constants::C_BLACK as i32,
        };
        let back_colour = match back_col {
            0 => None,
            1 => Some(constants::C_BLACK as i32),
            2 => Some(constants::C_GRAY as i32),
            3 => Some(constants::C_SILVER as i32),
            4 => Some(constants::C_WHITE as i32),
            5 => Some(constants::C_MAROON as i32),
            6 => Some(constants::C_GREEN as i32),
            7 => Some(constants::C_OLIVE as i32),
            8 => Some(constants::C_NAVY as i32),
            9 => Some(constants::C_PURPLE as i32),
            10 => Some(constants::C_TEAL as i32),
            11 => Some(constants::C_RED as i32),
            12 => Some(constants::C_LIME as i32),
            13 => Some(constants::C_YELLOW as i32),
            14 => Some(constants::C_BLUE as i32),
            15 => Some(constants::C_FUCHSIA as i32),
            16 => Some(constants::C_AQUA as i32),
            _ => Some(constants::C_PURPLE as i32),
        };

        if let Some(colour) = back_colour {
            self.renderer.draw_rectangle(x1.into(), y1.into(), x2.into(), y2.into(), colour, self.draw_alpha.into());
            self.renderer.draw_rectangle_outline(x1.into(), y1.into(), x2.into(), y2.into(), 0, self.draw_alpha.into());
        }

        x2 = x1 + ((x2 - x1) * health_ratio.into());
        self.renderer.draw_rectangle(x1.into(), y1.into(), x2.into(), y2.into(), bar_colour, self.draw_alpha.into());
        self.renderer.draw_rectangle_outline(x1.into(), y1.into(), x2.into(), y2.into(), 0, self.draw_alpha.into());

        Ok(Default::default())
    }

    pub fn action_set_caption(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (sc_show, sc_cap, lv_show, lv_cap, hl_show, hl_cap) =
            expect_args!(args, [bool, bytes, bool, bytes, bool, bytes])?;

        self.has_set_show_score = true;
        self.score_capt_d = sc_show;
        self.lives_capt_d = lv_show;
        self.health_capt_d = hl_show;

        self.score_capt = sc_cap;
        self.lives_capt = lv_cap;
        self.health_capt = hl_cap;

        Ok(Default::default())
    }

    pub fn action_partsyst_create(&mut self, args: &[Value]) -> gml::Result<Value> {
        let depth = expect_args!(args, [real])?;
        self.particles.get_dnd_system_mut().depth = depth;
        Ok(Default::default())
    }

    pub fn action_partsyst_destroy(&mut self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        self.particles.destroy_dnd_system();
        Ok(Default::default())
    }

    pub fn action_partsyst_clear(&mut self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        self.particles.clear_dnd_system();
        Ok(Default::default())
    }

    pub fn action_parttype_create_old(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (id, shape, size_min, size_max, col1, col2) = expect_args!(args, [int, int, real, real, int, int])?;
        let pt = self.particles.get_dnd_type_mut(id as usize);
        pt.graphic = particle::ParticleGraphic::Shape(shape);
        pt.size_min = size_min;
        pt.size_max = size_max;
        pt.size_incr = 0.into();
        pt.size_wiggle = 0.into();
        pt.colour = particle::ParticleColour::Two(col1, col2);
        Ok(Default::default())
    }

    pub fn action_parttype_create(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (id, shape, sprite, size_min, size_max, size_incr) = expect_args!(args, [int, int, int, real, real, real])?;
        let pt = self.particles.get_dnd_type_mut(id as usize);
        pt.graphic = if self.assets.sprites.get_asset(sprite).is_none() {
            particle::ParticleGraphic::Shape(shape)
        } else {
            particle::ParticleGraphic::Sprite { sprite, animat: true, random: false, stretch: false }
        };
        pt.size_min = size_min;
        pt.size_max = size_max;
        pt.size_incr = size_incr;
        pt.size_wiggle = 0.into();
        Ok(Default::default())
    }

    pub fn action_parttype_color(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (id, changing, col1, col2, start_alpha, end_alpha) = expect_args!(args, [int, bool, int, int, real, real])?;
        let pt = self.particles.get_dnd_type_mut(id as usize);
        pt.colour = if changing {
            particle::ParticleColour::Two(col1, col2)
        } else {
            particle::ParticleColour::Mix(col1, col2)
        };
        pt.alpha1 = start_alpha;
        pt.alpha2 = (start_alpha + end_alpha) / Real::from(2.0);
        pt.alpha3 = end_alpha;
        Ok(Default::default())
    }

    pub fn action_parttype_life(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (id, life_min, life_max) = expect_args!(args, [int, int, int])?;
        let pt = self.particles.get_dnd_type_mut(id as usize);
        pt.life_min = life_min;
        pt.life_max = life_max;
        Ok(Default::default())
    }

    pub fn action_parttype_speed(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (id, speed_min, speed_max, dir_min, dir_max, friction) =
            expect_args!(args, [int, real, real, real, real, real])?;
        let pt = self.particles.get_dnd_type_mut(id as usize);
        pt.speed_min = speed_min;
        pt.speed_max = speed_max;
        pt.dir_min = dir_min;
        pt.dir_max = dir_max;
        pt.speed_incr = -friction;
        Ok(Default::default())
    }

    pub fn action_parttype_gravity(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (id, grav_amount, grav_dir) = expect_args!(args, [int, real, real])?;
        let pt = self.particles.get_dnd_type_mut(id as usize);
        pt.grav_amount = grav_amount;
        pt.grav_dir = grav_dir;
        Ok(Default::default())
    }

    pub fn action_parttype_secondary(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (id, step_type, step_number, death_type, death_number) = expect_args!(args, [int, int, int, int, int])?;
        self.particles.dnd_type_secondary(
            id as usize,
            step_type as usize,
            step_number,
            death_type as usize,
            death_number,
        );
        Ok(Default::default())
    }

    pub fn action_partemit_create(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (id, shape, xmin, ymin, xmax, ymax) = expect_args!(args, [int, int, real, real, real, real])?;
        let em = self.particles.get_dnd_emitter_mut(id as usize);
        em.shape = match shape {
            1 => particle::Shape::Ellipse,
            2 => particle::Shape::Diamond,
            3 => particle::Shape::Line,
            _ => particle::Shape::Rectangle,
        };
        em.xmin = xmin;
        em.ymin = ymin;
        em.xmax = xmax;
        em.ymax = ymax;
        Ok(Default::default())
    }

    pub fn action_partemit_destroy(&mut self, args: &[Value]) -> gml::Result<Value> {
        let id = expect_args!(args, [int])?;
        self.particles.destroy_dnd_emitter(id as usize);
        Ok(Default::default())
    }

    pub fn action_partemit_burst(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (id, parttype, number) = expect_args!(args, [int, int, int])?;
        self.particles.dnd_emitter_burst(id as usize, parttype as usize, number, &mut self.rand);
        Ok(Default::default())
    }

    pub fn action_partemit_stream(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (id, parttype, number) = expect_args!(args, [int, int, int])?;
        self.particles.dnd_emitter_stream(id as usize, parttype as usize, number);
        Ok(Default::default())
    }

    pub fn action_set_cursor(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (sprite_id, show_window_cursor) = expect_args!(args, [int, bool])?;
        self.cursor_sprite = sprite_id;
        let cursor = if show_window_cursor {
            Cursor::Arrow // GM8 seems to always resets to default cursor on call of this function
        } else {
            Cursor::Blank
        };
        self.window.set_cursor(cursor);
        Ok(Default::default())
    }

    pub fn action_webpage(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        unimplemented!("Called unimplemented kernel function action_webpage")
    }

    pub fn action_draw_sprite(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (sprite_id, x, y, image_index) = expect_args!(args, [any, real, real, any])?;
        let instance = self.room.instance_list.get(context.this);
        let (x, y) = if context.relative { (x + instance.x.get(), y + instance.y.get()) } else { (x, y) };
        self.draw_sprite(context, &[sprite_id, image_index, x.into(), y.into()])
    }

    pub fn action_draw_background(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (bg_index, x, y, tiled) = expect_args!(args, [any, any, any, bool])?;
        let (x, y) = if context.relative {
            let instance = self.room.instance_list.get(context.this);
            ((instance.x.get() + x.into()).into(), (instance.y.get() + y.into()).into())
        } else {
            (x, y)
        };
        if tiled { self.draw_background_tiled(&[bg_index, x, y]) } else { self.draw_background(&[bg_index, x, y]) }
    }

    pub fn action_draw_text(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (text, mut x, mut y) = expect_args!(args, [any, real, real])?;
        if context.relative {
            let instance = self.room.instance_list.get(context.this);
            x += instance.x.get();
            y += instance.y.get();
        }
        self.draw_text(&[x.into(), y.into(), text])
    }

    pub fn action_draw_text_transformed(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (text, mut x, mut y, xscale, yscale, angle) = expect_args!(args, [any, real, real, any, any, any])?;
        if context.relative {
            let instance = self.room.instance_list.get(context.this);
            x += instance.x.get();
            y += instance.y.get();
        }
        self.draw_text_transformed(&[x.into(), y.into(), text, xscale, yscale, angle])
    }

    pub fn action_draw_rectangle(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (x1, y1, x2, y2, border) = expect_args!(args, [real, real, real, real, any])?;
        if context.relative {
            let instance = self.room.instance_list.get(context.this);
            let x = instance.x.get();
            let y = instance.y.get();
            self.draw_rectangle(&[
                Value::from(x1 + x),
                Value::from(y1 + y),
                Value::from(x2 + x),
                Value::from(y2 + y),
                border,
            ])
        } else {
            self.draw_rectangle(&[Value::from(x1), Value::from(y1), Value::from(x2), Value::from(y2), border])
        }
    }

    pub fn action_draw_gradient_hor(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (x1, y1, x2, y2, col1, col2) = expect_args!(args, [any, any, any, any, any, any])?;
        let (x1, y1, x2, y2) = if context.relative {
            let instance = self.room.instance_list.get(context.this);
            (
                (instance.x.get() + x1.into()).into(),
                (instance.y.get() + y1.into()).into(),
                (instance.x.get() + x2.into()).into(),
                (instance.y.get() + y2.into()).into(),
            )
        } else {
            (x1, y1, x2, y2)
        };
        self.draw_rectangle_color(&[x1, y1, x2, y2, col1.clone(), col2.clone(), col2, col1, false.into()])
    }

    pub fn action_draw_gradient_vert(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (x1, y1, x2, y2, col1, col2) = expect_args!(args, [any, any, any, any, any, any])?;
        let (x1, y1, x2, y2) = if context.relative {
            let instance = self.room.instance_list.get(context.this);
            (
                (instance.x.get() + x1.into()).into(),
                (instance.y.get() + y1.into()).into(),
                (instance.x.get() + x2.into()).into(),
                (instance.y.get() + y2.into()).into(),
            )
        } else {
            (x1, y1, x2, y2)
        };
        self.draw_rectangle_color(&[x1, y1, x2, y2, col1.clone(), col1, col2.clone(), col2, false.into()])
    }

    pub fn action_draw_ellipse(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (x1, y1, x2, y2, outline) = expect_args!(args, [any, any, any, any, any])?;
        let (x1, y1, x2, y2) = if context.relative {
            let instance = self.room.instance_list.get(context.this);
            (
                (instance.x.get() + x1.into()).into(),
                (instance.y.get() + y1.into()).into(),
                (instance.x.get() + x2.into()).into(),
                (instance.y.get() + y2.into()).into(),
            )
        } else {
            (x1, y1, x2, y2)
        };
        self.draw_ellipse(&[x1, y1, x2, y2, outline])
    }

    pub fn action_draw_ellipse_gradient(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (x1, y1, x2, y2, col1, col2) = expect_args!(args, [any, any, any, any, any, any])?;
        let (x1, y1, x2, y2) = if context.relative {
            let instance = self.room.instance_list.get(context.this);
            (
                (instance.x.get() + x1.into()).into(),
                (instance.y.get() + y1.into()).into(),
                (instance.x.get() + x2.into()).into(),
                (instance.y.get() + y2.into()).into(),
            )
        } else {
            (x1, y1, x2, y2)
        };
        self.draw_ellipse_color(&[x1, y1, x2, y2, col1, col2, false.into()])
    }

    pub fn action_draw_line(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (x1, y1, x2, y2) = expect_args!(args, [any, any, any, any])?;
        let (x1, y1, x2, y2) = if context.relative {
            let instance = self.room.instance_list.get(context.this);
            (
                (instance.x.get() + x1.into()).into(),
                (instance.y.get() + y1.into()).into(),
                (instance.x.get() + x2.into()).into(),
                (instance.y.get() + y2.into()).into(),
            )
        } else {
            (x1, y1, x2, y2)
        };
        self.draw_line(&[x1, y1, x2, y2])
    }

    pub fn action_draw_arrow(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (x1, y1, x2, y2, size) = expect_args!(args, [any, any, any, any, any])?;
        let (x1, y1, x2, y2) = if context.relative {
            let instance = self.room.instance_list.get(context.this);
            (
                (instance.x.get() + x1.into()).into(),
                (instance.y.get() + y1.into()).into(),
                (instance.x.get() + x2.into()).into(),
                (instance.y.get() + y2.into()).into(),
            )
        } else {
            (x1, y1, x2, y2)
        };
        self.draw_arrow(&[x1, y1, x2, y2, size])
    }

    pub fn action_font(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (font_id, align) = expect_args!(args, [int, int])?;
        if self.assets.fonts.get_asset(font_id).is_some() {
            self.draw_font_id = font_id;
        } else {
            self.draw_font_id = -1;
        }
        self.draw_halign = match align {
            1 => draw::Halign::Middle,
            2 => draw::Halign::Right,
            0 | _ => draw::Halign::Left,
        };
        Ok(Default::default())
    }

    pub fn action_fullscreen(&mut self, args: &[Value]) -> gml::Result<Value> {
        let _action = expect_args!(args, [int])?;
        // 1 is windowed, 2 is fullscreen 0/other is switch
        // TODO
        Ok(Default::default())
    }

    pub fn action_effect(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (kind, mut x, mut y, size, colour, below) = expect_args!(args, [any, real, real, any, any, bool])?;
        if context.relative {
            let instance = self.room.instance_list.get(context.this);
            x += instance.x.get();
            y += instance.y.get();
        }

        if below {
            self.effect_create_below(&[kind, x.into(), y.into(), size, colour])
        } else {
            self.effect_create_above(&[kind, x.into(), y.into(), size, colour])
        }
    }

    pub fn is_real(args: &[Value]) -> gml::Result<Value> {
        match expect_args!(args, [any])? {
            Value::Real(_) => Ok(gml::TRUE.into()),
            _ => Ok(gml::FALSE.into()),
        }
    }

    pub fn is_string(args: &[Value]) -> gml::Result<Value> {
        match expect_args!(args, [any])? {
            Value::Str(_) => Ok(gml::TRUE.into()),
            _ => Ok(gml::FALSE.into()),
        }
    }

    pub fn random(&mut self, args: &[Value]) -> gml::Result<Value> {
        let bound = expect_args!(args, [real])?;
        Ok(self.rand.next(bound.into()).into())
    }

    pub fn random_range(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (lower, upper) = expect_args!(args, [real, real])?;
        Ok((lower.min(upper) + Real::from(self.rand.next((upper - lower).abs().into()))).into())
    }

    pub fn irandom(&mut self, args: &[Value]) -> gml::Result<Value> {
        let bound = expect_args!(args, [int])?;
        Ok(self.rand.next_int(bound as _).into())
    }

    pub fn irandom_range(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (lower, upper) = expect_args!(args, [int, int])?;
        Ok((lower.min(upper) + self.rand.next_int((upper - lower).abs() as _)).into())
    }

    pub fn random_set_seed(&mut self, args: &[Value]) -> gml::Result<Value> {
        let seed = expect_args!(args, [int])?;
        self.rand.set_seed(seed);
        Ok(Default::default())
    }

    pub fn random_get_seed(&self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        Ok(self.rand.seed().into())
    }

    pub fn randomize(&mut self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        match self.play_type {
            PlayType::Normal => self.rand.randomize(),
            PlayType::Record => {
                self.rand.randomize();
                self.stored_events.push_back(replay::Event::Randomize(self.rand.seed()));
            },
            PlayType::Replay => {
                if let Some(replay::Event::Randomize(seed)) = self.stored_events.pop_front() {
                    self.rand.set_seed(seed);
                } else {
                    return Err(gml::Error::ReplayError("randomize".into()))
                }
            },
        }
        Ok(Default::default())
    }

    pub fn abs(args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [real]).map(|x| Value::Real(x.abs()))
    }

    pub fn round(args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [real]).map(|x| x.round().into())
    }

    pub fn floor(args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [real]).map(|x| Value::Real(x.floor()))
    }

    pub fn ceil(args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [real]).map(|x| Value::Real(x.ceil()))
    }

    pub fn sign(args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [real]).map(|x| {
            Value::Real(
                if x >= Real::CMP_EPSILON {
                    1
                } else if x <= -Real::CMP_EPSILON {
                    -1
                } else {
                    0
                }
                .into(),
            )
        })
    }

    pub fn frac(args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [real]).map(|x| Value::Real(x.fract()))
    }

    pub fn sqrt(args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [real]).and_then(|x| match x.sqrt() {
            n if !n.as_ref().is_nan() => Ok(Value::Real(n)),
            n => Err(gml::Error::FunctionError("sqrt".into(), format!("can't get square root of {}", n))),
        })
    }

    pub fn sqr(args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [real]).map(|x| Value::Real(x * x))
    }

    pub fn exp(args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [real]).map(|x| Value::Real(x.exp()))
    }

    pub fn ln(args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [real]).map(|x| Value::Real(x.ln()))
    }

    pub fn log2(args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [real]).map(|x| Value::Real(x.log2()))
    }

    pub fn log10(args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [real]).map(|x| Value::Real(x.log10()))
    }

    pub fn sin(args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [real]).map(|x| Value::Real(x.sin()))
    }

    pub fn cos(args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [real]).map(|x| Value::Real(x.cos()))
    }

    pub fn tan(args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [real]).map(|x| Value::Real(x.tan()))
    }

    pub fn arcsin(args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [real]).map(|x| Value::Real(x.arcsin()))
    }

    pub fn arccos(args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [real]).map(|x| Value::Real(x.arccos()))
    }

    pub fn arctan(args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [real]).map(|x| Value::Real(x.arctan()))
    }

    pub fn arctan2(args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [real, real]).map(|(y, x)| Value::Real(y.arctan2(x)))
    }

    pub fn degtorad(args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [real]).map(|x| Value::Real(x.to_radians()))
    }

    pub fn radtodeg(args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [real]).map(|x| Value::Real(x.to_degrees()))
    }

    pub fn power(args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [real, real]).map(|(x, n)| Value::Real(x.into_inner().powf(n.into()).into()))
    }

    pub fn logn(args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [real, real]).map(|(n, x)| Value::Real(x.logn(n)))
    }

    pub fn min(args: &[Value]) -> gml::Result<Value> {
        Ok(args.iter().reduce(Value::min).cloned().unwrap_or_default())
    }

    pub fn max(args: &[Value]) -> gml::Result<Value> {
        Ok(args.iter().reduce(Value::max).cloned().unwrap_or_default())
    }

    pub fn min3(args: &[Value]) -> gml::Result<Value> {
        let (a, b, c) = expect_args!(args, [any, any, any])?;
        Ok(a.min(&b).min(&c).clone())
    }

    pub fn max3(args: &[Value]) -> gml::Result<Value> {
        let (a, b, c) = expect_args!(args, [any, any, any])?;
        Ok(a.max(&b).max(&c).clone())
    }

    pub fn mean(args: &[Value]) -> gml::Result<Value> {
        if !args.is_empty() {
            Ok(Value::from(args.iter().cloned().map(Real::from).sum::<Real>() / Real::from(args.len() as f64)))
        } else {
            Ok(Default::default())
        }
    }

    pub fn median(args: &[Value]) -> gml::Result<Value> {
        Ok(args
            .iter()
            .cloned()
            .find(|v| {
                let v = Real::from(v.clone());
                let mut less = 0.0;
                let mut less_eq = 0.0;
                for arg in args.iter().cloned() {
                    let arg = Real::from(arg);
                    if arg <= v {
                        less_eq += 1.0;
                        if arg != v {
                            less += 1.0;
                        }
                    }
                }
                less < args.len() as f64 / 2.0 && less_eq >= args.len() as f64 / 2.0
            })
            .unwrap_or_default())
    }

    pub fn choose(&mut self, args: &[Value]) -> gml::Result<Value> {
        match args.len().checked_sub(1) {
            Some(i) => Ok(args[self.rand.next_int(i as _) as usize].clone()),
            None => Ok(Default::default()),
        }
    }

    pub fn clamp(args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [real, real, real]).map(|(n, lo, hi)| Value::Real(n.clamp(lo, hi)))
    }

    pub fn lerp(args: &[Value]) -> gml::Result<Value> {
        let (low, high, amount) = expect_args!(args, [real, real, real])?;
        Ok(Value::from(((high - low) * amount) + low))
    }

    pub fn real(&self, args: &[Value]) -> gml::Result<Value> {
        // TODO: Make this function pure.
        expect_args!(args, [any]).and_then(|v| match v {
            r @ Value::Real(_) => Ok(r),
            Value::Str(s) => match self.decode_str(s.as_ref()).trim() {
                x if x.len() == 0 => Ok(Default::default()),
                x => match x.parse::<f64>() {
                    Ok(r) => Ok(r.into()),
                    Err(e) => Err(gml::Error::FunctionError("real".into(), format!("can't convert {} - {}", s, e))),
                },
            },
        })
    }

    pub fn string(args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [any]).map(|v| v.repr().into())
    }

    pub fn string_format(args: &[Value]) -> gml::Result<Value> {
        let (val, mut tot, mut dec) = expect_args!(args, [any, int, int])?;
        match val {
            Value::Str(_) => Ok(val),
            Value::Real(mut x) => {
                dec = dec.min(18);
                tot = tot.max(0);
                if dec < 0 {
                    // Very strange behaviour here but I swear it's accurate
                    let power = Real::from(10f64.powi(-dec));
                    x = Real::from((x / power).round()) * power;
                    dec = 18;
                }
                Ok(format!("{num:>width$.prec$}", num = x, prec = dec as usize, width = tot as usize).into())
            },
        }
    }

    pub fn chr(&self, args: &[Value]) -> gml::Result<Value> {
        // TODO: use font to decode if not sprite font
        Self::ansi_char(args)
    }

    pub fn ansi_char(args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [int]).map(|x| vec![x as u8].into())
    }

    pub fn ord(args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [bytes]).map(|s| s.as_ref().get(0).copied().map(f64::from).unwrap_or_default().into())
    }

    pub fn string_length(&self, args: &[Value]) -> gml::Result<Value> {
        let string = expect_args!(args, [bytes])?;
        match self.gm_version {
            Version::GameMaker8_0 => Ok(Value::Real((string.as_ref().len() as f64).into())),
            Version::GameMaker8_1 => Ok(Value::Real((self.decode_str(string.as_ref()).chars().count() as f64).into())),
        }
    }

    pub fn string_byte_length(args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [bytes]).map(|s| Value::Real((s.as_ref().len() as f64).into()))
    }

    pub fn string_byte_at(args: &[Value]) -> gml::Result<Value> {
        // NOTE: The gamemaker 8 runner instead of defaulting to 0 just reads any memory address. LOL
        // We don't do this, unsurprisingly.
        expect_args!(args, [bytes, int]).map(|(s, ix)| {
            Value::Real((s.as_ref().get((ix as isize - 1).max(0) as usize).copied().unwrap_or_default() as f64).into())
        })
    }

    pub fn string_pos(&self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [bytes, bytes]).map(|(query, main_string)| match self.gm_version {
            Version::GameMaker8_0 => {
                let query = query.as_ref();
                Value::Real(Real::from(
                    main_string
                        .as_ref()
                        .windows(query.len())
                        .position(|x| x == query)
                        .map(|p| p + 1)
                        .unwrap_or_default() as f64,
                ))
            },
            Version::GameMaker8_1 => {
                let query = self.decode_str(query.as_ref());
                let main_string = self.decode_str(main_string.as_ref());
                Value::Real(Real::from(
                    main_string.as_ref().find(query.as_ref()).map(|p| p + 1).unwrap_or_default() as f64
                ))
            },
        })
    }

    pub fn string_copy(&self, args: &[Value]) -> gml::Result<Value> {
        let (s, start, len) = expect_args!(args, [bytes, int, int])?;
        let start = (start - 1).max(0) as usize;
        let len = len.max(0) as usize;
        let s = s.as_ref();
        Ok(match self.gm_version {
            Version::GameMaker8_0 => s.iter().skip(start).take(len).copied().collect::<Vec<_>>().into(),
            Version::GameMaker8_1 => self.decode_str(s).chars().skip(start).take(len).collect::<String>().into(),
        })
    }

    pub fn string_char_at(&self, args: &[Value]) -> gml::Result<Value> {
        let (string, pos) = expect_args!(args, [bytes, int])?;
        match self.gm_version {
            Version::GameMaker8_0 => {
                Ok(string.as_ref().get((pos as isize - 1).max(0) as usize).map_or("".into(), |ch| vec![*ch].into()))
            },
            Version::GameMaker8_1 => Ok(Value::Str(
                self.decode_str(string.as_ref())
                    .chars()
                    .nth((pos as isize - 1).max(0) as usize)
                    .map_or("".to_string().into(), |ch| ch.to_string().into()),
            )),
        }
    }

    pub fn string_delete(&self, args: &[Value]) -> gml::Result<Value> {
        let (s, start, len) = expect_args!(args, [bytes, int, int])?;
        let (start, len) = match (<usize>::try_from(start - 1), len) {
            (Ok(a), b) if b > 0 => (a, b as usize),
            _ => return Ok(s.into()),
        };
        let s = s.as_ref();
        Ok(match self.gm_version {
            Version::GameMaker8_0 => {
                s.iter().take(start).chain(s.iter().skip(start + len)).copied().collect::<Vec<_>>().into()
            },
            Version::GameMaker8_1 => self
                .decode_str(s)
                .chars()
                .enumerate()
                .filter_map(|(i, x)| if (start..start + len).contains(&i) { None } else { Some(x) })
                .collect::<String>()
                .into(),
        })
    }

    pub fn string_insert(args: &[Value]) -> gml::Result<Value> {
        // string_insert doesn't care about UTF-8
        expect_args!(args, [bytes, bytes, int]).map(|(ss, s, ix)| {
            // ghetto clamp is better here than .clamp() because of unsigned type boundary
            let ix = ((ix as isize - 1).max(0) as usize).min(s.as_ref().len());
            s.as_ref()[..ix].iter().chain(ss.as_ref()).chain(&s.as_ref()[ix..]).copied().collect::<Vec<_>>().into()
        })
    }

    pub fn string_lower(&self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [bytes]).map(|s| match self.gm_version {
            Version::GameMaker8_0 => Value::Str(s.as_ref().to_ascii_lowercase().into()),
            Version::GameMaker8_1 => Value::Str(self.decode_str(s.as_ref()).to_lowercase().into()),
        })
    }

    pub fn string_upper(&self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [bytes]).map(|s| match self.gm_version {
            Version::GameMaker8_0 => Value::Str(s.as_ref().to_ascii_uppercase().into()),
            Version::GameMaker8_1 => Value::Str(self.decode_str(s.as_ref()).to_uppercase().into()),
        })
    }

    pub fn string_repeat(args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [bytes, real]).map(|(s, n)| Value::Str(s.as_ref().repeat(n.into_inner() as usize).into()))
    }

    pub fn string_letters(args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [string])
            .map(|s| Value::Str(s.as_ref().chars().filter(|ch| ch.is_ascii_alphabetic()).collect::<String>().into()))
    }

    pub fn string_digits(args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [string])
            .map(|s| Value::Str(s.as_ref().chars().filter(|ch| ch.is_ascii_digit()).collect::<String>().into()))
    }

    pub fn string_lettersdigits(args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [string])
            .map(|s| Value::Str(s.as_ref().chars().filter(|ch| ch.is_ascii_alphanumeric()).collect::<String>().into()))
    }

    pub fn string_replace(args: &[Value]) -> gml::Result<Value> {
        let (s, sub, rep) = expect_args!(args, [bytes, bytes, bytes])?;
        let (s, sub, rep) = (s.as_ref(), sub.as_ref(), rep.as_ref());
        // could be faster but i'm feeling lazy
        for i in 0..s.as_ref().len() {
            if s[i..].starts_with(sub.as_ref()) {
                let mut out = Vec::with_capacity(s.len() + rep.len() - sub.len());
                out.extend_from_slice(&s[..i]);
                out.extend_from_slice(rep);
                out.extend_from_slice(&s[i + sub.len()..]);
                return Ok(out.into())
            }
        }
        Ok(s.into())
    }

    pub fn string_replace_all(args: &[Value]) -> gml::Result<Value> {
        let (s, sub, rep) = expect_args!(args, [bytes, bytes, bytes])?;
        let (s, sub, rep) = (s.as_ref(), sub.as_ref(), rep.as_ref());
        if sub.len() > 0 {
            // could be faster but i'm feeling lazy
            let mut out = Vec::new();
            let mut section_start = 0;
            let mut i = 0;
            while i < s.len() {
                if s[i..].starts_with(sub) {
                    out.extend_from_slice(&s[section_start..i]);
                    out.extend_from_slice(rep);
                    i += sub.len();
                    section_start = i;
                } else {
                    i += 1;
                }
            }
            out.extend_from_slice(&s[section_start..]);
            Ok(out.into())
        } else {
            Ok(s.into())
        }
    }

    pub fn string_count(args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [string, string])
            .map(|(ss, s)| Value::Real(Real::from(s.as_ref().matches(ss.as_ref()).count() as f64)))
    }

    pub fn dot_product(args: &[Value]) -> gml::Result<Value> {
        let (x1, y1, x2, y2) = expect_args!(args, [real, real, real, real])?;
        let l1 = Real::from(x1.into_inner().hypot(y1.into_inner()));
        let l2 = Real::from(x2.into_inner().hypot(y2.into_inner()));
        let (x1, y1) = (x1 / l1, y1 / l1);
        let (x2, y2) = (x2 / l2, y2 / l2);
        Ok((x1 * x2 + y1 * y2).into())
    }

    pub fn dot_product_3d(args: &[Value]) -> gml::Result<Value> {
        let (x1, y1, z1, x2, y2, z2) = expect_args!(args, [real, real, real, real, real, real])?;
        let l1 = (x1 * x1 + y1 * y1 + z1 * z1).sqrt();
        let l2 = (x2 * x2 + y2 * y2 + z2 * z2).sqrt();
        let (x1, y1, z1) = (x1 / l1, y1 / l1, z1 / l1);
        let (x2, y2, z2) = (x2 / l2, y2 / l2, z2 / l2);
        Ok((x1 * x2 + y1 * y2 + z1 * z2).into())
    }

    pub fn point_distance_3d(args: &[Value]) -> gml::Result<Value> {
        let (x1, y1, z1, x2, y2, z2) = expect_args!(args, [real, real, real, real, real, real])?;
        let xdist = x2 - x1;
        let ydist = y2 - y1;
        let zdist = z2 - z1;
        Ok((xdist * xdist + ydist * ydist + zdist * zdist).sqrt().into())
    }

    pub fn point_distance(args: &[Value]) -> gml::Result<Value> {
        let (x1, y1, x2, y2) = expect_args!(args, [real, real, real, real])?;
        let xdist = x2 - x1;
        let ydist = y2 - y1;
        Ok((xdist * xdist + ydist * ydist).sqrt().into())
    }

    pub fn point_direction(args: &[Value]) -> gml::Result<Value> {
        let (x1, y1, x2, y2) = expect_args!(args, [real, real, real, real])?;
        Ok((y1 - y2).arctan2(x2 - x1).to_degrees().rem_euclid(360.into()).into())
    }

    pub fn lengthdir_x(args: &[Value]) -> gml::Result<Value> {
        let (len, dir) = expect_args!(args, [real, real])?;
        Ok((dir.to_radians().cos() * len).into())
    }

    pub fn lengthdir_y(args: &[Value]) -> gml::Result<Value> {
        let (len, dir) = expect_args!(args, [real, real])?;
        Ok((dir.to_radians().sin() * -len).into())
    }

    pub fn move_random(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (hsnap, vsnap) = expect_args!(args, [int, int])?;
        let inst = self.room.instance_list.get(context.this);
        let (mut left, mut right, mut top, mut bottom) = (0, self.room.width, 0, self.room.height);
        if let Some(sprite) = self
            .assets
            .sprites
            .get_asset(inst.sprite_index.get())
            .or(self.assets.sprites.get_asset(inst.mask_index.get()))
        {
            inst.update_bbox(Some(sprite));
            left = (inst.x.get() - inst.bbox_left.get().into()).round().to_i32();
            right = (inst.x.get() + right.into() - inst.bbox_right.get().into()).round().to_i32();
            top = (inst.y.get() - inst.bbox_top.get().into()).round().to_i32();
            bottom = (inst.y.get() + bottom.into() - inst.bbox_bottom.get().into()).round().to_i32();
        };
        drop(inst); // le borrow
        let (mut x, mut y) = Default::default();
        for _ in 0..100 {
            x = Real::from(self.rand.next_int((right - left - 1) as u32) + left);
            if hsnap > 0 {
                x = (x / hsnap.into()).floor() * hsnap.into();
            }
            y = Real::from(self.rand.next_int((bottom - top - 1) as u32) + top);
            if vsnap > 0 {
                y = (y / vsnap.into()).floor() * vsnap.into();
            }
            if self.place_free(context, &[x.into(), y.into()])?.is_truthy() {
                break
            }
        }
        let inst = self.room.instance_list.get(context.this);
        inst.x.set(x);
        inst.y.set(y);
        inst.bbox_is_stale.set(true);
        Ok(Default::default())
    }

    pub fn place_free(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (x, y) = expect_args!(args, [real, real])?;

        // Set self's position to the new coordinates
        let instance = self.room.instance_list.get(context.this);
        let old_x = instance.x.get();
        let old_y = instance.y.get();
        instance.x.set(x);
        instance.y.set(y);
        instance.bbox_is_stale.set(true);

        // Check collision with any solids
        let free = self.check_collision_solid(context.this).is_none();

        // Move self back to where it was
        instance.x.set(old_x);
        instance.y.set(old_y);
        instance.bbox_is_stale.set(true);

        Ok(free.into())
    }

    pub fn place_empty(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (x, y) = expect_args!(args, [real, real])?;

        // Set self's position to the new coordinates
        let instance = self.room.instance_list.get(context.this);
        let old_x = instance.x.get();
        let old_y = instance.y.get();
        instance.x.set(x);
        instance.y.set(y);
        instance.bbox_is_stale.set(true);

        // Check collision with any instance
        let empty = self.check_collision_any(context.this).is_none();

        // Move self back to where it was
        instance.x.set(old_x);
        instance.y.set(old_y);
        instance.bbox_is_stale.set(true);

        Ok(empty.into())
    }

    pub fn place_meeting(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (x, y, obj) = expect_args!(args, [real, real, int])?;

        // Set self's position to the new coordinates
        let instance = self.room.instance_list.get(context.this);
        let old_x = instance.x.get();
        let old_y = instance.y.get();
        instance.x.set(x);
        instance.y.set(y);
        instance.bbox_is_stale.set(true);

        // Check collision with target
        let collision = match obj {
            gml::SELF => false,
            gml::OTHER => self.check_collision(context.this, context.other),
            obj => self.find_instance_with(obj, |handle| self.check_collision(context.this, handle)).is_some(),
        };

        // Move self back to where it was
        instance.x.set(old_x);
        instance.y.set(old_y);
        instance.bbox_is_stale.set(true);

        Ok(collision.into())
    }

    pub fn place_snapped(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (hsnap, vsnap) = expect_args!(args, [real, real])?;
        let instance = self.room.instance_list.get(context.this);
        Ok(((hsnap <= 0.into()
            || (instance.x.get() - (instance.x.get() / hsnap).round() * hsnap).abs() < 0.001.into())
            && (vsnap <= 0.into()
                || (instance.y.get() - (instance.y.get() / vsnap).round() * vsnap).abs() < 0.001.into()))
        .into())
    }

    pub fn move_snap(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (hsnap, vsnap) = expect_args!(args, [real, real])?;
        let instance = self.room.instance_list.get(context.this);
        if hsnap > 0.into() {
            instance.x.set(Real::from((instance.x.get() / hsnap).round()) * hsnap);
        }
        if vsnap > 0.into() {
            instance.y.set(Real::from((instance.y.get() / vsnap).round()) * vsnap);
        }
        instance.bbox_is_stale.set(true);
        Ok(Default::default())
    }

    pub fn move_towards_point(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (x, y, speed) = expect_args!(args, [real, real, real])?;
        let instance = self.room.instance_list.get(context.this);
        let direction = (instance.y.get() - y).arctan2(x - instance.x.get()).to_degrees();
        instance.set_speed_direction(speed, direction);
        Ok(Default::default())
    }

    pub fn move_contact(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let direction = expect_args!(args, [any])?;
        self.move_contact_all(context, &[direction, (-1).into()])
    }

    pub fn move_contact_solid(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (direction, max_distance) = expect_args!(args, [real, int])?;
        let max_distance = if max_distance > 0 {
            max_distance
        } else {
            1000 // GML default
        };

        // Figure out how far we're going to step in x and y between each check
        let step_x = direction.to_radians().cos();
        let step_y = -direction.to_radians().sin();

        // Check if we're already colliding with a solid, do nothing if so
        if self.check_collision_solid(context.this).is_none() {
            let instance = self.room.instance_list.get(context.this);
            for _ in 0..max_distance {
                // Step forward, but back up old coordinates
                let old_x = instance.x.get();
                let old_y = instance.y.get();
                instance.x.set(instance.x.get() + step_x);
                instance.y.set(instance.y.get() + step_y);
                instance.bbox_is_stale.set(true);

                // Check if we're colliding with a solid now
                if self.check_collision_solid(context.this).is_some() {
                    // Move self back to where it was, then exit
                    instance.x.set(old_x);
                    instance.y.set(old_y);
                    instance.bbox_is_stale.set(true);
                    break
                }
            }
        }

        Ok(Default::default())
    }

    pub fn move_contact_all(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (direction, max_distance) = expect_args!(args, [real, int])?;
        let max_distance = if max_distance > 0 {
            max_distance
        } else {
            1000 // GML default
        };

        // Figure out how far we're going to step in x and y between each check
        let step_x = direction.to_radians().cos();
        let step_y = -direction.to_radians().sin();

        // Check if we're already colliding with another instance, do nothing if so
        if self.check_collision_any(context.this).is_none() {
            let instance = self.room.instance_list.get(context.this);
            for _ in 0..max_distance {
                // Step forward, but back up old coordinates
                let old_x = instance.x.get();
                let old_y = instance.y.get();
                instance.x.set(instance.x.get() + step_x);
                instance.y.set(instance.y.get() + step_y);
                instance.bbox_is_stale.set(true);

                // Check if we're colliding with another instance now
                if self.check_collision_any(context.this).is_some() {
                    // Move self back to where it was, then exit
                    instance.x.set(old_x);
                    instance.y.set(old_y);
                    instance.bbox_is_stale.set(true);
                    break
                }
            }
        }

        Ok(Default::default())
    }

    pub fn move_outside_solid(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (direction, max_distance) = expect_args!(args, [real, int])?;
        let max_distance = if max_distance > 0 {
            max_distance
        } else {
            1000 // GML default
        };

        // Figure out how far we're going to step in x and y between each check
        let step_x = direction.to_radians().cos();
        let step_y = -direction.to_radians().sin();

        // Check if we're already outside all solids, do nothing if so
        if self.check_collision_solid(context.this).is_some() {
            let instance = self.room.instance_list.get(context.this);
            for _ in 0..max_distance {
                // Step forward
                instance.x.set(instance.x.get() + step_x);
                instance.y.set(instance.y.get() + step_y);
                instance.bbox_is_stale.set(true);

                // Check if we're outside all solids now
                if self.check_collision_solid(context.this).is_none() {
                    // Outside a solid, exit
                    break
                }
            }
        }

        Ok(Default::default())
    }

    pub fn move_outside_all(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (direction, max_distance) = expect_args!(args, [real, int])?;
        let max_distance = if max_distance > 0 {
            max_distance
        } else {
            1000 // GML default
        };

        // Figure out how far we're going to step in x and y between each check
        let step_x = direction.to_radians().cos();
        let step_y = -direction.to_radians().sin();

        // Check if we're already not colliding with anything, do nothing if so
        if self.check_collision_any(context.this).is_some() {
            let instance = self.room.instance_list.get(context.this);
            for _ in 0..max_distance {
                // Step forward
                instance.x.set(instance.x.get() + step_x);
                instance.y.set(instance.y.get() + step_y);
                instance.bbox_is_stale.set(true);

                // Check if we're not colliding with anything now
                if self.check_collision_any(context.this).is_none() {
                    // Outside a solid, exit
                    break
                }
            }
        }

        Ok(Default::default())
    }

    pub fn move_bounce_solid(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let advanced = expect_args!(args, [int])?;
        if advanced == 1 {
            self.bounce_advanced(context.this, true);
        } else {
            self.bounce(context.this, true);
        }
        Ok(Default::default())
    }

    pub fn move_bounce_all(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let advanced = expect_args!(args, [int])?;
        if advanced == 1 {
            self.bounce_advanced(context.this, false);
        } else {
            self.bounce(context.this, false);
        }
        Ok(Default::default())
    }

    pub fn move_wrap(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (horizontal_wrap, vertical_wrap, margin) = expect_args!(args, [bool, bool, real])?;
        let instance = self.room.instance_list.get(context.this);

        let mut update_bbox = false;

        if horizontal_wrap {
            let instance_x = instance.x.get();

            if instance_x < -margin {
                instance.x.set(Real::from(self.room.width) + instance_x + Real::from(2) * margin);
                update_bbox = true;
            }
            if instance_x > Real::from(self.room.width) + margin {
                instance.x.set(instance_x - Real::from(self.room.width) - Real::from(2) * margin);
                update_bbox = true;
            }
        }
        if vertical_wrap {
            let instance_y = instance.y.get();
            if instance_y < -margin {
                instance.y.set(Real::from(self.room.height) + instance_y + Real::from(2) * margin);
                update_bbox = true;
            }
            if instance_y > Real::from(self.room.height) + margin {
                instance.y.set(instance_y - Real::from(self.room.height) - Real::from(2) * margin);
                update_bbox = true;
            }
        }
        if update_bbox {
            instance.bbox_is_stale.set(true);
        }
        Ok(Default::default())
    }

    pub fn motion_set(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (direction, speed) = expect_args!(args, [real, real])?;
        self.room.instance_list.get(context.this).set_speed_direction(speed, direction);
        Ok(Default::default())
    }

    pub fn motion_add(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (direction, speed) = expect_args!(args, [real, real])?;
        let instance = self.room.instance_list.get(context.this);
        let hspeed = direction.to_radians().cos() * speed;
        let vspeed = -direction.to_radians().sin() * speed;
        instance.set_hvspeed(instance.hspeed.get() + hspeed, instance.vspeed.get() + vspeed);
        Ok(Default::default())
    }

    pub fn distance_to_point(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (x, y) = expect_args!(args, [real, real])?;
        let instance = self.room.instance_list.get(context.this);

        let sprite = self.get_instance_mask_sprite(context.this);
        instance.update_bbox(sprite);

        let distance_x = if x < instance.bbox_left.get().into() {
            x - instance.bbox_left.get().into()
        } else if x > instance.bbox_right.get().into() {
            x - instance.bbox_right.get().into()
        } else {
            0.into()
        };

        let distance_y = if y < instance.bbox_top.get().into() {
            y - instance.bbox_top.get().into()
        } else if y > instance.bbox_bottom.get().into() {
            y - instance.bbox_bottom.get().into()
        } else {
            0.into()
        };

        Ok(distance_x.into_inner().hypot(distance_y.into_inner()).into())
    }

    pub fn distance_to_object(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let object_id = expect_args!(args, [int])?;

        // Helper fn: distance between two instances (with a function name that's really hard to say quickly)
        fn instance_distance(inst1: &Instance, inst2: &Instance) -> f64 {
            let distance_x = if inst1.bbox_left.get() > inst2.bbox_right.get() {
                inst1.bbox_left.get() - inst2.bbox_right.get()
            } else if inst2.bbox_left.get() > inst1.bbox_right.get() {
                inst2.bbox_left.get() - inst1.bbox_right.get()
            } else {
                0
            };

            let distance_y = if inst1.bbox_top.get() > inst2.bbox_bottom.get() {
                inst1.bbox_top.get() - inst2.bbox_bottom.get()
            } else if inst2.bbox_top.get() > inst1.bbox_bottom.get() {
                inst2.bbox_top.get() - inst1.bbox_bottom.get()
            } else {
                0
            };

            match (distance_x, distance_y) {
                (0, 0) => 0.0,
                (x, 0) => x.into(),
                (0, y) => y.into(),
                (x, y) => f64::from(x).hypot(y.into()),
            }
        }

        let sprite = self.get_instance_mask_sprite(context.this);
        let this = self.room.instance_list.get(context.this);
        this.update_bbox(sprite);

        Ok(match object_id {
            gml::SELF => 0.0,
            gml::OTHER => {
                let sprite = self.get_instance_mask_sprite(context.other);
                let other = self.room.instance_list.get(context.other);
                other.update_bbox(sprite);
                instance_distance(this, other)
            },
            gml::ALL => {
                let mut closest = 1000000.0; // GML default
                let this = this;
                let mut iter = self.room.instance_list.iter_by_insertion();
                while let Some(other) = iter.next(&self.room.instance_list) {
                    let sprite = self.get_instance_mask_sprite(other);
                    let other = self.room.instance_list.get(other);
                    other.update_bbox(sprite);
                    let dist = instance_distance(this, other);
                    if dist < closest {
                        closest = dist;
                    }
                }
                closest
            },
            object_id if object_id <= 100000 => {
                let mut closest = 1000000.0; // GML default
                let this = this;
                let mut iter = self.room.instance_list.iter_by_identity(object_id);
                while let Some(other) = iter.next(&self.room.instance_list) {
                    let sprite = self.get_instance_mask_sprite(other);
                    let other = self.room.instance_list.get(other);
                    other.update_bbox(sprite);
                    let dist = instance_distance(this, other);
                    if dist < closest {
                        closest = dist;
                    }
                }
                closest
            },
            instance_id => {
                match self.room.instance_list.get_by_instid(instance_id) {
                    Some(handle) => {
                        let sprite = self.get_instance_mask_sprite(handle);
                        let other = self.room.instance_list.get(handle);
                        other.update_bbox(sprite);
                        instance_distance(this, other)
                    },
                    None => 1000000.0, // Again, GML default
                }
            },
        }
        .into())
    }

    pub fn path_start(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (path_id, speed, end_action, absolute) = expect_args!(args, [int, real, int, bool])?;
        let instance = self.room.instance_list.get(context.this);
        if let Some(path) = self.assets.paths.get_asset(path_id).filter(|p| p.length > 0.into()) {
            instance.path_index.set(path_id);
            instance.path_speed.set(speed);
            instance.path_endaction.set(end_action);
            let forwards = speed >= Real::from(0.0);
            instance.path_position.set(Real::from(if forwards { 0.0 } else { 1.0 }));
            instance.path_positionprevious.set(instance.path_position.get());
            instance.path_scale.set(Real::from(1.0));
            instance.path_orientation.set(Real::from(0.0));
            if absolute {
                let path_start = if forwards { path.start } else { path.end };
                instance.x.set(path_start.x);
                instance.y.set(path_start.y);
            }
            instance.path_xstart.set(instance.x.get());
            instance.path_ystart.set(instance.y.get());
        } else {
            instance.path_index.set(path_id.min(-1));
        }
        Ok(Default::default())
    }

    pub fn path_end(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        self.room.instance_list.get(context.this).path_index.set(-1);
        Ok(Default::default())
    }

    pub fn mp_linear_step(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (x, y, step_size, checkall) = expect_args!(args, [real, real, real, bool])?;
        Ok(pathfinding::linear_step(x, y, step_size, self.room.instance_list.get(context.this), || {
            if checkall {
                self.check_collision_any(context.this).is_some()
            } else {
                self.check_collision_solid(context.this).is_some()
            }
        })
        .into())
    }

    pub fn mp_linear_path(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (path_id, xg, yg, step_size, checkall) = expect_args!(args, [int, real, real, real, bool])?;
        // we use a closure that needs a &Game for the collision calls, so we can't have a &mut Path
        // so this function needs to own the path while that closure's being used
        if let Some(mut path) =
            usize::try_from(path_id).ok().and_then(|id| self.assets.paths.get_mut(id)).and_then(Option::take)
        {
            let inst = self.room.instance_list.get(context.this);
            let coll = || {
                if checkall {
                    self.check_collision_any(context.this).is_some()
                } else {
                    self.check_collision_solid(context.this).is_some()
                }
            };
            pathfinding::make_path(inst, &mut path, |inst| {
                let (old_x, old_y) = (inst.x.get(), inst.y.get());
                if pathfinding::linear_step(xg, yg, step_size, inst, coll) {
                    pathfinding::PathGenResult::Done
                } else if inst.x.get() == old_x && inst.y.get() == old_y {
                    pathfinding::PathGenResult::Failed
                } else {
                    pathfinding::PathGenResult::NotDone
                }
            });
            self.assets.paths[path_id as usize] = Some(path);
            Ok(Default::default())
        } else {
            Err(gml::Error::NonexistentAsset(asset::Type::Path, path_id))
        }
    }

    pub fn mp_linear_step_object(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (x, y, step_size, obj) = expect_args!(args, [real, real, real, int])?;
        Ok(pathfinding::linear_step(x, y, step_size, self.room.instance_list.get(context.this), || match obj {
            gml::SELF => false,
            gml::OTHER => self.check_collision(context.this, context.other),
            obj => self.find_instance_with(obj, |handle| self.check_collision(context.this, handle)).is_some(),
        })
        .into())
    }

    pub fn mp_linear_path_object(&mut self, _context: &mut Context, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 5
        unimplemented!("Called unimplemented kernel function mp_linear_path_object")
    }

    pub fn mp_potential_settings(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (max_rotation, rotate_step, check_distance, rotate_on_spot) = expect_args!(args, [real, real, real, bool])?;
        self.potential_step_settings = pathfinding::PotentialStepSettings {
            max_rotation,
            rotate_step,
            check_distance,
            rotate_on_spot: rotate_on_spot,
        };
        Ok(Default::default())
    }

    pub fn mp_potential_step(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (x, y, step_size, checkall) = expect_args!(args, [real, real, real, bool])?;
        Ok(pathfinding::potential_step(
            x,
            y,
            step_size,
            &self.potential_step_settings,
            self.room.instance_list.get(context.this),
            || {
                if checkall {
                    self.check_collision_any(context.this).is_some()
                } else {
                    self.check_collision_solid(context.this).is_some()
                }
            },
        )
        .into())
    }

    pub fn mp_potential_path(&mut self, _context: &mut Context, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 6
        unimplemented!("Called unimplemented kernel function mp_potential_path")
    }

    pub fn mp_potential_step_object(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (x, y, step_size, obj) = expect_args!(args, [real, real, real, int])?;
        Ok(pathfinding::potential_step(
            x,
            y,
            step_size,
            &self.potential_step_settings,
            self.room.instance_list.get(context.this),
            || match obj {
                gml::SELF => false,
                gml::OTHER => self.check_collision(context.this, context.other),
                obj => self.find_instance_with(obj, |handle| self.check_collision(context.this, handle)).is_some(),
            },
        )
        .into())
    }

    pub fn mp_potential_path_object(&mut self, _context: &mut Context, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 6
        unimplemented!("Called unimplemented kernel function mp_potential_path_object")
    }

    pub fn mp_grid_create(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (left, top, hcells, vcells, cellwidth, cellheight) = expect_args!(args, [int, int, int, int, int, int])?;
        if hcells < 0 || vcells < 0 {
            return Err(gml::Error::FunctionError(
                "mp_grid_create".into(),
                "mp grids cannot have negative dimensions".to_string(),
            ))
        }
        Ok(self
            .mpgrids
            .put(pathfinding::MpGrid::new(left, top, hcells as usize, vcells as usize, cellwidth, cellheight))
            .into())
    }

    pub fn mp_grid_destroy(&mut self, args: &[Value]) -> gml::Result<Value> {
        let id = expect_args!(args, [int])?;
        if self.mpgrids.delete(id) {
            Ok(Default::default())
        } else {
            Err(gml::Error::FunctionError(
                "mp_grid_destroy".into(),
                pathfinding::Error::NonexistentStructure(id).into(),
            ))
        }
    }

    pub fn mp_grid_clear_all(&mut self, args: &[Value]) -> gml::Result<Value> {
        let id = expect_args!(args, [int])?;
        match self.mpgrids.get_mut(id) {
            Some(mpgrid) => {
                for x in 0..mpgrid.hcells {
                    for y in 0..mpgrid.vcells {
                        mpgrid.set(x, y, 0);
                    }
                }
                Ok(Default::default())
            },
            None => Err(gml::Error::FunctionError(
                "mp_grid_clear_all".into(),
                pathfinding::Error::NonexistentStructure(id).into(),
            )),
        }
    }

    pub fn mp_grid_clear_cell(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (id, x, y) = expect_args!(args, [int, int, int])?;
        match self.mpgrids.get_mut(id) {
            Some(mpgrid) => {
                if x >= 0 && y >= 0 {
                    mpgrid.set(x as usize, y as usize, 0);
                }
                Ok(Default::default())
            },
            None => Err(gml::Error::FunctionError(
                "mp_grid_clear_cell".into(),
                pathfinding::Error::NonexistentStructure(id).into(),
            )),
        }
    }

    pub fn mp_grid_clear_rectangle(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (id, left, top, right, bottom) = expect_args!(args, [int, int, int, int, int])?;
        match self.mpgrids.get_mut(id) {
            Some(mpgrid) => {
                let (left, right) = if right < left { (right, left) } else { (left, right) };
                let (top, bottom) = if bottom < top { (bottom, top) } else { (top, bottom) };

                let gl = (((left - mpgrid.left) / mpgrid.cellwidth).max(0) as usize).min(mpgrid.hcells - 1);
                let gt = (((top - mpgrid.top) / mpgrid.cellheight).max(0) as usize).min(mpgrid.vcells - 1);
                let gr = (((right - mpgrid.left) / mpgrid.cellwidth).max(0) as usize).min(mpgrid.hcells - 1);
                let gb = (((bottom - mpgrid.top) / mpgrid.cellheight).max(0) as usize).min(mpgrid.vcells - 1);

                for x in gl..=gr {
                    for y in gt..=gb {
                        mpgrid.set(x, y, 0);
                    }
                }

                Ok(Default::default())
            },
            None => Err(gml::Error::FunctionError(
                "mp_grid_add_rectangle".into(),
                pathfinding::Error::NonexistentStructure(id).into(),
            )),
        }
    }

    pub fn mp_grid_add_cell(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (id, x, y) = expect_args!(args, [int, int, int])?;
        match self.mpgrids.get_mut(id) {
            Some(mpgrid) => {
                if x >= 0 && y >= 0 {
                    mpgrid.set(x as usize, y as usize, -1);
                }
                Ok(Default::default())
            },
            None => Err(gml::Error::FunctionError(
                "mp_grid_clear_cell".into(),
                pathfinding::Error::NonexistentStructure(id).into(),
            )),
        }
    }

    pub fn mp_grid_add_rectangle(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (id, left, top, right, bottom) = expect_args!(args, [int, int, int, int, int])?;
        match self.mpgrids.get_mut(id) {
            Some(mpgrid) => {
                let (left, right) = if right < left { (right, left) } else { (left, right) };
                let (top, bottom) = if bottom < top { (bottom, top) } else { (top, bottom) };

                let gl = (((left - mpgrid.left) / mpgrid.cellwidth).max(0) as usize).min(mpgrid.hcells - 1);
                let gt = (((top - mpgrid.top) / mpgrid.cellheight).max(0) as usize).min(mpgrid.vcells - 1);
                let gr = (((right - mpgrid.left) / mpgrid.cellwidth).max(0) as usize).min(mpgrid.hcells - 1);
                let gb = (((bottom - mpgrid.top) / mpgrid.cellheight).max(0) as usize).min(mpgrid.vcells - 1);

                for x in gl..=gr {
                    for y in gt..=gb {
                        mpgrid.set(x, y, -1);
                    }
                }

                Ok(Default::default())
            },
            None => Err(gml::Error::FunctionError(
                "mp_grid_add_rectangle".into(),
                pathfinding::Error::NonexistentStructure(id).into(),
            )),
        }
    }

    pub fn mp_grid_add_instances(&mut self, _context: &mut Context, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 3
        unimplemented!("Called unimplemented kernel function mp_grid_add_instances")
    }

    pub fn mp_grid_path(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 7
        unimplemented!("Called unimplemented kernel function mp_grid_path")
    }

    pub fn mp_grid_draw(&mut self, args: &[Value]) -> gml::Result<Value> {
        let id = expect_args!(args, [int])?;
        match self.mpgrids.get_mut(id) {
            Some(mpgrid) => {
                for x in 0..mpgrid.hcells {
                    for y in 0..mpgrid.vcells {
                        let x1 = mpgrid.left + x as i32 * mpgrid.cellwidth;
                        let y1 = mpgrid.top + y as i32 * mpgrid.cellheight;
                        let x2 = x1 + mpgrid.cellwidth;
                        let y2 = y1 + mpgrid.cellheight;
                        let c = if mpgrid.get(x, y) < 0 { 0x0000ff } else { 0x008000 };

                        self.renderer.draw_rectangle_gradient(
                            x1.into(),
                            y1.into(),
                            x2.into(),
                            y2.into(),
                            c,
                            c,
                            c,
                            c,
                            self.draw_alpha.into(),
                            false,
                        );
                    }
                }
                Ok(Default::default())
            },
            None => Err(gml::Error::FunctionError(
                "mp_grid_draw".into(),
                pathfinding::Error::NonexistentStructure(id).into(),
            )),
        }
    }

    pub fn collision_point(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (x, y, object_id, precise, exclude_self) = expect_args!(args, [real, real, int, bool, bool])?;
        match self.find_instance_with(object_id, |handle| {
            (!exclude_self || handle != context.this) && self.check_collision_point(handle, x, y, precise)
        }) {
            Some(handle) => Ok(self.room.instance_list.get(handle).id.get().into()),
            None => Ok(gml::NOONE.into()),
        }
    }

    pub fn collision_rectangle(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (x1, y1, x2, y2, object_id, precise, exclude_self) =
            expect_args!(args, [int, int, int, int, int, bool, bool])?;
        match self.find_instance_with(object_id, |handle| {
            (!exclude_self || handle != context.this) && self.check_collision_rectangle(handle, x1, y1, x2, y2, precise)
        }) {
            Some(handle) => Ok(self.room.instance_list.get(handle).id.get().into()),
            None => Ok(gml::NOONE.into()),
        }
    }

    pub fn collision_circle(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (x, y, r, object_id, precise, exclude_self) = expect_args!(args, [real, real, real, int, bool, bool])?;
        match self.find_instance_with(object_id, |handle| {
            (!exclude_self || handle != context.this)
                && self.check_collision_ellipse(handle, x - r, y - r, x + r, y + r, precise)
        }) {
            Some(handle) => Ok(self.room.instance_list.get(handle).id.get().into()),
            None => Ok(gml::NOONE.into()),
        }
    }

    pub fn collision_ellipse(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (x1, y1, x2, y2, object_id, precise, exclude_self) =
            expect_args!(args, [real, real, real, real, int, bool, bool])?;
        match self.find_instance_with(object_id, |handle| {
            (!exclude_self || handle != context.this) && self.check_collision_ellipse(handle, x1, y1, x2, y2, precise)
        }) {
            Some(handle) => Ok(self.room.instance_list.get(handle).id.get().into()),
            None => Ok(gml::NOONE.into()),
        }
    }

    pub fn collision_line(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (x1, y1, x2, y2, object_id, precise, exclude_self) =
            expect_args!(args, [real, real, real, real, int, bool, bool])?;
        match self.find_instance_with(object_id, |handle| {
            (!exclude_self || handle != context.this) && self.check_collision_line(handle, x1, y1, x2, y2, precise)
        }) {
            Some(handle) => Ok(self.room.instance_list.get(handle).id.get().into()),
            None => Ok(gml::NOONE.into()),
        }
    }

    pub fn instance_find(&self, args: &[Value]) -> gml::Result<Value> {
        let (obj, n) = expect_args!(args, [int, int])?;
        if n < 0 {
            return Ok(gml::NOONE.into())
        }
        let handle = match obj {
            gml::ALL => {
                let mut iter = self.room.instance_list.iter_by_insertion();
                (0..n + 1).filter_map(|_| iter.next(&self.room.instance_list)).nth(n as usize)
            },
            _ if obj < 0 => None,
            obj if obj < 100000 => {
                let mut iter = self.room.instance_list.iter_by_identity(obj);
                (0..n + 1).filter_map(|_| iter.next(&self.room.instance_list)).nth(n as usize)
            },
            inst_id => {
                if n != 0 {
                    None
                } else {
                    self.room
                        .instance_list
                        .get_by_instid(inst_id)
                        .filter(|h| self.room.instance_list.get(*h).state.get() == InstanceState::Active)
                }
            },
        };
        Ok(handle.map(|h| self.room.instance_list.get(h).id.get()).unwrap_or(gml::NOONE).into())
    }

    pub fn instance_exists(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let obj = expect_args!(args, [int])?;
        let exists = match obj {
            gml::SELF => self.room.instance_list.get(context.this).state.get() == InstanceState::Active,
            gml::OTHER => self.room.instance_list.get(context.other).state.get() == InstanceState::Active,
            gml::ALL => self.room.instance_list.any_active(),
            obj if obj <= 100000 => self.room.instance_list.count(obj) != 0,
            _ => self.room.instance_list.get_by_instid(obj).is_some(),
        };
        Ok(exists.into())
    }

    pub fn instance_number(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let object_id = expect_args!(args, [int])?;
        let number = match object_id {
            gml::SELF => {
                if self.room.instance_list.get(context.this).state.get() == InstanceState::Active {
                    1
                } else {
                    0
                }
            },
            gml::OTHER => {
                if self.room.instance_list.get(context.other).state.get() == InstanceState::Active {
                    1
                } else {
                    0
                }
            },
            gml::ALL => self.room.instance_list.count_all_active(),
            obj if obj <= 100000 => self.room.instance_list.count(obj),
            inst_id => {
                if self.room.instance_list.get_by_instid(inst_id).is_some() {
                    1
                } else {
                    0
                }
            },
        };
        Ok(number.into())
    }

    pub fn instance_position(&self, args: &[Value]) -> gml::Result<Value> {
        let (x, y, object_id) = expect_args!(args, [real, real, int])?;
        match self.find_instance_with(object_id, |handle| self.check_collision_point(handle, x, y, true)) {
            Some(handle) => Ok(self.room.instance_list.get(handle).id.get().into()),
            None => Ok(gml::NOONE.into()),
        }
    }

    pub fn instance_nearest(&self, args: &[Value]) -> gml::Result<Value> {
        let (x, y, obj) = expect_args!(args, [real, real, int])?;
        // Check collision with target
        let nearest = match obj {
            gml::ALL => {
                // Target is all objects
                let mut iter = self.room.instance_list.iter_by_insertion();
                let mut maxdist = Real::from(10000000000.0); // GML default
                let mut nearest = None;
                loop {
                    match iter.next(&self.room.instance_list) {
                        Some(target) => {
                            let ti = self.room.instance_list.get(target);
                            let xdist = ti.x.get() - x;
                            let ydist = ti.y.get() - y;
                            let dist = (xdist * xdist) + (ydist * ydist);
                            if dist < maxdist {
                                maxdist = dist;
                                nearest = Some(target);
                            }
                        },
                        None => break nearest,
                    }
                }
            },
            obj if obj >= 0 && obj < 100000 => {
                // Target is an object ID
                let mut iter = self.room.instance_list.iter_by_identity(obj);
                let mut maxdist = Real::from(10000000000.0); // GML default
                let mut nearest = None;
                loop {
                    match iter.next(&self.room.instance_list) {
                        Some(target) => {
                            let ti = self.room.instance_list.get(target);
                            let xdist = ti.x.get() - x;
                            let ydist = ti.y.get() - y;
                            let dist = (xdist * xdist) + (ydist * ydist);
                            if dist < maxdist {
                                maxdist = dist;
                                nearest = Some(target);
                            }
                        },
                        None => break nearest,
                    }
                }
            },
            // Target is an instance id
            _ => None,
        };

        match nearest {
            Some(t) => Ok(self.room.instance_list.get(t).id.get().into()),
            None => Ok(gml::NOONE.into()),
        }
    }

    pub fn instance_furthest(&self, args: &[Value]) -> gml::Result<Value> {
        let (x, y, obj) = expect_args!(args, [real, real, int])?;
        // Check collision with target
        let other: Option<usize> = match obj {
            gml::ALL => {
                // Target is an object ID
                let mut iter = self.room.instance_list.iter_by_insertion();
                let mut maxdist = Real::from(0.0);
                let mut nearest = None;
                loop {
                    match iter.next(&self.room.instance_list) {
                        Some(target) => {
                            let ti = self.room.instance_list.get(target);
                            let xdist = ti.x.get() - x;
                            let ydist = ti.y.get() - y;
                            let dist = (xdist * xdist) + (ydist * ydist);
                            if nearest.is_none() || dist > maxdist {
                                maxdist = dist;
                                nearest = Some(target);
                            }
                        },
                        None => break nearest,
                    }
                }
            },
            obj if obj >= 0 && obj < 100000 => {
                // Target is an object ID
                let mut iter = self.room.instance_list.iter_by_identity(obj);
                let mut maxdist = Real::from(0.0);
                let mut nearest = None;
                loop {
                    match iter.next(&self.room.instance_list) {
                        Some(target) => {
                            let ti = self.room.instance_list.get(target);
                            let xdist = ti.x.get() - x;
                            let ydist = ti.y.get() - y;
                            let dist = (xdist * xdist) + (ydist * ydist);
                            if nearest.is_none() || dist > maxdist {
                                maxdist = dist;
                                nearest = Some(target);
                            }
                        },
                        None => break nearest,
                    }
                }
            },
            // Target is an instance ID
            _ => None,
        };

        match other {
            Some(t) => Ok(self.room.instance_list.get(t).id.get().into()),
            None => Ok(gml::NOONE.into()),
        }
    }

    pub fn instance_place(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (x, y, obj) = expect_args!(args, [real, real, int])?;

        // Set self's position to the new coordinates
        let instance = self.room.instance_list.get(context.this);
        let old_x = instance.x.get();
        let old_y = instance.y.get();
        instance.x.set(x);
        instance.y.set(y);
        instance.bbox_is_stale.set(true);

        // Check collision with target
        let other =
            self.find_instance_with(obj, |handle| handle != context.this && self.check_collision(context.this, handle));

        // Move self back to where it was
        instance.x.set(old_x);
        instance.y.set(old_y);
        instance.bbox_is_stale.set(true);

        match other {
            Some(t) => Ok(self.room.instance_list.get(t).id.get().into()),
            None => Ok(gml::NOONE.into()),
        }
    }

    pub fn instance_create(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (x, y, object_id) = expect_args!(args, [real, real, int])?;
        let object = self
            .assets
            .objects
            .get_asset(object_id)
            .ok_or(gml::Error::NonexistentAsset(asset::Type::Object, object_id))?;
        self.last_instance_id += 1;
        let id = self.last_instance_id;
        let instance = self.room.instance_list.insert(Instance::new(id, x, y, object_id, object));
        self.run_instance_event(gml::ev::CREATE, 0, instance, instance, None)?;
        Ok(id.into())
    }

    pub fn instance_copy(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let run_event = expect_args!(args, [bool])?;
        let new_instance = self.room.instance_list.get(context.this).clone();
        self.last_instance_id += 1;
        let id = self.last_instance_id;
        new_instance.id.set(id);
        let handle = self.room.instance_list.insert(new_instance);
        if run_event {
            self.run_instance_event(gml::ev::CREATE, 0, handle, handle, None)?;
        }
        Ok(id.into())
    }

    pub fn instance_change(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (object_id, run_events) = expect_args!(args, [int, bool])?;

        if run_events {
            self.run_instance_event(gml::ev::DESTROY, 0, context.this, context.this, None)?;
        }

        let object = self
            .assets
            .objects
            .get_asset(object_id)
            .ok_or(gml::Error::NonexistentAsset(asset::Type::Object, object_id))?;
        let mut new_instance = self.room.instance_list.get(context.this).clone();
        new_instance.object_index.set(object_id);
        new_instance.sprite_index.set(object.sprite_index);
        new_instance.mask_index.set(object.mask_index);
        new_instance.depth.set(Real::from(object.depth));
        new_instance.solid.set(object.solid);
        new_instance.visible.set(object.visible);
        new_instance.persistent.set(object.persistent);
        new_instance.parents = object.parents.clone();
        self.last_instance_id += 1; // This is incremented by GM8 but not used

        let frame_count = if let Some(sprite) = self.assets.sprites.get_asset(object.sprite_index) {
            sprite.frames.len() as f64
        } else {
            0.0
        };
        if frame_count <= new_instance.image_index.get().floor().into() {
            new_instance.image_index.set(Real::from(0.0));
        }
        new_instance.bbox_is_stale.set(true);

        self.room.instance_list.mark_deleted(context.this);
        let handle = self.room.instance_list.insert(new_instance);

        if run_events {
            self.run_instance_event(gml::ev::CREATE, 0, handle, handle, None)?;
        }

        Ok(Default::default())
    }

    pub fn instance_destroy(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        self.run_instance_event(gml::ev::DESTROY, 0, context.this, context.this, None)?;
        self.room.instance_list.mark_deleted(context.this);
        Ok(Default::default())
    }

    pub fn instance_sprite(&mut self, _context: &mut Context, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        unimplemented!("Called unimplemented kernel function instance_sprite")
    }

    pub fn position_empty(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (x, y) = expect_args!(args, [any, any])?;
        Ok((!self.position_meeting(context, &[gml::ALL.into(), x, y])?.is_truthy()).into())
    }

    pub fn position_meeting(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (x, y, object_id) = expect_args!(args, [real, real, int])?;
        let meeting = match object_id {
            gml::SELF => self.check_collision_point(context.this, x, y, true),
            gml::OTHER => self.check_collision_point(context.other, x, y, true),
            obj => self.find_instance_with(obj, |handle| self.check_collision_point(handle, x, y, true)).is_some(),
        };
        Ok(meeting.into())
    }

    pub fn position_destroy(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (x, y) = expect_args!(args, [real, real])?;
        let mut iter = self.room.instance_list.iter_by_insertion();
        while let Some(handle) = iter.next(&self.room.instance_list) {
            if self.check_collision_point(handle, x, y, true) {
                self.run_instance_event(gml::ev::DESTROY, 0, handle, handle, None)?;
                self.room.instance_list.mark_deleted(handle);
            }
        }
        Ok(Default::default())
    }

    pub fn position_change(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 4
        unimplemented!("Called unimplemented kernel function position_change")
    }

    pub fn instance_deactivate_all(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let notme = expect_args!(args, [bool])?;
        let mut iter = self.room.instance_list.iter_by_insertion();
        while let Some(handle) = iter.next(&self.room.instance_list) {
            self.room.instance_list.deactivate(handle);
        }
        if notme {
            self.room.instance_list.activate(context.this);
        }
        Ok(Default::default())
    }

    pub fn instance_deactivate_object(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let obj = expect_args!(args, [int])?;
        match obj {
            gml::SELF => self.room.instance_list.deactivate(context.this),
            gml::OTHER => self.room.instance_list.deactivate(context.other),
            gml::ALL => {
                let mut iter = self.room.instance_list.iter_by_insertion();
                while let Some(handle) = iter.next(&self.room.instance_list) {
                    self.room.instance_list.deactivate(handle);
                }
            },
            obj if obj < 100000 => {
                let mut iter = self.room.instance_list.iter_by_identity(obj);
                while let Some(handle) = iter.next(&self.room.instance_list) {
                    self.room.instance_list.deactivate(handle);
                }
            },
            inst_id => {
                // fun fact: in gm8 you can deactivate dead instances
                // this changes nothing about their deadness
                if let Some(handle) = self.room.instance_list.get_by_instid(inst_id) {
                    self.room.instance_list.deactivate(handle);
                }
            },
        }
        Ok(Default::default())
    }

    pub fn instance_deactivate_region(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (left, top, width, height, inside, notme) = expect_args!(args, [real, real, real, real, bool, bool])?;
        let mut iter = self.room.instance_list.iter_by_insertion();
        while let Some(handle) = iter.next(&self.room.instance_list) {
            let inst = self.room.instance_list.get(handle);
            let mask = self.get_instance_mask_sprite(handle);
            let outside = if mask.is_some() {
                inst.update_bbox(mask);
                left > inst.bbox_right.get().into()
                    || top > inst.bbox_bottom.get().into()
                    || left + width < inst.bbox_left.get().into()
                    || top + height < inst.bbox_top.get().into()
            } else {
                inst.x.get() < left || inst.x.get() > left + width || inst.y.get() < top || inst.y.get() > top + height
            };
            if outside != inside {
                self.room.instance_list.deactivate(handle);
            }
        }
        if notme {
            self.room.instance_list.activate(context.this);
        }
        Ok(Default::default())
    }

    pub fn instance_activate_all(&mut self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        let mut iter = self.room.instance_list.iter_inactive();
        while let Some(handle) = iter.next(&self.room.instance_list) {
            self.room.instance_list.activate(handle);
        }
        Ok(Default::default())
    }

    pub fn instance_activate_object(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let obj = expect_args!(args, [int])?;
        match obj {
            gml::SELF => self.room.instance_list.activate(context.this),
            gml::OTHER => self.room.instance_list.activate(context.other),
            gml::ALL => {
                let mut iter = self.room.instance_list.iter_inactive();
                while let Some(handle) = iter.next(&self.room.instance_list) {
                    self.room.instance_list.activate(handle);
                }
            },
            obj if obj < 100000 => {
                let mut iter = self.room.instance_list.iter_inactive();
                while let Some(handle) = iter.next(&self.room.instance_list) {
                    let inst = self.room.instance_list.get(handle);
                    if inst.parents.borrow().contains(&obj) {
                        self.room.instance_list.activate(handle);
                    }
                }
            },
            inst_id => {
                let mut iter = self.room.instance_list.iter_inactive();
                while let Some(handle) = iter.next(&self.room.instance_list) {
                    let inst = self.room.instance_list.get(handle);
                    if inst.id.get() == inst_id {
                        self.room.instance_list.activate(handle);
                        break // gm8 doesn't short circuit
                    }
                }
            },
        }
        Ok(Default::default())
    }

    pub fn instance_activate_region(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (left, top, width, height, inside) = expect_args!(args, [real, real, real, real, bool])?;
        let mut iter = self.room.instance_list.iter_inactive();
        while let Some(handle) = iter.next(&self.room.instance_list) {
            let inst = self.room.instance_list.get(handle);
            let mask = self.get_instance_mask_sprite(handle);
            let outside = if mask.is_some() {
                inst.update_bbox(mask);
                left > inst.bbox_right.get().into()
                    || top > inst.bbox_bottom.get().into()
                    || left + width < inst.bbox_left.get().into()
                    || top + height < inst.bbox_top.get().into()
            } else {
                inst.x.get() < left || inst.x.get() > left + width || inst.y.get() < top || inst.y.get() > top + height
            };
            if outside != inside {
                self.room.instance_list.activate(handle);
            }
        }
        Ok(Default::default())
    }

    pub fn room_goto(&mut self, args: &[Value]) -> gml::Result<Value> {
        let target = expect_args!(args, [int])?;
        self.scene_change = Some(SceneChange::Room(target));
        Ok(Default::default())
    }

    pub fn room_goto_previous(&mut self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        match self
            .room_order
            .iter()
            .position(|x| *x == self.room.id)
            .and_then(|x| x.checked_sub(1))
            .and_then(|x| self.room_order.get(x).copied())
        {
            Some(i) => {
                self.scene_change = Some(SceneChange::Room(i));
                Ok(Default::default())
            },
            None => Err(gml::Error::EndOfRoomOrder),
        }
    }

    pub fn room_goto_next(&mut self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        match self.room_order.iter().position(|x| *x == self.room.id).and_then(|x| self.room_order.get(x + 1).copied())
        {
            Some(i) => {
                self.scene_change = Some(SceneChange::Room(i));
                Ok(Default::default())
            },
            None => Err(gml::Error::EndOfRoomOrder),
        }
    }

    pub fn room_previous(&self, args: &[Value]) -> gml::Result<Value> {
        let room = expect_args!(args, [int])?;
        Ok(self
            .room_order
            .iter()
            .position(|x| *x == room)
            .and_then(|x| x.checked_sub(1))
            .and_then(|x| self.room_order.get(x).copied())
            .unwrap_or(-1)
            .into())
    }

    pub fn room_next(&self, args: &[Value]) -> gml::Result<Value> {
        let room = expect_args!(args, [int])?;
        Ok(self
            .room_order
            .iter()
            .position(|x| *x == room)
            .and_then(|x| self.room_order.get(x + 1).copied())
            .unwrap_or(-1)
            .into())
    }

    pub fn room_restart(&mut self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        self.scene_change = Some(SceneChange::Room(self.room.id));
        Ok(Default::default())
    }

    pub fn game_end(&mut self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        self.scene_change = Some(SceneChange::End);
        Ok(Default::default())
    }

    pub fn game_restart(&mut self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        self.scene_change = Some(SceneChange::Restart);
        Ok(Default::default())
    }

    pub fn game_load(&mut self, args: &[Value]) -> gml::Result<Value> {
        let fname = expect_args!(args, [string])?;
        self.scene_change = Some(SceneChange::Load(fname.into_owned().into()));
        Ok(Default::default())
    }

    pub fn game_save(&mut self, args: &[Value]) -> gml::Result<Value> {
        let fname = expect_args!(args, [string])?;
        let save = GMSave::from_game(self);
        let mut file = std::fs::File::create(fname.as_ref())
            .map(std::io::BufWriter::new)
            .map_err(|e| gml::Error::FunctionError("game_save".into(), format!("{}", e)))?;
        // write magic number (0x21c in GM8)
        file.write(&[0x1d, 0x02, 0x00, 0x00])
            .map_err(|e| gml::Error::FunctionError("game_save".into(), format!("{}", e)))?;
        bincode::serialize_into(&mut file, &save)
            .map_err(|e| gml::Error::FunctionError("game_save".into(), format!("{}", e)))?;
        file.flush().map_err(|e| gml::Error::FunctionError("game_save".into(), e.to_string()))?;
        Ok(Default::default())
    }

    pub fn transition_define(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (id, script_name) = expect_args!(args, [int, bytes])?;
        self.user_transitions.insert(id, UserTransition { script_name });
        Ok(Default::default())
    }

    pub fn transition_exists(&mut self, args: &[Value]) -> gml::Result<Value> {
        let transition_id = expect_args!(args, [int])?;
        Ok(self.get_transition(transition_id).is_some().into())
    }

    pub fn sleep(&mut self, args: &[Value]) -> gml::Result<Value> {
        let millis = expect_args!(args, [int])?;
        if millis > 0 {
            datetime::sleep(std::time::Duration::from_millis(millis as u64));
            if let Some(ns) = self.spoofed_time_nanos.as_mut() {
                *ns += (millis as u128) * 1_000_000;
            }
            self.process_window_events();
        }
        Ok(Default::default())
    }

    pub fn yoyo_getplatform(args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        Ok(gml::GM81_OS_TYPE.into())
    }

    pub fn yoyo_getdevice(args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        Ok(gml::GM81_OS_DEVICE.into())
    }

    pub fn yoyo_openurl(args: &[Value]) -> gml::Result<Value> {
        // Special GM 8.1 function, has effect only in the original HTML5 runner.
        expect_args!(args, [any])?;
        Ok(Default::default())
    }

    pub fn yoyo_openurl_ext(args: &[Value]) -> gml::Result<Value> {
        // Special GM 8.1 function, has effect only in the original HTML5 runner.
        expect_args!(args, [any, any])?;
        Ok(Default::default())
    }

    pub fn yoyo_openurl_full(args: &[Value]) -> gml::Result<Value> {
        // Special GM 8.1 function, has effect only in the original HTML5 runner.
        expect_args!(args, [any, any, any])?;
        Ok(Default::default())
    }

    pub fn yoyo_getdomain(args: &[Value]) -> gml::Result<Value> {
        // Special GM 8.1 function, has effect only in the original HTML5 runner.
        expect_args!(args, [])?;
        Ok(b"unknown".as_ref().into())
    }

    pub fn yoyo_gettimer(args: &[Value]) -> gml::Result<Value> {
        // Special GM 8.1 function, has effect only in the original HTML5 runner.
        expect_args!(args, [])?;
        Ok(Default::default())
    }

    pub fn yoyo_addvirtualkey(args: &[Value]) -> gml::Result<Value> {
        // Special GM 8.1 function, has effect only in the original HTML5 runner.
        expect_args!(args, [any, any, any, any, any])?;
        Ok(Default::default())
    }

    pub fn yoyo_deletevirtualkey(args: &[Value]) -> gml::Result<Value> {
        // Special GM 8.1 function, has effect only in the original HTML5 runner.
        expect_args!(args, [any])?;
        Ok(Default::default())
    }

    pub fn yoyo_showvirtualkey(args: &[Value]) -> gml::Result<Value> {
        // Special GM 8.1 function, has effect only in the original HTML5 runner.
        expect_args!(args, [any])?;
        Ok(Default::default())
    }

    pub fn yoyo_hidevirtualkey(args: &[Value]) -> gml::Result<Value> {
        // Special GM 8.1 function, has effect only in the original HTML5 runner.
        expect_args!(args, [any])?;
        Ok(Default::default())
    }

    pub fn yoyo_enablealphablend(&mut self, args: &[Value]) -> gml::Result<Value> {
        let alphablend = expect_args!(args, [bool])?;
        self.renderer.set_alpha_blending(alphablend);
        Ok(Default::default())
    }

    pub fn file_bin_open(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (filename, mode) = expect_args!(args, [string, int])?;
        let mode = match mode {
            0 => file::AccessMode::Read,
            1 => file::AccessMode::Write,
            2 | _ => file::AccessMode::Special,
        };
        match self.binary_files.add_from(|| Ok(file::BinaryHandle::open(filename.as_ref(), mode)?)) {
            Ok(i) => Ok((i + 1).into()),
            Err(e) => Err(gml::Error::FunctionError("file_bin_open".into(), e.to_string())),
        }
    }

    pub fn file_bin_rewrite(&mut self, args: &[Value]) -> gml::Result<Value> {
        let handle = expect_args!(args, [int])?;
        match self.binary_files.get_mut(handle - 1).map_or(Err(file::Error::InvalidFile(handle)), |f| f.clear()) {
            Ok(()) => Ok(Default::default()),
            Err(e) => Err(gml::Error::FunctionError("file_bin_rewrite".into(), e.to_string())),
        }
    }

    pub fn file_bin_close(&mut self, args: &[Value]) -> gml::Result<Value> {
        let handle = expect_args!(args, [int])?;

        // flush buffer if possible
        self.binary_files
            .get_mut(handle - 1)
            .map_or(Err(file::Error::InvalidFile(handle)), |f| f.flush())
            .map_err(|e| gml::Error::FunctionError("file_bin_close".into(), e.to_string()))?;

        if self.binary_files.delete(handle - 1) {
            Ok(Default::default())
        } else {
            Err(gml::Error::FunctionError("file_bin_close".into(), file::Error::InvalidFile(handle).to_string()))
        }
    }

    pub fn file_bin_position(&mut self, args: &[Value]) -> gml::Result<Value> {
        let handle = expect_args!(args, [int])?;
        match self.binary_files.get_mut(handle - 1).map_or(Err(file::Error::InvalidFile(handle)), |f| f.tell()) {
            Ok(p) => Ok(f64::from(p as i32).into()),
            Err(e) => Err(gml::Error::FunctionError("file_bin_position".into(), e.to_string())),
        }
    }

    pub fn file_bin_size(&mut self, args: &[Value]) -> gml::Result<Value> {
        let handle = expect_args!(args, [int])?;
        match self.binary_files.get_mut(handle - 1).map_or(Err(file::Error::InvalidFile(handle)), |f| f.size()) {
            Ok(l) => Ok(f64::from(l as i32).into()),
            Err(e) => Err(gml::Error::FunctionError("file_bin_size".into(), e.to_string())),
        }
    }

    pub fn file_bin_seek(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (handle, pos) = expect_args!(args, [int, int])?;
        match self.binary_files.get_mut(handle - 1).map_or(Err(file::Error::InvalidFile(handle)), |f| f.seek(pos)) {
            Ok(()) => Ok(Default::default()),
            Err(e) => Err(gml::Error::FunctionError("file_bin_seek".into(), e.to_string())),
        }
    }

    pub fn file_bin_read_byte(&mut self, args: &[Value]) -> gml::Result<Value> {
        let handle = expect_args!(args, [int])?;
        match self.binary_files.get_mut(handle - 1).map_or(Err(file::Error::InvalidFile(handle)), |f| f.read_byte()) {
            Ok(b) => Ok(f64::from(b).into()),
            Err(e) => Err(gml::Error::FunctionError("file_bin_read_byte".into(), e.to_string())),
        }
    }

    pub fn file_bin_write_byte(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (handle, byte) = expect_args!(args, [int, int])?;
        match self
            .binary_files
            .get_mut(handle - 1)
            .map_or(Err(file::Error::InvalidFile(handle)), |f| f.write_byte(byte as u8))
        {
            Ok(()) => Ok(Default::default()),
            Err(e) => Err(gml::Error::FunctionError("file_bin_write_byte".into(), e.to_string())),
        }
    }

    pub fn file_text_open_read(&mut self, args: &[Value]) -> gml::Result<Value> {
        let filename = expect_args!(args, [string])?;
        use std::error::Error as _; // for .source() trait method

        match self.text_files.add_from(|| Ok(file::TextHandle::open(filename.as_ref(), file::AccessMode::Read)?)) {
            Ok(i) => Ok((i + 1).into()),
            Err(e)
                if e.source()
                    .and_then(|r| r.downcast_ref::<std::io::Error>())
                    .map_or(false, |s| s.kind() == std::io::ErrorKind::NotFound) =>
            {
                Ok((-1).into())
            },
            Err(e) => Err(gml::Error::FunctionError("file_text_open_read".into(), e.to_string())),
        }
    }

    pub fn file_text_open_write(&mut self, args: &[Value]) -> gml::Result<Value> {
        let filename = expect_args!(args, [string])?;
        match self.text_files.add_from(|| Ok(file::TextHandle::open(filename.as_ref(), file::AccessMode::Write)?)) {
            Ok(i) => Ok((i + 1).into()),
            Err(e) => Err(gml::Error::FunctionError("file_text_open_write".into(), e.to_string())),
        }
    }

    pub fn file_text_open_append(&mut self, args: &[Value]) -> gml::Result<Value> {
        let filename = expect_args!(args, [string])?;
        match self.text_files.add_from(|| Ok(file::TextHandle::open(filename.as_ref(), file::AccessMode::Special)?)) {
            Ok(i) => Ok((i + 1).into()),
            Err(e) => Err(gml::Error::FunctionError("file_text_open_append".into(), e.to_string())),
        }
    }

    pub fn file_text_close(&mut self, args: &[Value]) -> gml::Result<Value> {
        let handle = expect_args!(args, [int])?;
        let c = self.text_files.capacity();

        // flush buffer if possible
        self.text_files
            .get_mut(handle - 1)
            .map_or(Err(file::Error::InvalidFile(handle)), |f| f.flush())
            .map_err(|e| gml::Error::FunctionError("file_text_close".into(), e.to_string()))?;

        // NB: .delete() MUST be called - beware the short-circuit evaluation here!
        if self.text_files.delete(handle - 1) || (1..=c).contains(&handle) {
            Ok(Default::default())
        } else {
            Err(gml::Error::FunctionError("file_text_close".into(), file::Error::InvalidFile(handle).to_string()))
        }
    }

    pub fn file_text_read_string(&mut self, args: &[Value]) -> gml::Result<Value> {
        let handle = expect_args!(args, [int])?;
        match self.text_files.get_mut(handle - 1).map_or(Err(file::Error::InvalidFile(handle)), |f| f.read_string()) {
            Ok(s) => Ok(s.into()),
            Err(e) => Err(gml::Error::FunctionError("file_text_read_string".into(), e.to_string())),
        }
    }

    pub fn file_text_read_real(&mut self, args: &[Value]) -> gml::Result<Value> {
        let handle = expect_args!(args, [int])?;
        match self.text_files.get_mut(handle - 1).map_or(Err(file::Error::InvalidFile(handle)), |f| f.read_real()) {
            Ok(r) => Ok(r.into()),
            Err(e) => Err(gml::Error::FunctionError("file_text_read_real".into(), e.to_string())),
        }
    }

    pub fn file_text_readln(&mut self, args: &[Value]) -> gml::Result<Value> {
        let handle = expect_args!(args, [int])?;
        match self.text_files.get_mut(handle - 1).map_or(Err(file::Error::InvalidFile(handle)), |f| f.skip_line()) {
            Ok(()) => Ok(Default::default()),
            Err(e) => Err(gml::Error::FunctionError("file_text_readln".into(), e.to_string())),
        }
    }

    pub fn file_text_eof(&mut self, args: &[Value]) -> gml::Result<Value> {
        let handle = expect_args!(args, [int])?;
        match self.text_files.get_mut(handle - 1).map_or(Err(file::Error::InvalidFile(handle)), |f| f.is_eof()) {
            Ok(res) => Ok(res.into()),
            Err(e) => Err(gml::Error::FunctionError("file_text_eof".into(), e.to_string())),
        }
    }

    pub fn file_text_eoln(&mut self, args: &[Value]) -> gml::Result<Value> {
        let handle = expect_args!(args, [int])?;
        match self.text_files.get_mut(handle - 1).map_or(Err(file::Error::InvalidFile(handle)), |f| f.is_eoln()) {
            Ok(res) => Ok(res.into()),
            Err(e) => Err(gml::Error::FunctionError("file_text_eoln".into(), e.to_string())),
        }
    }

    pub fn file_text_write_string(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (handle, text) = expect_args!(args, [int, bytes])?;
        match self
            .text_files
            .get_mut(handle - 1)
            .map_or(Err(file::Error::InvalidFile(handle)), |f| f.write_string(text.as_ref()))
        {
            Ok(()) => Ok(Default::default()),
            Err(e) => Err(gml::Error::FunctionError("file_text_write_string".into(), e.to_string())),
        }
    }

    pub fn file_text_write_real(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (handle, num) = expect_args!(args, [int, real])?;
        match self
            .text_files
            .get_mut(handle - 1)
            .map_or(Err(file::Error::InvalidFile(handle)), |f| f.write_real(num.into()))
        {
            Ok(()) => Ok(Default::default()),
            Err(e) => Err(gml::Error::FunctionError("file_text_write_real".into(), e.to_string())),
        }
    }

    pub fn file_text_writeln(&mut self, args: &[Value]) -> gml::Result<Value> {
        let handle = expect_args!(args, [int])?;
        match self.text_files.get_mut(handle - 1).map_or(Err(file::Error::InvalidFile(handle)), |f| f.write_newline()) {
            Ok(()) => Ok(Default::default()),
            Err(e) => Err(gml::Error::FunctionError("file_text_writeln".into(), e.to_string())),
        }
    }

    pub fn file_open_read(&mut self, args: &[Value]) -> gml::Result<Value> {
        let filename = expect_args!(args, [string])?;
        match file::TextHandle::open(filename.as_ref(), file::AccessMode::Read) {
            Ok(f) => {
                self.open_file.replace(f);
            },
            Err(e) => {
                self.open_file.take();
                if e.kind() != std::io::ErrorKind::NotFound {
                    return Err(gml::Error::FunctionError("file_open_read".into(), e.to_string()))
                }
            },
        };
        Ok(Default::default())
    }

    pub fn file_open_write(&mut self, args: &[Value]) -> gml::Result<Value> {
        let filename = expect_args!(args, [string])?;
        match file::TextHandle::open(filename.as_ref(), file::AccessMode::Write) {
            Ok(f) => {
                self.open_file.replace(f);
                Ok(Default::default())
            },
            Err(e) => {
                self.open_file.take();
                Err(gml::Error::FunctionError("file_open_write".into(), e.to_string()))
            },
        }
    }

    pub fn file_open_append(&mut self, args: &[Value]) -> gml::Result<Value> {
        let filename = expect_args!(args, [string])?;
        match file::TextHandle::open(filename.as_ref(), file::AccessMode::Special) {
            Ok(f) => {
                self.open_file.replace(f);
                Ok(Default::default())
            },
            Err(e) => {
                self.open_file.take();
                Err(gml::Error::FunctionError("file_open_append".into(), e.to_string()))
            },
        }
    }

    pub fn file_close(&mut self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        match self.open_file.take() {
            Some(mut f) => match f.flush() {
                Ok(()) => Ok(Default::default()),
                Err(e) => Err(gml::Error::FunctionError("file_close".into(), e.to_string())),
            },
            None => Ok(Default::default()),
        }
    }

    pub fn file_read_string(&mut self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        match self.open_file.as_mut().map_or(Err(file::Error::LegacyFileUnopened), |f| f.read_string()) {
            Ok(s) => Ok(s.into()),
            Err(e) => Err(gml::Error::FunctionError("file_read_string".into(), e.to_string())),
        }
    }

    pub fn file_read_real(&mut self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        match self.open_file.as_mut().map_or(Err(file::Error::LegacyFileUnopened), |f| f.read_real()) {
            Ok(r) => Ok(r.into()),
            Err(e) => Err(gml::Error::FunctionError("file_read_real".into(), e.to_string())),
        }
    }

    pub fn file_readln(&mut self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        match self.open_file.as_mut().map_or(Err(file::Error::LegacyFileUnopened), |f| f.skip_line()) {
            Ok(()) => Ok(Default::default()),
            Err(e) => Err(gml::Error::FunctionError("file_readln".into(), e.to_string())),
        }
    }

    pub fn file_eof(&mut self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        match self.open_file.as_mut().map_or(Err(file::Error::LegacyFileUnopened), |f| f.is_eof()) {
            Ok(res) => Ok(res.into()),
            Err(e) => Err(gml::Error::FunctionError("file_eof".into(), e.to_string())),
        }
    }

    pub fn file_eoln(&mut self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        match self.open_file.as_mut().map_or(Err(file::Error::LegacyFileUnopened), |f| f.is_eoln()) {
            Ok(res) => Ok(res.into()),
            Err(e) => Err(gml::Error::FunctionError("file_eoln".into(), e.to_string())),
        }
    }

    pub fn file_write_string(&mut self, args: &[Value]) -> gml::Result<Value> {
        let text = expect_args!(args, [bytes])?;
        match self.open_file.as_mut().map_or(Err(file::Error::LegacyFileUnopened), |f| f.write_string(text.as_ref())) {
            Ok(()) => Ok(Default::default()),
            Err(e) => Err(gml::Error::FunctionError("file_write_string".into(), e.to_string())),
        }
    }

    pub fn file_write_real(&mut self, args: &[Value]) -> gml::Result<Value> {
        let num = expect_args!(args, [real])?;
        match self.open_file.as_mut().map_or(Err(file::Error::LegacyFileUnopened), |f| f.write_real(num.into())) {
            Ok(()) => Ok(Default::default()),
            Err(e) => Err(gml::Error::FunctionError("file_write_real".into(), e.to_string())),
        }
    }

    pub fn file_writeln(&mut self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        match self.open_file.as_mut().map_or(Err(file::Error::LegacyFileUnopened), |f| f.write_newline()) {
            Ok(()) => Ok(Default::default()),
            Err(e) => Err(gml::Error::FunctionError("file_writeln".into(), e.to_string())),
        }
    }

    pub fn file_exists(&self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [any]).map(|x| match x {
            Value::Str(s) => file::file_exists(&self.decode_str(s.as_ref())).into(),
            Value::Real(_) => gml::FALSE.into(),
        })
    }

    pub fn file_delete(&self, args: &[Value]) -> gml::Result<Value> {
        let filename = expect_args!(args, [string])?;
        match file::delete(filename.as_ref()) {
            Ok(()) => Ok(Default::default()),
            Err(e) => Err(gml::Error::FunctionError("file_delete".into(), e.to_string())),
        }
    }

    pub fn file_rename(&self, args: &[Value]) -> gml::Result<Value> {
        let (from, to) = expect_args!(args, [string, string])?;
        if file::rename(from.as_ref(), to.as_ref()).is_err() {
            // Fail silently
            eprintln!("Warning (file_rename): could not rename {} to {}", from, to);
        }
        Ok(Default::default())
    }

    pub fn file_copy(&self, args: &[Value]) -> gml::Result<Value> {
        let (from, to) = expect_args!(args, [string, string])?;
        if file::copy(from.as_ref(), to.as_ref()).is_err() {
            // Fail silently
            eprintln!("Warning (file_copy): could not copy {} to {}", from, to);
        }
        Ok(Default::default())
    }

    pub fn directory_exists(&self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [any]).map(|x| match x {
            Value::Str(s) => file::dir_exists(&self.decode_str(s.as_ref())).into(),
            Value::Real(_) => gml::FALSE.into(),
        })
    }

    pub fn directory_create(&self, args: &[Value]) -> gml::Result<Value> {
        let path = expect_args!(args, [string])?;
        match file::dir_create(path.as_ref()) {
            Ok(()) => Ok(Default::default()),
            Err(e) => Err(gml::Error::FunctionError("directory_create".into(), e.to_string())),
        }
    }

    pub fn file_find_first(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (path, attribs) = expect_args!(args, [string, int])?;
        if path.ends_with("/") || path.ends_with("\\") {
            // match nothing
            self.file_finder = None;
            return Ok(b"".as_ref().into())
        }
        // unwrap arguments
        let path: &str = path.as_ref();
        let include_read_only = (attribs & 1) != 0;
        let include_hidden = (attribs & 2) != 0;
        let include_sys_file = (attribs & 4) != 0;
        let include_volume_id = (attribs & 8) != 0;
        let include_directory = (attribs & 16) != 0;
        let include_archive = (attribs & 32) != 0;
        match glob::glob_with(path, glob::MatchOptions { case_sensitive: false, ..Default::default() }) {
            Ok(paths) => {
                // add . and .. to start if necessary
                let path: &std::path::Path = path.as_ref();
                let preceding: Vec<std::path::PathBuf> = match path.file_name().and_then(|p| p.to_str()) {
                    Some("*") | Some(".*") | Some("*.") => vec![".".into(), "..".into()],
                    Some(".") => vec![".".into()],
                    Some("..") => vec!["..".into()],
                    _ => vec![],
                };
                self.file_finder = Some(Box::new(
                    preceding.into_iter().chain(
                        paths
                            .filter_map(Result::ok)
                            .filter(move |p| {
                                let md = match p.metadata() {
                                    Ok(m) => m,
                                    Err(_) => return false,
                                };
                                let is_dir = md.is_dir();
                                // false means the check isn't in yet
                                // also note: apparently directories are read only?
                                (include_read_only || is_dir || !md.permissions().readonly())
                                    && (include_hidden || !false)
                                    && (include_sys_file || !false)
                                    && (include_volume_id || !false)
                                    && (include_directory || !is_dir)
                                    && (include_archive || !false)
                            })
                            .map(|p| p.file_name().map(|p| p.into()).unwrap_or(p)),
                    ),
                ));
                self.file_find_next(&[])
            },
            Err(e) => Err(gml::Error::FunctionError("file_find_first".into(), e.to_string())),
        }
    }

    pub fn file_find_next(&mut self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        while let Some(p) = self.file_finder.as_mut().and_then(|ff| ff.next()) {
            if let Some(p) = p.to_str().and_then(|p| self.encode_str_maybe(p)) {
                return Ok(Value::from(p.as_ref()))
            }
        }
        self.file_finder = None;
        Ok(b"".as_ref().into())
    }

    pub fn file_find_close(&mut self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        self.file_finder = None;
        Ok(Default::default())
    }

    pub fn file_attributes(&self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 2
        unimplemented!("Called unimplemented kernel function file_attributes")
    }

    pub fn filename_name(args: &[Value]) -> gml::Result<Value> {
        let full_path = expect_args!(args, [string])?;
        if let Some(name) = full_path.as_ref().rsplitn(2, '\\').next() {
            Ok(name.to_string().into())
        } else {
            Ok(full_path.as_ref().into())
        }
    }

    pub fn filename_path(args: &[Value]) -> gml::Result<Value> {
        let full_path = expect_args!(args, [string])?;
        if let Some(bs) = full_path.as_ref().rfind('\\') {
            Ok(full_path.as_ref()[..bs + 1].to_string().into())
        } else {
            Ok("".to_string().into())
        }
    }

    pub fn filename_dir(args: &[Value]) -> gml::Result<Value> {
        let full_path = expect_args!(args, [string])?;
        if let Some(bs) = full_path.as_ref().rfind('\\') {
            Ok(full_path.as_ref()[..bs].to_string().into())
        } else {
            Ok("".to_string().into())
        }
    }

    pub fn filename_drive(args: &[Value]) -> gml::Result<Value> {
        let full_path = expect_args!(args, [string])?;
        let drive = full_path.as_ref().chars().take(2).collect::<String>();
        if !drive.starts_with(':') && drive.ends_with(':') { Ok(drive.into()) } else { Ok("".to_string().into()) }
    }

    pub fn filename_ext(args: &[Value]) -> gml::Result<Value> {
        let full_path = expect_args!(args, [string])?;
        if let Some(dot) = full_path.as_ref().rfind('.') {
            Ok(full_path.as_ref()[dot..].to_string().into())
        } else {
            Ok("".to_string().into())
        }
    }

    pub fn filename_change_ext(args: &[Value]) -> gml::Result<Value> {
        let (full_path, new_ext) = expect_args!(args, [string, string])?;
        let mut new_path = full_path.as_ref().rsplitn(2, '.').last().unwrap_or(full_path.as_ref()).to_string();
        new_path.push_str(new_ext.as_ref());
        Ok(new_path.into())
    }

    pub fn export_include_file(&mut self, args: &[Value]) -> gml::Result<Value> {
        let name = expect_args!(args, [bytes])?;
        let temp_directory = self.decode_str(self.temp_directory.as_ref()).into_owned().into();
        let program_directory = self.decode_str(self.program_directory.as_ref()).into_owned().into();
        if let Some(file) = self.included_files.iter_mut().filter(|i| name.eq_ignore_ascii_case(i.name.as_ref())).next()
        {
            match file.export(temp_directory, program_directory) {
                Ok(()) => Ok(Default::default()),
                Err(e) => Err(gml::Error::FunctionError("export_include_file".into(), e.to_string())),
            }
        } else {
            Err(gml::Error::FunctionError("export_include_file".into(), "Trying to export non-existing file.".into()))
        }
    }

    pub fn export_include_file_location(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (name, path) = expect_args!(args, [bytes, string])?;
        if let Some(file) = self.included_files.iter_mut().filter(|i| name.eq_ignore_ascii_case(i.name.as_ref())).next()
        {
            let path_ref: &str = path.as_ref();
            match file.export_to(path_ref.as_ref()) {
                Ok(()) => Ok(Default::default()),
                Err(e) => Err(gml::Error::FunctionError("export_include_file_location".into(), e.to_string())),
            }
        } else {
            Err(gml::Error::FunctionError(
                "export_include_file_location".into(),
                "Trying to export non-existing file.".into(),
            ))
        }
    }

    pub fn discard_include_file(&mut self, args: &[Value]) -> gml::Result<Value> {
        let name = expect_args!(args, [bytes])?;
        if let Some(file) = self.included_files.iter_mut().filter(|i| name.eq_ignore_ascii_case(i.name.as_ref())).next()
        {
            file.data = None;
            Ok(Default::default())
        } else {
            Err(gml::Error::FunctionError("discard_include_file".into(), "Trying to discard non-existing file.".into()))
        }
    }

    pub fn execute_program(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (prog, prog_args, wait) = expect_args!(args, [string, string, bool])?;
        // Rust doesn't let you execute a program with just a string, so unescape it manually
        let mut command_array = Vec::new();
        let mut buf = Some(String::new());
        let mut quote_count = 0;
        for c in format!("{} {}", prog, prog_args).replace("\\\"", "\"\"\"").chars() {
            match c {
                '"' => {
                    buf = buf.or(Some("".into()));
                    quote_count += 1;
                    if quote_count > 2 {
                        quote_count = 0;
                        buf.as_mut().unwrap().push('"');
                    }
                },
                c if c.is_whitespace() && quote_count != 1 => {
                    quote_count %= 2;
                    if let Some(s) = buf {
                        command_array.push(s);
                        buf = None;
                    }
                },
                c => {
                    quote_count %= 2;
                    buf = buf.or(Some("".into()));
                    buf.as_mut().unwrap().push(c);
                },
            }
        }
        if let Some(s) = buf {
            command_array.push(s);
        }
        if command_array.is_empty() {
            return Err(gml::Error::FunctionError("execute_program".into(), "Cannot execute an empty string".into()))
        }
        // Actually run the program
        match Command::new(&command_array[0]).args(&command_array[1..]).spawn() {
            Ok(mut child) => {
                if wait {
                    // wait() closes stdin. This is inaccurate, but Rust doesn't offer an alternative.
                    if let Err(e) = child.wait() {
                        return Err(gml::Error::FunctionError(
                            "execute_program".into(),
                            format!("Cannot wait for {}: {}", prog, e),
                        ))
                    }
                    self.process_window_events();
                }
                Ok(Default::default())
            },
            Err(e) => {
                Err(gml::Error::FunctionError("execute_program".into(), format!("Cannot execute {}: {}", prog, e)))
            },
        }
    }

    pub fn execute_shell(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 2
        unimplemented!("Called unimplemented kernel function execute_shell")
    }

    pub fn parameter_count(&self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        // Gamemaker doesn't count parameter 0 (the game exe) as a "parameter"
        return Ok((self.parameters.len() - 1).into())
    }

    pub fn parameter_string(&self, args: &[Value]) -> gml::Result<Value> {
        let param_index = expect_args!(args, [int])?;
        if param_index >= 0 {
            Ok(match self.parameters.get(param_index as usize) {
                Some(a) => a.clone(),
                None => "".to_string(),
            }
            .into())
        } else {
            Ok("".into())
        }
    }

    pub fn environment_get_variable(&self, args: &[Value]) -> gml::Result<Value> {
        let name = expect_args!(args, [bytes])?;
        // get environment variable
        let env_os = std::env::var_os(self.decode_str(name.as_ref()).as_ref()).unwrap_or("".into());
        // convert to bytes, "" if impossible
        let env = env_os.to_str().and_then(|s| self.encode_str_maybe(s)).unwrap_or(b"".as_ref().into());
        Ok(env.as_ref().into())
    }

    pub fn registry_write_string(&self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 2
        unimplemented!("Called unimplemented kernel function registry_write_string")
    }

    pub fn registry_write_real(&self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 2
        unimplemented!("Called unimplemented kernel function registry_write_real")
    }

    pub fn registry_read_string(&self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        unimplemented!("Called unimplemented kernel function registry_read_string")
    }

    pub fn registry_read_real(&self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        unimplemented!("Called unimplemented kernel function registry_read_real")
    }

    pub fn registry_exists(&self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        unimplemented!("Called unimplemented kernel function registry_exists")
    }

    pub fn registry_write_string_ext(&self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 3
        unimplemented!("Called unimplemented kernel function registry_write_string_ext")
    }

    pub fn registry_write_real_ext(&self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 3
        unimplemented!("Called unimplemented kernel function registry_write_real_ext")
    }

    pub fn registry_read_string_ext(&self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 2
        unimplemented!("Called unimplemented kernel function registry_read_string_ext")
    }

    pub fn registry_read_real_ext(&self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 2
        unimplemented!("Called unimplemented kernel function registry_read_real_ext")
    }

    pub fn registry_exists_ext(&self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 2
        unimplemented!("Called unimplemented kernel function registry_exists_ext")
    }

    pub fn registry_set_root(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        unimplemented!("Called unimplemented kernel function registry_set_root")
    }

    pub fn ini_open(&mut self, args: &[Value]) -> gml::Result<Value> {
        let name = expect_args!(args, [bytes])?;
        let name_str = self.decode_str(name.as_ref());
        if file::file_exists(&name_str) {
            match ini::Ini::load_from_file(name_str.as_ref()) {
                Ok(ini) => {
                    self.open_ini = Some((ini, name));
                    Ok(Default::default())
                },
                Err(e) => Err(gml::Error::FunctionError("ini_open".into(), format!("{}", e))),
            }
        } else {
            self.open_ini = Some((ini::Ini::new(), name));
            Ok(Default::default())
        }
    }

    pub fn ini_close(&mut self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        match self.open_ini.as_ref() {
            Some((ini, path)) => match ini.write_to_file(self.decode_str(path.as_ref()).as_ref()) {
                Ok(()) => {
                    self.open_ini = None;
                    Ok(Default::default())
                },
                Err(e) => Err(gml::Error::FunctionError("ini_close".into(), format!("{}", e))),
            },
            None => Ok(Default::default()),
        }
    }

    pub fn ini_read_string(&self, args: &[Value]) -> gml::Result<Value> {
        let (section, key, default) = expect_args!(args, [string, string, string])?;
        match self.open_ini.as_ref() {
            Some((ini, _)) => Ok(ini
                .section(Some(section.as_ref()))
                .and_then(|s| s.get(key))
                .unwrap_or(default.as_ref())
                .to_string()
                .into()),
            None => Err(gml::Error::FunctionError(
                "ini_read_string".into(),
                "Trying to read from undefined INI file".to_string(),
            )),
        }
    }

    pub fn ini_read_real(&self, args: &[Value]) -> gml::Result<Value> {
        let (section, key, default) = expect_args!(args, [string, string, real])?;
        match self.open_ini.as_ref() {
            Some((ini, _)) => match ini.section(Some(section.as_ref())).and_then(|s| s.get(key)) {
                Some(val) => match val.parse::<f64>() {
                    Ok(x) => Ok(x.into()),
                    Err(_) => Ok(Default::default()),
                },
                None => Ok(default.into()),
            },
            None => Err(gml::Error::FunctionError(
                "ini_read_real".into(),
                "Trying to read from undefined INI file".to_string(),
            )),
        }
    }

    pub fn ini_write_string(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (section, key, val) = expect_args!(args, [string, string, string])?;
        match self.open_ini.as_mut() {
            Some((ini, _)) => {
                ini.with_section(Some(section.as_ref())).set(key.as_ref(), val.as_ref());
                Ok(Default::default())
            },
            None => Err(gml::Error::FunctionError(
                "ini_write_string".into(),
                "Trying to write to undefined INI file".to_string(),
            )),
        }
    }

    pub fn ini_write_real(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (section, key, val) = expect_args!(args, [string, string, real])?;
        match self.open_ini.as_mut() {
            Some((ini, _)) => {
                ini.with_section(Some(section.as_ref())).set(key.as_ref(), val.to_string());
                Ok(Default::default())
            },
            None => Err(gml::Error::FunctionError(
                "ini_write_real".into(),
                "Trying to write to undefined INI file".to_string(),
            )),
        }
    }

    pub fn ini_key_exists(&self, args: &[Value]) -> gml::Result<Value> {
        let (section, key) = expect_args!(args, [string, string])?;
        match self.open_ini.as_ref() {
            Some((ini, _)) => {
                Ok(ini.section(Some(section.as_ref())).map(|s| s.contains_key(key)).unwrap_or(false).into())
            },
            None => Err(gml::Error::FunctionError(
                "ini_key_exists".into(),
                "Trying to read from undefined INI file".to_string(),
            )),
        }
    }

    pub fn ini_section_exists(&self, args: &[Value]) -> gml::Result<Value> {
        let section = expect_args!(args, [string])?;
        match self.open_ini.as_ref() {
            Some((ini, _)) => Ok(ini.section(Some(section.as_ref())).is_some().into()),
            None => Err(gml::Error::FunctionError(
                "ini_section_exists".into(),
                "Trying to read from undefined INI file".to_string(),
            )),
        }
    }

    pub fn ini_key_delete(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (section, key) = expect_args!(args, [string, string])?;
        match self.open_ini.as_mut() {
            Some((ini, _)) => {
                ini.delete_from(Some(section.as_ref()), key.as_ref());
                Ok(Default::default())
            },
            None => Err(gml::Error::FunctionError(
                "ini_key_delete".into(),
                "Trying to change undefined INI file".to_string(),
            )),
        }
    }

    pub fn ini_section_delete(&mut self, args: &[Value]) -> gml::Result<Value> {
        let section = expect_args!(args, [string])?;
        match self.open_ini.as_mut() {
            Some((ini, _)) => {
                ini.delete(Some(section.as_ref()));
                Ok(Default::default())
            },
            None => Err(gml::Error::FunctionError(
                "ini_section_delete".into(),
                "Trying to change undefined INI file".to_string(),
            )),
        }
    }

    pub fn disk_free(&self, _args: &[Value]) -> gml::Result<Value> {
        // let path = match args.get(0).clone() {
        //     Some(Value::Str(p)) => p.as_ref().get(0).map(|&x| x as char),
        //     _ => None,
        // };
        // Ok(self.window.disk_free(path).map(|x| x as f64).unwrap_or(-1f64).into())
        todo!()
    }

    pub fn disk_size(&self, _args: &[Value]) -> gml::Result<Value> {
        // let path = match args.get(0).clone() {
        //     Some(Value::Str(p)) => p.as_ref().get(0).map(|&x| x as char),
        //     _ => None,
        // };
        // Ok(self.window.disk_size(path).map(|x| x as f64).unwrap_or(-1f64).into())
        todo!()
    }

    pub fn splash_set_caption(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        unimplemented!("Called unimplemented kernel function splash_set_caption")
    }

    pub fn splash_set_fullscreen(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        unimplemented!("Called unimplemented kernel function splash_set_fullscreen")
    }

    pub fn splash_set_border(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        unimplemented!("Called unimplemented kernel function splash_set_border")
    }

    pub fn splash_set_size(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 2
        unimplemented!("Called unimplemented kernel function splash_set_size")
    }

    pub fn splash_set_position(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 2
        unimplemented!("Called unimplemented kernel function splash_set_position")
    }

    pub fn splash_set_adapt(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        unimplemented!("Called unimplemented kernel function splash_set_adapt")
    }

    pub fn splash_set_top(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        unimplemented!("Called unimplemented kernel function splash_set_top")
    }

    pub fn splash_set_color(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        unimplemented!("Called unimplemented kernel function splash_set_color")
    }

    pub fn splash_set_main(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        unimplemented!("Called unimplemented kernel function splash_set_main")
    }

    pub fn splash_set_scale(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        unimplemented!("Called unimplemented kernel function splash_set_scale")
    }

    pub fn splash_set_cursor(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        unimplemented!("Called unimplemented kernel function splash_set_cursor")
    }

    pub fn splash_set_interrupt(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        unimplemented!("Called unimplemented kernel function splash_set_interrupt")
    }

    pub fn splash_set_stop_key(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        unimplemented!("Called unimplemented kernel function splash_set_stop_key")
    }

    pub fn splash_set_close_button(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        unimplemented!("Called unimplemented kernel function splash_set_close_button")
    }

    pub fn splash_set_stop_mouse(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        unimplemented!("Called unimplemented kernel function splash_set_stop_mouse")
    }

    pub fn splash_show_video(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 2
        unimplemented!("Called unimplemented kernel function splash_show_video")
    }

    pub fn splash_show_image(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 2
        unimplemented!("Called unimplemented kernel function splash_show_image")
    }

    pub fn splash_show_text(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 2
        unimplemented!("Called unimplemented kernel function splash_show_text")
    }

    pub fn splash_show_web(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 2
        unimplemented!("Called unimplemented kernel function splash_show_web")
    }

    pub fn show_image(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 3
        unimplemented!("Called unimplemented kernel function show_image")
    }

    pub fn show_video(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 3
        unimplemented!("Called unimplemented kernel function show_video")
    }

    pub fn show_text(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 4
        unimplemented!("Called unimplemented kernel function show_text")
    }

    pub fn show_message(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        unimplemented!("Called unimplemented kernel function show_message")
    }

    pub fn show_question(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        unimplemented!("Called unimplemented kernel function show_question")
    }

    pub fn show_error(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (text, _abort) = expect_args!(args, [string, bool])?;
        Err(gml::Error::FunctionError("show_error".into(), text.into()))
    }

    pub fn show_info(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 0
        unimplemented!("Called unimplemented kernel function show_info")
    }

    pub fn load_info(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        unimplemented!("Called unimplemented kernel function load_info")
    }

    pub fn highscore_show(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        unimplemented!("Called unimplemented kernel function highscore_show")
    }

    pub fn highscore_set_background(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        unimplemented!("Called unimplemented kernel function highscore_set_background")
    }

    pub fn highscore_set_border(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        unimplemented!("Called unimplemented kernel function highscore_set_border")
    }

    pub fn highscore_set_font(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 3
        unimplemented!("Called unimplemented kernel function highscore_set_font")
    }

    pub fn highscore_set_strings(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 3
        unimplemented!("Called unimplemented kernel function highscore_set_strings")
    }

    pub fn highscore_set_colors(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 3
        unimplemented!("Called unimplemented kernel function highscore_set_colors")
    }

    pub fn highscore_show_ext(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 7
        unimplemented!("Called unimplemented kernel function highscore_show_ext")
    }

    pub fn highscore_clear(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 0
        unimplemented!("Called unimplemented kernel function highscore_clear")
    }

    pub fn highscore_add(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 2
        unimplemented!("Called unimplemented kernel function highscore_add")
    }

    pub fn highscore_add_current(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 0
        unimplemented!("Called unimplemented kernel function highscore_add_current")
    }

    pub fn highscore_value(&self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        unimplemented!("Called unimplemented kernel function highscore_value")
    }

    pub fn highscore_name(&self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        unimplemented!("Called unimplemented kernel function highscore_name")
    }

    pub fn draw_highscore(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 4
        unimplemented!("Called unimplemented kernel function draw_highscore")
    }

    pub fn show_message_ext(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 4
        unimplemented!("Called unimplemented kernel function show_message_ext")
    }

    pub fn message_background(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        //unimplemented!("Called unimplemented kernel function message_background")
        // TODO
        Ok(Default::default())
    }

    pub fn message_button(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        //unimplemented!("Called unimplemented kernel function message_button")
        // TODO
        Ok(Default::default())
    }

    pub fn message_alpha(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        //unimplemented!("Called unimplemented kernel function message_alpha")
        // TODO
        Ok(Default::default())
    }

    pub fn message_text_font(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 4
        //unimplemented!("Called unimplemented kernel function message_text_font")
        // TODO
        Ok(Default::default())
    }

    pub fn message_button_font(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 4
        //unimplemented!("Called unimplemented kernel function message_button_font")
        // TODO
        Ok(Default::default())
    }

    pub fn message_input_font(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 4
        //unimplemented!("Called unimplemented kernel function message_input_font")
        // TODO
        Ok(Default::default())
    }

    pub fn message_text_charset(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 2
        //unimplemented!("Called unimplemented kernel function message_text_charset")
        // TODO
        Ok(Default::default())
    }

    pub fn message_mouse_color(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        //unimplemented!("Called unimplemented kernel function message_mouse_color")
        // TODO
        Ok(Default::default())
    }

    pub fn message_input_color(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        //unimplemented!("Called unimplemented kernel function message_input_color")
        // TODO
        Ok(Default::default())
    }

    pub fn message_position(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 2
        //unimplemented!("Called unimplemented kernel function message_position")
        // TODO
        Ok(Default::default())
    }

    pub fn message_size(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 2
        //unimplemented!("Called unimplemented kernel function message_size")
        // TODO
        Ok(Default::default())
    }

    pub fn message_caption(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 2
        //unimplemented!("Called unimplemented kernel function message_caption")
        // TODO
        Ok(Default::default())
    }

    pub fn show_menu(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 2
        unimplemented!("Called unimplemented kernel function show_menu")
    }

    pub fn show_menu_pos(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 4
        unimplemented!("Called unimplemented kernel function show_menu_pos")
    }

    pub fn get_integer(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 2
        unimplemented!("Called unimplemented kernel function get_integer")
    }

    pub fn get_string(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 2
        unimplemented!("Called unimplemented kernel function get_string")
    }

    pub fn get_color(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        unimplemented!("Called unimplemented kernel function get_color")
    }

    pub fn get_open_filename(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 2
        unimplemented!("Called unimplemented kernel function get_open_filename")
    }

    pub fn get_save_filename(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 2
        unimplemented!("Called unimplemented kernel function get_save_filename")
    }

    pub fn get_directory(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        unimplemented!("Called unimplemented kernel function get_directory")
    }

    pub fn get_directory_alt(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 2
        unimplemented!("Called unimplemented kernel function get_directory_alt")
    }

    // NB: This function is constant because numlock state is tracked.
    pub fn keyboard_get_numlock(&self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        Ok(self.input.keyboard_get_numlock().into())
    }

    pub fn keyboard_set_numlock(&mut self, args: &[Value]) -> gml::Result<Value> {
        let state = expect_args!(args, [bool])?;
        self.input.keyboard_set_numlock(state);
        Ok(Default::default())
    }

    pub fn keyboard_key_press(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // let key = expect_args!(args, [int])?;
        // if let Ok(vk) = u8::try_from(key) {
        //     self.input.button_press(vk, true);
        // }
        // Ok(Default::default())
        todo!() // should go on next event poll
    }

    pub fn keyboard_key_release(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // let key = expect_args!(args, [int])?;
        // if let Ok(vk) = u8::try_from(key) {
        //     self.input.button_release(vk, true);
        // }
        // Ok(Default::default())
        todo!() // should go on next event poll
    }

    pub fn keyboard_set_map(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (real, mapped) = expect_args!(args, [int, int])?;
        if let (Ok(from), Ok(to)) = (u8::try_from(real), u8::try_from(mapped)) {
            self.input.keyboard_set_map(from, to);
        }
        Ok(Default::default())
    }

    pub fn keyboard_get_map(&mut self, args: &[Value]) -> gml::Result<Value> {
        let key = expect_args!(args, [int])?;
        if let Ok(vk) = u8::try_from(key) {
            Ok(i32::from(self.input.keyboard_get_map(vk)).into())
        } else {
            Ok(key.into())
        }
    }

    pub fn keyboard_unset_map(&mut self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        self.input.keyboard_unset_map();
        Ok(Default::default())
    }

    pub fn keyboard_check(&self, args: &[Value]) -> gml::Result<Value> {
        let key = expect_args!(args, [int])?;
        match u8::try_from(key) {
            Ok(vk) => Ok(self.input.keyboard_check(vk).into()),
            _ => Ok(gml::FALSE.into()),
        }
    }

    pub fn keyboard_check_pressed(&self, args: &[Value]) -> gml::Result<Value> {
        let key = expect_args!(args, [int])?;
        match u8::try_from(key) {
            Ok(vk) => Ok(self.input.keyboard_check_pressed(vk).into()),
            _ => Ok(gml::FALSE.into()),
        }
    }

    pub fn keyboard_check_released(&self, args: &[Value]) -> gml::Result<Value> {
        let key = expect_args!(args, [int])?;
        match u8::try_from(key) {
            Ok(vk) => Ok(self.input.keyboard_check_released(vk).into()),
            _ => Ok(gml::FALSE.into()),
        }
    }

    pub fn keyboard_check_direct(&self, args: &[Value]) -> gml::Result<Value> {
        let key = expect_args!(args, [int])?;
        match u8::try_from(key) {
            Ok(vk) => Ok(self.input.keyboard_check_direct(vk).into()),
            _ => Ok(gml::FALSE.into()),
        }
    }

    pub fn mouse_check_button(&self, args: &[Value]) -> gml::Result<Value> {
        let button = expect_args!(args, [int])?;
        match i8::try_from(button) {
            Ok(mb) => Ok(self.input.mouse_check_button(mb).into()),
            _ => Ok(gml::FALSE.into()),
        }
    }

    pub fn mouse_check_button_pressed(&self, args: &[Value]) -> gml::Result<Value> {
        let button = expect_args!(args, [int])?;
        match i8::try_from(button) {
            Ok(mb) => Ok(self.input.mouse_check_button_pressed(mb).into()),
            _ => Ok(gml::FALSE.into()),
        }
    }

    pub fn mouse_check_button_released(&self, args: &[Value]) -> gml::Result<Value> {
        let button = expect_args!(args, [int])?;
        match i8::try_from(button) {
            Ok(mb) => Ok(self.input.mouse_check_button_released(mb).into()),
            _ => Ok(gml::FALSE.into()),
        }
    }

    pub fn mouse_wheel_up(&self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        Ok(self.input.mouse_wheel_up().into())
    }

    pub fn mouse_wheel_down(&self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        Ok(self.input.mouse_wheel_down().into())
    }

    pub fn joystick_exists(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        //unimplemented!("Called unimplemented kernel function joystick_exists")
        // TODO
        Ok(gml::FALSE.into())
    }

    pub fn joystick_direction(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        //unimplemented!("Called unimplemented kernel function joystick_direction")
        // TODO
        Ok(101.into())
    }

    pub fn joystick_name(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        //unimplemented!("Called unimplemented kernel function joystick_name")
        // TODO
        Ok("".into())
    }

    pub fn joystick_axes(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        //unimplemented!("Called unimplemented kernel function joystick_axes")
        // TODO
        Ok(0.into())
    }

    pub fn joystick_buttons(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        //unimplemented!("Called unimplemented kernel function joystick_buttons")
        // TODO
        Ok(0.into())
    }

    pub fn joystick_has_pov(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        //unimplemented!("Called unimplemented kernel function joystick_has_pov")
        // TODO
        Ok(gml::FALSE.into())
    }

    pub fn joystick_check_button(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 2
        //unimplemented!("Called unimplemented kernel function joystick_check_button")
        // TODO
        Ok(gml::FALSE.into())
    }

    pub fn joystick_xpos(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        //unimplemented!("Called unimplemented kernel function joystick_xpos")
        // TODO
        Ok(0.into())
    }

    pub fn joystick_ypos(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        //unimplemented!("Called unimplemented kernel function joystick_ypos")
        // TODO
        Ok(0.into())
    }

    pub fn joystick_zpos(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        //unimplemented!("Called unimplemented kernel function joystick_zpos")
        // TODO
        Ok(0.into())
    }

    pub fn joystick_rpos(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        //unimplemented!("Called unimplemented kernel function joystick_rpos")
        // TODO
        Ok(0.into())
    }

    pub fn joystick_upos(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        //unimplemented!("Called unimplemented kernel function joystick_upos")
        // TODO
        Ok(0.into())
    }

    pub fn joystick_vpos(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        //unimplemented!("Called unimplemented kernel function joystick_vpos")
        // TODO
        Ok(0.into())
    }

    pub fn joystick_pov(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        //unimplemented!("Called unimplemented kernel function joystick_pov")
        // TODO
        Ok((-1).into())
    }

    pub fn keyboard_clear(&mut self, args: &[Value]) -> gml::Result<Value> {
        let key = expect_args!(args, [int])?;
        self.process_window_events();
        if let Ok(vk) = u8::try_from(key) {
            self.input.keyboard_clear(vk);
        }
        Ok(Default::default())
    }

    pub fn mouse_clear(&mut self, args: &[Value]) -> gml::Result<Value> {
        let button = expect_args!(args, [int])?;
        self.process_window_events();
        if let Ok(mb) = i8::try_from(button) {
            self.input.mouse_clear(mb);
        }
        Ok(Default::default())
    }

    pub fn io_clear(&mut self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        self.process_window_events();
        // TODO: clear keyboard_string
        self.input.keyboard_clear_all();
        self.input.mouse_clear_all();
        Ok(Default::default())
    }

    pub fn io_handle(&mut self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        self.process_window_events();
        Ok(Default::default())
    }

    pub fn keyboard_wait(&mut self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        if self.play_type == PlayType::Normal {
            self.input.set_keyboard_lastkey(0);
            while self.input.keyboard_lastkey() == 0 {
                datetime::sleep(std::time::Duration::from_millis(50));
                self.process_window_events();
            }
        }
        Ok(Default::default())
    }

    pub fn mouse_wait(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 0
        unimplemented!("Called unimplemented kernel function mouse_wait")
    }

    pub fn mplay_init_ipx(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 0
        unimplemented!("Called unimplemented kernel function mplay_init_ipx")
    }

    pub fn mplay_init_tcpip(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        unimplemented!("Called unimplemented kernel function mplay_init_tcpip")
    }

    pub fn mplay_init_modem(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 2
        unimplemented!("Called unimplemented kernel function mplay_init_modem")
    }

    pub fn mplay_init_serial(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 5
        unimplemented!("Called unimplemented kernel function mplay_init_serial")
    }

    pub fn mplay_connect_status(&self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 0
        unimplemented!("Called unimplemented kernel function mplay_connect_status")
    }

    pub fn mplay_end(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 0
        unimplemented!("Called unimplemented kernel function mplay_end")
    }

    pub fn mplay_session_mode(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        unimplemented!("Called unimplemented kernel function mplay_session_mode")
    }

    pub fn mplay_session_create(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 3
        unimplemented!("Called unimplemented kernel function mplay_session_create")
    }

    pub fn mplay_session_find(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 0
        unimplemented!("Called unimplemented kernel function mplay_session_find")
    }

    pub fn mplay_session_name(&self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        unimplemented!("Called unimplemented kernel function mplay_session_name")
    }

    pub fn mplay_session_join(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 2
        unimplemented!("Called unimplemented kernel function mplay_session_join")
    }

    pub fn mplay_session_status(&self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 0
        unimplemented!("Called unimplemented kernel function mplay_session_status")
    }

    pub fn mplay_session_end(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 0
        unimplemented!("Called unimplemented kernel function mplay_session_end")
    }

    pub fn mplay_player_find(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 0
        unimplemented!("Called unimplemented kernel function mplay_player_find")
    }

    pub fn mplay_player_name(&self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        unimplemented!("Called unimplemented kernel function mplay_player_name")
    }

    pub fn mplay_player_id(&self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        unimplemented!("Called unimplemented kernel function mplay_player_id")
    }

    pub fn mplay_data_write(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 2
        unimplemented!("Called unimplemented kernel function mplay_data_write")
    }

    pub fn mplay_data_read(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        unimplemented!("Called unimplemented kernel function mplay_data_read")
    }

    pub fn mplay_data_mode(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        unimplemented!("Called unimplemented kernel function mplay_data_mode")
    }

    pub fn mplay_message_send(&self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 3
        unimplemented!("Called unimplemented kernel function mplay_message_send")
    }

    pub fn mplay_message_send_guaranteed(&self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 3
        unimplemented!("Called unimplemented kernel function mplay_message_send_guaranteed")
    }

    pub fn mplay_message_receive(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        unimplemented!("Called unimplemented kernel function mplay_message_receive")
    }

    pub fn mplay_message_id(&self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 0
        unimplemented!("Called unimplemented kernel function mplay_message_id")
    }

    pub fn mplay_message_value(&self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 0
        unimplemented!("Called unimplemented kernel function mplay_message_value")
    }

    pub fn mplay_message_player(&self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 0
        unimplemented!("Called unimplemented kernel function mplay_message_player")
    }

    pub fn mplay_message_name(&self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 0
        unimplemented!("Called unimplemented kernel function mplay_message_name")
    }

    pub fn mplay_message_count(&self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        unimplemented!("Called unimplemented kernel function mplay_message_count")
    }

    pub fn mplay_message_clear(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        unimplemented!("Called unimplemented kernel function mplay_message_clear")
    }

    pub fn mplay_ipaddress(&mut self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        Ok(network::get_local_ip().unwrap_or(std::net::Ipv4Addr::LOCALHOST.into()).to_string().into())
    }

    pub fn event_inherited(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        let parent = self
            .assets
            .objects
            .get_asset(context.event_object)
            .ok_or(gml::Error::NonexistentAsset(asset::Type::Object, context.event_object))?
            .parent_index;
        if parent >= 0 {
            self.run_instance_event(
                context.event_type,
                context.event_number as _,
                context.this,
                context.other,
                Some(parent),
            )?;
        }
        Ok(Default::default())
    }

    pub fn event_perform(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (event_type, event_number) = expect_args!(args, [int, int])?;
        self.run_instance_event(event_type as _, event_number as _, context.this, context.other, None)?;
        Ok(Default::default())
    }

    pub fn event_user(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let number = expect_args!(args, [int])?;
        if number >= 0 && number <= 15 {
            self.run_instance_event(7, (10 + number) as _, context.this, context.other, None)?;
        }
        Ok(Default::default())
    }

    pub fn event_perform_object(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (object, event_type, event_number) = expect_args!(args, [int, int, int])?;
        self.run_instance_event(event_type as _, event_number as _, context.this, context.other, Some(object))?;
        Ok(Default::default())
    }

    pub fn external_define(&mut self, args: &[Value]) -> gml::Result<Value> {
        if let (Some(dll_name), Some(fn_name), Some(call_conv), Some(res_type), Some(argnumb)) =
            (args.get(0), args.get(1), args.get(2), args.get(3), args.get(4))
        {
            let encoding = match self.gm_version {
                Version::GameMaker8_0 => self.encoding,
                Version::GameMaker8_1 => encoding_rs::UTF_8,
            };
            let gm_dll = gml::String::from(dll_name.clone());
            let dll = gm_dll.decode(encoding);
            let gm_function = gml::String::from(fn_name.clone());
            let function = gm_function.decode(encoding);

            let dummy = external::should_dummy(&*dll, &*function, self.play_type);

            let call_conv = match call_conv.round() {
                0 => external::dll::CallConv::Cdecl,
                _ => external::dll::CallConv::Stdcall,
            };
            let res_type = match res_type.round() {
                0 => external::dll::ValueType::Real,
                _ => external::dll::ValueType::Str,
            };
            let argnumb = argnumb.round();
            if args.len() as i32 != 5 + argnumb {
                return Err(gml::Error::WrongArgumentCount(5 + argnumb.max(5) as usize, args.len()))
            }

            if let Some(dummy) = dummy {
                // safety: arg count was checked above
                let argc = argnumb as usize;
                self.externals.define_dummy(&*dll, &*function, dummy, argc)
            } else {
                let arg_types = args[5..]
                    .iter()
                    .map(|v| match v.round() {
                        0 => external::dll::ValueType::Real,
                        _ => external::dll::ValueType::Str,
                    })
                    .collect::<Vec<_>>();
                self.externals.define(&*dll, &*function, call_conv, &arg_types, res_type)
            }
            .map(Value::from)
            .map_err(|e| gml::Error::FunctionError("external_define".into(), e))
        } else {
            Err(gml::Error::WrongArgumentCount(5, args.len()))
        }
    }

    pub fn external_call(&mut self, args: &[Value]) -> gml::Result<Value> {
        if let Some(id) = args.get(0) {
            let id = id.round();
            let dll_args: Vec<external::dll::Value> =
                (&args[1..]).iter().cloned().map(external::dll::Value::from).collect();
            self.externals
                .call(id, &dll_args)
                .map(Value::from)
                .map_err(|e| gml::Error::FunctionError("external_call".into(), e))
        } else {
            Ok(Default::default())
        }
    }

    pub fn external_free(&mut self, args: &[Value]) -> gml::Result<Value> {
        let dll_name = expect_args!(args, [bytes])?;
        let encoding = match self.gm_version {
            Version::GameMaker8_0 => self.encoding,
            Version::GameMaker8_1 => encoding_rs::UTF_8,
        };
        let dll = gml::String::from(dll_name);
        self.externals
            .free(&*dll.decode(encoding))
            .map_err(|e| gml::Error::FunctionError("external_free".into(), e))?;
        Ok(Default::default())
    }

    pub fn get_function_address(args: &[Value]) -> gml::Result<Value> {
        let _name = expect_args!(args, [any])?;
        // We're definitely not ABI compliant with the original GM (at least for now),
        // and don't have any compatibility layer as well, so let it just return -1,
        // as the GM implementation does for unknown function names.
        Ok((-1).into())
    }

    pub fn external_define0(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 3
        unimplemented!("Called unimplemented kernel function external_define0")
    }

    pub fn external_call0(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        unimplemented!("Called unimplemented kernel function external_call0")
    }

    pub fn external_define1(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 4
        unimplemented!("Called unimplemented kernel function external_define1")
    }

    pub fn external_call1(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 2
        unimplemented!("Called unimplemented kernel function external_call1")
    }

    pub fn external_define2(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 5
        unimplemented!("Called unimplemented kernel function external_define2")
    }

    pub fn external_call2(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 3
        unimplemented!("Called unimplemented kernel function external_call2")
    }

    pub fn external_define3(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 6
        unimplemented!("Called unimplemented kernel function external_define3")
    }

    pub fn external_call3(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 4
        unimplemented!("Called unimplemented kernel function external_call3")
    }

    pub fn external_define4(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 7
        unimplemented!("Called unimplemented kernel function external_define4")
    }

    pub fn external_call4(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 5
        unimplemented!("Called unimplemented kernel function external_call4")
    }

    pub fn external_define5(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 3
        unimplemented!("Called unimplemented kernel function external_define5")
    }

    pub fn external_call5(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 6
        unimplemented!("Called unimplemented kernel function external_call5")
    }

    pub fn external_define6(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 3
        unimplemented!("Called unimplemented kernel function external_define6")
    }

    pub fn external_call6(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 7
        unimplemented!("Called unimplemented kernel function external_call6")
    }

    pub fn external_define7(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 3
        unimplemented!("Called unimplemented kernel function external_define7")
    }

    pub fn external_call7(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 8
        unimplemented!("Called unimplemented kernel function external_call7")
    }

    pub fn external_define8(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 3
        unimplemented!("Called unimplemented kernel function external_define8")
    }

    pub fn external_call8(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 9
        unimplemented!("Called unimplemented kernel function external_call8")
    }

    pub fn execute_string(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        if let Some(Value::Str(code)) = args.get(0) {
            match self.compiler.compile(code.as_ref()) {
                Ok(instrs) => {
                    let mut new_args: [Value; 16] = Default::default();
                    for (src, dest) in args[1..].iter().zip(new_args.iter_mut()) {
                        *dest = src.clone();
                    }
                    // Note: GM8 does not update the argument_count here to (args.len() - 1) as it should
                    let mut new_context = Context::copy_with_args(context, new_args, context.argument_count);
                    self.execute(&instrs, &mut new_context)?;
                    Ok(new_context.return_value)
                },
                Err(e) => Err(gml::Error::FunctionError("execute_string".into(), e.message)),
            }
        } else {
            // eg execute_string(42) - does nothing, returns 0
            Ok(Default::default())
        }
    }

    pub fn execute_file(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        if let Some(Value::Str(path)) = args.get(0) {
            let mut new_args: [Value; 16] = Default::default();
            for (src, dest) in args.iter().zip(new_args.iter_mut()) {
                *dest = src.clone();
            }
            match std::fs::read(self.decode_str(path.as_ref()).as_ref()) {
                Ok(code) => {
                    new_args[0] = code.into();
                    self.execute_string(context, &new_args)
                },
                Err(e) => Err(gml::Error::FunctionError("execute_file".into(), format!("{}", e))),
            }
        } else {
            Err(gml::Error::FunctionError("execute_file".into(), "Trying to execute a number.".to_string()))
        }
    }

    pub fn window_handle(&self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        #[cfg(target_os = "windows")]
        {
            use ramen::platform::win32::WindowExt as _;
            Ok((self.window.hwnd() as usize).into())
        }
        // TODO: Others! (They'll compile error here so it'll remind me)
    }

    pub fn show_debug_message(&self, args: &[Value]) -> gml::Result<Value> {
        let message = expect_args!(args, [any])?;
        println!("{}", self.decode_str(message.repr().as_ref()));
        Ok(Default::default())
    }

    pub fn set_program_priority(&mut self, args: &[Value]) -> gml::Result<Value> {
        let _priority = expect_args!(args, [int])?;
        // do nothing
        Ok(Default::default())
    }

    pub fn set_application_title(args: &[Value]) -> gml::Result<Value> {
        let _title = expect_args!(args, [any])?;
        // In GM8, the game is made out of two windows. One is the one you see, and its caption is
        // managed by room_caption and (somewhat) window_set_caption. The other's caption is set by
        // set_application_title, and its caption only shows up in the taskbar and task manager.
        // The emulator only uses one window, and emulating this behaviour isn't possible with just
        // one window, so emulating set_application_title isn't possible.
        // It's a write-only attribute, so simply making it a NOP doesn't hurt anything.
        Ok(Default::default())
    }

    pub fn variable_global_exists(&self, args: &[Value]) -> gml::Result<Value> {
        let identifier = expect_args!(args, [bytes])?;
        if let Some(var) = mappings::get_instance_variable_by_name(identifier.as_ref()) {
            Ok(self.globals.vars.contains_key(var).into())
        } else {
            Ok(self
                .compiler
                .find_field_id(identifier.as_ref())
                .map_or(false, |i| self.globals.fields.contains_key(&i))
                .into())
        }
    }

    pub fn variable_global_get(&self, args: &[Value]) -> gml::Result<Value> {
        let identifier = expect_args!(args, [any])?;
        self.variable_global_array_get(&[identifier, 0.into()])
    }

    pub fn variable_global_array_get(&self, args: &[Value]) -> gml::Result<Value> {
        let (identifier, index) = expect_args!(args, [bytes, int])?;
        let index = index as u32;
        if let Some(var) = mappings::get_instance_variable_by_name(identifier.as_ref()) {
            Ok(self.globals.vars.get(var).and_then(|x| x.get(index)).unwrap_or_default())
        } else {
            Ok(self
                .compiler
                .find_field_id(identifier.as_ref())
                .and_then(|i| self.globals.fields.get(&i))
                .and_then(|x| x.get(index))
                .unwrap_or_default())
        }
    }

    pub fn variable_global_array2_get(&self, args: &[Value]) -> gml::Result<Value> {
        let (identifier, index1, index2) = expect_args!(args, [any, int, int])?;
        self.variable_global_array_get(&[identifier, ((index1 * 32000) + index2).into()])
    }

    pub fn variable_global_set(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (identifier, value) = expect_args!(args, [any, any])?;
        self.variable_global_array_set(&[identifier, 0.into(), value])
    }

    pub fn variable_global_array_set(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (identifier, index, value) = expect_args!(args, [bytes, int, any])?;
        let index = index as u32;
        if let Some(var) = mappings::get_instance_variable_by_name(identifier.as_ref()) {
            if let Some(field) = self.globals.vars.get_mut(var) {
                field.set(index, value);
            } else {
                self.globals.vars.insert(*var, Field::new(index, value));
            }
        } else {
            let field_id = self.compiler.get_field_id(identifier.as_ref());
            if let Some(field) = self.globals.fields.get_mut(&field_id) {
                field.set(index, value);
            } else {
                self.globals.fields.insert(field_id, Field::new(index, value));
            }
        }
        Ok(Default::default())
    }

    pub fn variable_global_array2_set(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (identifier, index1, index2, value) = expect_args!(args, [any, int, int, any])?;
        self.variable_global_array_set(&[identifier, ((index1 * 32000) + index2).into(), value])
    }

    pub fn variable_local_exists(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let identifier = expect_args!(args, [bytes])?;
        if mappings::get_instance_variable_by_name(identifier.as_ref()).is_some() {
            Ok(gml::TRUE.into())
        } else {
            Ok(self
                .compiler
                .find_field_id(identifier.as_ref())
                .map_or(false, |i| self.room.instance_list.get(context.this).fields.borrow().contains_key(&i))
                .into())
        }
    }

    pub fn variable_local_get(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let identifier = expect_args!(args, [any])?;
        self.variable_local_array_get(context, &[identifier, 0.into()])
    }

    pub fn variable_local_array_get(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (identifier, index) = expect_args!(args, [bytes, int])?;
        let index = index as u32;
        if let Some(var) = mappings::get_instance_variable_by_name(identifier.as_ref()) {
            self.get_instance_var(context.this, var, index, context)
        } else {
            let fields_ref = self.room.instance_list.get(context.this).fields.borrow();
            Ok(self
                .compiler
                .find_field_id(identifier.as_ref())
                .and_then(|i| fields_ref.get(&i))
                .and_then(|x| x.get(index))
                .unwrap_or_default())
        }
    }

    pub fn variable_local_array2_get(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (identifier, index1, index2) = expect_args!(args, [any, int, int])?;
        self.variable_local_array_get(context, &[identifier, ((index1 * 32000) + index2).into()])
    }

    pub fn variable_local_set(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (identifier, value) = expect_args!(args, [any, any])?;
        self.variable_local_array_set(context, &[identifier, 0.into(), value])
    }

    pub fn variable_local_array_set(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (identifier, index, value) = expect_args!(args, [bytes, int, any])?;
        let index = index as u32;
        if let Some(var) = mappings::get_instance_variable_by_name(identifier.as_ref()) {
            self.set_instance_var(context.this, var, index, value, context)?;
        } else {
            let mut fields = self.room.instance_list.get(context.this).fields.borrow_mut();
            let field_id = self.compiler.get_field_id(identifier.as_ref());
            if let Some(field) = fields.get_mut(&field_id) {
                field.set(index, value);
            } else {
                fields.insert(field_id, Field::new(index, value));
            }
        }
        Ok(Default::default())
    }

    pub fn variable_local_array2_set(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        let (identifier, index1, index2, value) = expect_args!(args, [any, int, int, any])?;
        self.variable_local_array_set(context, &[identifier, ((index1 * 32000) + index2).into(), value])
    }

    pub fn clipboard_has_text(&self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 0
        unimplemented!("Called unimplemented kernel function clipboard_has_text")
    }

    pub fn clipboard_set_text(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        unimplemented!("Called unimplemented kernel function clipboard_set_text")
    }

    pub fn clipboard_get_text(&self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 0
        unimplemented!("Called unimplemented kernel function clipboard_get_text")
    }

    pub fn date_current_datetime(&self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        Ok(DateTime::now_or_nanos(self.spoofed_time_nanos).into())
    }

    pub fn date_current_date(&self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        Ok(DateTime::now_or_nanos(self.spoofed_time_nanos).date().into())
    }

    pub fn date_current_time(&self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        Ok(DateTime::now_or_nanos(self.spoofed_time_nanos).time().into())
    }

    pub fn date_create_datetime(args: &[Value]) -> gml::Result<Value> {
        // todo: this may give different results than gm8 due to floating point error?
        let (year, month, day, hour, minute, second) = expect_args!(args, [int, int, int, int, int, int])?;
        Ok(DateTime::from_ymdhms(year, month, day, hour, minute, second).map(Real::from).unwrap_or(0.into()).into())
    }

    pub fn date_create_date(args: &[Value]) -> gml::Result<Value> {
        let (year, month, day) = expect_args!(args, [int, int, int])?;
        Ok(DateTime::from_ymd(year, month, day).map(Real::from).unwrap_or(0.into()).into())
    }

    pub fn date_create_time(args: &[Value]) -> gml::Result<Value> {
        let (hour, minute, second) = expect_args!(args, [int, int, int])?;
        Ok(DateTime::from_hms(hour, minute, second).map(Real::from).unwrap_or(0.into()).into())
    }

    pub fn date_valid_datetime(args: &[Value]) -> gml::Result<Value> {
        let (year, month, day, hour, minute, second) = expect_args!(args, [int, int, int, int, int, int])?;
        Ok(DateTime::from_ymd(year, month, day).and_then(|_| DateTime::from_hms(hour, minute, second)).is_some().into())
    }

    pub fn date_valid_date(args: &[Value]) -> gml::Result<Value> {
        let (y, m, d) = expect_args!(args, [int, int, int])?;
        Ok(DateTime::from_ymd(y, m, d).is_some().into())
    }

    pub fn date_valid_time(args: &[Value]) -> gml::Result<Value> {
        let (h, m, s) = expect_args!(args, [int, int, int])?;
        // 24:00:00 is counted as valid, even though making such a date would cause a crash
        Ok((((0..24).contains(&h) && (0..60).contains(&m) && (0..60).contains(&s)) || (h, m, s) == (24, 0, 0)).into())
    }

    pub fn date_inc_year(_args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 2
        unimplemented!("Called unimplemented kernel function date_inc_year")
    }

    pub fn date_inc_month(_args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 2
        unimplemented!("Called unimplemented kernel function date_inc_month")
    }

    pub fn date_inc_week(args: &[Value]) -> gml::Result<Value> {
        let (datetime, amount) = expect_args!(args, [real, int])?;
        Ok((datetime + Real::from(amount * 7)).into())
    }

    pub fn date_inc_day(args: &[Value]) -> gml::Result<Value> {
        let (datetime, amount) = expect_args!(args, [real, int])?;
        Ok((datetime + Real::from(amount)).into())
    }

    pub fn date_inc_hour(args: &[Value]) -> gml::Result<Value> {
        let (datetime, amount) = expect_args!(args, [real, int])?;
        Ok((datetime + Real::from(amount) / 24.into() * if datetime <= 0.into() { Real::from(-1) } else { 1.into() })
            .into())
    }

    pub fn date_inc_minute(args: &[Value]) -> gml::Result<Value> {
        let (datetime, amount) = expect_args!(args, [real, int])?;
        Ok((datetime + Real::from(amount) / 1440.into() * if datetime <= 0.into() { Real::from(-1) } else { 1.into() })
            .into())
    }

    pub fn date_inc_second(args: &[Value]) -> gml::Result<Value> {
        let (datetime, amount) = expect_args!(args, [real, int])?;
        Ok((datetime
            + Real::from(amount) / 86400.into() * if datetime <= 0.into() { Real::from(-1) } else { 1.into() })
        .into())
    }

    pub fn date_get_year(args: &[Value]) -> gml::Result<Value> {
        let datetime = expect_args!(args, [real])?;
        Ok(DateTime::from(datetime).year().into())
    }

    pub fn date_get_month(args: &[Value]) -> gml::Result<Value> {
        let datetime = expect_args!(args, [real])?;
        Ok(DateTime::from(datetime).month().into())
    }

    pub fn date_get_week(args: &[Value]) -> gml::Result<Value> {
        let datetime = expect_args!(args, [real])?;
        Ok(DateTime::from(datetime).week().into())
    }

    pub fn date_get_day(args: &[Value]) -> gml::Result<Value> {
        let datetime = expect_args!(args, [real])?;
        Ok(DateTime::from(datetime).day().into())
    }

    pub fn date_get_hour(args: &[Value]) -> gml::Result<Value> {
        let datetime = expect_args!(args, [real])?;
        Ok(DateTime::from(datetime).hour().into())
    }

    pub fn date_get_minute(args: &[Value]) -> gml::Result<Value> {
        let datetime = expect_args!(args, [real])?;
        Ok(DateTime::from(datetime).minute().into())
    }

    pub fn date_get_second(args: &[Value]) -> gml::Result<Value> {
        let datetime = expect_args!(args, [real])?;
        Ok(DateTime::from(datetime).second().into())
    }

    pub fn date_get_weekday(args: &[Value]) -> gml::Result<Value> {
        let datetime = expect_args!(args, [real])?;
        Ok(DateTime::from(datetime).weekday().into())
    }

    pub fn date_get_day_of_year(args: &[Value]) -> gml::Result<Value> {
        let datetime = expect_args!(args, [real])?;
        Ok(DateTime::from(datetime).day_of_year().into())
    }

    pub fn date_get_hour_of_year(args: &[Value]) -> gml::Result<Value> {
        let datetime = expect_args!(args, [real])?;
        Ok(DateTime::from(datetime).hour_of_year().into())
    }

    pub fn date_get_minute_of_year(args: &[Value]) -> gml::Result<Value> {
        let datetime = expect_args!(args, [real])?;
        Ok(DateTime::from(datetime).minute_of_year().into())
    }

    pub fn date_get_second_of_year(args: &[Value]) -> gml::Result<Value> {
        let datetime = expect_args!(args, [real])?;
        Ok(DateTime::from(datetime).second_of_year().into())
    }

    pub fn date_year_span(_args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 2
        unimplemented!("Called unimplemented kernel function date_year_span")
    }

    pub fn date_month_span(_args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 2
        unimplemented!("Called unimplemented kernel function date_month_span")
    }

    pub fn date_week_span(_args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 2
        unimplemented!("Called unimplemented kernel function date_week_span")
    }

    pub fn date_day_span(_args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 2
        unimplemented!("Called unimplemented kernel function date_day_span")
    }

    pub fn date_hour_span(_args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 2
        unimplemented!("Called unimplemented kernel function date_hour_span")
    }

    pub fn date_minute_span(_args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 2
        unimplemented!("Called unimplemented kernel function date_minute_span")
    }

    pub fn date_second_span(_args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 2
        unimplemented!("Called unimplemented kernel function date_second_span")
    }

    pub fn date_compare_datetime(_args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 2
        unimplemented!("Called unimplemented kernel function date_compare_datetime")
    }

    pub fn date_compare_date(_args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 2
        unimplemented!("Called unimplemented kernel function date_compare_date")
    }

    pub fn date_compare_time(_args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 2
        unimplemented!("Called unimplemented kernel function date_compare_time")
    }

    pub fn date_date_of(_args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        unimplemented!("Called unimplemented kernel function date_date_of")
    }

    pub fn date_time_of(_args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        unimplemented!("Called unimplemented kernel function date_time_of")
    }

    pub fn date_datetime_string(&self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        unimplemented!("Called unimplemented kernel function date_datetime_string")
    }

    pub fn date_date_string(&self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        unimplemented!("Called unimplemented kernel function date_date_string")
    }

    pub fn date_time_string(&self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        unimplemented!("Called unimplemented kernel function date_time_string")
    }

    pub fn date_days_in_month(_args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        unimplemented!("Called unimplemented kernel function date_days_in_month")
    }

    pub fn date_days_in_year(_args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        unimplemented!("Called unimplemented kernel function date_days_in_year")
    }

    pub fn date_leap_year(_args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        unimplemented!("Called unimplemented kernel function date_leap_year")
    }

    pub fn date_is_today(&self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        unimplemented!("Called unimplemented kernel function date_is_today")
    }

    pub fn sprite_exists(&self, args: &[Value]) -> gml::Result<Value> {
        let sprite = expect_args!(args, [int])?;
        Ok(self.assets.sprites.get_asset(sprite).is_some().into())
    }

    pub fn sprite_get_name(&self, args: &[Value]) -> gml::Result<Value> {
        let asset_id = expect_args!(args, [int])?;
        Ok(self.assets.sprites.get_asset(asset_id).map(|x| x.name.clone().into()).unwrap_or("<undefined>".into()))
    }

    pub fn sprite_get_number(&self, args: &[Value]) -> gml::Result<Value> {
        let sprite = expect_args!(args, [int])?;
        if let Some(sprite) = self.assets.sprites.get_asset(sprite) {
            Ok(sprite.frames.len().into())
        } else {
            Ok((-1).into())
        }
    }

    pub fn sprite_get_width(&self, args: &[Value]) -> gml::Result<Value> {
        let sprite = expect_args!(args, [int])?;
        if let Some(sprite) = self.assets.sprites.get_asset(sprite) { Ok(sprite.width.into()) } else { Ok((-1).into()) }
    }

    pub fn sprite_get_height(&self, args: &[Value]) -> gml::Result<Value> {
        let sprite = expect_args!(args, [int])?;
        if let Some(sprite) = self.assets.sprites.get_asset(sprite) {
            Ok(sprite.height.into())
        } else {
            Ok((-1).into())
        }
    }

    pub fn sprite_get_xoffset(&self, args: &[Value]) -> gml::Result<Value> {
        let sprite = expect_args!(args, [int])?;
        if let Some(sprite) = self.assets.sprites.get_asset(sprite) {
            Ok(sprite.origin_x.into())
        } else {
            Ok((-1).into())
        }
    }

    pub fn sprite_get_yoffset(&self, args: &[Value]) -> gml::Result<Value> {
        let sprite = expect_args!(args, [int])?;
        if let Some(sprite) = self.assets.sprites.get_asset(sprite) {
            Ok(sprite.origin_y.into())
        } else {
            Ok((-1).into())
        }
    }

    pub fn sprite_get_bbox_left(&self, args: &[Value]) -> gml::Result<Value> {
        let sprite = expect_args!(args, [int])?;
        if let Some(sprite) = self.assets.sprites.get_asset(sprite) {
            Ok(sprite.bbox_left.into())
        } else {
            Ok((-1).into())
        }
    }

    pub fn sprite_get_bbox_right(&self, args: &[Value]) -> gml::Result<Value> {
        let sprite = expect_args!(args, [int])?;
        if let Some(sprite) = self.assets.sprites.get_asset(sprite) {
            Ok(sprite.bbox_right.into())
        } else {
            Ok((-1).into())
        }
    }

    pub fn sprite_get_bbox_top(&self, args: &[Value]) -> gml::Result<Value> {
        let sprite = expect_args!(args, [int])?;
        if let Some(sprite) = self.assets.sprites.get_asset(sprite) {
            Ok(sprite.bbox_top.into())
        } else {
            Ok((-1).into())
        }
    }

    pub fn sprite_get_bbox_bottom(&self, args: &[Value]) -> gml::Result<Value> {
        let sprite = expect_args!(args, [int])?;
        if let Some(sprite) = self.assets.sprites.get_asset(sprite) {
            Ok(sprite.bbox_bottom.into())
        } else {
            Ok((-1).into())
        }
    }

    pub fn sprite_set_offset(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (sprite, x, y) = expect_args!(args, [int, int, int])?;
        if let Some(sprite) = self.assets.sprites.get_asset_mut(sprite) {
            sprite.origin_x = x;
            sprite.origin_y = y;
        }
        Ok(Default::default())
    }

    pub fn sprite_set_alpha_from_sprite(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (dst_id, src_id) = expect_args!(args, [int, int])?;
        if let Some(src) = self.assets.sprites.get_asset(src_id) {
            let src_frames = src.frames.clone();
            if let Some(dst) = self.assets.sprites.get_asset_mut(dst_id) {
                for (dst_frame, src_frame) in dst.frames.iter_mut().zip(src_frames.iter().cycle()) {
                    let src_data = self.renderer.dump_sprite(&src_frame.atlas_ref);
                    let mut dst_data = self.renderer.dump_sprite(&dst_frame.atlas_ref);
                    // TODO: delete sprite when this is safe for sprite fonts
                    // self.renderer.delete_sprite(dst_frame.atlas_ref);
                    for (dst_row, src_row) in dst_data
                        .chunks_mut(dst_frame.width as usize * 4)
                        .zip(src_data.chunks(src_frame.width as usize * 4))
                    {
                        for (dst_col, src_col) in dst_row.chunks_mut(4).zip(src_row.chunks(4)) {
                            dst_col[3] = (src_col[..3].iter().map(|&x| u16::from(x)).sum::<u16>() / 3u16) as u8;
                        }
                    }
                    dst_frame.atlas_ref = self
                        .renderer
                        .upload_sprite(
                            dst_data,
                            dst_frame.width as _,
                            dst_frame.height as _,
                            dst.origin_x,
                            dst.origin_y,
                        )
                        .map_err(|e| gml::Error::FunctionError("sprite_set_alpha_from_sprite".into(), e))?;
                }
            }
        }
        Ok(Default::default())
    }

    pub fn sprite_create_from_screen(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (x, y, width, height, transparency, smooth, origin_x, origin_y) =
            expect_args!(args, [int, int, int, int, int, bool, int, int])?;
        let (removeback, fill_transparent) = match self.gm_version {
            Version::GameMaker8_0 => (transparency != 0, true),
            Version::GameMaker8_1 => (transparency == 1, transparency != 2),
        };
        // i know we're downloading the thing and reuploading it instead of doing it all in one go
        // but we need the pixel data to make the colliders
        let x = x.max(0);
        let y = y.max(0);
        let width = width.min(self.unscaled_width as i32 - x);
        let height = height.min(self.unscaled_height as i32 - y);
        self.renderer.flush_queue();
        let rgba = self.renderer.get_pixels(x, y, width, height);
        let mut image = RgbaImage::from_vec(width as _, height as _, rgba.into_vec()).unwrap();
        asset::sprite::process_image(&mut image, removeback, smooth, fill_transparent);
        if self.gm_version == Version::GameMaker8_1 && transparency == -1 {
            // make entire image opaque
            image.pixels_mut().for_each(|p| p[3] = 255);
        }
        let colliders = asset::sprite::make_colliders_precise(std::slice::from_ref(&image), 0, false);
        let frames = vec![asset::sprite::Frame {
            width: width as _,
            height: height as _,
            atlas_ref: self
                .renderer
                .upload_sprite(image.into_raw().into_boxed_slice(), width, height, origin_x, origin_y)
                .map_err(|e| gml::Error::FunctionError("sprite_create_from_screen".into(), e.into()))?,
        }];
        let sprite_id = self.assets.sprites.len();
        self.assets.sprites.push(Some(Box::new(asset::Sprite {
            name: format!("__newsprite{}", sprite_id).into(),
            frames,
            bbox_left: colliders[0].bbox_left,
            bbox_right: colliders[0].bbox_right,
            bbox_top: colliders[0].bbox_top,
            bbox_bottom: colliders[0].bbox_bottom,
            colliders: colliders,
            width: width as _,
            height: height as _,
            origin_x,
            origin_y,
            per_frame_colliders: false,
        })));
        Ok(sprite_id.into())
    }

    pub fn sprite_add_from_screen(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (sprite_id, x, y, width, height, removeback, smooth) =
            expect_args!(args, [int, int, int, int, int, bool, bool])?;
        if let Some(sprite) = self.assets.sprites.get_asset_mut(sprite_id) {
            // get image
            let x = x.max(0);
            let y = y.max(0);
            let width = width.min(self.unscaled_width as i32 - x);
            let height = height.min(self.unscaled_height as i32 - y);
            self.renderer.flush_queue();
            let rgba = self.renderer.get_pixels(x, y, width, height);
            let mut image = RgbaImage::from_vec(width as _, height as _, rgba.into_vec()).unwrap();
            asset::sprite::process_image(&mut image, removeback, smooth, true);
            asset::sprite::scale(&mut image, sprite.width, sprite.height);
            // generate collision
            let mut images = Vec::with_capacity(sprite.frames.len() + 1);
            // can't use .map() because closures cause borrowing issues
            for f in sprite.frames.iter() {
                images.push(
                    RgbaImage::from_vec(f.width, f.height, self.renderer.dump_sprite(&f.atlas_ref).into_vec()).unwrap(),
                );
            }
            images.push(image);
            let sprite = self.assets.sprites.get_asset_mut(sprite_id).unwrap();
            sprite.colliders = asset::sprite::make_colliders_precise(&images, 0, sprite.per_frame_colliders);
            sprite.bbox_left = sprite.colliders.iter().map(|c| c.bbox_left).min().unwrap();
            sprite.bbox_top = sprite.colliders.iter().map(|c| c.bbox_top).min().unwrap();
            sprite.bbox_right = sprite.colliders.iter().map(|c| c.bbox_right).max().unwrap();
            sprite.bbox_bottom = sprite.colliders.iter().map(|c| c.bbox_bottom).max().unwrap();
            // upload frame
            let image = images.pop().unwrap();
            sprite.frames.push(asset::sprite::Frame {
                width: sprite.width as _,
                height: sprite.height as _,
                atlas_ref: self
                    .renderer
                    .upload_sprite(
                        image.into_raw().into_boxed_slice(),
                        sprite.width as _,
                        sprite.height as _,
                        sprite.origin_x,
                        sprite.origin_y,
                    )
                    .map_err(|e| gml::Error::FunctionError("sprite_add_from_screen".into(), e.into()))?,
            });
            Ok(Default::default())
        } else {
            Err(gml::Error::NonexistentAsset(asset::Type::Sprite, sprite_id))
        }
    }

    pub fn sprite_create_from_surface(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (surf_id, x, y, width, height, transparency, smooth, origin_x, origin_y) =
            expect_args!(args, [int, int, int, int, int, int, bool, int, int])?;
        if self.surface_target == Some(surf_id) {
            self.renderer.flush_queue();
        }
        if let Some(surf) = self.surfaces.get_asset(surf_id) {
            let x = x.max(0);
            let y = y.max(0);
            let width = width.min(surf.width as i32 - x);
            let height = height.min(surf.height as i32 - y);
            let (removeback, fill_transparent) = match self.gm_version {
                Version::GameMaker8_0 => (transparency != 0, true),
                Version::GameMaker8_1 => (transparency == 1, transparency != 2),
            };
            let rgba = self.renderer.dump_sprite_part(&surf.atlas_ref, x, y, width, height);
            let mut image = RgbaImage::from_vec(width as _, height as _, rgba.into_vec()).unwrap();
            asset::sprite::process_image(&mut image, removeback, smooth, fill_transparent);
            if self.gm_version == Version::GameMaker8_1 && transparency == -1 {
                // make entire image opaque
                image.pixels_mut().for_each(|p| p[3] = 255);
            }
            let colliders = asset::sprite::make_colliders_precise(std::slice::from_ref(&image), 0, false);
            let frames = vec![asset::sprite::Frame {
                width: width as _,
                height: height as _,
                atlas_ref: self
                    .renderer
                    .upload_sprite(image.into_raw().into_boxed_slice(), width, height, origin_x, origin_y)
                    .map_err(|e| gml::Error::FunctionError("sprite_create_from_surface".into(), e.into()))?,
            }];
            let sprite_id = self.assets.sprites.len();
            self.assets.sprites.push(Some(Box::new(asset::Sprite {
                name: format!("__newsprite{}", sprite_id).into(),
                frames,
                bbox_left: colliders[0].bbox_left,
                bbox_right: colliders[0].bbox_right,
                bbox_top: colliders[0].bbox_top,
                bbox_bottom: colliders[0].bbox_bottom,
                colliders: colliders,
                width: width as _,
                height: height as _,
                origin_x,
                origin_y,
                per_frame_colliders: false,
            })));
            Ok(sprite_id.into())
        } else {
            Err(gml::Error::FunctionError(
                "sprite_create_from_surface".into(),
                format!("Surface {} does not exist", surf_id),
            ))
        }
    }

    pub fn sprite_add_from_surface(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (sprite_id, surf_id, x, y, width, height, removeback, smooth) =
            expect_args!(args, [int, int, int, int, int, int, bool, bool])?;
        if let Some(sprite) = self.assets.sprites.get_asset_mut(sprite_id) {
            if let Some(surf) = self.surfaces.get_asset(surf_id) {
                // get image
                let x = x.max(0);
                let y = y.max(0);
                let width = width.min(surf.width as i32 - x);
                let height = height.min(surf.height as i32 - y);
                let rgba = self.renderer.dump_sprite_part(&surf.atlas_ref, x, y, width, height);
                let mut image = RgbaImage::from_vec(width as _, height as _, rgba.into_vec()).unwrap();
                asset::sprite::process_image(&mut image, removeback, smooth, true);
                asset::sprite::scale(&mut image, sprite.width, sprite.height);
                // generate collision
                let mut images = Vec::with_capacity(sprite.frames.len() + 1);
                // can't use .map() because closures cause borrowing issues
                for f in sprite.frames.iter() {
                    images.push(
                        RgbaImage::from_vec(f.width, f.height, self.renderer.dump_sprite(&f.atlas_ref).into_vec())
                            .unwrap(),
                    );
                }
                images.push(image);
                let sprite = self.assets.sprites.get_asset_mut(sprite_id).unwrap();
                sprite.colliders = asset::sprite::make_colliders_precise(&images, 0, sprite.per_frame_colliders);
                sprite.bbox_left = sprite.colliders.iter().map(|c| c.bbox_left).min().unwrap();
                sprite.bbox_top = sprite.colliders.iter().map(|c| c.bbox_top).min().unwrap();
                sprite.bbox_right = sprite.colliders.iter().map(|c| c.bbox_right).max().unwrap();
                sprite.bbox_bottom = sprite.colliders.iter().map(|c| c.bbox_bottom).max().unwrap();
                // upload frame
                let image = images.pop().unwrap();
                sprite.frames.push(asset::sprite::Frame {
                    width: sprite.width as _,
                    height: sprite.height as _,
                    atlas_ref: self
                        .renderer
                        .upload_sprite(
                            image.into_raw().into_boxed_slice(),
                            sprite.width as _,
                            sprite.height as _,
                            sprite.origin_x,
                            sprite.origin_y,
                        )
                        .map_err(|e| gml::Error::FunctionError("sprite_add_from_surface".into(), e.into()))?,
                });
                Ok(Default::default())
            } else {
                Err(gml::Error::FunctionError(
                    "sprite_add_from_surface".into(),
                    format!("Surface {} does not exist", surf_id),
                ))
            }
        } else {
            Err(gml::Error::NonexistentAsset(asset::Type::Sprite, sprite_id))
        }
    }

    pub fn sprite_add(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (fname, imgnumb, removeback, smooth, origin_x, origin_y) =
            expect_args!(args, [string, int, bool, bool, int, int])?;
        let imgnumb = imgnumb.max(1) as usize;
        let mut images = match file::load_animation(fname.as_ref(), imgnumb) {
            Ok(frames) => frames,
            Err(e) => {
                eprintln!("Warning: sprite_add on {} failed: {}", fname, e);
                return Ok((-1).into())
            },
        };
        for image in images.iter_mut() {
            asset::sprite::process_image(image, removeback, smooth, true);
        }
        let (width, height) = images[0].dimensions();
        // make colliders
        let colliders = asset::sprite::make_colliders_precise(&images, 0, false);
        // collect atlas refs
        // yes i know it's a new texture for every frame like in gm8 but it's fine
        let frames = images
            .drain(..)
            .map(|i| {
                Ok(asset::sprite::Frame {
                    width,
                    height,
                    atlas_ref: self
                        .renderer
                        .upload_sprite(i.into_raw().into_boxed_slice(), width as _, height as _, origin_x, origin_y)
                        .map_err(|e| gml::Error::FunctionError("sprite_add".into(), e.into()))?,
                })
            })
            .collect::<gml::Result<_>>()?;
        let sprite_id = self.assets.sprites.len();
        self.assets.sprites.push(Some(Box::new(asset::Sprite {
            name: format!("__newsprite{}", sprite_id).into(),
            frames,
            bbox_left: colliders[0].bbox_left,
            bbox_right: colliders[0].bbox_right,
            bbox_top: colliders[0].bbox_top,
            bbox_bottom: colliders[0].bbox_bottom,
            colliders,
            width,
            height,
            origin_x,
            origin_y,
            per_frame_colliders: false,
        })));
        Ok(sprite_id.into())
    }

    pub fn sprite_replace(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (sprite_id, fname, imgnumb, removeback, smooth, origin_x, origin_y) =
            expect_args!(args, [int, string, int, bool, bool, int, int])?;
        if let Some(sprite) = self.assets.sprites.get_asset_mut(sprite_id) {
            for frame in &sprite.frames {
                self.renderer.delete_sprite(frame.atlas_ref);
            }
            let imgnumb = imgnumb.max(1) as usize;
            let mut images = match file::load_animation(fname.as_ref(), imgnumb) {
                Ok(frames) => frames,
                Err(e) => {
                    eprintln!("Warning: sprite_replace on {} failed: {}", fname, e);
                    return Ok((-1).into())
                },
            };
            for image in images.iter_mut() {
                asset::sprite::process_image(image, removeback, smooth, true);
            }
            let (width, height) = images[0].dimensions();
            // make colliders
            let colliders = asset::sprite::make_colliders_precise(&images, 0, false);
            // collect atlas refs
            let renderer = &mut self.renderer;
            let frames = images
                .drain(..)
                .map(|i| {
                    Ok(asset::sprite::Frame {
                        width,
                        height,
                        atlas_ref: renderer
                            .upload_sprite(i.into_raw().into_boxed_slice(), width as _, height as _, origin_x, origin_y)
                            .map_err(|e| gml::Error::FunctionError("sprite_replace".into(), e.into()))?,
                    })
                })
                .collect::<gml::Result<_>>()?;
            *sprite = Box::new(asset::Sprite {
                name: sprite.name.clone(),
                frames,
                bbox_left: colliders[0].bbox_left,
                bbox_right: colliders[0].bbox_right,
                bbox_top: colliders[0].bbox_top,
                bbox_bottom: colliders[0].bbox_bottom,
                colliders,
                width,
                height,
                origin_x,
                origin_y,
                per_frame_colliders: false,
            });
            Ok(Default::default())
        } else {
            Err(gml::Error::FunctionError("sprite_replace".into(), "Trying to replace non-existing sprite.".into()))
        }
    }

    pub fn sprite_add_sprite(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        unimplemented!("Called unimplemented kernel function sprite_add_sprite")
    }

    pub fn sprite_replace_sprite(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 2
        unimplemented!("Called unimplemented kernel function sprite_replace_sprite")
    }

    pub fn sprite_delete(&mut self, args: &[Value]) -> gml::Result<Value> {
        let sprite_id = expect_args!(args, [int])?;
        if let Some(sprite) = self.assets.sprites.get_asset(sprite_id) {
            for frame in &sprite.frames {
                self.renderer.delete_sprite(frame.atlas_ref);
            }
        } else {
            return Err(gml::Error::FunctionError("sprite_delete".into(), "Trying to delete non-existing sprite".into()))
        }
        self.assets.sprites[sprite_id as usize] = None;
        Ok(Default::default())
    }

    pub fn sprite_duplicate(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        unimplemented!("Called unimplemented kernel function sprite_duplicate")
    }

    pub fn sprite_assign(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (dst_id, src_id) = expect_args!(args, [int, int])?;
        if let Some(src) = self.assets.sprites.get_asset(src_id) {
            if let Some(sprite) = self.assets.sprites.get_asset(dst_id) {
                for frame in &sprite.frames {
                    self.renderer.delete_sprite(frame.atlas_ref);
                }
            }
            if dst_id >= 0 && self.assets.sprites.len() > dst_id as usize {
                let renderer = &mut self.renderer; // borrowck
                let frames = src
                    .frames
                    .iter()
                    .map(|f| {
                        Ok(asset::sprite::Frame {
                            atlas_ref: renderer
                                .duplicate_sprite(&f.atlas_ref)
                                .map_err(|e| gml::Error::FunctionError("sprite_assign".into(), e.into()))?,
                            width: f.width,
                            height: f.height,
                        })
                    })
                    .collect::<gml::Result<_>>()?;
                self.assets.sprites[dst_id as usize] = Some(Box::new(asset::Sprite {
                    name: src.name.clone(),
                    frames,
                    colliders: src.colliders.clone(),
                    width: src.width,
                    height: src.height,
                    origin_x: src.origin_x,
                    origin_y: src.origin_y,
                    per_frame_colliders: src.per_frame_colliders,
                    bbox_left: src.bbox_left,
                    bbox_right: src.bbox_right,
                    bbox_top: src.bbox_top,
                    bbox_bottom: src.bbox_bottom,
                }));
                Ok(Default::default())
            } else {
                Err(gml::Error::FunctionError("sprite_assign".into(), "Destination sprite has an invalid index".into()))
            }
        } else {
            Err(gml::Error::FunctionError("sprite_assign".into(), "Source sprite does not exist".into()))
        }
    }

    pub fn sprite_merge(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 2
        unimplemented!("Called unimplemented kernel function sprite_merge")
    }

    pub fn sprite_save(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (sprite_id, subimg, fname) = expect_args!(args, [int, int, string])?;
        if let Some(sprite) = self.assets.sprites.get_asset(sprite_id) {
            let image_index = subimg % sprite.frames.len() as i32;
            if let Some(frame) = sprite.get_frame(image_index) {
                // get RGBA
                if let Err(e) = file::save_image(
                    fname.as_ref(),
                    RgbaImage::from_vec(frame.width, frame.height, self.renderer.dump_sprite(&frame.atlas_ref).into())
                        .unwrap(),
                ) {
                    return Err(gml::Error::FunctionError("sprite_save".into(), e.to_string()))
                }
            }
        }
        Ok(Default::default())
    }

    pub fn sprite_save_strip(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 2
        unimplemented!("Called unimplemented kernel function sprite_save_strip")
    }

    pub fn sprite_collision_mask(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (sprite_id, sepmasks, bboxmode, bbleft, bbtop, bbright, bbbottom, kind, tolerance) =
            expect_args!(args, [int, bool, int, int, int, int, int, int, int])?;
        let tolerance = tolerance.clamp(0, 255) as u8;
        let sepmasks = sepmasks;
        if let Some(sprite) = self.assets.sprites.get_asset_mut(sprite_id) {
            // formulate requested bounding box
            let bbox = match bboxmode {
                0 => None, // automatic
                1 => Some(asset::sprite::BoundingBox {
                    // full image
                    left: 0,
                    right: sprite.width - 1,
                    top: 0,
                    bottom: sprite.height - 1,
                }),
                _ => Some(asset::sprite::BoundingBox {
                    // user defined
                    left: bbleft.max(0) as u32,
                    right: (bbright as u32).min(sprite.width),
                    top: bbtop.max(0) as u32,
                    bottom: (bbbottom as u32).min(sprite.height),
                }),
            };

            // download frames from gpu
            let renderer = &mut self.renderer;
            let frames = sprite
                .frames
                .iter()
                .map(|f| RgbaImage::from_vec(f.width, f.height, renderer.dump_sprite(&f.atlas_ref).to_vec()).unwrap())
                .collect::<Vec<RgbaImage>>();

            // make colliders
            sprite.colliders = match kind {
                0 => asset::sprite::make_colliders_precise(&frames, tolerance, sepmasks), // precise
                _ => asset::sprite::make_colliders_shaped(&frames, tolerance, sepmasks, bbox, match kind {
                    1 => Some(asset::sprite::ColliderShape::Rectangle),
                    2 => Some(asset::sprite::ColliderShape::Ellipse),
                    3 => Some(asset::sprite::ColliderShape::Diamond),
                    _ => None,
                }),
            };

            // set bbox variables manually if needed (even if using precise collision)
            if let Some(bbox) = bbox {
                for c in &mut sprite.colliders {
                    c.bbox_left = bbox.left;
                    c.bbox_top = bbox.top;
                    c.bbox_right = bbox.right;
                    c.bbox_bottom = bbox.bottom;
                }
            }
            sprite.bbox_left = sprite.colliders.iter().map(|c| c.bbox_left).min().unwrap();
            sprite.bbox_top = sprite.colliders.iter().map(|c| c.bbox_top).min().unwrap();
            sprite.bbox_right = sprite.colliders.iter().map(|c| c.bbox_right).max().unwrap();
            sprite.bbox_bottom = sprite.colliders.iter().map(|c| c.bbox_bottom).max().unwrap();
        }
        Ok(Default::default())
    }

    pub fn sprite_set_cache_size(args: &[Value]) -> gml::Result<Value> {
        // Special GM 8.1 function, has effect only in the original HTML5 runner.
        expect_args!(args, [any, any])?;
        Ok(Default::default())
    }

    pub fn sprite_set_cache_size_ext(args: &[Value]) -> gml::Result<Value> {
        // Special GM 8.1 function, has effect only in the original HTML5 runner.
        expect_args!(args, [any, any, any])?;
        Ok(Default::default())
    }

    pub fn background_exists(&self, args: &[Value]) -> gml::Result<Value> {
        let background_id = expect_args!(args, [int])?;
        Ok(self.assets.backgrounds.get_asset(background_id).is_some().into())
    }

    pub fn background_get_name(&self, args: &[Value]) -> gml::Result<Value> {
        let asset_id = expect_args!(args, [int])?;
        Ok(self.assets.backgrounds.get_asset(asset_id).map(|x| x.name.clone().into()).unwrap_or("<undefined>".into()))
    }

    pub fn background_get_width(&self, args: &[Value]) -> gml::Result<Value> {
        let background_id = expect_args!(args, [int])?;
        if let Some(background) = self.assets.backgrounds.get_asset(background_id) {
            Ok(background.width.into())
        } else {
            Ok((-1).into())
        }
    }

    pub fn background_get_height(&self, args: &[Value]) -> gml::Result<Value> {
        let background_id = expect_args!(args, [int])?;
        if let Some(background) = self.assets.backgrounds.get_asset(background_id) {
            Ok(background.height.into())
        } else {
            Ok((-1).into())
        }
    }

    pub fn background_set_alpha_from_background(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (dst_id, src_id) = expect_args!(args, [int, int])?;
        if self.assets.backgrounds.get_asset(dst_id).filter(|bg| bg.atlas_ref.is_some()).is_none() {
            return Ok(Default::default())
        }
        let (alpha_src, src_w) =
            match self.assets.backgrounds.get_asset(src_id).map(|bg| (bg.atlas_ref.as_ref(), bg.width)) {
                Some((Some(atlas_ref), w)) => (self.renderer.dump_sprite(atlas_ref), w),
                _ => return Ok(Default::default()),
            };
        if let Some((Some(atlas_ref), dst_w, dst_h)) =
            self.assets.backgrounds.get_asset_mut(dst_id).map(|bg| (bg.atlas_ref.as_mut(), bg.width, bg.height))
        {
            let mut dst = self.renderer.dump_sprite(atlas_ref);
            self.renderer.delete_sprite(*atlas_ref);
            for (dst_row, src_row) in dst.chunks_mut(dst_w as usize * 4).zip(alpha_src.chunks(src_w as usize * 4)) {
                for (dst_col, src_col) in dst_row.chunks_mut(4).zip(src_row.chunks(4)) {
                    dst_col[3] = (src_col[..3].iter().map(|&x| u16::from(x)).sum::<u16>() / 3u16) as u8;
                }
            }
            *atlas_ref = self
                .renderer
                .upload_sprite(dst, dst_w as _, dst_h as _, 0, 0)
                .map_err(|e| gml::Error::FunctionError("background_set_alpha_from_background".into(), e))?;
        }
        Ok(Default::default())
    }

    pub fn background_create_from_screen(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (x, y, width, height, removeback, smooth) = expect_args!(args, [int, int, int, int, bool, bool])?;
        let x = x.max(0);
        let y = y.max(0);
        let width = width.min(self.unscaled_width as i32 - x);
        let height = height.min(self.unscaled_height as i32 - y);
        self.renderer.flush_queue();
        let rgba = self.renderer.get_pixels(x, y, width, height);
        let mut image = RgbaImage::from_vec(width as _, height as _, rgba.into_vec()).unwrap();
        asset::sprite::process_image(&mut image, removeback, smooth, true);
        let background_id = self.assets.backgrounds.len();
        self.assets.backgrounds.push(Some(Box::new(asset::Background {
            name: format!("__newbackground{}", background_id).into(),
            width: width as _,
            height: height as _,
            atlas_ref: Some(
                self.renderer
                    .upload_sprite(image.into_raw().into_boxed_slice(), width, height, 0, 0)
                    .map_err(|e| gml::Error::FunctionError("background_create_from_screen".into(), e.into()))?,
            ),
        })));
        Ok(background_id.into())
    }

    pub fn background_create_from_surface(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (surf_id, x, y, width, height, removeback, smooth) =
            expect_args!(args, [int, int, int, int, int, bool, bool])?;
        if self.surface_target == Some(surf_id) {
            self.renderer.flush_queue();
        }
        if let Some(surf) = self.surfaces.get_asset(surf_id) {
            let x = x.max(0);
            let y = y.max(0);
            let width = width.min(surf.width as i32 - x);
            let height = height.min(surf.height as i32 - y);
            let rgba = self.renderer.dump_sprite_part(&surf.atlas_ref, x, y, width, height);
            let mut image = RgbaImage::from_vec(width as _, height as _, rgba.into_vec()).unwrap();
            asset::sprite::process_image(&mut image, removeback, smooth, true);
            let background_id = self.assets.backgrounds.len();
            self.assets.backgrounds.push(Some(Box::new(asset::Background {
                name: format!("__newbackground{}", background_id).into(),
                width: width as _,
                height: height as _,
                atlas_ref: Some(
                    self.renderer
                        .upload_sprite(image.into_raw().into_boxed_slice(), width, height, 0, 0)
                        .map_err(|e| gml::Error::FunctionError("background_create_from_surface".into(), e.into()))?,
                ),
            })));
            Ok(background_id.into())
        } else {
            Err(gml::Error::FunctionError(
                "background_create_from_surface".into(),
                format!("Surface {} does not exist", surf_id),
            ))
        }
    }

    pub fn background_create_color(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (w, h, col) = expect_args!(args, [int, int, int])?;
        let background_id = self.assets.backgrounds.len();
        self.assets.backgrounds.push(Some(Box::new(asset::Background {
            name: format!("__newbackground{}", background_id).into(),
            width: w as _,
            height: h as _,
            atlas_ref: Some(
                self.renderer
                    .create_sprite_colour(w, h, (col as u32).into())
                    .map_err(|e| gml::Error::FunctionError("background_create_color".into(), e))?,
            ),
        })));
        Ok(background_id.into())
    }

    pub fn background_create_gradient(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 5
        unimplemented!("Called unimplemented kernel function background_create_gradient")
    }

    pub fn background_add(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (fname, removeback, smooth) = expect_args!(args, [string, bool, bool])?;
        let mut image = match file::load_image(fname.as_ref()) {
            Ok(im) => im,
            Err(e) => {
                eprintln!("Warning: background_add on {} failed: {}", fname, e);
                return Ok((-1).into())
            },
        };
        asset::sprite::process_image(&mut image, removeback, smooth, true);
        let width = image.width();
        let height = image.height();
        let atlas_ref = self
            .renderer
            .upload_sprite(image.into_raw().into_boxed_slice(), width as _, height as _, 0, 0)
            .map_err(|e| gml::Error::FunctionError("background_add".into(), e.into()))?;
        let background_id = self.assets.backgrounds.len();
        self.assets.backgrounds.push(Some(Box::new(asset::Background {
            name: format!("__newbackground{}", background_id).into(),
            width,
            height,
            atlas_ref: Some(atlas_ref),
        })));
        Ok(background_id.into())
    }

    pub fn background_replace(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (background_id, fname, removeback, smooth) = expect_args!(args, [int, string, bool, bool])?;
        if let Some(background) = self.assets.backgrounds.get_asset_mut(background_id) {
            if let Some(atlas_ref) = background.atlas_ref {
                self.renderer.delete_sprite(atlas_ref);
            }
            let mut image = match file::load_image(fname.as_ref()) {
                Ok(im) => im,
                Err(e) => {
                    eprintln!("Warning: background_replace on {} failed: {}", fname, e);
                    return Ok((-1).into())
                },
            };
            asset::sprite::process_image(&mut image, removeback, smooth, true);
            let width = image.width();
            let height = image.height();
            let atlas_ref = self
                .renderer
                .upload_sprite(image.into_raw().into_boxed_slice(), width as _, height as _, 0, 0)
                .map_err(|e| gml::Error::FunctionError("background_replace".into(), e.into()))?;
            background.atlas_ref = Some(atlas_ref);
            background.width = width;
            background.height = height;
            Ok(Default::default())
        } else {
            Err(gml::Error::FunctionError(
                "background_replace".into(),
                "Trying to replace non-existing background.".into(),
            ))
        }
    }

    pub fn background_add_background(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        unimplemented!("Called unimplemented kernel function background_add_background")
    }

    pub fn background_replace_background(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 2
        unimplemented!("Called unimplemented kernel function background_replace_background")
    }

    pub fn background_delete(&mut self, args: &[Value]) -> gml::Result<Value> {
        let background_id = expect_args!(args, [int])?;
        if let Some(background) = self.assets.backgrounds.get_asset(background_id) {
            if let Some(atlas_ref) = background.atlas_ref {
                self.renderer.delete_sprite(atlas_ref);
            }
        } else {
            return Err(gml::Error::FunctionError(
                "background_delete".into(),
                "Trying to delete non-existing background".into(),
            ))
        }
        self.assets.backgrounds[background_id as usize] = None;
        Ok(Default::default())
    }

    pub fn background_duplicate(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        unimplemented!("Called unimplemented kernel function background_duplicate")
    }

    pub fn background_assign(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (dst_id, src_id) = expect_args!(args, [int, int])?;
        if let Some(src) = self.assets.backgrounds.get_asset(src_id) {
            if let Some(background) = self.assets.backgrounds.get_asset(dst_id) {
                if let Some(atlas_ref) = background.atlas_ref {
                    self.renderer.delete_sprite(atlas_ref);
                }
            }
            if dst_id >= 0 && self.assets.backgrounds.len() > dst_id as usize {
                let dst_atlref = match src.atlas_ref.as_ref() {
                    Some(ar) => Some(
                        self.renderer
                            .duplicate_sprite(ar)
                            .map_err(|e| gml::Error::FunctionError("background_assign".into(), e.into()))?,
                    ),
                    None => None,
                };
                self.assets.backgrounds[dst_id as usize] = Some(Box::new(asset::Background {
                    atlas_ref: dst_atlref,
                    width: src.width,
                    height: src.height,
                    name: src.name.clone(),
                }));
                Ok(Default::default())
            } else {
                Err(gml::Error::FunctionError(
                    "background_assign".into(),
                    "Destination background has an invalid index".into(),
                ))
            }
        } else {
            Err(gml::Error::FunctionError("background_assign".into(), "Source background does not exist".into()))
        }
    }

    pub fn background_save(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (background_id, fname) = expect_args!(args, [int, string])?;
        if let Some(background) = self.assets.backgrounds.get_asset(background_id) {
            if let Some(atlas_ref) = background.atlas_ref.as_ref() {
                // get RGBA
                if let Err(e) = file::save_image(
                    fname.as_ref(),
                    RgbaImage::from_vec(
                        background.width,
                        background.height,
                        self.renderer.dump_sprite(atlas_ref).into(),
                    )
                    .unwrap(),
                ) {
                    return Err(gml::Error::FunctionError("background_save".into(), e.to_string()))
                }
            }
        }
        Ok(Default::default())
    }

    pub fn sound_exists(&self, args: &[Value]) -> gml::Result<Value> {
        let sound = expect_args!(args, [int])?;
        Ok(self.assets.sounds.get_asset(sound).is_some().into())
    }

    pub fn sound_get_name(&self, args: &[Value]) -> gml::Result<Value> {
        let asset_id = expect_args!(args, [int])?;
        Ok(self.assets.sounds.get_asset(asset_id).map(|x| x.name.clone().into()).unwrap_or("<undefined>".into()))
    }

    pub fn sound_get_kind(&self, args: &[Value]) -> gml::Result<Value> {
        let sound_id = expect_args!(args, [int])?;
        Ok(self.assets.sounds.get_asset(sound_id).map(|x| x.gml_kind).unwrap_or(Real::from(-1.0)).into())
    }

    pub fn sound_get_preload(&self, args: &[Value]) -> gml::Result<Value> {
        let sound_id = expect_args!(args, [int])?;
        Ok(self.assets.sounds.get_asset(sound_id).map(|x| x.gml_preload).unwrap_or(Real::from(-1.0)).into())
    }

    pub fn sound_discard(&mut self, args: &[Value]) -> gml::Result<Value> {
        // Dynamically un-preloads a sound, but we preload all sounds, so all we need to do is call sound_stop()
        self.sound_stop(args)
    }

    pub fn sound_restore(&mut self, args: &[Value]) -> gml::Result<Value> {
        let _sound_id = expect_args!(args, [int])?;
        // Dynamically preloads a sound, but we preload all sounds so this does nothing
        Ok(Default::default())
    }

    pub fn sound_add(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (fname, kind, preload) = expect_args!(args, [string, int, bool])?;
        let path_buf = std::path::PathBuf::from(fname.as_ref());
        let data = match std::fs::read(&path_buf) {
            Ok(b) => b.into_boxed_slice(),
            Err(_) => return Ok((-1).into()),
        };
        let sound_id = self.assets.sounds.len() as i32;
        let handle = match path_buf.extension().and_then(std::ffi::OsStr::to_str) {
            Some("mp3") => match self.audio.add_mp3(data, sound_id as i32) {
                Some(x) => asset::sound::FileType::Mp3(x),
                None => return Ok((-1).into()),
            },
            Some("wav") => match self.audio.add_wav(data, sound_id as i32, 1.0, kind == 2, kind >= 3) {
                Some(x) => asset::sound::FileType::Wav(x),
                None => return Ok((-1).into()),
            },
            _ => return Ok((-1).into()),
        };
        self.assets.sounds.push(Some(Box::new(asset::Sound {
            name: format!("__newsound{}", sound_id).into(),
            handle,
            gml_kind: kind.into(),
            gml_preload: f64::from(u8::from(preload)).into(),
        })));
        Ok(sound_id.into())
    }

    pub fn sound_replace(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (sound_id, fname, kind, preload) = expect_args!(args, [int, string, int, bool])?;
        if let Some(sound) = self.assets.sounds.get_asset_mut(sound_id) {
            self.audio.stop_sound(sound_id);
            sound.gml_kind = kind.into();
            sound.gml_preload = f64::from(u8::from(preload)).into();

            if matches!(sound.handle, asset::sound::FileType::None) {
                let path_buf = std::path::PathBuf::from(fname.as_ref());
                let data = match std::fs::read(&path_buf) {
                    Ok(b) => b.into_boxed_slice(),
                    Err(_) => return Ok(0.into()),
                };
                sound.handle = match path_buf.extension().and_then(std::ffi::OsStr::to_str) {
                    Some("mp3") => match self.audio.add_mp3(data, sound_id as i32) {
                        Some(x) => asset::sound::FileType::Mp3(x),
                        None => return Ok(0.into()),
                    },
                    Some("wav") => match self.audio.add_wav(data, sound_id as i32, 1.0, kind == 2, kind >= 3) {
                        Some(x) => asset::sound::FileType::Wav(x),
                        None => return Ok(0.into()),
                    },
                    _ => return Ok(0.into()),
                };
                Ok(1.into())
            } else {
                // This appears to be a GM8 bug, I could never get it to actually load the new sound if
                // the one we're replacing already had a sound loaded. (tested GM 8.1.141)
                sound.handle = asset::sound::FileType::None;
                Ok(1.into())
            }
        } else {
            Err(gml::Error::FunctionError("sound_replace".into(), "Trying to replace non-existing sound.".into()))
        }
    }

    pub fn sound_delete(&mut self, args: &[Value]) -> gml::Result<Value> {
        let sound_id = expect_args!(args, [int])?;
        self.audio.stop_sound(sound_id);
        if self.assets.sounds.get_asset(sound_id).is_some() {
            self.assets.sounds[sound_id as usize] = None;
        }
        Ok(Default::default())
    }

    pub fn font_exists(&self, args: &[Value]) -> gml::Result<Value> {
        let font = expect_args!(args, [int])?;
        Ok(self.assets.fonts.get_asset(font).is_some().into())
    }

    pub fn font_get_name(&self, args: &[Value]) -> gml::Result<Value> {
        let asset_id = expect_args!(args, [int])?;
        Ok(self.assets.fonts.get_asset(asset_id).map(|x| x.name.clone().into()).unwrap_or("<undefined>".into()))
    }

    pub fn font_get_fontname(&self, args: &[Value]) -> gml::Result<Value> {
        let id = expect_args!(args, [int])?;
        Ok(self.assets.fonts.get_asset(id).map(|x| x.sys_name.clone().into()).unwrap_or("".into()))
    }

    pub fn font_get_size(&self, args: &[Value]) -> gml::Result<Value> {
        let id = expect_args!(args, [int])?;
        Ok(self.assets.fonts.get_asset(id).map(|x| x.size.into()).unwrap_or((-1).into()))
    }

    pub fn font_get_bold(&self, args: &[Value]) -> gml::Result<Value> {
        let id = expect_args!(args, [int])?;
        Ok(self.assets.fonts.get_asset(id).map(|x| x.bold.into()).unwrap_or((-1).into()))
    }

    pub fn font_get_italic(&self, args: &[Value]) -> gml::Result<Value> {
        let id = expect_args!(args, [int])?;
        Ok(self.assets.fonts.get_asset(id).map(|x| x.italic.into()).unwrap_or((-1).into()))
    }

    pub fn font_get_first(&self, args: &[Value]) -> gml::Result<Value> {
        let id = expect_args!(args, [int])?;
        Ok(self.assets.fonts.get_asset(id).map(|x| x.first.into()).unwrap_or((-1).into()))
    }

    pub fn font_get_last(&self, args: &[Value]) -> gml::Result<Value> {
        let id = expect_args!(args, [int])?;
        Ok(self.assets.fonts.get_asset(id).map(|x| x.last.into()).unwrap_or((-1).into()))
    }

    pub fn font_add(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 6
        unimplemented!("Called unimplemented kernel function font_add")
    }

    pub fn font_replace(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 7
        unimplemented!("Called unimplemented kernel function font_replace")
    }

    pub fn font_add_sprite(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (sprite_id, first, prop, sep) = expect_args!(args, [int, int, bool, int])?;
        if let Some(sprite) = self.assets.sprites.get_asset(sprite_id) {
            let chars = asset::font::create_chars_from_sprite(sprite, prop, sep, &self.renderer);
            let font_id = self.assets.fonts.len();
            let first = first.clamp(0, 255) as _;
            let last = (first as usize + chars.len() - 1).min(255) as _;
            self.assets.fonts.push(Some(Box::new(asset::Font {
                name: format!("__newfont{}", font_id).into(),
                sys_name: "".into(),
                charset: 1,
                size: 12,
                bold: false,
                italic: false,
                first,
                last,
                tallest_char_height: sprite.height,
                chars,
                own_graphics: false,
            })));
            Ok(font_id.into())
        } else {
            Err(gml::Error::NonexistentAsset(asset::Type::Sprite, sprite_id))
        }
    }

    pub fn font_replace_sprite(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (font_id, sprite_id, first, prop, sep) = expect_args!(args, [int, int, int, bool, int])?;
        if let Some(font) = self.assets.fonts.get_asset_mut(font_id) {
            if let Some(sprite) = self.assets.sprites.get_asset(sprite_id) {
                if font.own_graphics {
                    // font_add isn't in yet but atm for ttfs all characters are on the same texture
                    if let Some(c) = font.get_char(font.first) {
                        self.renderer.delete_sprite(c.atlas_ref);
                    }
                }
                let chars = asset::font::create_chars_from_sprite(sprite, prop, sep, &self.renderer);
                font.sys_name = "".into();
                font.size = 12;
                font.bold = false;
                font.italic = false;
                font.first = first.clamp(0, 255) as _;
                font.last = (first as usize + chars.len() - 1).min(255) as _;
                font.chars = chars;
                font.own_graphics = false;
                Ok(Default::default())
            } else {
                Err(gml::Error::NonexistentAsset(asset::Type::Sprite, sprite_id))
            }
        } else {
            Err(gml::Error::NonexistentAsset(asset::Type::Font, font_id))
        }
    }

    pub fn font_delete(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        unimplemented!("Called unimplemented kernel function font_delete")
    }

    pub fn script_exists(&self, args: &[Value]) -> gml::Result<Value> {
        let script_id = expect_args!(args, [int])?;
        Ok(self.assets.scripts.get_asset(script_id).is_some().into())
    }

    pub fn script_get_name(&self, args: &[Value]) -> gml::Result<Value> {
        let asset_id = expect_args!(args, [int])?;
        Ok(self.assets.scripts.get_asset(asset_id).map(|x| x.name.clone().into()).unwrap_or("<undefined>".into()))
    }

    pub fn script_get_text(&self, args: &[Value]) -> gml::Result<Value> {
        let script_id = expect_args!(args, [int])?;
        Ok(self.assets.scripts.get_asset(script_id).map(|s| s.source.clone().into()).unwrap_or("".into()))
    }

    pub fn script_execute(&mut self, context: &mut Context, args: &[Value]) -> gml::Result<Value> {
        if let Some(script_id) = args.get(0) {
            let script_id = script_id.round();
            if let Some(script) = self.assets.scripts.get_asset(script_id) {
                let instructions = script.compiled.clone();
                let mut new_args: [Value; 16] = Default::default();
                for (src, dest) in args[1..].iter().zip(new_args.iter_mut()) {
                    *dest = src.clone();
                }
                let mut new_context = Context::copy_with_args(context, new_args, args.len() - 1);
                self.execute(&instructions, &mut new_context)?;
                Ok(new_context.return_value)
            } else {
                Err(gml::Error::NonexistentAsset(asset::Type::Script, script_id))
            }
        } else {
            Err(gml::runtime::Error::WrongArgumentCount(1, 0))
        }
    }

    pub fn path_exists(&self, args: &[Value]) -> gml::Result<Value> {
        let path_id = expect_args!(args, [int])?;
        Ok(self.assets.paths.get_asset(path_id).is_some().into())
    }

    pub fn path_get_name(&self, args: &[Value]) -> gml::Result<Value> {
        let asset_id = expect_args!(args, [int])?;
        Ok(self.assets.paths.get_asset(asset_id).map(|x| x.name.clone().into()).unwrap_or("<undefined>".into()))
    }

    pub fn path_get_length(&self, args: &[Value]) -> gml::Result<Value> {
        let path_id = expect_args!(args, [int])?;
        match self.assets.paths.get_asset(path_id) {
            Some(path) => Ok(path.length.into()),
            None => Ok((-1).into()),
        }
    }

    pub fn path_get_kind(&self, args: &[Value]) -> gml::Result<Value> {
        let path_id = expect_args!(args, [int])?;
        match self.assets.paths.get_asset(path_id) {
            Some(path) => Ok(path.curve.into()),
            None => Ok((-1).into()),
        }
    }

    pub fn path_get_closed(&self, args: &[Value]) -> gml::Result<Value> {
        let path_id = expect_args!(args, [int])?;
        match self.assets.paths.get_asset(path_id) {
            Some(path) => Ok(path.closed.into()),
            None => Ok((-1).into()),
        }
    }

    pub fn path_get_precision(&self, args: &[Value]) -> gml::Result<Value> {
        let path_id = expect_args!(args, [int])?;
        match self.assets.paths.get_asset(path_id) {
            Some(path) => Ok(path.precision.into()),
            None => Ok((-1).into()),
        }
    }

    pub fn path_get_number(&self, args: &[Value]) -> gml::Result<Value> {
        let path_id = expect_args!(args, [int])?;
        match self.assets.paths.get_asset(path_id) {
            Some(path) => Ok(path.points.len().into()),
            None => Ok((-1).into()),
        }
    }

    pub fn path_get_point_x(&self, args: &[Value]) -> gml::Result<Value> {
        let (path_id, point_id) = expect_args!(args, [int, int])?;
        match self.assets.paths.get_asset(path_id) {
            Some(path) => {
                if point_id < 0 || point_id >= path.points.len() as i32 {
                    Ok(0.into())
                } else {
                    Ok(path.points.get(point_id as usize).unwrap().x.into())
                }
            },
            None => Ok((-1).into()),
        }
    }

    pub fn path_get_point_y(&self, args: &[Value]) -> gml::Result<Value> {
        let (path_id, point_id) = expect_args!(args, [int, int])?;
        match self.assets.paths.get_asset(path_id) {
            Some(path) => {
                if point_id < 0 || point_id >= path.points.len() as i32 {
                    Ok(0.into())
                } else {
                    Ok(path.points.get(point_id as usize).unwrap().y.into())
                }
            },
            None => Ok((-1).into()),
        }
    }

    pub fn path_get_point_speed(&self, args: &[Value]) -> gml::Result<Value> {
        let (path_id, point_id) = expect_args!(args, [int, int])?;
        match self.assets.paths.get_asset(path_id) {
            Some(path) => {
                if point_id < 0 || point_id >= path.points.len() as i32 {
                    Ok(1.into())
                } else {
                    Ok(path.points.get(point_id as usize).unwrap().speed.into())
                }
            },
            None => Ok((-1).into()),
        }
    }

    pub fn path_get_x(&self, args: &[Value]) -> gml::Result<Value> {
        let (path_id, offset) = expect_args!(args, [int, real])?;
        match self.assets.paths.get_asset(path_id) {
            Some(path) => Ok(path.get_point(offset).x.into()),
            None => Ok((-1).into()),
        }
    }

    pub fn path_get_y(&self, args: &[Value]) -> gml::Result<Value> {
        let (path_id, offset) = expect_args!(args, [int, real])?;
        match self.assets.paths.get_asset(path_id) {
            Some(path) => Ok(path.get_point(offset).y.into()),
            None => Ok((-1).into()),
        }
    }

    pub fn path_get_speed(&self, args: &[Value]) -> gml::Result<Value> {
        let (path_id, offset) = expect_args!(args, [int, real])?;
        match self.assets.paths.get_asset(path_id) {
            Some(path) => Ok(path.get_point(offset).speed.into()),
            None => Ok((-1).into()),
        }
    }

    pub fn path_set_kind(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (path_id, kind) = expect_args!(args, [int, int])?;
        self.assets.paths.get_asset_mut(path_id).map(|path| {
            path.curve = kind == 1;
            path.update();
        });
        Ok(Default::default())
    }

    pub fn path_set_closed(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (path_id, closed) = expect_args!(args, [int, int])?;
        self.assets.paths.get_asset_mut(path_id).map(|path| {
            path.closed = closed != 0;
            path.update();
        });
        Ok(Default::default())
    }

    pub fn path_set_precision(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (path_id, precision) = expect_args!(args, [int, int])?;
        self.assets.paths.get_asset_mut(path_id).map(|path| {
            path.precision = precision.clamp(0, 8);
            path.update();
        });
        Ok(Default::default())
    }

    pub fn path_add(&mut self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        let path_id = self.assets.paths.len();
        self.assets.paths.push(Some(Box::new(asset::Path {
            name: format!("__newpath{}", path_id).into(),
            points: Vec::new(),
            control_nodes: Default::default(),
            length: Default::default(),
            curve: false,
            closed: false,
            precision: 4,
            start: Default::default(),
            end: Default::default(),
        })));
        Ok(path_id.into())
    }

    pub fn path_duplicate(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        unimplemented!("Called unimplemented kernel function path_duplicate")
    }

    pub fn path_assign(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 2
        unimplemented!("Called unimplemented kernel function path_assign")
    }

    pub fn path_append(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 2
        unimplemented!("Called unimplemented kernel function path_append")
    }

    pub fn path_delete(&mut self, args: &[Value]) -> gml::Result<Value> {
        let path_id = expect_args!(args, [int])?;
        if self.assets.paths.get_asset(path_id).is_some() {
            self.assets.paths[path_id as usize] = None;
        }
        Ok(Default::default())
    }

    pub fn path_add_point(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (path_id, x, y, speed) = expect_args!(args, [int, real, real, real])?;
        if let Some(path) = self.assets.paths.get_asset_mut(path_id) {
            path.points.push(asset::path::Point { x, y, speed });
            path.update();
        }
        Ok(Default::default())
    }

    pub fn path_insert_point(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 5
        unimplemented!("Called unimplemented kernel function path_insert_point")
    }

    pub fn path_change_point(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (path_id, n, x, y, speed) = expect_args!(args, [int, int, real, real, real])?;
        if n >= 0 {
            if let Some(path) = self.assets.paths.get_asset_mut(path_id) {
                if let Some(point) = path.points.get_mut(n as usize) {
                    point.x = x;
                    point.y = y;
                    point.speed = speed;
                    path.update();
                }
            }
        }
        Ok(Default::default())
    }

    pub fn path_delete_point(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 2
        unimplemented!("Called unimplemented kernel function path_delete_point")
    }

    pub fn path_clear_points(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        unimplemented!("Called unimplemented kernel function path_clear_points")
    }

    pub fn path_reverse(&mut self, args: &[Value]) -> gml::Result<Value> {
        let id = expect_args!(args, [int])?;
        if let Some(path) = self.assets.paths.get_asset_mut(id) {
            path.points.reverse();
            path.update();
        }
        Ok(Default::default())
    }

    pub fn path_mirror(&mut self, args: &[Value]) -> gml::Result<Value> {
        let id = expect_args!(args, [int])?;
        if let Some(path) = self.assets.paths.get_asset_mut(id) {
            let (xcenter, _) = path.center();
            for path in &mut path.points {
                path.x = xcenter - (path.x - xcenter);
            }
            path.update();
        }
        Ok(Default::default())
    }

    pub fn path_flip(&mut self, args: &[Value]) -> gml::Result<Value> {
        let id = expect_args!(args, [int])?;
        if let Some(path) = self.assets.paths.get_asset_mut(id) {
            let (_, ycenter) = path.center();
            for path in &mut path.points {
                path.y = ycenter - (path.y - ycenter);
            }
            path.update();
        }
        Ok(Default::default())
    }

    pub fn path_rotate(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (id, angle) = expect_args!(args, [int, real])?;
        let sin = -angle.to_radians().sin().into_inner();
        let cos = angle.to_radians().cos().into_inner();
        if let Some(path) = self.assets.paths.get_asset_mut(id) {
            let (xcenter, ycenter) = path.center();
            for point in &mut path.points {
                crate::util::rotate_around(
                    point.x.as_mut_ref(),
                    point.y.as_mut_ref(),
                    xcenter.into(),
                    ycenter.into(),
                    sin,
                    cos,
                );
            }
            path.update();
        }
        Ok(Default::default())
    }

    pub fn path_scale(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (id, xscale, yscale) = expect_args!(args, [int, real, real])?;
        if let Some(path) = self.assets.paths.get_asset_mut(id) {
            let (xcenter, ycenter) = path.center();
            for path in &mut path.points {
                path.x = xcenter + xscale * (path.x - xcenter);
                path.y = ycenter + yscale * (path.y - ycenter);
            }
            path.update();
        }
        Ok(Default::default())
    }

    pub fn path_shift(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (id, xshift, yshift) = expect_args!(args, [int, real, real])?;
        if let Some(path) = self.assets.paths.get_asset_mut(id) {
            for path in &mut path.points {
                path.x += xshift;
                path.y += yshift;
            }
            path.update();
        }
        Ok(Default::default())
    }

    pub fn timeline_exists(&self, args: &[Value]) -> gml::Result<Value> {
        let timeline = expect_args!(args, [int])?;
        Ok(self.assets.timelines.get_asset(timeline).is_some().into())
    }

    pub fn timeline_get_name(&self, args: &[Value]) -> gml::Result<Value> {
        let asset_id = expect_args!(args, [int])?;
        Ok(self.assets.timelines.get_asset(asset_id).map(|x| x.name.clone().into()).unwrap_or("<undefined>".into()))
    }

    pub fn timeline_add(&mut self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        let idx = self.assets.timelines.len();
        self.assets.timelines.push(Some(Box::new(asset::Timeline {
            name: format!("__newtimeline{}", idx).into(),
            moments: Default::default(),
        })));

        Ok(idx.into())
    }

    pub fn timeline_delete(&mut self, args: &[Value]) -> gml::Result<Value> {
        let timeline = expect_args!(args, [int])?;
        if self.assets.timelines.get_asset(timeline).is_some() {
            self.assets.timelines[timeline as usize] = None;
            Ok(Default::default())
        } else {
            Err(gml::Error::FunctionError("timeline_delete".into(), "Trying to delete non-existing timeline".into()))
        }
    }

    pub fn timeline_clear(&mut self, args: &[Value]) -> gml::Result<Value> {
        let timeline = expect_args!(args, [int])?;
        if let Some(timeline) = self.assets.timelines.get_asset_mut(timeline) {
            // Instead of timeline.moments.borrow().clear(), which could panic if this is called from
            // within a timeline step, just drop the old list and create a new one
            timeline.moments = Default::default();
            Ok(Default::default())
        } else {
            Err(gml::Error::NonexistentAsset(asset::Type::Timeline, timeline))
        }
    }

    pub fn timeline_moment_clear(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (timeline, moment) = expect_args!(args, [int, int])?;
        if let Some(timeline) = self.assets.timelines.get_asset(timeline) {
            timeline.moments.borrow_mut().remove(&moment);
        }
        Ok(Default::default())
    }

    pub fn timeline_moment_add(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (timeline, moment, code) = expect_args!(args, [int, int, bytes])?;
        // Note: GM8 does not attempt to compile the string if the timeline doesn't exist
        if let Some(timeline) = self.assets.timelines.get_asset(timeline) {
            let instrs = self
                .compiler
                .compile(code.as_ref())
                .map_err(|e| gml::Error::FunctionError("timeline_moment_add".into(), e.message))?;

            timeline.moments.borrow_mut().entry(moment).or_insert(Default::default()).borrow_mut().push_code(instrs);
        }
        Ok(Default::default())
    }

    pub fn object_exists(&self, args: &[Value]) -> gml::Result<Value> {
        let object = expect_args!(args, [int])?;
        Ok(self.assets.objects.get_asset(object).is_some().into())
    }

    pub fn object_get_name(&self, args: &[Value]) -> gml::Result<Value> {
        let asset_id = expect_args!(args, [int])?;
        Ok(self.assets.objects.get_asset(asset_id).map(|x| x.name.clone().into()).unwrap_or("<undefined>".into()))
    }

    pub fn object_get_sprite(&self, args: &[Value]) -> gml::Result<Value> {
        let object_id = expect_args!(args, [int])?;
        if let Some(Some(object)) = self.assets.objects.get(object_id as usize) {
            Ok(object.sprite_index.into())
        } else {
            Ok((-1).into())
        }
    }

    pub fn object_get_solid(&self, args: &[Value]) -> gml::Result<Value> {
        let object_id = expect_args!(args, [int])?;
        if let Some(Some(object)) = self.assets.objects.get(object_id as usize) {
            Ok(object.solid.into())
        } else {
            Ok((-1).into())
        }
    }

    pub fn object_get_visible(&self, args: &[Value]) -> gml::Result<Value> {
        let object_id = expect_args!(args, [int])?;
        if let Some(Some(object)) = self.assets.objects.get(object_id as usize) {
            Ok(object.visible.into())
        } else {
            Ok((-1).into())
        }
    }

    pub fn object_get_depth(&self, args: &[Value]) -> gml::Result<Value> {
        let object_id = expect_args!(args, [int])?;
        if let Some(Some(object)) = self.assets.objects.get(object_id as usize) {
            Ok(object.depth.into())
        } else {
            Ok((-1).into())
        }
    }

    pub fn object_get_persistent(&self, args: &[Value]) -> gml::Result<Value> {
        let object_id = expect_args!(args, [int])?;
        if let Some(Some(object)) = self.assets.objects.get(object_id as usize) {
            Ok(object.persistent.into())
        } else {
            Ok((-1).into())
        }
    }

    pub fn object_get_mask(&self, args: &[Value]) -> gml::Result<Value> {
        let object_id = expect_args!(args, [int])?;
        if let Some(Some(object)) = self.assets.objects.get(object_id as usize) {
            Ok(object.mask_index.into())
        } else {
            Ok((-1).into())
        }
    }

    pub fn object_get_parent(&self, args: &[Value]) -> gml::Result<Value> {
        let object_id = expect_args!(args, [int])?;
        if let Some(Some(object)) = self.assets.objects.get(object_id as usize) {
            Ok(object.parent_index.into())
        } else {
            Ok((-1).into())
        }
    }

    pub fn object_is_ancestor(&self, args: &[Value]) -> gml::Result<Value> {
        let (child_id, parent_id) = expect_args!(args, [int, int])?;
        if child_id != parent_id {
            if let Some(parent) = self.assets.objects.get_asset(parent_id) {
                return Ok(parent.children.borrow().contains(&child_id).into())
            }
        }
        Ok(false.into())
    }

    pub fn object_set_sprite(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (object_id, sprite_id) = expect_args!(args, [int, int])?;
        self.assets.objects.get_asset_mut(object_id).map(|o| o.sprite_index = sprite_id);
        Ok(Default::default())
    }

    pub fn object_set_solid(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (object_id, visible) = expect_args!(args, [int, bool])?;
        self.assets.objects.get_asset_mut(object_id).map(|o| o.visible = visible);
        Ok(Default::default())
    }

    pub fn object_set_visible(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (object_id, visible) = expect_args!(args, [int, bool])?;
        self.assets.objects.get_asset_mut(object_id).map(|o| o.visible = visible);
        Ok(Default::default())
    }

    pub fn object_set_depth(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (object_id, depth) = expect_args!(args, [int, int])?;
        self.assets.objects.get_asset_mut(object_id).map(|o| o.depth = depth);
        Ok(Default::default())
    }

    pub fn object_set_persistent(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (object_id, persistent) = expect_args!(args, [int, bool])?;
        self.assets.objects.get_asset_mut(object_id).map(|o| o.persistent = persistent);
        Ok(Default::default())
    }

    pub fn object_set_mask(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (object_id, mask_id) = expect_args!(args, [int, int])?;
        self.assets.objects.get_asset_mut(object_id).map(|o| o.mask_index = mask_id);
        Ok(Default::default())
    }

    pub fn object_set_parent(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (object_id, new_parent) = expect_args!(args, [int, int])?;
        if let Some(object) = self.assets.objects.get_asset(object_id) {
            let parents = object.parents.borrow();
            let children = object.children.borrow();
            // Remove object and its parents from all its children
            for &child_id in children.iter().filter(|&&id| id != object_id) {
                if let Some(child) = self.assets.objects.get_asset(child_id) {
                    child.parents.borrow_mut().retain(|p| !parents.contains(p));
                }
            }
            drop(parents);
            // Remove object and all its children from old parents
            let mut parent_index = object.parent_index;
            while let Some(parent) = self.assets.objects.get_asset(parent_index) {
                parent.children.borrow_mut().retain(|c| !children.contains(c));
                parent_index = parent.parent_index;
            }
            // Calculate new parents
            let mut new_parents = self
                .assets
                .objects
                .get_asset(new_parent)
                .map(|o| o.parents.as_ref().borrow().clone())
                .unwrap_or_default();
            new_parents.insert(object_id);
            // Add object and all its new parents to children
            for &child_id in children.iter().filter(|&&id| id != object_id) {
                if let Some(child) = self.assets.objects.get_asset(child_id) {
                    child.parents.borrow_mut().extend(&new_parents);
                }
            }
            self.assets.objects.get_asset(object_id).map(|o| *o.parents.borrow_mut() = new_parents);
            // Add object and all its children to new parents
            parent_index = new_parent;
            while let Some(parent) = self.assets.objects.get_asset(parent_index) {
                parent.children.borrow_mut().extend(children.iter());
                parent_index = parent.parent_index;
            }
        }

        self.assets.objects.get_asset_mut(object_id).map(|o| o.parent_index = new_parent);
        self.refresh_event_holders();
        Ok(Default::default())
    }

    pub fn object_add(&mut self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        let id = self.assets.objects.len() as i32;
        let children = Default::default();
        let object = Box::new(asset::Object {
            name: format!("__newobject{}", id).into(),
            solid: false,
            visible: true,
            persistent: false,
            depth: 0,
            sprite_index: -1,
            mask_index: -1,
            parent_index: -1,
            events: Default::default(),
            children,
            parents: Default::default(),
        });
        object.children.borrow_mut().insert(id);
        object.parents.borrow_mut().insert(id);
        self.assets.objects.push(Some(object));
        Ok(id.into())
    }

    pub fn object_delete(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        unimplemented!("Called unimplemented kernel function object_delete")
    }

    pub fn object_event_clear(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (object_index, ev_type, ev_number) = expect_args!(args, [int, int, int])?;
        if let Some(object) = self.assets.objects.get_asset_mut(object_index) {
            object.events[ev_type as usize].remove(&(ev_number as u32));
            self.refresh_event_holders();
        }
        Ok(Default::default())
    }

    pub fn object_event_add(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (object_index, ev_type, ev_number, code) = expect_args!(args, [int, int, int, bytes])?;
        if let Some(object) = self.assets.objects.get_asset_mut(object_index) {
            let instrs = match self.compiler.compile(code.as_ref()) {
                Ok(instrs) => instrs,
                Err(e) => return Err(gml::Error::FunctionError("object_event_add".into(), e.message)),
            };
            let object_event_map = &mut object.events[ev_type as usize];
            match object_event_map.get_mut(&(ev_number as u32)) {
                Some(tree) => {
                    tree.borrow_mut().push_code(instrs);
                },
                None => {
                    object_event_map.insert(ev_number as u32, action::Tree::new_from_code(instrs));
                    self.refresh_event_holders();
                },
            }
        }
        Ok(Default::default())
    }

    pub fn room_exists(&self, args: &[Value]) -> gml::Result<Value> {
        let room_id = expect_args!(args, [int])?;
        Ok(self.assets.rooms.get_asset(room_id).is_some().into())
    }

    pub fn room_get_name(&self, args: &[Value]) -> gml::Result<Value> {
        let asset_id = expect_args!(args, [int])?;
        Ok(self.assets.rooms.get_asset(asset_id).map(|x| x.name.clone().into()).unwrap_or("<undefined>".into()))
    }

    pub fn room_set_width(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (room_id, width) = expect_args!(args, [int, int])?;
        if let Some(room) = self.assets.rooms.get_asset_mut(room_id) {
            room.width = width as _;
        }
        Ok(Default::default())
    }

    pub fn room_set_height(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (room_id, height) = expect_args!(args, [int, int])?;
        if let Some(room) = self.assets.rooms.get_asset_mut(room_id) {
            room.height = height as _;
        }
        Ok(Default::default())
    }

    pub fn room_set_caption(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (room_id, caption) = expect_args!(args, [int, bytes])?;
        if let Some(room) = self.assets.rooms.get_asset_mut(room_id) {
            room.caption = caption;
        }
        Ok(Default::default())
    }

    pub fn room_set_persistent(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (room_id, persistent) = expect_args!(args, [int, bool])?;
        if let Some(room) = self.assets.rooms.get_asset_mut(room_id) {
            room.persistent = persistent;
        }
        Ok(Default::default())
    }

    pub fn room_set_code(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 2
        unimplemented!("Called unimplemented kernel function room_set_code")
    }

    pub fn room_set_background_color(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (room_id, colour, show) = expect_args!(args, [int, int, bool])?;
        if let Some(room) = self.assets.rooms.get_asset_mut(room_id) {
            room.bg_colour = (colour as u32).into();
            room.clear_screen = show;
        }
        Ok(Default::default())
    }

    pub fn room_set_background(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (room_id, bg, visible, is_foreground, bg_id, x, y, htiled, vtiled, hspeed, vspeed, alpha) =
            expect_args!(args, [int, int, bool, bool, int, real, real, bool, bool, real, real, real])?;
        if bg >= 0 {
            if let Some(room) = self.assets.rooms.get_asset_mut(room_id) {
                if let Some(bg) = room.backgrounds.get_mut(bg as usize) {
                    bg.visible = visible;
                    bg.is_foreground = is_foreground;
                    bg.background_id = bg_id;
                    bg.x_offset = x;
                    bg.y_offset = y;
                    bg.tile_horizontal = htiled;
                    bg.tile_vertical = vtiled;
                    bg.hspeed = hspeed;
                    bg.vspeed = vspeed;
                    bg.alpha = alpha;
                }
            }
        }
        Ok(Default::default())
    }

    pub fn room_set_view(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (
            room_id,
            view_id,
            visible,
            source_x,
            source_y,
            source_w,
            source_h,
            port_x,
            port_y,
            port_w,
            port_h,
            follow_hborder,
            follow_vborder,
            follow_hspeed,
            follow_vspeed,
            follow_target,
        ) = expect_args!(args, [int, int, bool, int, int, int, int, int, int, int, int, int, int, int, int, int])?;
        let view_id = if view_id >= 0 { view_id as usize } else { return Ok(Default::default()) };
        if let Some(room) = self.assets.rooms.get_asset_mut(room_id) {
            if let Some(view) = room.views.get_mut(view_id) {
                *view = View {
                    visible,
                    source_x,
                    source_y,
                    source_w,
                    source_h,
                    port_x,
                    port_y,
                    port_w: port_w as _,
                    port_h: port_h as _,
                    follow_hborder,
                    follow_vborder,
                    follow_hspeed,
                    follow_vspeed,
                    follow_target,
                    angle: view.angle,
                };
            }
        }
        Ok(Default::default())
    }

    pub fn room_set_view_enabled(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (room_id, enabled) = expect_args!(args, [int, bool])?;
        if let Some(room) = self.assets.rooms.get_asset_mut(room_id) {
            room.views_enabled = enabled;
        }
        Ok(Default::default())
    }

    pub fn room_add(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 0
        unimplemented!("Called unimplemented kernel function room_add")
    }

    pub fn room_duplicate(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        unimplemented!("Called unimplemented kernel function room_duplicate")
    }

    pub fn room_assign(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 2
        unimplemented!("Called unimplemented kernel function room_assign")
    }

    pub fn room_instance_add(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (room_id, x, y, object) = expect_args!(args, [int, int, int, int])?;
        self.last_instance_id += 1;
        if let Some(room) = self.assets.rooms.get_asset_mut(room_id) {
            room.instances.push(asset::room::Instance {
                x,
                y,
                object,
                id: self.last_instance_id,
                creation: Ok(std::rc::Rc::new([])),
                xscale: 1.0,
                yscale: 1.0,
                blend: u32::MAX,
                angle: 0.0,
            });
            Ok(Default::default())
        } else {
            Err(gml::Error::NonexistentAsset(asset::Type::Room, room_id))
        }
    }

    pub fn room_instance_clear(&mut self, args: &[Value]) -> gml::Result<Value> {
        let room_id = expect_args!(args, [int])?;
        if let Some(room) = self.assets.rooms.get_asset_mut(room_id) {
            room.instances.clear();
            Ok(Default::default())
        } else {
            Err(gml::Error::NonexistentAsset(asset::Type::Room, room_id))
        }
    }

    pub fn room_tile_add(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 9
        unimplemented!("Called unimplemented kernel function room_tile_add")
    }

    pub fn room_tile_add_ext(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 12
        unimplemented!("Called unimplemented kernel function room_tile_add_ext")
    }

    pub fn room_tile_clear(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        unimplemented!("Called unimplemented kernel function room_tile_clear")
    }

    pub fn part_type_create(&mut self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        Ok(self.particles.create_type().into())
    }

    pub fn part_type_destroy(&mut self, args: &[Value]) -> gml::Result<Value> {
        let id = expect_args!(args, [int])?;
        self.particles.destroy_type(id);
        Ok(Default::default())
    }

    pub fn part_type_exists(&self, args: &[Value]) -> gml::Result<Value> {
        let id = expect_args!(args, [int])?;
        Ok(self.particles.get_type(id).is_some().into())
    }

    pub fn part_type_clear(&mut self, args: &[Value]) -> gml::Result<Value> {
        let id = expect_args!(args, [int])?;
        if let Some(pt) = self.particles.get_type_mut(id) {
            *pt = Box::new(particle::ParticleType::new());
        }
        Ok(Default::default())
    }

    pub fn part_type_shape(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (id, shape) = expect_args!(args, [int, int])?;
        if let Some(pt) = self.particles.get_type_mut(id) {
            pt.graphic = particle::ParticleGraphic::Shape(shape);
        }
        Ok(Default::default())
    }

    pub fn part_type_sprite(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (id, sprite, animat, stretch, random) = expect_args!(args, [int, int, bool, bool, bool])?;
        if let Some(pt) = self.particles.get_type_mut(id) {
            pt.graphic = particle::ParticleGraphic::Sprite { sprite, animat, stretch, random };
        }
        Ok(Default::default())
    }

    pub fn part_type_size(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (id, size_min, size_max, size_incr, size_wiggle) = expect_args!(args, [int, real, real, real, real])?;
        if let Some(pt) = self.particles.get_type_mut(id) {
            pt.size_min = size_min;
            pt.size_max = size_max;
            pt.size_incr = size_incr;
            pt.size_wiggle = size_wiggle;
        }
        Ok(Default::default())
    }

    pub fn part_type_scale(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (id, xscale, yscale) = expect_args!(args, [int, real, real])?;
        if let Some(pt) = self.particles.get_type_mut(id) {
            pt.xscale = xscale;
            pt.yscale = yscale;
        }
        Ok(Default::default())
    }

    pub fn part_type_life(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (id, life_min, life_max) = expect_args!(args, [int, int, int])?;
        if let Some(pt) = self.particles.get_type_mut(id) {
            pt.life_min = life_min;
            pt.life_max = life_max;
        }
        Ok(Default::default())
    }

    pub fn part_type_step(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (id, step_number, step_type) = expect_args!(args, [int, int, int])?;
        if let Some(pt) = self.particles.get_type_mut(id) {
            pt.step_number = step_number;
            pt.step_type = step_type;
        }
        Ok(Default::default())
    }

    pub fn part_type_death(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (id, death_number, death_type) = expect_args!(args, [int, int, int])?;
        if let Some(pt) = self.particles.get_type_mut(id) {
            pt.death_number = death_number;
            pt.death_type = death_type;
        }
        Ok(Default::default())
    }

    pub fn part_type_speed(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (id, speed_min, speed_max, speed_incr, speed_wiggle) = expect_args!(args, [int, real, real, real, real])?;
        if let Some(pt) = self.particles.get_type_mut(id) {
            pt.speed_min = speed_min;
            pt.speed_max = speed_max;
            pt.speed_incr = speed_incr;
            pt.speed_wiggle = speed_wiggle;
        }
        Ok(Default::default())
    }

    pub fn part_type_direction(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (id, dir_min, dir_max, dir_incr, dir_wiggle) = expect_args!(args, [int, real, real, real, real])?;
        if let Some(pt) = self.particles.get_type_mut(id) {
            pt.dir_min = dir_min;
            pt.dir_max = dir_max;
            pt.dir_incr = dir_incr;
            pt.dir_wiggle = dir_wiggle;
        }
        Ok(Default::default())
    }

    pub fn part_type_orientation(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (id, ang_min, ang_max, ang_incr, ang_wiggle, ang_relative) =
            expect_args!(args, [int, real, real, real, real, bool])?;
        if let Some(pt) = self.particles.get_type_mut(id) {
            pt.ang_min = ang_min;
            pt.ang_max = ang_max;
            pt.ang_incr = ang_incr;
            pt.ang_wiggle = ang_wiggle;
            pt.ang_relative = ang_relative;
        }
        Ok(Default::default())
    }

    pub fn part_type_gravity(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (id, grav_amount, grav_dir) = expect_args!(args, [int, real, real])?;
        if let Some(pt) = self.particles.get_type_mut(id) {
            pt.grav_amount = grav_amount;
            pt.grav_dir = grav_dir.rem_euclid(Real::from(360.0));
        }
        Ok(Default::default())
    }

    pub fn part_type_color_mix(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (id, c1, c2) = expect_args!(args, [int, int, int])?;
        if let Some(pt) = self.particles.get_type_mut(id) {
            pt.colour = particle::ParticleColour::Mix(c1, c2);
        }
        Ok(Default::default())
    }

    pub fn part_type_color_rgb(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (id, rmin, rmax, gmin, gmax, bmin, bmax) = expect_args!(args, [int, int, int, int, int, int, int])?;
        if let Some(pt) = self.particles.get_type_mut(id) {
            pt.colour = particle::ParticleColour::RGB { rmin, rmax, gmin, gmax, bmin, bmax };
        }
        Ok(Default::default())
    }

    pub fn part_type_color_hsv(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (id, hmin, hmax, smin, smax, vmin, vmax) = expect_args!(args, [int, int, int, int, int, int, int])?;
        if let Some(pt) = self.particles.get_type_mut(id) {
            pt.colour = particle::ParticleColour::HSV { hmin, hmax, smin, smax, vmin, vmax };
        }
        Ok(Default::default())
    }

    pub fn part_type_color1(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (id, col) = expect_args!(args, [int, int])?;
        if let Some(pt) = self.particles.get_type_mut(id) {
            pt.colour = particle::ParticleColour::One(col);
        }
        Ok(Default::default())
    }

    pub fn part_type_color2(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (id, c1, c2) = expect_args!(args, [int, int, int])?;
        if let Some(pt) = self.particles.get_type_mut(id) {
            pt.colour = particle::ParticleColour::Two(c1, c2);
        }
        Ok(Default::default())
    }

    pub fn part_type_color3(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (id, c1, c2, c3) = expect_args!(args, [int, int, int, int])?;
        if let Some(pt) = self.particles.get_type_mut(id) {
            pt.colour = particle::ParticleColour::Three(c1, c2, c3);
        }
        Ok(Default::default())
    }

    pub fn part_type_alpha1(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (id, alpha) = expect_args!(args, [int, real])?;
        if let Some(pt) = self.particles.get_type_mut(id) {
            pt.alpha1 = alpha;
            pt.alpha2 = alpha;
            pt.alpha3 = alpha;
        }
        Ok(Default::default())
    }

    pub fn part_type_alpha2(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (id, alpha1, alpha2) = expect_args!(args, [int, real, real])?;
        if let Some(pt) = self.particles.get_type_mut(id) {
            pt.alpha1 = alpha1;
            pt.alpha2 = (alpha1 + alpha2) / Real::from(2.0);
            pt.alpha3 = alpha2;
        }
        Ok(Default::default())
    }

    pub fn part_type_alpha3(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (id, alpha1, alpha2, alpha3) = expect_args!(args, [int, real, real, real])?;
        if let Some(pt) = self.particles.get_type_mut(id) {
            pt.alpha1 = alpha1;
            pt.alpha2 = alpha2;
            pt.alpha3 = alpha3;
        }
        Ok(Default::default())
    }

    pub fn part_type_blend(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (id, additive) = expect_args!(args, [int, bool])?;
        if let Some(pt) = self.particles.get_type_mut(id) {
            pt.additive_blending = additive;
        }
        Ok(Default::default())
    }

    pub fn part_system_create(&mut self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        Ok(self.particles.create_system().into())
    }

    pub fn part_system_destroy(&mut self, args: &[Value]) -> gml::Result<Value> {
        let id = expect_args!(args, [int])?;
        self.particles.destroy_system(id);
        Ok(Default::default())
    }

    pub fn part_system_exists(&self, args: &[Value]) -> gml::Result<Value> {
        let id = expect_args!(args, [int])?;
        Ok(self.particles.get_system(id).is_some().into())
    }

    pub fn part_system_clear(&mut self, args: &[Value]) -> gml::Result<Value> {
        let id = expect_args!(args, [int])?;
        if let Some(ps) = self.particles.get_system_mut(id) {
            *ps = Box::new(particle::System::new());
        }
        Ok(Default::default())
    }

    pub fn part_system_draw_order(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (id, oldtonew) = expect_args!(args, [int, bool])?;
        if let Some(ps) = self.particles.get_system_mut(id) {
            ps.draw_old_to_new = oldtonew;
        }
        Ok(Default::default())
    }

    pub fn part_system_depth(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (id, depth) = expect_args!(args, [int, real])?;
        if let Some(ps) = self.particles.get_system_mut(id) {
            ps.depth = depth;
        }
        Ok(Default::default())
    }

    pub fn part_system_position(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (id, x, y) = expect_args!(args, [int, real, real])?;
        if let Some(ps) = self.particles.get_system_mut(id) {
            ps.x = x;
            ps.y = y;
        }
        Ok(Default::default())
    }

    pub fn part_system_automatic_update(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (id, automatic) = expect_args!(args, [int, bool])?;
        if let Some(ps) = self.particles.get_system_mut(id) {
            ps.auto_update = automatic;
        }
        Ok(Default::default())
    }

    pub fn part_system_automatic_draw(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (id, automatic) = expect_args!(args, [int, bool])?;
        if let Some(ps) = self.particles.get_system_mut(id) {
            ps.auto_draw = automatic;
        }
        Ok(Default::default())
    }

    pub fn part_system_update(&mut self, args: &[Value]) -> gml::Result<Value> {
        let id = expect_args!(args, [int])?;
        self.particles.update_system(id, &mut self.rand);
        Ok(Default::default())
    }

    pub fn part_system_drawit(&mut self, args: &[Value]) -> gml::Result<Value> {
        let id = expect_args!(args, [int])?;
        self.particles.draw_system(id, &mut self.renderer, &self.assets, false);
        Ok(Default::default())
    }

    pub fn part_particles_create(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (id, x, y, parttype, number) = expect_args!(args, [int, real, real, int, int])?;
        self.particles.system_create_particles(id, x, y, parttype, None, number, &mut self.rand);
        Ok(Default::default())
    }

    pub fn part_particles_create_color(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (id, x, y, parttype, colour, number) = expect_args!(args, [int, real, real, int, int, int])?;
        self.particles.system_create_particles(id, x, y, parttype, Some(colour), number, &mut self.rand);
        Ok(Default::default())
    }

    pub fn part_particles_clear(&mut self, args: &[Value]) -> gml::Result<Value> {
        let id = expect_args!(args, [int])?;
        if let Some(ps) = self.particles.get_system_mut(id) {
            ps.particles.clear();
        }
        Ok(Default::default())
    }

    pub fn part_particles_count(&self, args: &[Value]) -> gml::Result<Value> {
        let id = expect_args!(args, [int])?;
        if let Some(ps) = self.particles.get_system(id) {
            Ok(ps.particles.len().into())
        } else {
            Ok(Default::default())
        }
    }

    pub fn part_emitter_create(&mut self, args: &[Value]) -> gml::Result<Value> {
        let id = expect_args!(args, [int])?;
        if let Some(ps) = self.particles.get_system_mut(id) {
            let em = particle::Emitter::new();
            if let Some(id) = ps.emitters.iter().position(|x| x.is_none()) {
                ps.emitters[id] = Some(em);
                Ok(id.into())
            } else {
                ps.emitters.push(Some(em));
                Ok((ps.emitters.len() - 1).into())
            }
        } else {
            Ok((-1).into())
        }
    }

    pub fn part_emitter_destroy(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (psid, id) = expect_args!(args, [int, int])?;
        if let Some(ps) = self.particles.get_system_mut(psid) {
            if ps.emitters.get_asset(id).is_some() {
                ps.emitters[id as usize] = None;
            }
        }
        Ok(Default::default())
    }

    pub fn part_emitter_destroy_all(&mut self, args: &[Value]) -> gml::Result<Value> {
        let psid = expect_args!(args, [int])?;
        if let Some(ps) = self.particles.get_system_mut(psid) {
            ps.emitters.clear();
        }
        Ok(Default::default())
    }

    pub fn part_emitter_exists(&self, args: &[Value]) -> gml::Result<Value> {
        let (psid, id) = expect_args!(args, [int, int])?;
        if let Some(ps) = self.particles.get_system(psid) {
            Ok(ps.emitters.get_asset(id).is_some().into())
        } else {
            Ok(gml::FALSE.into())
        }
    }

    pub fn part_emitter_clear(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (psid, id) = expect_args!(args, [int, int])?;
        if let Some(ps) = self.particles.get_system_mut(psid) {
            if let Some(em) = ps.emitters.get_asset_mut(id) {
                *em = particle::Emitter::new();
            }
        }
        Ok(Default::default())
    }

    pub fn part_emitter_region(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (psid, id, xmin, xmax, ymin, ymax, shape, distr) =
            expect_args!(args, [int, int, real, real, real, real, int, int])?;
        let shape = match shape {
            1 => particle::Shape::Ellipse,
            2 => particle::Shape::Diamond,
            3 => particle::Shape::Line,
            _ => particle::Shape::Rectangle,
        };
        let distr = match distr {
            1 => particle::Distribution::Gaussian,
            2 => particle::Distribution::InvGaussian,
            _ => particle::Distribution::Linear,
        };
        if let Some(ps) = self.particles.get_system_mut(psid) {
            if let Some(em) = ps.emitters.get_asset_mut(id) {
                em.xmin = xmin;
                em.xmax = xmax;
                em.ymin = ymin;
                em.ymax = ymax;
                em.shape = shape;
                em.distribution = distr;
            }
        }
        Ok(Default::default())
    }

    pub fn part_emitter_burst(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (psid, id, parttype, number) = expect_args!(args, [int, int, int, int])?;
        self.particles.emitter_burst(psid, id, parttype, number, &mut self.rand);
        Ok(Default::default())
    }

    pub fn part_emitter_stream(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (psid, id, parttype, number) = expect_args!(args, [int, int, int, int])?;
        if let Some(ps) = self.particles.get_system_mut(psid) {
            if let Some(em) = ps.emitters.get_asset_mut(id) {
                em.ptype = parttype;
                em.number = number;
            }
        }
        Ok(Default::default())
    }

    pub fn part_attractor_create(&mut self, args: &[Value]) -> gml::Result<Value> {
        let id = expect_args!(args, [int])?;
        if let Some(ps) = self.particles.get_system_mut(id) {
            let at = particle::Attractor::new();
            if let Some(id) = ps.attractors.iter().position(|x| x.is_none()) {
                ps.attractors[id] = Some(at);
                Ok(id.into())
            } else {
                ps.attractors.push(Some(at));
                Ok((ps.attractors.len() - 1).into())
            }
        } else {
            Ok((-1).into())
        }
    }

    pub fn part_attractor_destroy(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (psid, id) = expect_args!(args, [int, int])?;
        if let Some(ps) = self.particles.get_system_mut(psid) {
            if ps.attractors.get_asset(id).is_some() {
                ps.attractors[id as usize] = None;
            }
        }
        Ok(Default::default())
    }

    pub fn part_attractor_destroy_all(&mut self, args: &[Value]) -> gml::Result<Value> {
        let psid = expect_args!(args, [int])?;
        if let Some(ps) = self.particles.get_system_mut(psid) {
            ps.attractors.clear();
        }
        Ok(Default::default())
    }

    pub fn part_attractor_exists(&self, args: &[Value]) -> gml::Result<Value> {
        let (psid, id) = expect_args!(args, [int, int])?;
        if let Some(ps) = self.particles.get_system(psid) {
            Ok(ps.attractors.get_asset(id).is_some().into())
        } else {
            Ok(gml::FALSE.into())
        }
    }

    pub fn part_attractor_clear(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (psid, id) = expect_args!(args, [int, int])?;
        if let Some(ps) = self.particles.get_system_mut(psid) {
            if let Some(at) = ps.attractors.get_asset_mut(id) {
                *at = particle::Attractor::new();
            }
        }
        Ok(Default::default())
    }

    pub fn part_attractor_position(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (psid, id, x, y) = expect_args!(args, [int, int, real, real])?;
        if let Some(ps) = self.particles.get_system_mut(psid) {
            if let Some(at) = ps.attractors.get_asset_mut(id) {
                at.x = x;
                at.y = y;
            }
        }
        Ok(Default::default())
    }

    pub fn part_attractor_force(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (psid, id, force, dist, kind, additive) = expect_args!(args, [int, int, real, real, int, bool])?;
        if let Some(ps) = self.particles.get_system_mut(psid) {
            if let Some(at) = ps.attractors.get_asset_mut(id) {
                at.force = force;
                at.dist = dist;
                at.kind = match kind {
                    1 => particle::ForceKind::Linear,
                    2 => particle::ForceKind::Quadratic,
                    _ => particle::ForceKind::Constant,
                };
                at.additive = additive;
            }
        }
        Ok(Default::default())
    }

    pub fn part_destroyer_create(&mut self, args: &[Value]) -> gml::Result<Value> {
        let id = expect_args!(args, [int])?;
        if let Some(ps) = self.particles.get_system_mut(id) {
            let de = particle::Destroyer::new();
            if let Some(id) = ps.destroyers.iter().position(|x| x.is_none()) {
                ps.destroyers[id] = Some(de);
                Ok(id.into())
            } else {
                ps.destroyers.push(Some(de));
                Ok((ps.destroyers.len() - 1).into())
            }
        } else {
            Ok((-1).into())
        }
    }

    pub fn part_destroyer_destroy(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (psid, id) = expect_args!(args, [int, int])?;
        if let Some(ps) = self.particles.get_system_mut(psid) {
            if ps.destroyers.get_asset(id).is_some() {
                ps.destroyers[id as usize] = None;
            }
        }
        Ok(Default::default())
    }

    pub fn part_destroyer_destroy_all(&mut self, args: &[Value]) -> gml::Result<Value> {
        let psid = expect_args!(args, [int])?;
        if let Some(ps) = self.particles.get_system_mut(psid) {
            ps.destroyers.clear();
        }
        Ok(Default::default())
    }

    pub fn part_destroyer_exists(&self, args: &[Value]) -> gml::Result<Value> {
        let (psid, id) = expect_args!(args, [int, int])?;
        if let Some(ps) = self.particles.get_system(psid) {
            Ok(ps.destroyers.get_asset(id).is_some().into())
        } else {
            Ok(gml::FALSE.into())
        }
    }

    pub fn part_destroyer_clear(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (psid, id) = expect_args!(args, [int, int])?;
        if let Some(ps) = self.particles.get_system_mut(psid) {
            if let Some(de) = ps.destroyers.get_asset_mut(id) {
                *de = particle::Destroyer::new();
            }
        }
        Ok(Default::default())
    }

    pub fn part_destroyer_region(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (psid, id, xmin, xmax, ymin, ymax, shape) = expect_args!(args, [int, int, real, real, real, real, int])?;
        let shape = match shape {
            1 => particle::Shape::Ellipse,
            2 => particle::Shape::Diamond,
            _ => particle::Shape::Rectangle,
        };
        if let Some(ps) = self.particles.get_system_mut(psid) {
            if let Some(de) = ps.destroyers.get_asset_mut(id) {
                de.xmin = xmin;
                de.xmax = xmax;
                de.ymin = ymin;
                de.ymax = ymax;
                de.shape = shape;
            }
        }
        Ok(Default::default())
    }

    pub fn part_deflector_create(&mut self, args: &[Value]) -> gml::Result<Value> {
        let id = expect_args!(args, [int])?;
        if let Some(ps) = self.particles.get_system_mut(id) {
            let de = particle::Deflector::new();
            if let Some(id) = ps.deflectors.iter().position(|x| x.is_none()) {
                ps.deflectors[id] = Some(de);
                Ok(id.into())
            } else {
                ps.deflectors.push(Some(de));
                Ok((ps.deflectors.len() - 1).into())
            }
        } else {
            Ok((-1).into())
        }
    }

    pub fn part_deflector_destroy(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (psid, id) = expect_args!(args, [int, int])?;
        if let Some(ps) = self.particles.get_system_mut(psid) {
            if ps.deflectors.get_asset(id).is_some() {
                ps.deflectors[id as usize] = None;
            }
        }
        Ok(Default::default())
    }

    pub fn part_deflector_destroy_all(&mut self, args: &[Value]) -> gml::Result<Value> {
        let psid = expect_args!(args, [int])?;
        if let Some(ps) = self.particles.get_system_mut(psid) {
            ps.deflectors.clear();
        }
        Ok(Default::default())
    }

    pub fn part_deflector_exists(&self, args: &[Value]) -> gml::Result<Value> {
        let (psid, id) = expect_args!(args, [int, int])?;
        if let Some(ps) = self.particles.get_system(psid) {
            Ok(ps.deflectors.get_asset(id).is_some().into())
        } else {
            Ok(gml::FALSE.into())
        }
    }

    pub fn part_deflector_clear(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (psid, id) = expect_args!(args, [int, int])?;
        if let Some(ps) = self.particles.get_system_mut(psid) {
            if let Some(de) = ps.deflectors.get_asset_mut(id) {
                *de = particle::Deflector::new();
            }
        }
        Ok(Default::default())
    }

    pub fn part_deflector_region(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (psid, id, xmin, xmax, ymin, ymax) = expect_args!(args, [int, int, real, real, real, real])?;
        if let Some(ps) = self.particles.get_system_mut(psid) {
            if let Some(de) = ps.deflectors.get_asset_mut(id) {
                de.xmin = xmin;
                de.xmax = xmax;
                de.ymin = ymin;
                de.ymax = ymax;
            }
        }
        Ok(Default::default())
    }

    pub fn part_deflector_kind(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (psid, id, kind) = expect_args!(args, [int, int, int])?;
        if let Some(ps) = self.particles.get_system_mut(psid) {
            if let Some(de) = ps.deflectors.get_asset_mut(id) {
                de.kind = match kind {
                    1 => particle::DeflectorKind::Horizontal,
                    _ => particle::DeflectorKind::Vertical,
                }
            }
        }
        Ok(Default::default())
    }

    pub fn part_deflector_friction(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (psid, id, friction) = expect_args!(args, [int, int, real])?;
        if let Some(ps) = self.particles.get_system_mut(psid) {
            if let Some(de) = ps.deflectors.get_asset_mut(id) {
                de.friction = friction;
            }
        }
        Ok(Default::default())
    }

    pub fn part_changer_create(&mut self, args: &[Value]) -> gml::Result<Value> {
        let id = expect_args!(args, [int])?;
        if let Some(ps) = self.particles.get_system_mut(id) {
            let ch = particle::Changer::new();
            if let Some(id) = ps.changers.iter().position(|x| x.is_none()) {
                ps.changers[id] = Some(ch);
                Ok(id.into())
            } else {
                ps.changers.push(Some(ch));
                Ok((ps.changers.len() - 1).into())
            }
        } else {
            Ok((-1).into())
        }
    }

    pub fn part_changer_destroy(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (psid, id) = expect_args!(args, [int, int])?;
        if let Some(ps) = self.particles.get_system_mut(psid) {
            if ps.changers.get_asset(id).is_some() {
                ps.changers[id as usize] = None;
            }
        }
        Ok(Default::default())
    }

    pub fn part_changer_destroy_all(&mut self, args: &[Value]) -> gml::Result<Value> {
        let psid = expect_args!(args, [int])?;
        if let Some(ps) = self.particles.get_system_mut(psid) {
            ps.changers.clear();
        }
        Ok(Default::default())
    }

    pub fn part_changer_exists(&self, args: &[Value]) -> gml::Result<Value> {
        let (psid, id) = expect_args!(args, [int, int])?;
        if let Some(ps) = self.particles.get_system(psid) {
            Ok(ps.changers.get_asset(id).is_some().into())
        } else {
            Ok(gml::FALSE.into())
        }
    }

    pub fn part_changer_clear(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (psid, id) = expect_args!(args, [int, int])?;
        if let Some(ps) = self.particles.get_system_mut(psid) {
            if let Some(ch) = ps.changers.get_asset_mut(id) {
                *ch = particle::Changer::new();
            }
        }
        Ok(Default::default())
    }

    pub fn part_changer_region(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (psid, id, xmin, xmax, ymin, ymax, shape) = expect_args!(args, [int, int, real, real, real, real, int])?;
        let shape = match shape {
            1 => particle::Shape::Ellipse,
            2 => particle::Shape::Diamond,
            _ => particle::Shape::Rectangle,
        };
        if let Some(ps) = self.particles.get_system_mut(psid) {
            if let Some(ch) = ps.changers.get_asset_mut(id) {
                ch.xmin = xmin;
                ch.xmax = xmax;
                ch.ymin = ymin;
                ch.ymax = ymax;
                ch.shape = shape;
            }
        }
        Ok(Default::default())
    }

    pub fn part_changer_kind(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (psid, id, kind) = expect_args!(args, [int, int, int])?;
        if let Some(ps) = self.particles.get_system_mut(psid) {
            if let Some(ch) = ps.changers.get_asset_mut(id) {
                ch.kind = match kind {
                    0 => particle::ChangerKind::All,
                    1 => particle::ChangerKind::Shape,
                    _ => particle::ChangerKind::Motion,
                };
            }
        }
        Ok(Default::default())
    }

    pub fn part_changer_types(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (psid, id, parttype1, parttype2) = expect_args!(args, [int, int, int, int])?;
        if let Some(ps) = self.particles.get_system_mut(psid) {
            if let Some(ch) = ps.changers.get_asset_mut(id) {
                ch.parttype1 = parttype1;
                ch.parttype2 = parttype2;
            }
        }
        Ok(Default::default())
    }

    pub fn effect_create_below(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (kind, x, y, size, colour) = expect_args!(args, [int, real, real, int, int])?;
        let kind = match kind {
            0 => particle::EffectType::Explosion,
            1 => particle::EffectType::Ring,
            2 => particle::EffectType::Ellipse,
            3 => particle::EffectType::Firework,
            4 => particle::EffectType::Smoke,
            5 => particle::EffectType::SmokeUp,
            6 => particle::EffectType::Star,
            7 => particle::EffectType::Spark,
            8 => particle::EffectType::Flare,
            9 => particle::EffectType::Cloud,
            10 => particle::EffectType::Rain,
            11 => particle::EffectType::Snow,
            _ => return Ok(Default::default()),
        };
        let size = match size {
            0 => particle::EffectSize::Small,
            2 => particle::EffectSize::Large,
            _ => particle::EffectSize::Medium,
        };
        self.particles.create_effect(
            kind,
            x,
            y,
            size,
            colour,
            true,
            (Real::from(30) / self.room.speed.into()).max(1.into()),
            self.room.width,
            self.room.height,
            &mut self.rand,
        );
        Ok(Default::default())
    }

    pub fn effect_create_above(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (kind, x, y, size, colour) = expect_args!(args, [int, real, real, int, int])?;
        let kind = match kind {
            0 => particle::EffectType::Explosion,
            1 => particle::EffectType::Ring,
            2 => particle::EffectType::Ellipse,
            3 => particle::EffectType::Firework,
            4 => particle::EffectType::Smoke,
            5 => particle::EffectType::SmokeUp,
            6 => particle::EffectType::Star,
            7 => particle::EffectType::Spark,
            8 => particle::EffectType::Flare,
            9 => particle::EffectType::Cloud,
            10 => particle::EffectType::Rain,
            11 => particle::EffectType::Snow,
            _ => return Ok(Default::default()),
        };
        let size = match size {
            0 => particle::EffectSize::Small,
            2 => particle::EffectSize::Large,
            _ => particle::EffectSize::Medium,
        };
        self.particles.create_effect(
            kind,
            x,
            y,
            size,
            colour,
            false,
            (Real::from(30) / self.room.speed.into()).max(1.into()),
            self.room.width,
            self.room.height,
            &mut self.rand,
        );
        Ok(Default::default())
    }

    pub fn effect_clear(&mut self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        self.particles.effect_clear();
        Ok(Default::default())
    }

    pub fn ds_set_precision(&mut self, args: &[Value]) -> gml::Result<Value> {
        self.ds_precision = expect_args!(args, [real])?;
        Ok(Default::default())
    }

    pub fn ds_stack_create(&mut self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        Ok(self.stacks.put(ds::Stack::new()).into())
    }

    pub fn ds_stack_destroy(&mut self, args: &[Value]) -> gml::Result<Value> {
        let id = expect_args!(args, [int])?;
        if self.stacks.delete(id) {
            Ok(Default::default())
        } else {
            Err(gml::Error::FunctionError("ds_stack_destroy".into(), ds::Error::NonexistentStructure(id).into()))
        }
    }

    pub fn ds_stack_clear(&mut self, args: &[Value]) -> gml::Result<Value> {
        let id = expect_args!(args, [int])?;
        match self.stacks.get_mut(id) {
            Some(stack) => {
                stack.clear();
                Ok(Default::default())
            },
            None => Err(gml::Error::FunctionError("ds_stack_clear".into(), ds::Error::NonexistentStructure(id).into())),
        }
    }

    pub fn ds_stack_copy(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (id, src_id) = expect_args!(args, [int, int])?;
        let src = match self.stacks.get(src_id) {
            Some(stack) => stack.clone(),
            None => {
                return Err(gml::Error::FunctionError(
                    "ds_stack_copy".into(),
                    ds::Error::NonexistentStructure(src_id).into(),
                ))
            },
        };
        match self.stacks.get_mut(id) {
            Some(stack) => {
                *stack = src;
                Ok(Default::default())
            },
            None => Err(gml::Error::FunctionError("ds_stack_copy".into(), ds::Error::NonexistentStructure(id).into())),
        }
    }

    pub fn ds_stack_size(&self, args: &[Value]) -> gml::Result<Value> {
        let id = expect_args!(args, [int])?;
        match self.stacks.get(id) {
            Some(stack) => Ok(stack.len().into()),
            None => Err(gml::Error::FunctionError("ds_stack_size".into(), ds::Error::NonexistentStructure(id).into())),
        }
    }

    pub fn ds_stack_empty(&self, args: &[Value]) -> gml::Result<Value> {
        let id = expect_args!(args, [int])?;
        match self.stacks.get(id) {
            Some(stack) => Ok(stack.is_empty().into()),
            None => Err(gml::Error::FunctionError("ds_stack_empty".into(), ds::Error::NonexistentStructure(id).into())),
        }
    }

    pub fn ds_stack_push(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (id, val) = expect_args!(args, [int, any])?;
        match self.stacks.get_mut(id) {
            Some(stack) => {
                stack.push(val);
                Ok(Default::default())
            },
            None => Err(gml::Error::FunctionError("ds_stack_push".into(), ds::Error::NonexistentStructure(id).into())),
        }
    }

    pub fn ds_stack_pop(&mut self, args: &[Value]) -> gml::Result<Value> {
        let id = expect_args!(args, [int])?;
        match self.stacks.get_mut(id) {
            Some(stack) => Ok(stack.pop().unwrap_or_default()),
            None => Err(gml::Error::FunctionError("ds_stack_pop".into(), ds::Error::NonexistentStructure(id).into())),
        }
    }

    pub fn ds_stack_top(&self, args: &[Value]) -> gml::Result<Value> {
        let id = expect_args!(args, [int])?;
        match self.stacks.get(id) {
            Some(stack) => Ok(stack.last().map(Value::clone).unwrap_or_default()),
            None => Err(gml::Error::FunctionError("ds_stack_top".into(), ds::Error::NonexistentStructure(id).into())),
        }
    }

    pub fn ds_stack_write(&self, args: &[Value]) -> gml::Result<Value> {
        let id = expect_args!(args, [int])?;
        match self.stacks.get(id) {
            Some(stack) => {
                let mut output = "65000000".to_string();
                output.push_str(&hex::encode_upper((stack.len() as u32).to_le_bytes()));
                output.extend(stack.iter().map(|v| hex::encode_upper(v.as_bytes())));
                Ok(output.into())
            },
            None => Err(gml::Error::FunctionError("ds_stack_write".into(), ds::Error::NonexistentStructure(id).into())),
        }
    }

    pub fn ds_stack_read(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (id, hex_data) = expect_args!(args, [int, string])?;
        match self.stacks.get_mut(id) {
            Some(old_stack) => {
                match hex::decode(hex_data.as_ref()) {
                    Ok(data) => {
                        let mut reader = data.as_slice();
                        // Read header and size
                        let mut buf = [0u8; 4];
                        if reader.read_exact(&mut buf).is_ok()
                            && u32::from_le_bytes(buf) == 0x65
                            && reader.read_exact(&mut buf).is_ok()
                        {
                            let size = u32::from_le_bytes(buf) as usize;
                            // Read each item
                            let mut stack = ds::Stack::with_capacity(size);
                            for _ in 0..size {
                                if let Some(val) = Value::from_reader(&mut reader) {
                                    stack.push(val);
                                } else {
                                    return Ok(Default::default())
                                }
                            }
                            *old_stack = stack;
                        }
                    },
                    Err(e) => eprintln!("Warning (ds_stack_read): {}", e),
                }
                Ok(Default::default())
            },
            None => Err(gml::Error::FunctionError("ds_stack_read".into(), ds::Error::NonexistentStructure(id).into())),
        }
    }

    pub fn ds_queue_create(&mut self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        Ok(self.queues.put(ds::Queue::new()).into())
    }

    pub fn ds_queue_destroy(&mut self, args: &[Value]) -> gml::Result<Value> {
        let id = expect_args!(args, [int])?;
        if self.queues.delete(id) {
            Ok(Default::default())
        } else {
            Err(gml::Error::FunctionError("ds_queue_destroy".into(), ds::Error::NonexistentStructure(id).into()))
        }
    }

    pub fn ds_queue_clear(&mut self, args: &[Value]) -> gml::Result<Value> {
        let id = expect_args!(args, [int])?;
        match self.queues.get_mut(id) {
            Some(queue) => {
                queue.clear();
                Ok(Default::default())
            },
            None => Err(gml::Error::FunctionError("ds_queue_clear".into(), ds::Error::NonexistentStructure(id).into())),
        }
    }

    pub fn ds_queue_copy(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (id, src_id) = expect_args!(args, [int, int])?;
        let src = match self.queues.get(src_id) {
            Some(queue) => queue.clone(),
            None => {
                return Err(gml::Error::FunctionError(
                    "ds_queue_copy".into(),
                    ds::Error::NonexistentStructure(src_id).into(),
                ))
            },
        };
        match self.queues.get_mut(id) {
            Some(queue) => {
                *queue = src;
                Ok(Default::default())
            },
            None => Err(gml::Error::FunctionError("ds_queue_copy".into(), ds::Error::NonexistentStructure(id).into())),
        }
    }

    pub fn ds_queue_size(&self, args: &[Value]) -> gml::Result<Value> {
        let id = expect_args!(args, [int])?;
        match self.queues.get(id) {
            Some(queue) => Ok(queue.len().into()),
            None => Err(gml::Error::FunctionError("ds_queue_size".into(), ds::Error::NonexistentStructure(id).into())),
        }
    }

    pub fn ds_queue_empty(&self, args: &[Value]) -> gml::Result<Value> {
        let id = expect_args!(args, [int])?;
        match self.queues.get(id) {
            Some(queue) => Ok(queue.is_empty().into()),
            None => Err(gml::Error::FunctionError("ds_queue_empty".into(), ds::Error::NonexistentStructure(id).into())),
        }
    }

    pub fn ds_queue_enqueue(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (id, val) = expect_args!(args, [int, any])?;
        match self.queues.get_mut(id) {
            Some(queue) => {
                queue.push_back(val);
                Ok(Default::default())
            },
            None => {
                Err(gml::Error::FunctionError("ds_queue_enqueue".into(), ds::Error::NonexistentStructure(id).into()))
            },
        }
    }

    pub fn ds_queue_dequeue(&mut self, args: &[Value]) -> gml::Result<Value> {
        let id = expect_args!(args, [int])?;
        match self.queues.get_mut(id) {
            Some(queue) => Ok(queue.pop_front().unwrap_or_default()),
            None => {
                Err(gml::Error::FunctionError("ds_queue_dequeue".into(), ds::Error::NonexistentStructure(id).into()))
            },
        }
    }

    pub fn ds_queue_head(&self, args: &[Value]) -> gml::Result<Value> {
        let id = expect_args!(args, [int])?;
        match self.queues.get(id) {
            Some(queue) => Ok(queue.front().map(Value::clone).unwrap_or_default()),
            None => Err(gml::Error::FunctionError("ds_queue_head".into(), ds::Error::NonexistentStructure(id).into())),
        }
    }

    pub fn ds_queue_tail(&self, args: &[Value]) -> gml::Result<Value> {
        let id = expect_args!(args, [int])?;
        match self.queues.get(id) {
            Some(queue) => Ok(queue.back().map(Value::clone).unwrap_or_default()),
            None => Err(gml::Error::FunctionError("ds_queue_tail".into(), ds::Error::NonexistentStructure(id).into())),
        }
    }

    pub fn ds_queue_write(&self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        unimplemented!("Called unimplemented kernel function ds_queue_write")
    }

    pub fn ds_queue_read(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 2
        unimplemented!("Called unimplemented kernel function ds_queue_read")
    }

    pub fn ds_list_create(&mut self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        Ok(self.lists.put(ds::List::new()).into())
    }

    pub fn ds_list_destroy(&mut self, args: &[Value]) -> gml::Result<Value> {
        let id = expect_args!(args, [int])?;
        if self.lists.delete(id) {
            Ok(Default::default())
        } else {
            Err(gml::Error::FunctionError("ds_list_destroy".into(), ds::Error::NonexistentStructure(id).into()))
        }
    }

    pub fn ds_list_clear(&mut self, args: &[Value]) -> gml::Result<Value> {
        let id = expect_args!(args, [int])?;
        match self.lists.get_mut(id) {
            Some(list) => {
                list.clear();
                Ok(Default::default())
            },
            None => Err(gml::Error::FunctionError("ds_list_clear".into(), ds::Error::NonexistentStructure(id).into())),
        }
    }

    pub fn ds_list_copy(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (id, src_id) = expect_args!(args, [int, int])?;
        let src = match self.lists.get(src_id) {
            Some(list) => list.clone(),
            None => {
                return Err(gml::Error::FunctionError(
                    "ds_list_copy".into(),
                    ds::Error::NonexistentStructure(src_id).into(),
                ))
            },
        };
        match self.lists.get_mut(id) {
            Some(list) => {
                *list = src;
                Ok(Default::default())
            },
            None => Err(gml::Error::FunctionError("ds_list_copy".into(), ds::Error::NonexistentStructure(id).into())),
        }
    }

    pub fn ds_list_size(&self, args: &[Value]) -> gml::Result<Value> {
        let id = expect_args!(args, [int])?;
        match self.lists.get(id) {
            Some(list) => Ok(list.len().into()),
            None => Err(gml::Error::FunctionError("ds_list_size".into(), ds::Error::NonexistentStructure(id).into())),
        }
    }

    pub fn ds_list_empty(&self, args: &[Value]) -> gml::Result<Value> {
        let id = expect_args!(args, [int])?;
        match self.lists.get(id) {
            Some(list) => Ok(list.is_empty().into()),
            None => Err(gml::Error::FunctionError("ds_list_empty".into(), ds::Error::NonexistentStructure(id).into())),
        }
    }

    pub fn ds_list_add(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (id, val) = expect_args!(args, [int, any])?;
        match self.lists.get_mut(id) {
            Some(list) => {
                list.push(val);
                Ok(Default::default())
            },
            None => Err(gml::Error::FunctionError("ds_list_add".into(), ds::Error::NonexistentStructure(id).into())),
        }
    }

    pub fn ds_list_insert(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (id, index, val) = expect_args!(args, [int, int, any])?;
        match self.lists.get_mut(id) {
            Some(list) => {
                if index >= 0 && (index as usize) <= list.len() {
                    list.insert(index as usize, val);
                }
                Ok(Default::default())
            },
            None => Err(gml::Error::FunctionError("ds_list_insert".into(), ds::Error::NonexistentStructure(id).into())),
        }
    }

    pub fn ds_list_replace(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (id, index, val) = expect_args!(args, [int, int, any])?;
        match self.lists.get_mut(id) {
            Some(list) => {
                if index >= 0 && (index as usize) < list.len() {
                    list[index as usize] = val;
                }
                Ok(Default::default())
            },
            None => {
                Err(gml::Error::FunctionError("ds_list_replace".into(), ds::Error::NonexistentStructure(id).into()))
            },
        }
    }

    pub fn ds_list_delete(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (id, index) = expect_args!(args, [int, int])?;
        match self.lists.get_mut(id) {
            Some(list) => {
                if index >= 0 && (index as usize) < list.len() {
                    list.remove(index as usize);
                }
                Ok(Default::default())
            },
            None => Err(gml::Error::FunctionError("ds_list_delete".into(), ds::Error::NonexistentStructure(id).into())),
        }
    }

    pub fn ds_list_find_index(&self, args: &[Value]) -> gml::Result<Value> {
        let (id, val) = expect_args!(args, [int, any])?;
        match self.lists.get(id) {
            Some(list) => Ok(list
                .iter()
                .enumerate()
                .find(|(_, x)| ds::eq(x, &val, self.ds_precision))
                .map(|(i, _)| i as i32)
                .unwrap_or(-1)
                .into()),
            None => {
                Err(gml::Error::FunctionError("ds_list_find_index".into(), ds::Error::NonexistentStructure(id).into()))
            },
        }
    }

    pub fn ds_list_find_value(&self, args: &[Value]) -> gml::Result<Value> {
        let (id, index) = expect_args!(args, [int, int])?;
        match self.lists.get(id) {
            Some(list) => {
                if index >= 0 && (index as usize) < list.len() {
                    Ok(list[index as usize].clone())
                } else {
                    Ok(Default::default())
                }
            },
            None => {
                Err(gml::Error::FunctionError("ds_list_find_value".into(), ds::Error::NonexistentStructure(id).into()))
            },
        }
    }

    pub fn ds_list_sort(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (id, asc) = expect_args!(args, [int, bool])?;
        match self.lists.get_mut(id) {
            Some(list) => {
                let precision = self.ds_precision; // otherwise we get borrowing issues
                if asc {
                    list.sort_by(|x, y| ds::cmp(x, y, precision));
                } else {
                    list.sort_by(|x, y| ds::cmp(y, x, precision));
                }
                Ok(Default::default())
            },
            None => Err(gml::Error::FunctionError("ds_list_sort".into(), ds::Error::NonexistentStructure(id).into())),
        }
    }

    pub fn ds_list_shuffle(&mut self, args: &[Value]) -> gml::Result<Value> {
        let id = expect_args!(args, [int])?;
        match self.lists.get_mut(id) {
            Some(list) => {
                for _ in 1..list.len() {
                    let id1 = self.rand.next_int(list.len() as u32 - 1);
                    let id2 = self.rand.next_int(list.len() as u32 - 1);
                    list.swap(id1 as usize, id2 as usize);
                }
                Ok(Default::default())
            },
            None => {
                Err(gml::Error::FunctionError("ds_list_shuffle".into(), ds::Error::NonexistentStructure(id).into()))
            },
        }
    }

    pub fn ds_list_write(&self, args: &[Value]) -> gml::Result<Value> {
        let id = expect_args!(args, [int])?;
        match self.lists.get(id) {
            Some(list) => {
                let mut output = "2D010000".to_string();
                output.push_str(&hex::encode_upper((list.len() as u32).to_le_bytes()));
                output.extend(list.iter().map(|v| hex::encode_upper(v.as_bytes())));
                Ok(output.into())
            },
            None => Err(gml::Error::FunctionError("ds_list_write".into(), ds::Error::NonexistentStructure(id).into())),
        }
    }

    pub fn ds_list_read(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (id, hex_data) = expect_args!(args, [int, string])?;
        fn read_list(mut reader: &[u8]) -> Option<ds::List> {
            let mut buf = [0u8; 4];
            reader.read_exact(&mut buf).ok()?;
            if u32::from_le_bytes(buf) != 0x12d {
                return None
            }
            reader.read_exact(&mut buf).ok()?;
            let size = u32::from_le_bytes(buf) as usize;
            let mut list = ds::List::with_capacity(size);
            for _ in 0..size {
                list.push(Value::from_reader(&mut reader)?);
            }
            Some(list)
        }
        match self.lists.get_mut(id) {
            Some(old_list) => {
                match hex::decode(hex_data.as_ref()) {
                    Ok(data) => {
                        if let Some(list) = read_list(data.as_slice()) {
                            *old_list = list;
                        }
                    },
                    Err(e) => eprintln!("Warning (ds_list_read): {}", e),
                }
                Ok(Default::default())
            },
            None => Err(gml::Error::FunctionError("ds_list_read".into(), ds::Error::NonexistentStructure(id).into())),
        }
    }

    pub fn ds_map_create(&mut self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        Ok(self.maps.put(ds::Map { keys: Vec::new(), values: Vec::new() }).into())
    }

    pub fn ds_map_destroy(&mut self, args: &[Value]) -> gml::Result<Value> {
        let id = expect_args!(args, [int])?;
        if self.maps.delete(id) {
            Ok(Default::default())
        } else {
            Err(gml::Error::FunctionError("ds_map_destroy".into(), ds::Error::NonexistentStructure(id).into()))
        }
    }

    pub fn ds_map_clear(&mut self, args: &[Value]) -> gml::Result<Value> {
        let id = expect_args!(args, [int])?;
        match self.maps.get_mut(id) {
            Some(map) => {
                map.keys.clear();
                map.values.clear();
                Ok(Default::default())
            },
            None => Err(gml::Error::FunctionError("ds_map_clear".into(), ds::Error::NonexistentStructure(id).into())),
        }
    }

    pub fn ds_map_copy(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (id, src_id) = expect_args!(args, [int, int])?;
        let src = match self.maps.get(src_id) {
            Some(map) => map.clone(),
            None => {
                return Err(gml::Error::FunctionError(
                    "ds_map_copy".into(),
                    ds::Error::NonexistentStructure(src_id).into(),
                ))
            },
        };
        match self.maps.get_mut(id) {
            Some(map) => {
                *map = src;
                Ok(Default::default())
            },
            None => Err(gml::Error::FunctionError("ds_map_copy".into(), ds::Error::NonexistentStructure(id).into())),
        }
    }

    pub fn ds_map_size(&self, args: &[Value]) -> gml::Result<Value> {
        let id = expect_args!(args, [int])?;
        match self.maps.get(id) {
            Some(map) => Ok(map.keys.len().into()),
            None => Err(gml::Error::FunctionError("ds_map_size".into(), ds::Error::NonexistentStructure(id).into())),
        }
    }

    pub fn ds_map_empty(&self, args: &[Value]) -> gml::Result<Value> {
        let id = expect_args!(args, [int])?;
        match self.maps.get(id) {
            Some(map) => Ok(map.keys.is_empty().into()),
            None => Err(gml::Error::FunctionError("ds_map_empty".into(), ds::Error::NonexistentStructure(id).into())),
        }
    }

    pub fn ds_map_add(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (id, key, val) = expect_args!(args, [int, any, any])?;
        match self.maps.get_mut(id) {
            Some(map) => {
                let index = map.get_next_index(&key, self.ds_precision);
                map.keys.insert(index, key);
                map.values.insert(index, val);
                Ok(Default::default())
            },
            None => Err(gml::Error::FunctionError("ds_map_add".into(), ds::Error::NonexistentStructure(id).into())),
        }
    }

    pub fn ds_map_replace(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (id, key, val) = expect_args!(args, [int, any, any])?;
        match self.maps.get_mut(id) {
            Some(map) => {
                if let Some(index) = map.get_index(&key, self.ds_precision) {
                    map.values[index] = val;
                }
                Ok(Default::default())
            },
            None => Err(gml::Error::FunctionError("ds_map_replace".into(), ds::Error::NonexistentStructure(id).into())),
        }
    }

    pub fn ds_map_delete(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (id, key) = expect_args!(args, [int, any])?;
        match self.maps.get_mut(id) {
            Some(map) => {
                if let Some(index) = map.get_index(&key, self.ds_precision) {
                    map.keys.remove(index);
                    map.values.remove(index);
                }
                Ok(Default::default())
            },
            None => Err(gml::Error::FunctionError("ds_map_delete".into(), ds::Error::NonexistentStructure(id).into())),
        }
    }

    pub fn ds_map_exists(&self, args: &[Value]) -> gml::Result<Value> {
        let (id, key) = expect_args!(args, [int, any])?;
        match self.maps.get(id) {
            Some(map) => Ok(map.contains_key(&key, self.ds_precision).into()),
            None => Err(gml::Error::FunctionError("ds_map_exists".into(), ds::Error::NonexistentStructure(id).into())),
        }
    }

    pub fn ds_map_find_value(&self, args: &[Value]) -> gml::Result<Value> {
        let (id, key) = expect_args!(args, [int, any])?;
        match self.maps.get(id) {
            Some(map) => Ok(map.get_index(&key, self.ds_precision).map_or(0.into(), |i| map.values[i].clone())),
            None => {
                Err(gml::Error::FunctionError("ds_map_find_value".into(), ds::Error::NonexistentStructure(id).into()))
            },
        }
    }

    pub fn ds_map_find_previous(&self, args: &[Value]) -> gml::Result<Value> {
        let (id, key) = expect_args!(args, [int, any])?;
        match self.maps.get(id) {
            Some(map) => {
                let index = map.get_index_unchecked(&key, self.ds_precision);
                if index > 0 { Ok(map.keys[index - 1].clone()) } else { Ok(Default::default()) }
            },
            None => Err(gml::Error::FunctionError(
                "ds_map_find_previous".into(),
                ds::Error::NonexistentStructure(id).into(),
            )),
        }
    }

    pub fn ds_map_find_next(&self, args: &[Value]) -> gml::Result<Value> {
        let (id, key) = expect_args!(args, [int, any])?;
        match self.maps.get(id) {
            Some(map) => {
                let index = map.get_next_index(&key, self.ds_precision);
                if index < map.keys.len() { Ok(map.keys[index].clone()) } else { Ok(Default::default()) }
            },
            None => {
                Err(gml::Error::FunctionError("ds_map_find_next".into(), ds::Error::NonexistentStructure(id).into()))
            },
        }
    }

    pub fn ds_map_find_first(&self, args: &[Value]) -> gml::Result<Value> {
        let id = expect_args!(args, [int])?;
        match self.maps.get(id) {
            Some(map) => Ok(map.keys.first().map(Value::clone).unwrap_or_default()),
            None => {
                Err(gml::Error::FunctionError("ds_map_find_first".into(), ds::Error::NonexistentStructure(id).into()))
            },
        }
    }

    pub fn ds_map_find_last(&self, args: &[Value]) -> gml::Result<Value> {
        let id = expect_args!(args, [int])?;
        match self.maps.get(id) {
            Some(map) => Ok(map.keys.last().map(Value::clone).unwrap_or_default()),
            None => {
                Err(gml::Error::FunctionError("ds_map_find_last".into(), ds::Error::NonexistentStructure(id).into()))
            },
        }
    }

    pub fn ds_map_write(&self, args: &[Value]) -> gml::Result<Value> {
        let id = expect_args!(args, [int])?;
        match self.maps.get(id) {
            Some(map) => {
                let mut output = "91010000".to_string();
                output.push_str(&hex::encode_upper((map.keys.len() as u32).to_le_bytes()));
                output.extend(map.keys.iter().map(|v| hex::encode_upper(v.as_bytes())));
                output.extend(map.values.iter().map(|v| hex::encode_upper(v.as_bytes())));
                Ok(output.into())
            },
            None => Err(gml::Error::FunctionError("ds_map_write".into(), ds::Error::NonexistentStructure(id).into())),
        }
    }

    pub fn ds_map_read(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (id, hex_data) = expect_args!(args, [int, string])?;
        fn read_map(mut reader: &[u8]) -> Option<ds::Map> {
            let mut buf = [0u8; 4];
            reader.read_exact(&mut buf).ok()?;
            if u32::from_le_bytes(buf) != 0x191 {
                return None
            }
            reader.read_exact(&mut buf).ok()?;
            let size = u32::from_le_bytes(buf) as usize;
            let mut keys = Vec::with_capacity(size);
            let mut values = Vec::with_capacity(size);
            for _ in 0..size {
                keys.push(Value::from_reader(&mut reader)?);
            }
            for _ in 0..size {
                values.push(Value::from_reader(&mut reader)?);
            }
            Some(ds::Map { keys, values })
        }
        match self.maps.get_mut(id) {
            Some(old_map) => {
                match hex::decode(hex_data.as_ref()) {
                    Ok(data) => {
                        if let Some(map) = read_map(data.as_slice()) {
                            *old_map = map;
                        }
                    },
                    Err(e) => eprintln!("Warning (ds_map_read): {}", e),
                }
                Ok(Default::default())
            },
            None => Err(gml::Error::FunctionError("ds_map_read".into(), ds::Error::NonexistentStructure(id).into())),
        }
    }

    pub fn ds_priority_create(&mut self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        Ok(self.priority_queues.put(ds::Priority { priorities: Vec::new(), values: Vec::new() }).into())
    }

    pub fn ds_priority_destroy(&mut self, args: &[Value]) -> gml::Result<Value> {
        let id = expect_args!(args, [int])?;
        if self.priority_queues.delete(id) {
            Ok(Default::default())
        } else {
            Err(gml::Error::FunctionError("ds_priority_destroy".into(), ds::Error::NonexistentStructure(id).into()))
        }
    }

    pub fn ds_priority_clear(&mut self, args: &[Value]) -> gml::Result<Value> {
        let id = expect_args!(args, [int])?;
        match self.priority_queues.get_mut(id) {
            Some(pq) => {
                pq.priorities.clear();
                pq.values.clear();
                Ok(Default::default())
            },
            None => {
                Err(gml::Error::FunctionError("ds_priority_clear".into(), ds::Error::NonexistentStructure(id).into()))
            },
        }
    }

    pub fn ds_priority_copy(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (id, src_id) = expect_args!(args, [int, int])?;
        let src = match self.priority_queues.get(src_id) {
            Some(queue) => queue.clone(),
            None => {
                return Err(gml::Error::FunctionError(
                    "ds_priority_copy".into(),
                    ds::Error::NonexistentStructure(src_id).into(),
                ))
            },
        };
        match self.priority_queues.get_mut(id) {
            Some(queue) => {
                *queue = src;
                Ok(Default::default())
            },
            None => {
                Err(gml::Error::FunctionError("ds_priority_copy".into(), ds::Error::NonexistentStructure(id).into()))
            },
        }
    }

    pub fn ds_priority_size(&self, args: &[Value]) -> gml::Result<Value> {
        let id = expect_args!(args, [int])?;
        match self.priority_queues.get(id) {
            Some(pq) => Ok(pq.priorities.len().into()),
            None => {
                Err(gml::Error::FunctionError("ds_priority_size".into(), ds::Error::NonexistentStructure(id).into()))
            },
        }
    }

    pub fn ds_priority_empty(&self, args: &[Value]) -> gml::Result<Value> {
        let id = expect_args!(args, [int])?;
        match self.priority_queues.get(id) {
            Some(pq) => Ok(pq.priorities.is_empty().into()),
            None => {
                Err(gml::Error::FunctionError("ds_priority_empty".into(), ds::Error::NonexistentStructure(id).into()))
            },
        }
    }

    pub fn ds_priority_add(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (id, val, prio) = expect_args!(args, [int, any, any])?;
        match self.priority_queues.get_mut(id) {
            Some(pq) => {
                pq.priorities.push(prio);
                pq.values.push(val);
                Ok(Default::default())
            },
            None => {
                Err(gml::Error::FunctionError("ds_priority_add".into(), ds::Error::NonexistentStructure(id).into()))
            },
        }
    }

    pub fn ds_priority_change_priority(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (id, val, prio) = expect_args!(args, [int, any, any])?;
        match self.priority_queues.get_mut(id) {
            Some(pq) => {
                let precision = self.ds_precision;
                if let Some(pos) = pq.values.iter().position(|x| ds::eq(x, &val, precision)) {
                    pq.priorities[pos] = prio;
                }
                Ok(Default::default())
            },
            None => Err(gml::Error::FunctionError(
                "ds_priority_change_priority".into(),
                ds::Error::NonexistentStructure(id).into(),
            )),
        }
    }

    pub fn ds_priority_find_priority(&self, args: &[Value]) -> gml::Result<Value> {
        let (id, val) = expect_args!(args, [int, any])?;
        match self.priority_queues.get(id) {
            Some(pq) => {
                let precision = self.ds_precision;
                if let Some(pos) = pq.values.iter().position(|x| ds::eq(x, &val, precision)) {
                    Ok(pq.priorities[pos].clone())
                } else {
                    Ok(Default::default())
                }
            },
            None => Err(gml::Error::FunctionError(
                "ds_priority_find_priority".into(),
                ds::Error::NonexistentStructure(id).into(),
            )),
        }
    }

    pub fn ds_priority_delete_value(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (id, val) = expect_args!(args, [int, any])?;
        match self.priority_queues.get_mut(id) {
            Some(pq) => {
                let precision = self.ds_precision;
                if let Some(pos) = pq.values.iter().position(|x| ds::eq(x, &val, precision)) {
                    pq.priorities.remove(pos);
                    pq.values.remove(pos);
                }
                Ok(Default::default())
            },
            None => Err(gml::Error::FunctionError(
                "ds_priority_delete_value".into(),
                ds::Error::NonexistentStructure(id).into(),
            )),
        }
    }

    pub fn ds_priority_delete_min(&mut self, args: &[Value]) -> gml::Result<Value> {
        let id = expect_args!(args, [int])?;
        match self.priority_queues.get_mut(id) {
            Some(pq) => {
                if let Some(min) = pq.min_id(self.ds_precision) {
                    pq.priorities.remove(min);
                    Ok(pq.values.remove(min))
                } else {
                    Ok(Default::default())
                }
            },
            None => Err(gml::Error::FunctionError(
                "ds_priority_delete_min".into(),
                ds::Error::NonexistentStructure(id).into(),
            )),
        }
    }

    pub fn ds_priority_find_min(&self, args: &[Value]) -> gml::Result<Value> {
        let id = expect_args!(args, [int])?;
        match self.priority_queues.get(id) {
            Some(pq) => {
                if let Some(min) = pq.min_id(self.ds_precision) {
                    Ok(pq.values[min].clone())
                } else {
                    Ok(Default::default())
                }
            },
            None => Err(gml::Error::FunctionError(
                "ds_priority_find_min".into(),
                ds::Error::NonexistentStructure(id).into(),
            )),
        }
    }

    pub fn ds_priority_delete_max(&mut self, args: &[Value]) -> gml::Result<Value> {
        let id = expect_args!(args, [int])?;
        match self.priority_queues.get_mut(id) {
            Some(pq) => {
                if let Some(max) = pq.max_id(self.ds_precision) {
                    pq.priorities.remove(max);
                    Ok(pq.values.remove(max))
                } else {
                    Ok(Default::default())
                }
            },
            None => Err(gml::Error::FunctionError(
                "ds_priority_delete_max".into(),
                ds::Error::NonexistentStructure(id).into(),
            )),
        }
    }

    pub fn ds_priority_find_max(&self, args: &[Value]) -> gml::Result<Value> {
        let id = expect_args!(args, [int])?;
        match self.priority_queues.get(id) {
            Some(pq) => {
                if let Some(max) = pq.max_id(self.ds_precision) {
                    Ok(pq.values[max].clone())
                } else {
                    Ok(Default::default())
                }
            },
            None => Err(gml::Error::FunctionError(
                "ds_priority_find_max".into(),
                ds::Error::NonexistentStructure(id).into(),
            )),
        }
    }

    pub fn ds_priority_write(&self, args: &[Value]) -> gml::Result<Value> {
        let id = expect_args!(args, [int])?;
        match self.priority_queues.get(id) {
            Some(pq) => {
                let mut output = "F5010000".to_string();
                output.push_str(&hex::encode_upper((pq.priorities.len() as u32).to_le_bytes()));
                output.extend(pq.priorities.iter().map(|v| hex::encode_upper(v.as_bytes())));
                output.extend(pq.values.iter().map(|v| hex::encode_upper(v.as_bytes())));
                Ok(output.into())
            },
            None => {
                Err(gml::Error::FunctionError("ds_priority_write".into(), ds::Error::NonexistentStructure(id).into()))
            },
        }
    }

    pub fn ds_priority_read(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (id, hex_data) = expect_args!(args, [int, string])?;
        fn read_priority(mut reader: &[u8]) -> Option<ds::Priority> {
            let mut buf = [0u8; 4];
            reader.read_exact(&mut buf).ok()?;
            if u32::from_le_bytes(buf) != 0x1f5 {
                return None
            }
            reader.read_exact(&mut buf).ok()?;
            let size = u32::from_le_bytes(buf) as usize;
            let mut priorities = Vec::with_capacity(size);
            let mut values = Vec::with_capacity(size);
            for _ in 0..size {
                priorities.push(Value::from_reader(&mut reader)?);
            }
            for _ in 0..size {
                values.push(Value::from_reader(&mut reader)?);
            }
            Some(ds::Priority { priorities, values })
        }
        match self.priority_queues.get_mut(id) {
            Some(old_pq) => {
                match hex::decode(hex_data.as_ref()) {
                    Ok(data) => {
                        if let Some(pq) = read_priority(data.as_slice()) {
                            *old_pq = pq;
                        }
                    },
                    Err(e) => eprintln!("Warning (ds_priority_read): {}", e),
                }
                Ok(Default::default())
            },
            None => {
                Err(gml::Error::FunctionError("ds_priority_read".into(), ds::Error::NonexistentStructure(id).into()))
            },
        }
    }

    pub fn ds_grid_create(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (width, height) = expect_args!(args, [int, int])?;
        if width < 0 || height < 0 {
            return Err(gml::Error::FunctionError(
                "ds_grid_create".into(),
                "grids cannot have negative dimensions".to_string(),
            ))
        }
        Ok(self.grids.put(ds::Grid::new(width as usize, height as usize)).into())
    }

    pub fn ds_grid_destroy(&mut self, args: &[Value]) -> gml::Result<Value> {
        let id = expect_args!(args, [int])?;
        if self.grids.delete(id) {
            Ok(Default::default())
        } else {
            Err(gml::Error::FunctionError("ds_grid_destroy".into(), ds::Error::NonexistentStructure(id).into()))
        }
    }

    pub fn ds_grid_copy(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (id, src_id) = expect_args!(args, [int, int])?;
        let src_grid = match self.grids.get(src_id) {
            Some(grid) => grid.clone(),
            None => {
                return Err(gml::Error::FunctionError(
                    "ds_grid_copy".into(),
                    ds::Error::NonexistentStructure(src_id).into(),
                ))
            },
        };
        match self.grids.get_mut(id) {
            Some(grid) => {
                *grid = src_grid;
                Ok(Default::default())
            },
            None => Err(gml::Error::FunctionError("ds_grid_copy".into(), ds::Error::NonexistentStructure(id).into())),
        }
    }

    pub fn ds_grid_resize(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (id, width, height) = expect_args!(args, [int, int, int])?;
        match self.grids.get_mut(id) {
            Some(grid) => {
                if width < 0 || height < 0 {
                    return Err(gml::Error::FunctionError(
                        "ds_grid_resize".into(),
                        "grids cannot have negative dimensions".to_string(),
                    ))
                }
                grid.resize(width as usize, height as usize);
                Ok(Default::default())
            },
            None => Err(gml::Error::FunctionError("ds_grid_resize".into(), ds::Error::NonexistentStructure(id).into())),
        }
    }

    pub fn ds_grid_width(&self, args: &[Value]) -> gml::Result<Value> {
        let id = expect_args!(args, [int])?;
        match self.grids.get(id) {
            Some(grid) => Ok(grid.width().into()),
            None => Err(gml::Error::FunctionError("ds_grid_width".into(), ds::Error::NonexistentStructure(id).into())),
        }
    }

    pub fn ds_grid_height(&self, args: &[Value]) -> gml::Result<Value> {
        let id = expect_args!(args, [int])?;
        match self.grids.get(id) {
            Some(grid) => Ok(grid.height().into()),
            None => Err(gml::Error::FunctionError("ds_grid_height".into(), ds::Error::NonexistentStructure(id).into())),
        }
    }

    pub fn ds_grid_clear(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (id, val) = expect_args!(args, [int, any])?;
        match self.grids.get_mut(id) {
            Some(grid) => {
                for x in 0..grid.width() {
                    for y in 0..grid.height() {
                        grid.set(x, y, val.clone());
                    }
                }
                Ok(Default::default())
            },
            None => Err(gml::Error::FunctionError("ds_grid_clear".into(), ds::Error::NonexistentStructure(id).into())),
        }
    }

    pub fn ds_grid_set(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (id, x, y, val) = expect_args!(args, [int, int, int, any])?;
        match self.grids.get_mut(id) {
            Some(grid) => {
                if x >= 0 && y >= 0 && (x as usize) < grid.width() && (y as usize) < grid.height() {
                    grid.set(x as usize, y as usize, val);
                }
                Ok(Default::default())
            },
            None => Err(gml::Error::FunctionError("ds_grid_set".into(), ds::Error::NonexistentStructure(id).into())),
        }
    }

    pub fn ds_grid_add(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 4
        unimplemented!("Called unimplemented kernel function ds_grid_add")
    }

    pub fn ds_grid_multiply(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 4
        unimplemented!("Called unimplemented kernel function ds_grid_multiply")
    }

    pub fn ds_grid_set_region(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 6
        unimplemented!("Called unimplemented kernel function ds_grid_set_region")
    }

    pub fn ds_grid_add_region(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 6
        unimplemented!("Called unimplemented kernel function ds_grid_add_region")
    }

    pub fn ds_grid_multiply_region(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 6
        unimplemented!("Called unimplemented kernel function ds_grid_multiply_region")
    }

    pub fn ds_grid_set_disk(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 5
        unimplemented!("Called unimplemented kernel function ds_grid_set_disk")
    }

    pub fn ds_grid_add_disk(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 5
        unimplemented!("Called unimplemented kernel function ds_grid_add_disk")
    }

    pub fn ds_grid_multiply_disk(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 5
        unimplemented!("Called unimplemented kernel function ds_grid_multiply_disk")
    }

    pub fn ds_grid_set_grid_region(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 8
        unimplemented!("Called unimplemented kernel function ds_grid_set_grid_region")
    }

    pub fn ds_grid_add_grid_region(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 8
        unimplemented!("Called unimplemented kernel function ds_grid_add_grid_region")
    }

    pub fn ds_grid_multiply_grid_region(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 8
        unimplemented!("Called unimplemented kernel function ds_grid_multiply_grid_region")
    }

    pub fn ds_grid_get(&self, args: &[Value]) -> gml::Result<Value> {
        let (id, x, y) = expect_args!(args, [int, int, int])?;
        match self.grids.get(id) {
            Some(grid) => {
                if x >= 0 && y >= 0 && (x as usize) < grid.width() && (y as usize) < grid.height() {
                    Ok(grid.get(x as usize, y as usize).clone())
                } else {
                    Ok(Default::default())
                }
            },
            None => Err(gml::Error::FunctionError("ds_grid_get".into(), ds::Error::NonexistentStructure(id).into())),
        }
    }

    pub fn ds_grid_get_sum(&self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 5
        unimplemented!("Called unimplemented kernel function ds_grid_get_sum")
    }

    pub fn ds_grid_get_max(&self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 5
        unimplemented!("Called unimplemented kernel function ds_grid_get_max")
    }

    pub fn ds_grid_get_min(&self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 5
        unimplemented!("Called unimplemented kernel function ds_grid_get_min")
    }

    pub fn ds_grid_get_mean(&self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 5
        unimplemented!("Called unimplemented kernel function ds_grid_get_mean")
    }

    pub fn ds_grid_get_disk_sum(&self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 4
        unimplemented!("Called unimplemented kernel function ds_grid_get_disk_sum")
    }

    pub fn ds_grid_get_disk_max(&self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 4
        unimplemented!("Called unimplemented kernel function ds_grid_get_disk_max")
    }

    pub fn ds_grid_get_disk_min(&self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 4
        unimplemented!("Called unimplemented kernel function ds_grid_get_disk_min")
    }

    pub fn ds_grid_get_disk_mean(&self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 4
        unimplemented!("Called unimplemented kernel function ds_grid_get_disk_mean")
    }

    pub fn ds_grid_value_exists(&self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 6
        unimplemented!("Called unimplemented kernel function ds_grid_value_exists")
    }

    pub fn ds_grid_value_x(&self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 6
        unimplemented!("Called unimplemented kernel function ds_grid_value_x")
    }

    pub fn ds_grid_value_y(&self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 6
        unimplemented!("Called unimplemented kernel function ds_grid_value_y")
    }

    pub fn ds_grid_value_disk_exists(&self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 5
        unimplemented!("Called unimplemented kernel function ds_grid_value_disk_exists")
    }

    pub fn ds_grid_value_disk_x(&self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 5
        unimplemented!("Called unimplemented kernel function ds_grid_value_disk_x")
    }

    pub fn ds_grid_value_disk_y(&self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 5
        unimplemented!("Called unimplemented kernel function ds_grid_value_disk_y")
    }

    pub fn ds_grid_shuffle(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        unimplemented!("Called unimplemented kernel function ds_grid_shuffle")
    }

    pub fn ds_grid_write(&self, args: &[Value]) -> gml::Result<Value> {
        let id = expect_args!(args, [int])?;
        match self.grids.get(id) {
            Some(grid) => {
                let mut output = "59020000".to_string();
                output.push_str(&hex::encode_upper((grid.width() as u32).to_le_bytes()));
                output.push_str(&hex::encode_upper((grid.height() as u32).to_le_bytes()));
                for x in 0..grid.width() {
                    for y in 0..grid.height() {
                        output.push_str(&hex::encode_upper(grid.get(x, y).as_bytes()));
                    }
                }
                Ok(output.into())
            },
            None => Err(gml::Error::FunctionError("ds_grid_write".into(), ds::Error::NonexistentStructure(id).into())),
        }
    }

    pub fn ds_grid_read(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (id, hex_data) = expect_args!(args, [int, string])?;
        fn read_grid(mut reader: &[u8]) -> Option<ds::Grid> {
            let mut buf = [0u8; 4];
            reader.read_exact(&mut buf).ok()?;
            if u32::from_le_bytes(buf) != 0x259 {
                return None
            }
            reader.read_exact(&mut buf).ok()?;
            let width = u32::from_le_bytes(buf) as usize;
            reader.read_exact(&mut buf).ok()?;
            let height = u32::from_le_bytes(buf) as usize;
            let mut grid = ds::Grid::new(width, height);
            for x in 0..width {
                for y in 0..height {
                    grid.set(x, y, Value::from_reader(&mut reader)?);
                }
            }
            Some(grid)
        }
        match self.grids.get_mut(id) {
            Some(old_grid) => {
                match hex::decode(hex_data.as_ref()) {
                    Ok(data) => {
                        if let Some(grid) = read_grid(data.as_slice()) {
                            *old_grid = grid;
                        }
                    },
                    Err(e) => eprintln!("Warning (ds_grid_read): {}", e),
                }
                Ok(Default::default())
            },
            None => Err(gml::Error::FunctionError("ds_grid_read".into(), ds::Error::NonexistentStructure(id).into())),
        }
    }

    pub fn sound_play(&mut self, args: &[Value]) -> gml::Result<Value> {
        let sound_id = expect_args!(args, [int])?;
        if let Some(sound) = self.assets.sounds.get_asset(sound_id) {
            use asset::sound::FileType;
            let nanos = self.spoofed_time_nanos.unwrap_or_else(|| datetime::now_as_nanos());
            match &sound.handle {
                FileType::Mp3(handle) => self.audio.play_mp3(handle, nanos),
                FileType::Wav(handle) => self.audio.play_wav(handle, nanos),
                FileType::None => (),
            }
            Ok(Default::default())
        } else {
            Err(gml::Error::NonexistentAsset(asset::Type::Sound, sound_id))
        }
    }

    pub fn sound_loop(&mut self, args: &[Value]) -> gml::Result<Value> {
        let sound_id = expect_args!(args, [int])?;
        if let Some(sound) = self.assets.sounds.get_asset(sound_id) {
            use asset::sound::FileType;
            match &sound.handle {
                FileType::Mp3(handle) => self.audio.loop_mp3(handle),
                FileType::Wav(handle) => self.audio.loop_wav(handle),
                FileType::None => (),
            }
            Ok(Default::default())
        } else {
            Err(gml::Error::NonexistentAsset(asset::Type::Sound, sound_id))
        }
    }

    pub fn sound_stop(&mut self, args: &[Value]) -> gml::Result<Value> {
        let sound_id = expect_args!(args, [int])?;
        self.audio.stop_sound(sound_id);
        Ok(Default::default())
    }

    pub fn sound_stop_all(&mut self, _args: &[Value]) -> gml::Result<Value> {
        self.audio.stop_all();
        Ok(Default::default())
    }

    pub fn sound_isplaying(&self, args: &[Value]) -> gml::Result<Value> {
        let sound_id = expect_args!(args, [int])?;
        let nanos = self.spoofed_time_nanos.unwrap_or_else(|| datetime::now_as_nanos());
        Ok(self.audio.sound_playing(sound_id, nanos).into())
    }

    pub fn sound_volume(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (sound_id, volume) = expect_args!(args, [int, real])?;
        if let Some(sound) = self.assets.sounds.get_asset(sound_id) {
            // Deliberately written in a way that will produce an error when Kind::Midi is added
            use asset::sound::FileType;
            match &sound.handle {
                FileType::Wav(handle) => handle.set_volume(volume.into()),
                FileType::Mp3(_) => (),
                FileType::None => (),
            }
            Ok(Default::default())
        } else {
            Err(gml::Error::NonexistentAsset(asset::Type::Sound, sound_id))
        }
    }

    pub fn sound_fade(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 3
        unimplemented!("Called unimplemented kernel function sound_fade")
    }

    pub fn sound_pan(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 2
        unimplemented!("Called unimplemented kernel function sound_pan")
    }

    pub fn sound_background_tempo(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        // Does nothing unless the sound is a midi, which we don't support yet
        Ok(Default::default())
    }

    pub fn sound_global_volume(&mut self, args: &[Value]) -> gml::Result<Value> {
        let volume = expect_args!(args, [real])?;
        self.audio.set_global_volume(volume.into());
        Ok(Default::default())
    }

    pub fn sound_set_search_directory(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        unimplemented!("Called unimplemented kernel function sound_set_search_directory")
    }

    pub fn sound_effect_set(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 2
        unimplemented!("Called unimplemented kernel function sound_effect_set")
    }

    pub fn sound_effect_chorus(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 8
        unimplemented!("Called unimplemented kernel function sound_effect_chorus")
    }

    pub fn sound_effect_compressor(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 7
        unimplemented!("Called unimplemented kernel function sound_effect_compressor")
    }

    pub fn sound_effect_echo(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 6
        unimplemented!("Called unimplemented kernel function sound_effect_echo")
    }

    pub fn sound_effect_flanger(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 8
        unimplemented!("Called unimplemented kernel function sound_effect_flanger")
    }

    pub fn sound_effect_gargle(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 3
        unimplemented!("Called unimplemented kernel function sound_effect_gargle")
    }

    pub fn sound_effect_equalizer(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 4
        unimplemented!("Called unimplemented kernel function sound_effect_equalizer")
    }

    pub fn sound_effect_reverb(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 5
        unimplemented!("Called unimplemented kernel function sound_effect_reverb")
    }

    pub fn sound_3d_set_sound_position(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 4
        unimplemented!("Called unimplemented kernel function sound_3d_set_sound_position")
    }

    pub fn sound_3d_set_sound_velocity(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 4
        unimplemented!("Called unimplemented kernel function sound_3d_set_sound_velocity")
    }

    pub fn sound_3d_set_sound_distance(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 3
        unimplemented!("Called unimplemented kernel function sound_3d_set_sound_distance")
    }

    pub fn sound_3d_set_sound_cone(&mut self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 7
        unimplemented!("Called unimplemented kernel function sound_3d_set_sound_cone")
    }

    pub fn cd_init(&self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 0
        unimplemented!("Called unimplemented kernel function cd_init")
    }

    pub fn cd_present(&self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 0
        unimplemented!("Called unimplemented kernel function cd_present")
    }

    pub fn cd_number(&self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 0
        unimplemented!("Called unimplemented kernel function cd_number")
    }

    pub fn cd_playing(&self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 0
        unimplemented!("Called unimplemented kernel function cd_playing")
    }

    pub fn cd_paused(&self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 0
        unimplemented!("Called unimplemented kernel function cd_paused")
    }

    pub fn cd_track(&self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 0
        unimplemented!("Called unimplemented kernel function cd_track")
    }

    pub fn cd_length(&self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 0
        unimplemented!("Called unimplemented kernel function cd_length")
    }

    pub fn cd_track_length(&self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        unimplemented!("Called unimplemented kernel function cd_track_length")
    }

    pub fn cd_position(&self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 0
        unimplemented!("Called unimplemented kernel function cd_position")
    }

    pub fn cd_track_position(&self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 0
        unimplemented!("Called unimplemented kernel function cd_track_position")
    }

    pub fn cd_play(&self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 2
        unimplemented!("Called unimplemented kernel function cd_play")
    }

    pub fn cd_stop(&self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 0
        unimplemented!("Called unimplemented kernel function cd_stop")
    }

    pub fn cd_pause(&self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 0
        unimplemented!("Called unimplemented kernel function cd_pause")
    }

    pub fn cd_resume(&self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 0
        unimplemented!("Called unimplemented kernel function cd_resume")
    }

    pub fn cd_set_position(&self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        unimplemented!("Called unimplemented kernel function cd_set_position")
    }

    pub fn cd_set_track_position(&self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        unimplemented!("Called unimplemented kernel function cd_set_track_position")
    }

    pub fn cd_open_door(&self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 0
        unimplemented!("Called unimplemented kernel function cd_open_door")
    }

    pub fn cd_close_door(&self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 0
        unimplemented!("Called unimplemented kernel function cd_close_door")
    }

    pub fn mci_command(&self, _args: &[Value]) -> gml::Result<Value> {
        // Expected arg count: 1
        unimplemented!("Called unimplemented kernel function MCI_command")
    }

    pub fn d3d_start(&mut self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        self.renderer.set_3d(true);
        Ok(1.into())
    }

    pub fn d3d_end(&mut self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        self.renderer.set_3d(false);
        Ok(1.into())
    }

    pub fn d3d_set_perspective(&mut self, args: &[Value]) -> gml::Result<Value> {
        let perspective = expect_args!(args, [bool])?;
        self.renderer.set_perspective(perspective);
        Ok(Default::default())
    }

    pub fn d3d_set_hidden(&mut self, args: &[Value]) -> gml::Result<Value> {
        let hidden = expect_args!(args, [bool])?;
        self.renderer.set_depth_test(hidden);
        Ok(Default::default())
    }

    pub fn d3d_set_depth(&mut self, args: &[Value]) -> gml::Result<Value> {
        let depth = expect_args!(args, [real])?;
        self.renderer.set_depth(depth.into_inner() as f32);
        Ok(Default::default())
    }

    pub fn d3d_set_zwriteenable(&mut self, args: &[Value]) -> gml::Result<Value> {
        let write_depth = expect_args!(args, [bool])?;
        if self.renderer.get_3d() {
            self.renderer.set_write_depth(write_depth);
        }
        Ok(Default::default())
    }

    pub fn d3d_set_lighting(&mut self, args: &[Value]) -> gml::Result<Value> {
        let enabled = expect_args!(args, [bool])?;
        self.renderer.set_lighting_enabled(enabled);
        Ok(Default::default())
    }

    pub fn d3d_set_shading(&mut self, args: &[Value]) -> gml::Result<Value> {
        let gouraud = expect_args!(args, [bool])?;
        self.renderer.set_gouraud(gouraud);
        Ok(Default::default())
    }

    pub fn d3d_set_fog(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (enabled, colour, begin, end) = expect_args!(args, [bool, int, real, real])?;
        let fog = if enabled {
            Some(Fog { colour, begin: begin.into_inner() as f32, end: end.into_inner() as f32 })
        } else {
            None
        };
        self.renderer.set_fog(fog);
        Ok(Default::default())
    }

    pub fn d3d_set_culling(&mut self, args: &[Value]) -> gml::Result<Value> {
        let cull = expect_args!(args, [bool])?;
        self.renderer.set_culling(cull);
        Ok(Default::default())
    }

    pub fn d3d_primitive_begin(&mut self, args: &[Value]) -> gml::Result<Value> {
        let kind = expect_args!(args, [int])?;
        self.renderer.reset_primitive_3d(kind.into(), None);
        Ok(Default::default())
    }

    pub fn d3d_primitive_begin_texture(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (kind, texture) = expect_args!(args, [int, int])?;
        self.renderer.reset_primitive_3d(kind.into(), self.renderer.get_texture_from_id(texture as _).copied());
        Ok(Default::default())
    }

    pub fn d3d_primitive_end(&mut self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        self.renderer.draw_primitive_3d();
        Ok(Default::default())
    }

    pub fn d3d_vertex(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (x, y, z) = expect_args!(args, [real, real, real])?;
        // And here we see the really weird GM8 colour rules where when drawing 3D vertices,
        // the LSB of the blue component is set to 0 when the colour isn't specified, and 1 when it is.
        let col = u32::from(self.draw_colour) as i32 & 0xfeffff;
        self.renderer.vertex_3d(x.into(), y.into(), z.into(), 0.0, 0.0, 0.0, 0.0, 0.0, col, self.draw_alpha.into());
        Ok(Default::default())
    }

    pub fn d3d_vertex_color(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (x, y, z, col, alpha) = expect_args!(args, [real, real, real, int, real])?;
        let col = col | 0x010000;
        self.renderer.vertex_3d(x.into(), y.into(), z.into(), 0.0, 0.0, 0.0, 0.0, 0.0, col, alpha.into());
        Ok(Default::default())
    }

    pub fn d3d_vertex_texture(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (x, y, z, xtex, ytex) = expect_args!(args, [real, real, real, real, real])?;
        let col = u32::from(self.draw_colour) as i32 & 0xfeffff;
        self.renderer.vertex_3d(
            x.into(),
            y.into(),
            z.into(),
            0.0,
            0.0,
            0.0,
            xtex.into(),
            ytex.into(),
            col,
            self.draw_alpha.into(),
        );
        Ok(Default::default())
    }

    pub fn d3d_vertex_texture_color(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (x, y, z, xtex, ytex, col, alpha) = expect_args!(args, [real, real, real, real, real, int, real])?;
        let col = col | 0x010000;
        self.renderer.vertex_3d(
            x.into(),
            y.into(),
            z.into(),
            0.0,
            0.0,
            0.0,
            xtex.into(),
            ytex.into(),
            col,
            alpha.into(),
        );
        Ok(Default::default())
    }

    pub fn d3d_vertex_normal(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (x, y, z, nx, ny, nz) = expect_args!(args, [real, real, real, real, real, real])?;
        let col = u32::from(self.draw_colour) as i32 & 0xfeffff;
        self.renderer.vertex_3d(
            x.into(),
            y.into(),
            z.into(),
            nx.into(),
            ny.into(),
            nz.into(),
            0.0,
            0.0,
            col,
            self.draw_alpha.into(),
        );
        Ok(Default::default())
    }

    pub fn d3d_vertex_normal_color(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (x, y, z, nx, ny, nz, col, alpha) = expect_args!(args, [real, real, real, real, real, real, int, real])?;
        let col = col | 0x010000;
        self.renderer.vertex_3d(
            x.into(),
            y.into(),
            z.into(),
            nx.into(),
            ny.into(),
            nz.into(),
            0.0,
            0.0,
            col,
            alpha.into(),
        );
        Ok(Default::default())
    }

    pub fn d3d_vertex_normal_texture(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (x, y, z, nx, ny, nz, xtex, ytex) = expect_args!(args, [real, real, real, real, real, real, real, real])?;
        let col = u32::from(self.draw_colour) as i32 & 0xfeffff;
        self.renderer.vertex_3d(
            x.into(),
            y.into(),
            z.into(),
            nx.into(),
            ny.into(),
            nz.into(),
            xtex.into(),
            ytex.into(),
            col,
            self.draw_alpha.into(),
        );
        Ok(Default::default())
    }

    pub fn d3d_vertex_normal_texture_color(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (x, y, z, nx, ny, nz, xtex, ytex, col, alpha) =
            expect_args!(args, [real, real, real, real, real, real, real, real, int, real])?;
        let col = col | 0x010000;
        self.renderer.vertex_3d(
            x.into(),
            y.into(),
            z.into(),
            nx.into(),
            ny.into(),
            nz.into(),
            xtex.into(),
            ytex.into(),
            col,
            alpha.into(),
        );
        Ok(Default::default())
    }

    pub fn d3d_draw_block(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (x1, y1, z1, x2, y2, z2, tex_id, hrepeat, vrepeat) =
            expect_args!(args, [real, real, real, real, real, real, int, real, real])?;
        let atlas_ref = self.renderer.get_texture_from_id(tex_id as _).copied();
        model::draw_block(
            &mut self.renderer,
            atlas_ref,
            &mut |r: &mut Renderer| r.draw_primitive_3d(),
            x1.into(),
            y1.into(),
            z1.into(),
            x2.into(),
            y2.into(),
            z2.into(),
            hrepeat.into(),
            vrepeat.into(),
            u32::from(self.draw_colour) as i32 & 0xfeffff,
            self.draw_alpha.into(),
        );
        Ok(Default::default())
    }

    pub fn d3d_draw_cylinder(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (x1, y1, z1, x2, y2, z2, tex_id, hrepeat, vrepeat, closed, steps) =
            expect_args!(args, [real, real, real, real, real, real, int, real, real, bool, int])?;
        let atlas_ref = self.renderer.get_texture_from_id(tex_id as _).copied();
        model::draw_cylinder(
            &mut self.renderer,
            atlas_ref,
            &mut |r: &mut Renderer| r.draw_primitive_3d(),
            x1.into_inner(),
            y1.into_inner(),
            z1.into_inner(),
            x2.into_inner(),
            y2.into_inner(),
            z2.into_inner(),
            hrepeat.into_inner(),
            vrepeat.into_inner(),
            closed,
            steps,
            u32::from(self.draw_colour) as i32 & 0xfeffff,
            self.draw_alpha.into(),
        );
        Ok(Default::default())
    }

    pub fn d3d_draw_cone(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (x1, y1, z1, x2, y2, z2, tex_id, hrepeat, vrepeat, closed, steps) =
            expect_args!(args, [real, real, real, real, real, real, int, real, real, bool, int])?;
        let atlas_ref = self.renderer.get_texture_from_id(tex_id as _).copied();
        model::draw_cone(
            &mut self.renderer,
            atlas_ref,
            &mut |r: &mut Renderer| r.draw_primitive_3d(),
            x1.into_inner(),
            y1.into_inner(),
            z1.into_inner(),
            x2.into_inner(),
            y2.into_inner(),
            z2.into_inner(),
            hrepeat.into_inner(),
            vrepeat.into_inner(),
            closed,
            steps,
            u32::from(self.draw_colour) as i32 & 0xfeffff,
            self.draw_alpha.into(),
        );
        Ok(Default::default())
    }

    pub fn d3d_draw_ellipsoid(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (x1, y1, z1, x2, y2, z2, tex_id, hrepeat, vrepeat, steps) =
            expect_args!(args, [real, real, real, real, real, real, int, real, real, int])?;
        let atlas_ref = self.renderer.get_texture_from_id(tex_id as _).copied();
        model::draw_ellipsoid(
            &mut self.renderer,
            atlas_ref,
            &mut |r: &mut Renderer| r.draw_primitive_3d(),
            x1.into_inner(),
            y1.into_inner(),
            z1.into_inner(),
            x2.into_inner(),
            y2.into_inner(),
            z2.into_inner(),
            hrepeat.into_inner(),
            vrepeat.into_inner(),
            steps,
            u32::from(self.draw_colour) as i32 & 0xfeffff,
            self.draw_alpha.into(),
        );
        Ok(Default::default())
    }

    pub fn d3d_draw_wall(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (x1, y1, z1, x2, y2, z2, tex_id, hrepeat, vrepeat) =
            expect_args!(args, [real, real, real, real, real, real, int, real, real])?;
        let atlas_ref = self.renderer.get_texture_from_id(tex_id as _).copied();
        model::draw_wall(
            &mut self.renderer,
            atlas_ref,
            &mut |r: &mut Renderer| r.draw_primitive_3d(),
            x1.into(),
            y1.into(),
            z1.into(),
            x2.into(),
            y2.into(),
            z2.into(),
            hrepeat.into(),
            vrepeat.into(),
            u32::from(self.draw_colour) as i32 & 0xfeffff,
            self.draw_alpha.into(),
        );
        Ok(Default::default())
    }

    pub fn d3d_draw_floor(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (x1, y1, z1, x2, y2, z2, tex_id, hrepeat, vrepeat) =
            expect_args!(args, [real, real, real, real, real, real, int, real, real])?;
        let atlas_ref = self.renderer.get_texture_from_id(tex_id as _).copied();
        model::draw_floor(
            &mut self.renderer,
            atlas_ref,
            &mut |r: &mut Renderer| r.draw_primitive_3d(),
            x1.into(),
            y1.into(),
            z1.into(),
            x2.into(),
            y2.into(),
            z2.into(),
            hrepeat.into(),
            vrepeat.into(),
            u32::from(self.draw_colour) as i32 & 0xfeffff,
            self.draw_alpha.into(),
        );
        Ok(Default::default())
    }

    pub fn d3d_set_projection(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (eye_x, eye_y, eye_z, at_x, at_y, at_z, up_x, up_y, up_z) =
            expect_args!(args, [real, real, real, real, real, real, real, real, real])?;

        // zaxis = normal(at - eye)
        let (za_x, za_y, za_z) = (at_x - eye_x, at_y - eye_y, at_z - eye_z);
        let za_len = (za_x * za_x + za_y * za_y + za_z * za_z).sqrt();
        let (za_x, za_y, za_z) = (za_x / za_len, za_y / za_len, za_z / za_len);
        // xaxis = normal(cross(up, zaxis))
        let (xa_x, xa_y, xa_z) = (up_y * za_z - up_z * za_y, up_z * za_x - up_x * za_z, up_x * za_y - up_y * za_x);
        let xa_len = (xa_x * xa_x + xa_y * xa_y + xa_z * xa_z).sqrt();
        let (xa_x, xa_y, xa_z) = (xa_x / xa_len, xa_y / xa_len, xa_z / xa_len);
        // yaxis = cross(zaxis, xaxis)
        let (ya_x, ya_y, ya_z) = (za_y * xa_z - za_z * xa_y, za_z * xa_x - za_x * xa_z, za_x * xa_y - za_y * xa_x);
        // bottom row
        let (xa_w, ya_w, za_w) = (
            -(xa_x * eye_x + xa_y * eye_y + xa_z * eye_z),
            -(ya_x * eye_x + ya_y * eye_y + ya_z * eye_z),
            -(za_x * eye_x + za_y * eye_y + za_z * eye_z),
        );

        #[rustfmt::skip]
        let view_matrix: [f32; 16] = [
            xa_x.into_inner() as f32, ya_x.into_inner() as f32, za_x.into_inner() as f32, 0.0,
            xa_y.into_inner() as f32, ya_y.into_inner() as f32, za_y.into_inner() as f32, 0.0,
            xa_z.into_inner() as f32, ya_z.into_inner() as f32, za_z.into_inner() as f32, 0.0,
            xa_w.into_inner() as f32, ya_w.into_inner() as f32, za_w.into_inner() as f32, 1.0,
        ];

        self.renderer.set_view_matrix(view_matrix);
        Ok(Default::default())
    }

    pub fn d3d_set_projection_ext(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (eye_x, eye_y, eye_z, at_x, at_y, at_z, up_x, up_y, up_z, angle, aspect, znear, zfar) =
            expect_args!(args, [real, real, real, real, real, real, real, real, real, real, real, real, real])?;

        // zaxis = normal(at - eye)
        let (za_x, za_y, za_z) = (at_x - eye_x, at_y - eye_y, at_z - eye_z);
        let za_len = (za_x * za_x + za_y * za_y + za_z * za_z).sqrt();
        let (za_x, za_y, za_z) = (za_x / za_len, za_y / za_len, za_z / za_len);
        // xaxis = normal(cross(up, zaxis))
        let (xa_x, xa_y, xa_z) = (up_y * za_z - up_z * za_y, up_z * za_x - up_x * za_z, up_x * za_y - up_y * za_x);
        let xa_len = (xa_x * xa_x + xa_y * xa_y + xa_z * xa_z).sqrt();
        let (xa_x, xa_y, xa_z) = (xa_x / xa_len, xa_y / xa_len, xa_z / xa_len);
        // yaxis = cross(zaxis, xaxis)
        let (ya_x, ya_y, ya_z) = (za_y * xa_z - za_z * xa_y, za_z * xa_x - za_x * xa_z, za_x * xa_y - za_y * xa_x);
        // bottom row
        let (xa_w, ya_w, za_w) = (
            -(xa_x * eye_x + xa_y * eye_y + xa_z * eye_z),
            -(ya_x * eye_x + ya_y * eye_y + ya_z * eye_z),
            -(za_x * eye_x + za_y * eye_y + za_z * eye_z),
        );

        #[rustfmt::skip]
        let view_matrix: [f32; 16] = [
            xa_x.into_inner() as f32, ya_x.into_inner() as f32, za_x.into_inner() as f32, 0.0,
            xa_y.into_inner() as f32, ya_y.into_inner() as f32, za_y.into_inner() as f32, 0.0,
            xa_z.into_inner() as f32, ya_z.into_inner() as f32, za_z.into_inner() as f32, 0.0,
            xa_w.into_inner() as f32, ya_w.into_inner() as f32, za_w.into_inner() as f32, 1.0,
        ];

        let half_angle = angle.to_radians() / 2.into();
        let yscale = half_angle.cos() / half_angle.sin();
        let xscale = (yscale / aspect).into_inner() as f32;
        let yscale = yscale.into_inner() as f32;
        let upper_z = (zfar / (zfar - znear)).into_inner() as f32;
        let lower_z = (-znear * zfar / (zfar - znear)).into_inner() as f32;
        #[rustfmt::skip]
        let proj_matrix: [f32; 16] = [
            xscale, 0.0,    0.0,     0.0,
            0.0,    yscale, 0.0,     0.0,
            0.0,    0.0,    upper_z, 1.0,
            0.0,    0.0,    lower_z, 0.0,
        ];

        self.renderer.set_viewproj_matrix(view_matrix, proj_matrix);
        Ok(Default::default())
    }

    pub fn d3d_set_projection_ortho(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (x, y, w, h, angle) = expect_args!(args, [real, real, real, real, real])?;
        self.renderer.set_projection_ortho(x.into(), y.into(), w.into(), h.into(), angle.into());
        Ok(Default::default())
    }

    pub fn d3d_set_projection_perspective(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (x, y, w, h, angle) = expect_args!(args, [real, real, real, real, real])?;
        self.renderer.set_projection_perspective(x.into(), y.into(), w.into(), h.into(), angle.into());
        Ok(Default::default())
    }

    pub fn d3d_transform_set_identity(&mut self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        #[rustfmt::skip]
        let model_matrix: [f32; 16] = [
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        ];
        self.renderer.set_model_matrix(model_matrix);
        Ok(Default::default())
    }

    pub fn d3d_transform_set_translation(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (xt, yt, zt) = expect_args!(args, [real, real, real])?;
        #[rustfmt::skip]
        let model_matrix: [f32; 16] = [
            1.0,                    0.0,                    0.0,                    0.0,
            0.0,                    1.0,                    0.0,                    0.0,
            0.0,                    0.0,                    1.0,                    0.0,
            xt.into_inner() as f32, yt.into_inner() as f32, zt.into_inner() as f32, 1.0,
        ];
        self.renderer.set_model_matrix(model_matrix);
        Ok(Default::default())
    }

    pub fn d3d_transform_set_scaling(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (xs, ys, zs) = expect_args!(args, [real, real, real])?;
        #[rustfmt::skip]
        let model_matrix: [f32; 16] = [
            xs.into_inner() as f32, 0.0,                    0.0,                    0.0,
            0.0,                    ys.into_inner() as f32, 0.0,                    0.0,
            0.0,                    0.0,                    zs.into_inner() as f32, 0.0,
            0.0,                    0.0,                    0.0,                    1.0,
        ];
        self.renderer.set_model_matrix(model_matrix);
        Ok(Default::default())
    }

    pub fn d3d_transform_set_rotation_x(&mut self, args: &[Value]) -> gml::Result<Value> {
        let angle = expect_args!(args, [real])?.to_radians();
        let sin = -angle.sin().into_inner() as f32;
        let cos = angle.cos().into_inner() as f32;
        #[rustfmt::skip]
        let model_matrix: [f32; 16] = [
            1.0, 0.0,  0.0, 0.0,
            0.0, cos,  sin, 0.0,
            0.0, -sin, cos, 0.0,
            0.0, 0.0,  0.0, 1.0,
        ];
        self.renderer.set_model_matrix(model_matrix);
        Ok(Default::default())
    }

    pub fn d3d_transform_set_rotation_y(&mut self, args: &[Value]) -> gml::Result<Value> {
        let angle = expect_args!(args, [real])?.to_radians();
        let sin = -angle.sin().into_inner() as f32;
        let cos = angle.cos().into_inner() as f32;
        #[rustfmt::skip]
        let model_matrix: [f32; 16] = [
            cos, 0.0, -sin, 0.0,
            0.0, 1.0, 0.0,  0.0,
            sin, 0.0, cos,  0.0,
            0.0, 0.0, 0.0,  1.0,
        ];
        self.renderer.set_model_matrix(model_matrix);
        Ok(Default::default())
    }

    pub fn d3d_transform_set_rotation_z(&mut self, args: &[Value]) -> gml::Result<Value> {
        let angle = expect_args!(args, [real])?.to_radians();
        let sin = -angle.sin().into_inner() as f32;
        let cos = angle.cos().into_inner() as f32;
        #[rustfmt::skip]
        let model_matrix: [f32; 16] = [
            cos,  sin, 0.0, 0.0,
            -sin, cos, 0.0, 0.0,
            0.0,  0.0, 1.0, 0.0,
            0.0,  0.0, 0.0, 1.0,
        ];
        self.renderer.set_model_matrix(model_matrix);
        Ok(Default::default())
    }

    pub fn d3d_transform_set_rotation_axis(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (xa, ya, za, angle) = expect_args!(args, [real, real, real, real])?;
        let axis_len = (xa * xa + ya * ya + za * za).sqrt();
        let (xa, ya, za) = (xa / axis_len, ya / axis_len, za / axis_len);
        let angle = angle.to_radians();
        let sin = -angle.sin();
        let cos = angle.cos();
        let anticos = Real::from(1.0) - cos;
        let m00 = (cos + xa * xa * anticos).into_inner() as f32;
        let m01 = (xa * ya * anticos - za * sin).into_inner() as f32;
        let m02 = (xa * za * anticos + ya * sin).into_inner() as f32;
        let m10 = (ya * xa * anticos + za * sin).into_inner() as f32;
        let m11 = (cos + ya * ya * anticos).into_inner() as f32;
        let m12 = (ya * za * anticos - xa * sin).into_inner() as f32;
        let m20 = (za * za * anticos - ya * sin).into_inner() as f32;
        let m21 = (za * ya * anticos + xa * sin).into_inner() as f32;
        let m22 = (cos + za * za * anticos).into_inner() as f32;
        #[rustfmt::skip]
        let model_matrix = [
            m00, m10, m20, 0.0,
            m01, m11, m21, 0.0,
            m02, m12, m22, 0.0,
            0.0, 0.0, 0.0, 1.0,
        ];
        self.renderer.set_model_matrix(model_matrix);
        Ok(Default::default())
    }

    pub fn d3d_transform_add_translation(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (xt, yt, zt) = expect_args!(args, [real, real, real])?;
        #[rustfmt::skip]
        let model_matrix: [f32; 16] = [
            1.0,                    0.0,                    0.0,                    0.0,
            0.0,                    1.0,                    0.0,                    0.0,
            0.0,                    0.0,                    1.0,                    0.0,
            xt.into_inner() as f32, yt.into_inner() as f32, zt.into_inner() as f32, 1.0,
        ];
        self.renderer.mult_model_matrix(model_matrix);
        Ok(Default::default())
    }

    pub fn d3d_transform_add_scaling(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (xs, ys, zs) = expect_args!(args, [real, real, real])?;
        #[rustfmt::skip]
        let model_matrix: [f32; 16] = [
            xs.into_inner() as f32, 0.0,                    0.0,                    0.0,
            0.0,                    ys.into_inner() as f32, 0.0,                    0.0,
            0.0,                    0.0,                    zs.into_inner() as f32, 0.0,
            0.0,                    0.0,                    0.0,                    1.0,
        ];
        self.renderer.mult_model_matrix(model_matrix);
        Ok(Default::default())
    }

    pub fn d3d_transform_add_rotation_x(&mut self, args: &[Value]) -> gml::Result<Value> {
        let angle = expect_args!(args, [real])?.to_radians();
        let sin = -angle.sin().into_inner() as f32;
        let cos = angle.cos().into_inner() as f32;
        #[rustfmt::skip]
        let model_matrix: [f32; 16] = [
            1.0, 0.0,  0.0, 0.0,
            0.0, cos,  sin, 0.0,
            0.0, -sin, cos, 0.0,
            0.0, 0.0,  0.0, 1.0,
        ];
        self.renderer.mult_model_matrix(model_matrix);
        Ok(Default::default())
    }

    pub fn d3d_transform_add_rotation_y(&mut self, args: &[Value]) -> gml::Result<Value> {
        let angle = expect_args!(args, [real])?.to_radians();
        let sin = -angle.sin().into_inner() as f32;
        let cos = angle.cos().into_inner() as f32;
        #[rustfmt::skip]
        let model_matrix: [f32; 16] = [
            cos, 0.0, -sin, 0.0,
            0.0, 1.0, 0.0,  0.0,
            sin, 0.0, cos,  0.0,
            0.0, 0.0, 0.0,  1.0,
        ];
        self.renderer.mult_model_matrix(model_matrix);
        Ok(Default::default())
    }

    pub fn d3d_transform_add_rotation_z(&mut self, args: &[Value]) -> gml::Result<Value> {
        let angle = expect_args!(args, [real])?.to_radians();
        let sin = -angle.sin().into_inner() as f32;
        let cos = angle.cos().into_inner() as f32;
        #[rustfmt::skip]
        let model_matrix: [f32; 16] = [
            cos,  sin, 0.0, 0.0,
            -sin, cos, 0.0, 0.0,
            0.0,  0.0, 1.0, 0.0,
            0.0,  0.0, 0.0, 1.0,
        ];
        self.renderer.mult_model_matrix(model_matrix);
        Ok(Default::default())
    }

    pub fn d3d_transform_add_rotation_axis(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (xa, ya, za, angle) = expect_args!(args, [real, real, real, real])?;
        let axis_len = (xa * xa + ya * ya + za * za).sqrt();
        let (xa, ya, za) = (xa / axis_len, ya / axis_len, za / axis_len);
        let angle = angle.to_radians();
        let sin = -angle.sin();
        let cos = angle.cos();
        let anticos = Real::from(1.0) - cos;
        let m00 = (cos + xa * xa * anticos).into_inner() as f32;
        let m01 = (xa * ya * anticos - za * sin).into_inner() as f32;
        let m02 = (xa * za * anticos + ya * sin).into_inner() as f32;
        let m10 = (ya * xa * anticos + za * sin).into_inner() as f32;
        let m11 = (cos + ya * ya * anticos).into_inner() as f32;
        let m12 = (ya * za * anticos - xa * sin).into_inner() as f32;
        let m20 = (za * za * anticos - ya * sin).into_inner() as f32;
        let m21 = (za * ya * anticos + xa * sin).into_inner() as f32;
        let m22 = (cos + za * za * anticos).into_inner() as f32;
        #[rustfmt::skip]
        let model_matrix = [
            m00, m10, m20, 0.0,
            m01, m11, m21, 0.0,
            m02, m12, m22, 0.0,
            0.0, 0.0, 0.0, 1.0,
        ];
        self.renderer.mult_model_matrix(model_matrix);
        Ok(Default::default())
    }

    pub fn d3d_transform_stack_clear(&mut self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        self.model_matrix_stack.clear();
        Ok(Default::default())
    }

    pub fn d3d_transform_stack_empty(&self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        Ok(self.model_matrix_stack.is_empty().into())
    }

    pub fn d3d_transform_stack_push(&mut self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        if self.model_matrix_stack.len() <= 1000 {
            self.model_matrix_stack.push(self.renderer.get_model_matrix());
            Ok(true.into())
        } else {
            Ok(false.into())
        }
    }

    pub fn d3d_transform_stack_pop(&mut self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        if let Some(mat) = self.model_matrix_stack.pop() {
            self.renderer.set_model_matrix(mat);
            Ok(true.into())
        } else {
            Ok(false.into())
        }
    }

    pub fn d3d_transform_stack_top(&mut self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        if let Some(mat) = self.model_matrix_stack.last() {
            self.renderer.set_model_matrix(*mat);
            Ok(true.into())
        } else {
            Ok(false.into())
        }
    }

    pub fn d3d_transform_stack_discard(&mut self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        Ok(self.model_matrix_stack.pop().is_some().into())
    }

    pub fn d3d_light_define_ambient(&mut self, args: &[Value]) -> gml::Result<Value> {
        let colour = expect_args!(args, [int])?;
        self.renderer.set_ambient_colour(colour);
        Ok(Default::default())
    }

    pub fn d3d_light_define_direction(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (id, dx, dy, dz, colour) = expect_args!(args, [int, real, real, real, int])?;
        if (0..8).contains(&id) {
            self.renderer.set_light(id as usize, Light::Directional {
                direction: [dx.into_inner() as f32, dy.into_inner() as f32, dz.into_inner() as f32],
                colour,
            });
        }
        Ok(Default::default())
    }

    pub fn d3d_light_define_point(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (id, x, y, z, range, colour) = expect_args!(args, [int, real, real, real, real, int])?;
        if (0..8).contains(&id) {
            self.renderer.set_light(id as usize, Light::Point {
                position: [x.into_inner() as f32, y.into_inner() as f32, z.into_inner() as f32],
                range: range.into_inner() as f32,
                colour,
            });
        }
        Ok(Default::default())
    }

    pub fn d3d_light_enable(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (id, enabled) = expect_args!(args, [int, bool])?;
        if (0..8).contains(&id) {
            self.renderer.set_light_enabled(id as usize, enabled);
        }
        Ok(Default::default())
    }

    pub fn d3d_model_create(&mut self, args: &[Value]) -> gml::Result<Value> {
        expect_args!(args, [])?;
        let model = Default::default();
        if let Some(id) = self.models.iter().position(|x| x.is_none()) {
            self.models[id] = Some(model);
            Ok(id.into())
        } else {
            self.models.push(Some(model));
            Ok((self.models.len() - 1).into())
        }
    }

    pub fn d3d_model_destroy(&mut self, args: &[Value]) -> gml::Result<Value> {
        let model_id = expect_args!(args, [int])?;
        if self.models.get_asset(model_id).is_some() {
            self.models[model_id as usize] = None;
        }
        Ok(Default::default())
    }

    pub fn d3d_model_clear(&mut self, args: &[Value]) -> gml::Result<Value> {
        let model_id = expect_args!(args, [int])?;
        if let Some(model) = self.models.get_asset_mut(model_id) {
            *model = Default::default();
        }
        Ok(Default::default())
    }

    pub fn d3d_model_load(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (model_id, fname) = expect_args!(args, [int, string])?;
        fn load_model(fname: &str) -> Result<model::Model, Box<dyn std::error::Error>> {
            let mut file = std::io::BufReader::new(std::fs::File::open(fname)?);
            let version = file::read_real(&mut file)?;
            if version != 100.0 {
                return Err("invalid version".into())
            };
            file::skip_line(&mut file)?;
            let command_count = match file::read_real(&mut file)? as i32 {
                x if x < 0 => return Err("negative command count".into()),
                x => x as usize,
            };
            file::skip_line(&mut file)?;
            let mut commands = Vec::with_capacity(command_count);
            for _ in 0..command_count {
                let cmd = file::read_real(&mut file)?.round() as i32;
                let mut args = [0.0; 10];
                for x in &mut args {
                    *x = file::read_real(&mut file)?;
                }
                file::skip_line(&mut file)?;
                commands.push(match cmd {
                    0 => model::Command::Begin((args[0] as i32).into()),
                    1 => model::Command::End,
                    2 => model::Command::Vertex {
                        pos: [args[0].into(), args[1].into(), args[2].into()],
                        normal: [0.into(); 3],
                        tex_coord: [0.into(); 2],
                    },
                    3 => model::Command::VertexColour {
                        pos: [args[0].into(), args[1].into(), args[2].into()],
                        normal: [0.into(); 3],
                        tex_coord: [0.into(); 2],
                        col: (args[3] as i32, args[4].into()),
                    },
                    4 => model::Command::Vertex {
                        pos: [args[0].into(), args[1].into(), args[2].into()],
                        normal: [0.into(); 3],
                        tex_coord: [args[3].into(), args[4].into()],
                    },
                    5 => model::Command::VertexColour {
                        pos: [args[0].into(), args[1].into(), args[2].into()],
                        normal: [0.into(); 3],
                        tex_coord: [args[3].into(), args[4].into()],
                        col: (args[5] as i32, args[6].into()),
                    },
                    6 => model::Command::Vertex {
                        pos: [args[0].into(), args[1].into(), args[2].into()],
                        normal: [args[3].into(), args[4].into(), args[5].into()],
                        tex_coord: [0.into(); 2],
                    },
                    7 => model::Command::VertexColour {
                        pos: [args[0].into(), args[1].into(), args[2].into()],
                        normal: [args[3].into(), args[4].into(), args[5].into()],
                        tex_coord: [0.into(); 2],
                        col: (args[6] as i32, args[7].into()),
                    },
                    8 => model::Command::Vertex {
                        pos: [args[0].into(), args[1].into(), args[2].into()],
                        normal: [args[3].into(), args[4].into(), args[5].into()],
                        tex_coord: [args[6].into(), args[7].into()],
                    },
                    9 => model::Command::VertexColour {
                        pos: [args[0].into(), args[1].into(), args[2].into()],
                        normal: [args[3].into(), args[4].into(), args[5].into()],
                        tex_coord: [args[6].into(), args[7].into()],
                        col: (args[8] as i32, args[9].into()),
                    },
                    10 => model::Command::Block {
                        pos1: [args[0].into(), args[1].into(), args[2].into()],
                        pos2: [args[3].into(), args[4].into(), args[5].into()],
                        tex_repeat: [args[6].into(), args[7].into()],
                    },
                    11 => model::Command::Cylinder {
                        pos1: [args[0].into(), args[1].into(), args[2].into()],
                        pos2: [args[3].into(), args[4].into(), args[5].into()],
                        tex_repeat: [args[6].into(), args[7].into()],
                        closed: args[8] >= 0.5,
                        steps: args[9] as _,
                    },
                    12 => model::Command::Cone {
                        pos1: [args[0].into(), args[1].into(), args[2].into()],
                        pos2: [args[3].into(), args[4].into(), args[5].into()],
                        tex_repeat: [args[6].into(), args[7].into()],
                        closed: args[8] >= 0.5,
                        steps: args[9] as _,
                    },
                    13 => model::Command::Ellipsoid {
                        pos1: [args[0].into(), args[1].into(), args[2].into()],
                        pos2: [args[3].into(), args[4].into(), args[5].into()],
                        tex_repeat: [args[6].into(), args[7].into()],
                        steps: args[8] as _,
                    },
                    14 => model::Command::Wall {
                        pos1: [args[0].into(), args[1].into(), args[2].into()],
                        pos2: [args[3].into(), args[4].into(), args[5].into()],
                        tex_repeat: [args[6].into(), args[7].into()],
                    },
                    15 => model::Command::Floor {
                        pos1: [args[0].into(), args[1].into(), args[2].into()],
                        pos2: [args[3].into(), args[4].into(), args[5].into()],
                        tex_repeat: [args[6].into(), args[7].into()],
                    },
                    _ => continue,
                });
            }
            Ok(model::Model { old_draw_colour: None, commands, cache: None })
        }
        if let Some(model) = self.models.get_asset_mut(model_id) {
            match load_model(&fname) {
                Ok(new_model) => *model = new_model,
                Err(e) => return Err(gml::Error::FunctionError("d3d_model_load".into(), e.to_string())),
            }
        }
        Ok(Default::default())
    }

    pub fn d3d_model_save(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (model_id, fname) = expect_args!(args, [int, string])?;
        fn save_model(model: &model::Model, fname: &str) -> std::io::Result<()> {
            let mut file = std::io::BufWriter::new(std::fs::File::create(fname)?);
            writeln!(&mut file, "100\r\n{}\r", model.commands.len())?;
            for cmd in &model.commands {
                let (cmd, args) = cmd.to_line();
                write!(&mut file, "{}", cmd)?;
                for arg in args.iter() {
                    write!(&mut file, " {:.4}", arg)?;
                }
                writeln!(&mut file, "\r")?;
            }
            file.flush()?;
            Ok(())
        }
        if let Some(model) = self.models.get_asset(model_id) {
            if let Err(e) = save_model(model, &fname) {
                return Err(gml::Error::FunctionError("d3d_model_save".into(), format!("{}", e)))
            }
        }
        Ok(Default::default())
    }

    pub fn d3d_model_draw(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (model_id, x, y, z, tex_id) = expect_args!(args, [int, real, real, real, int])?;
        let atlas_ref = self.renderer.get_texture_from_id(tex_id as _).copied();
        if let Some(model) = self.models.get_asset_mut(model_id) {
            // translate according to given position
            let old_model_matrix = self.renderer.get_model_matrix();
            #[rustfmt::skip]
            let translation: [f32; 16] = [
                1.0,                    0.0,                    0.0,                    0.0,
                0.0,                    1.0,                    0.0,                    0.0,
                0.0,                    0.0,                    1.0,                    0.0,
                x.into_inner() as f32,  y.into_inner() as f32,  z.into_inner() as f32,  1.0,
            ];
            self.renderer.mult_model_matrix(translation);

            let draw_colour = (u32::from(self.draw_colour) as i32 & 0xfeffff, self.draw_alpha.into_inner());
            if model.cache.is_none() || self.gm_version == Version::GameMaker8_0 {
                // GM8.0 does not use model caching.
                // GM8.1 draws the model semi-normally once, then caches that and redraws.
                let mut buffers = Default::default();
                let mut primitive_draw: Box<dyn FnMut(&mut Renderer)> = match self.gm_version {
                    Version::GameMaker8_0 => Box::new(|r| r.draw_primitive_3d()),
                    Version::GameMaker8_1 => Box::new(|r| r.extend_buffers(&mut buffers)),
                };
                let mut uses_draw_colour = false;
                for command in &model.commands {
                    match command {
                        model::Command::Begin(ptype) => self.renderer.reset_primitive_3d(*ptype, atlas_ref),
                        model::Command::Vertex { pos: [x, y, z], normal: [nx, ny, nz], tex_coord: [xtex, ytex] } => {
                            self.renderer.vertex_3d(
                                x.into_inner(),
                                y.into_inner(),
                                z.into_inner(),
                                nx.into_inner(),
                                ny.into_inner(),
                                nz.into_inner(),
                                xtex.into_inner(),
                                ytex.into_inner(),
                                draw_colour.0,
                                draw_colour.1,
                            );
                            uses_draw_colour = true;
                        },
                        model::Command::VertexColour {
                            pos: [x, y, z],
                            normal: [nx, ny, nz],
                            tex_coord: [xtex, ytex],
                            col: (col, alpha),
                        } => {
                            self.renderer.vertex_3d(
                                x.into_inner(),
                                y.into_inner(),
                                z.into_inner(),
                                nx.into_inner(),
                                ny.into_inner(),
                                nz.into_inner(),
                                xtex.into_inner(),
                                ytex.into_inner(),
                                *col | 1,
                                alpha.into_inner(),
                            );
                        },
                        model::Command::Block { pos1: [x1, y1, z1], pos2: [x2, y2, z2], tex_repeat: [hr, vr] } => {
                            model::draw_block(
                                &mut self.renderer,
                                atlas_ref,
                                &mut primitive_draw,
                                x1.into_inner(),
                                y1.into_inner(),
                                z1.into_inner(),
                                x2.into_inner(),
                                y2.into_inner(),
                                z2.into_inner(),
                                hr.into_inner(),
                                vr.into_inner(),
                                draw_colour.0,
                                draw_colour.1,
                            );
                            uses_draw_colour = true;
                        },
                        model::Command::Cylinder {
                            pos1: [x1, y1, z1],
                            pos2: [x2, y2, z2],
                            tex_repeat: [hr, vr],
                            closed,
                            steps,
                        } => {
                            model::draw_cylinder(
                                &mut self.renderer,
                                atlas_ref,
                                &mut primitive_draw,
                                x1.into_inner(),
                                y1.into_inner(),
                                z1.into_inner(),
                                x2.into_inner(),
                                y2.into_inner(),
                                z2.into_inner(),
                                hr.into_inner(),
                                vr.into_inner(),
                                *closed,
                                *steps,
                                draw_colour.0,
                                draw_colour.1,
                            );
                            uses_draw_colour = true;
                        },
                        model::Command::Cone {
                            pos1: [x1, y1, z1],
                            pos2: [x2, y2, z2],
                            tex_repeat: [hr, vr],
                            closed,
                            steps,
                        } => {
                            let x1 = match self.gm_version {
                                Version::GameMaker8_0 => *x1,
                                Version::GameMaker8_1 => x + *x1, // why is gm8 like this
                            };
                            model::draw_cone(
                                &mut self.renderer,
                                atlas_ref,
                                &mut primitive_draw,
                                x1.into_inner(),
                                y1.into_inner(),
                                z1.into_inner(),
                                x2.into_inner(),
                                y2.into_inner(),
                                z2.into_inner(),
                                hr.into_inner(),
                                vr.into_inner(),
                                *closed,
                                *steps,
                                draw_colour.0,
                                draw_colour.1,
                            );
                            uses_draw_colour = true;
                        },
                        model::Command::Ellipsoid {
                            pos1: [x1, y1, z1],
                            pos2: [x2, y2, z2],
                            tex_repeat: [hr, vr],
                            steps,
                        } => {
                            model::draw_ellipsoid(
                                &mut self.renderer,
                                atlas_ref,
                                &mut primitive_draw,
                                x1.into_inner(),
                                y1.into_inner(),
                                z1.into_inner(),
                                x2.into_inner(),
                                y2.into_inner(),
                                z2.into_inner(),
                                hr.into_inner(),
                                vr.into_inner(),
                                *steps,
                                draw_colour.0,
                                draw_colour.1,
                            );
                            uses_draw_colour = true;
                        },
                        model::Command::Wall { pos1: [x1, y1, z1], pos2: [x2, y2, z2], tex_repeat: [hr, vr] } => {
                            model::draw_wall(
                                &mut self.renderer,
                                atlas_ref,
                                &mut primitive_draw,
                                x1.into_inner(),
                                y1.into_inner(),
                                z1.into_inner(),
                                x2.into_inner(),
                                y2.into_inner(),
                                z2.into_inner(),
                                hr.into_inner(),
                                vr.into_inner(),
                                draw_colour.0,
                                draw_colour.1,
                            );
                            uses_draw_colour = true;
                        },
                        model::Command::Floor { pos1: [x1, y1, z1], pos2: [x2, y2, z2], tex_repeat: [hr, vr] } => {
                            model::draw_floor(
                                &mut self.renderer,
                                atlas_ref,
                                &mut primitive_draw,
                                x1.into_inner(),
                                y1.into_inner(),
                                z1.into_inner(),
                                x2.into_inner(),
                                y2.into_inner(),
                                z2.into_inner(),
                                hr.into_inner(),
                                vr.into_inner(),
                                draw_colour.0,
                                draw_colour.1,
                            );
                            uses_draw_colour = true;
                        },
                        model::Command::End => primitive_draw(&mut self.renderer),
                    }
                }
                if uses_draw_colour {
                    model.old_draw_colour = Some(draw_colour);
                }
                drop(primitive_draw);
                model.cache = Some(buffers);
            }
            if self.gm_version == Version::GameMaker8_1 {
                let cache = model.cache.as_mut().unwrap();
                if let Some(old_col) = model.old_draw_colour {
                    cache.swap_colour(old_col, draw_colour);
                    model.old_draw_colour = Some(draw_colour);
                }
                self.renderer.draw_buffers(atlas_ref, cache);
            }

            self.renderer.set_model_matrix(old_model_matrix);
        }
        Ok(Default::default())
    }

    pub fn d3d_model_primitive_begin(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (model_id, kind) = expect_args!(args, [int, int])?;
        if let Some(model) = self.models.get_asset_mut(model_id) {
            model.commands.push(model::Command::Begin(kind.into()));
        }
        Ok(Default::default())
    }

    pub fn d3d_model_primitive_end(&mut self, args: &[Value]) -> gml::Result<Value> {
        let model_id = expect_args!(args, [int])?;
        if let Some(model) = self.models.get_asset_mut(model_id) {
            model.commands.push(model::Command::End);
        }
        Ok(Default::default())
    }

    pub fn d3d_model_vertex(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (model_id, x, y, z) = expect_args!(args, [int, real, real, real])?;
        if let Some(model) = self.models.get_asset_mut(model_id) {
            model.commands.push(model::Command::Vertex {
                pos: [x, y, z],
                normal: [0.into(); 3],
                tex_coord: [0.into(); 2],
            });
        }
        Ok(Default::default())
    }

    pub fn d3d_model_vertex_color(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (model_id, x, y, z, col, alpha) = expect_args!(args, [int, real, real, real, int, real])?;
        if let Some(model) = self.models.get_asset_mut(model_id) {
            model.commands.push(model::Command::VertexColour {
                pos: [x, y, z],
                normal: [0.into(); 3],
                tex_coord: [0.into(); 2],
                col: (col, alpha),
            });
        }
        Ok(Default::default())
    }

    pub fn d3d_model_vertex_texture(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (model_id, x, y, z, xtex, ytex) = expect_args!(args, [int, real, real, real, real, real])?;
        if let Some(model) = self.models.get_asset_mut(model_id) {
            model.commands.push(model::Command::Vertex {
                pos: [x, y, z],
                normal: [0.into(); 3],
                tex_coord: [xtex, ytex],
            });
        }
        Ok(Default::default())
    }

    pub fn d3d_model_vertex_texture_color(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (model_id, x, y, z, xtex, ytex, col, alpha) =
            expect_args!(args, [int, real, real, real, real, real, int, real])?;
        if let Some(model) = self.models.get_asset_mut(model_id) {
            model.commands.push(model::Command::VertexColour {
                pos: [x, y, z],
                normal: [0.into(); 3],
                tex_coord: [xtex, ytex],
                col: (col, alpha),
            });
        }
        Ok(Default::default())
    }

    pub fn d3d_model_vertex_normal(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (model_id, x, y, z, nx, ny, nz) = expect_args!(args, [int, real, real, real, real, real, real])?;
        if let Some(model) = self.models.get_asset_mut(model_id) {
            model.commands.push(model::Command::Vertex {
                pos: [x, y, z],
                normal: [nx, ny, nz],
                tex_coord: [0.into(); 2],
            });
        }
        Ok(Default::default())
    }

    pub fn d3d_model_vertex_normal_color(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (model_id, x, y, z, nx, ny, nz, col, alpha) =
            expect_args!(args, [int, real, real, real, real, real, real, int, real])?;
        if let Some(model) = self.models.get_asset_mut(model_id) {
            model.commands.push(model::Command::VertexColour {
                pos: [x, y, z],
                normal: [nx, ny, nz],
                tex_coord: [0.into(); 2],
                col: (col, alpha),
            });
        }
        Ok(Default::default())
    }

    pub fn d3d_model_vertex_normal_texture(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (model_id, x, y, z, nx, ny, nz, xtex, ytex) =
            expect_args!(args, [int, real, real, real, real, real, real, real, real])?;
        if let Some(model) = self.models.get_asset_mut(model_id) {
            model.commands.push(model::Command::Vertex {
                pos: [x, y, z],
                normal: [nx, ny, nz],
                tex_coord: [xtex, ytex],
            });
        }
        Ok(Default::default())
    }

    pub fn d3d_model_vertex_normal_texture_color(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (model_id, x, y, z, nx, ny, nz, xtex, ytex, col, alpha) =
            expect_args!(args, [int, real, real, real, real, real, real, real, real, int, real])?;
        if let Some(model) = self.models.get_asset_mut(model_id) {
            model.commands.push(model::Command::VertexColour {
                pos: [x, y, z],
                normal: [nx, ny, nz],
                tex_coord: [xtex, ytex],
                col: (col, alpha),
            });
        }
        Ok(Default::default())
    }

    pub fn d3d_model_block(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (model_id, x1, y1, z1, x2, y2, z2, hrepeat, vrepeat) =
            expect_args!(args, [int, real, real, real, real, real, real, real, real])?;
        if let Some(model) = self.models.get_asset_mut(model_id) {
            model.commands.push(model::Command::Block {
                pos1: [x1, y1, z1],
                pos2: [x2, y2, z2],
                tex_repeat: [hrepeat, vrepeat],
            });
        }
        Ok(Default::default())
    }

    pub fn d3d_model_cylinder(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (model_id, x1, y1, z1, x2, y2, z2, hrepeat, vrepeat, closed, steps) =
            expect_args!(args, [int, real, real, real, real, real, real, real, real, bool, int])?;
        if let Some(model) = self.models.get_asset_mut(model_id) {
            model.commands.push(model::Command::Cylinder {
                pos1: [x1, y1, z1],
                pos2: [x2, y2, z2],
                tex_repeat: [hrepeat, vrepeat],
                closed,
                steps,
            });
        }
        Ok(Default::default())
    }

    pub fn d3d_model_cone(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (model_id, x1, y1, z1, x2, y2, z2, hrepeat, vrepeat, closed, steps) =
            expect_args!(args, [int, real, real, real, real, real, real, real, real, bool, int])?;
        if let Some(model) = self.models.get_asset_mut(model_id) {
            model.commands.push(model::Command::Cone {
                pos1: [x1, y1, z1],
                pos2: [x2, y2, z2],
                tex_repeat: [hrepeat, vrepeat],
                closed,
                steps,
            });
        }
        Ok(Default::default())
    }

    pub fn d3d_model_ellipsoid(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (model_id, x1, y1, z1, x2, y2, z2, hrepeat, vrepeat, steps) =
            expect_args!(args, [int, real, real, real, real, real, real, real, real, int])?;
        if let Some(model) = self.models.get_asset_mut(model_id) {
            model.commands.push(model::Command::Ellipsoid {
                pos1: [x1, y1, z1],
                pos2: [x2, y2, z2],
                tex_repeat: [hrepeat, vrepeat],
                steps,
            });
        }
        Ok(Default::default())
    }

    pub fn d3d_model_wall(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (model_id, x1, y1, z1, x2, y2, z2, hrepeat, vrepeat) =
            expect_args!(args, [int, real, real, real, real, real, real, real, real])?;
        if let Some(model) = self.models.get_asset_mut(model_id) {
            model.commands.push(model::Command::Wall {
                pos1: [x1, y1, z1],
                pos2: [x2, y2, z2],
                tex_repeat: [hrepeat, vrepeat],
            });
        }
        Ok(Default::default())
    }

    pub fn d3d_model_floor(&mut self, args: &[Value]) -> gml::Result<Value> {
        let (model_id, x1, y1, z1, x2, y2, z2, hrepeat, vrepeat) =
            expect_args!(args, [int, real, real, real, real, real, real, real, real])?;
        if let Some(model) = self.models.get_asset_mut(model_id) {
            model.commands.push(model::Command::Floor {
                pos1: [x1, y1, z1],
                pos2: [x2, y2, z2],
                tex_repeat: [hrepeat, vrepeat],
            });
        }
        Ok(Default::default())
    }
}
