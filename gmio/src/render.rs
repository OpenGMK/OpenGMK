//! Game rendering functionality

mod opengl;

use crate::{atlas::AtlasBuilder, window::Window};
use serde::{Deserialize, Serialize};
use shared::types::Colour;
use std::any::Any;

// Re-export for more logical module pathing
pub use crate::atlas::AtlasRef;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Scaling {
    /// Fixed scale, with a multiplier. The multiplier always be strictly positive.
    Fixed(f64),
    /// Scale with window, but preserve aspect ratio.
    /// The f64 must be strictly negative and has no meaning, but can still be accessed with window_get_region_scale().
    Aspect(f64),
    /// Scale to fill window.
    Full,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SavedTexture {
    width: i32,
    height: i32,
    pixels: Box<[u8]>,
    zbuf: Option<Box<[f32]>>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Fog {
    pub colour: i32,
    pub begin: f32,
    pub end: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Light {
    Directional { direction: [f32; 3], colour: i32 },
    Point { position: [f32; 3], range: f32, colour: i32 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlendType {
    Zero,
    One,
    SrcColour,
    InvSrcColour,
    SrcAlpha,
    InvSrcAlpha,
    DestAlpha,
    InvDestAlpha,
    DestColour,
    InvDestColour,
    SrcAlphaSaturate,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrimitiveType {
    PointList,
    LineList,
    LineStrip,
    TriList,
    TriStrip,
    TriFan,
}

impl From<i32> for PrimitiveType {
    fn from(pt: i32) -> Self {
        match pt {
            0 | 1 => PrimitiveType::PointList,
            2 => PrimitiveType::LineList,
            3 => PrimitiveType::LineStrip,
            4 => PrimitiveType::TriList,
            5 => PrimitiveType::TriStrip,
            6 => PrimitiveType::TriFan,
            _ => PrimitiveType::PointList, // GM8 just draws nothing in this case
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
enum PrimitiveShape {
    Point,
    Line,
    Triangle,
}

impl From<PrimitiveType> for PrimitiveShape {
    fn from(pt: PrimitiveType) -> Self {
        match pt {
            PrimitiveType::PointList => PrimitiveShape::Point,
            PrimitiveType::LineList | PrimitiveType::LineStrip => PrimitiveShape::Line,
            PrimitiveType::TriList | PrimitiveType::TriStrip | PrimitiveType::TriFan => PrimitiveShape::Triangle,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
struct Vertex {
    pub pos: [f32; 3],
    pub tex_coord: [f32; 2],
    pub blend: [f32; 4],
    pub atlas_xywh: [f32; 4],
    pub normal: [f32; 3],
}

/// A builder to be used for building primitives.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PrimitiveBuilder {
    vertices: Vec<Vertex>,
    ptype: PrimitiveType,
    atlas_ref: AtlasRef,
}

impl PrimitiveBuilder {
    fn new(atlas_ref: AtlasRef, ptype: PrimitiveType) -> Self {
        Self { vertices: Vec::new(), ptype, atlas_ref }
    }

    fn push_vertex_raw(&mut self, v: Vertex) -> &mut Self {
        // if we need to fill out a shape get the other two points
        let (v1, v2) = match self.ptype {
            PrimitiveType::PointList | PrimitiveType::LineList | PrimitiveType::TriList => (None, None),
            PrimitiveType::LineStrip => (self.vertices.last().filter(|_| self.vertices.len() >= 2).copied(), None),
            PrimitiveType::TriFan | PrimitiveType::TriStrip if self.vertices.len() < 3 => (None, None),
            PrimitiveType::TriStrip | PrimitiveType::TriFan => {
                (Some(self.vertices[self.vertices.len() - 2]), self.vertices.last().copied())
            },
        };
        if let Some(v1) = v1 {
            self.vertices.push(v1);
        }
        self.vertices.push(v);
        if let Some(v2) = v2 {
            self.vertices.push(v2);
            let len = self.vertices.len();
            if len % 6 == 3 && self.ptype == PrimitiveType::TriStrip {
                self.vertices.swap(len - 2, len - 1);
                self.vertices.swap(len - 3, len - 2);
            }
        } else if self.vertices.len() == 3 && self.ptype == PrimitiveType::TriFan {
            // i hate that this works
            self.vertices.swap(0, 1);
            self.vertices.swap(1, 2);
        }
        self
    }

    fn push_vertex(&mut self, pos: [f32; 3], tex_coord: [f32; 2], blend: [f32; 4], normal: [f32; 3]) -> &mut Self {
        self.push_vertex_raw(Vertex { pos, tex_coord, blend, normal, atlas_xywh: self.atlas_ref.into() });
        self
    }

    fn get_atlas_id(&self) -> u32 {
        self.atlas_ref.atlas_id
    }

    fn get_shape(&self) -> PrimitiveShape {
        self.ptype.into()
    }

    fn get_vertices(&self) -> &[Vertex] {
        &self.vertices
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct VertexBuffer {
    points: Vec<Vertex>,
    lines: Vec<Vertex>,
    tris: Vec<Vertex>,
}

pub struct Renderer(Box<dyn RendererTrait>);

pub trait RendererTrait {
    fn as_any(&self) -> &dyn Any;
    fn max_texture_size(&self) -> u32;
    fn push_atlases(&mut self, atl: AtlasBuilder) -> Result<(), String>;
    fn upload_sprite(
        &mut self,
        data: Box<[u8]>,
        width: i32,
        height: i32,
        origin_x: i32,
        origin_y: i32,
    ) -> Result<AtlasRef, String>;
    fn duplicate_sprite(&mut self, atlas_ref: &AtlasRef) -> Result<AtlasRef, String>;
    fn delete_sprite(&mut self, atlas_ref: AtlasRef);

    fn resize_framebuffer(&mut self, width: u32, height: u32);

    fn set_vsync(&self, vsync: bool);
    fn get_vsync(&self) -> bool;
    fn wait_vsync(&self);

    fn draw_sprite(&mut self, tex: &AtlasRef, x: f64, y: f64, xs: f64, ys: f64, ang: f64, col: i32, alpha: f64) {
        self.draw_sprite_general(
            tex,
            0.0,
            0.0,
            tex.w.into(),
            tex.h.into(),
            x,
            y,
            xs,
            ys,
            ang,
            col,
            col,
            col,
            col,
            alpha,
            true,
        );
    }

    fn draw_sprite_colour(
        &mut self,
        tex: &AtlasRef,
        x: f64,
        y: f64,
        xs: f64,
        ys: f64,
        ang: f64,
        col1: i32,
        col2: i32,
        col3: i32,
        col4: i32,
        alpha: f64,
    ) {
        self.draw_sprite_general(
            tex,
            0.0,
            0.0,
            tex.w.into(),
            tex.h.into(),
            x,
            y,
            xs,
            ys,
            ang,
            col1,
            col2,
            col3,
            col4,
            alpha,
            true,
        );
    }

    fn draw_sprite_general(
        &mut self,
        texture: &AtlasRef,
        part_x: f64,
        part_y: f64,
        part_w: f64,
        part_h: f64,
        x: f64,
        y: f64,
        xscale: f64,
        yscale: f64,
        angle: f64,
        col1: i32,
        col2: i32,
        col3: i32,
        col4: i32,
        alpha: f64,
        use_origin: bool,
    );
    fn set_view_matrix(&mut self, view: [f32; 16]);
    fn set_viewproj_matrix(&mut self, view: [f32; 16], proj: [f32; 16]);
    fn get_model_matrix(&self) -> [f32; 16];
    fn set_model_matrix(&mut self, model: [f32; 16]);
    fn mult_model_matrix(&mut self, model: [f32; 16]);
    fn set_projection_ortho(&mut self, x: f64, y: f64, w: f64, h: f64, angle: f64);
    fn set_projection_perspective(&mut self, x: f64, y: f64, w: f64, h: f64, angle: f64);
    fn set_view(
        &mut self,
        src_x: i32,
        src_y: i32,
        src_w: i32,
        src_h: i32,
        src_angle: f64,
        port_x: i32,
        port_y: i32,
        port_w: i32,
        port_h: i32,
    );
    fn flush_queue(&mut self);
    fn present(&mut self, window_width: u32, window_height: u32, scaling: Scaling);
    fn finish(&mut self, window_width: u32, window_height: u32, clear_colour: Colour);

    fn dump_sprite(&self, atlas_ref: &AtlasRef) -> Box<[u8]>;
    fn dump_sprite_part(&self, texture: &AtlasRef, part_x: i32, part_y: i32, part_w: i32, part_h: i32) -> Box<[u8]> {
        self.dump_sprite(&AtlasRef {
            atlas_id: texture.atlas_id,
            sprite_id: texture.sprite_id,
            w: part_w,
            h: part_h,
            x: texture.x + part_x,
            y: texture.y + part_y,
            origin_x: 0.0,
            origin_y: 0.0,
        })
    }
    fn get_blend_mode(&self) -> (BlendType, BlendType);
    fn set_blend_mode(&mut self, src: BlendType, dst: BlendType);
    fn get_pixel_interpolation(&self) -> bool;
    fn set_pixel_interpolation(&mut self, lerping: bool);
    fn get_texture_repeat(&self) -> bool;
    fn set_texture_repeat(&mut self, repeat: bool);

    fn get_pixels(&self, x: i32, y: i32, w: i32, h: i32) -> Box<[u8]>;
    fn dump_zbuffer(&self) -> Box<[f32]>;
    fn draw_raw_frame(
        &mut self,
        rgba: Box<[u8]>,
        zbuf: Box<[f32]>,
        fb_w: i32,
        fb_h: i32,
        window_w: u32,
        window_h: u32,
        scaling: Scaling,
    );

    fn dump_dynamic_textures(&self) -> Vec<Option<SavedTexture>>;
    fn upload_dynamic_textures(&mut self, textures: &[Option<SavedTexture>]);

    fn create_sprite_colour(&mut self, width: i32, height: i32, col: Colour) -> Result<AtlasRef, String>;
    fn create_surface(&mut self, w: i32, h: i32, has_zbuffer: bool) -> Result<AtlasRef, String>;
    fn set_target(&mut self, atlas_ref: &AtlasRef);
    fn reset_target(&mut self);

    fn get_texture_id(&mut self, atl_ref: &AtlasRef) -> i32;
    fn get_texture_from_id(&self, id: i32) -> Option<&AtlasRef>;

    fn get_sprite_count(&self) -> i32;
    fn set_sprite_count(&mut self, sprite_count: i32);

    fn draw_sprite_partial(
        &mut self,
        texture: &AtlasRef,
        part_x: f64,
        part_y: f64,
        part_w: f64,
        part_h: f64,
        x: f64,
        y: f64,
        xscale: f64,
        yscale: f64,
        angle: f64,
        colour: i32,
        alpha: f64,
    ) {
        self.draw_sprite_general(
            texture, part_x, part_y, part_w, part_h, x, y, xscale, yscale, angle, colour, colour, colour, colour,
            alpha, false,
        )
    }
    fn draw_sprite_tiled(
        &mut self,
        texture: &AtlasRef,
        mut x: f64,
        mut y: f64,
        xscale: f64,
        yscale: f64,
        colour: i32,
        alpha: f64,
        tile_end_x: Option<f64>,
        tile_end_y: Option<f64>,
    ) {
        let width = f64::from(texture.w) * xscale;
        let height = f64::from(texture.h) * yscale;

        if tile_end_x.is_some() {
            x = x.rem_euclid(width);
            if x > 0.0 {
                x -= width;
            }
        }
        if tile_end_y.is_some() {
            y = y.rem_euclid(height);
            if y > 0.0 {
                y -= height;
            }
        }

        let start_x = x;

        loop {
            loop {
                self.draw_sprite(texture, x, y, xscale, yscale, 0.0, colour, alpha);
                x += width;
                match tile_end_x {
                    Some(end_x) if x < end_x => (),
                    _ => break,
                }
            }
            x = start_x;
            y += height;
            match tile_end_y {
                Some(end_y) if y < end_y => (),
                _ => break,
            }
        }
    }

    fn draw_rectangle(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, colour: i32, alpha: f64);
    fn draw_rectangle_outline(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, colour: i32, alpha: f64);
    fn draw_rectangle_gradient(
        &mut self,
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        c1: i32,
        c2: i32,
        c3: i32,
        c4: i32,
        alpha: f64,
        outline: bool,
    );
    fn draw_point(&mut self, x: f64, y: f64, colour: i32, alpha: f64);
    fn draw_line(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, width: Option<f64>, c1: i32, c2: i32, alpha: f64);
    fn draw_triangle(
        &mut self,
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        x3: f64,
        y3: f64,
        c1: i32,
        c2: i32,
        c3: i32,
        alpha: f64,
        outline: bool,
    );
    fn draw_ellipse(&mut self, x: f64, y: f64, rad_x: f64, rad_y: f64, c1: i32, c2: i32, alpha: f64, outline: bool);
    fn draw_roundrect(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, c1: i32, c2: i32, alpha: f64, outline: bool);
    fn set_circle_precision(&mut self, prec: i32);
    fn get_circle_precision(&self) -> i32;
    fn reset_primitive_2d(&mut self, ptype: PrimitiveType, atlas_ref: Option<AtlasRef>);
    fn vertex_2d(&mut self, x: f64, y: f64, xtex: f64, ytex: f64, col: i32, alpha: f64);
    fn draw_primitive_2d(&mut self);
    fn get_primitive_2d(&self) -> PrimitiveBuilder;
    fn set_primitive_2d(&mut self, prim: PrimitiveBuilder);
    fn reset_primitive_3d(&mut self, ptype: PrimitiveType, atlas_ref: Option<AtlasRef>);
    fn vertex_3d(
        &mut self,
        x: f64,
        y: f64,
        z: f64,
        nx: f64,
        ny: f64,
        nz: f64,
        xtex: f64,
        ytex: f64,
        col: i32,
        alpha: f64,
    );
    fn draw_primitive_3d(&mut self);
    fn get_primitive_3d(&self) -> PrimitiveBuilder;
    fn set_primitive_3d(&mut self, prim: PrimitiveBuilder);
    fn extend_buffers(&self, buf: &mut VertexBuffer);
    fn draw_buffers(&mut self, atlas_ref: Option<AtlasRef>, buf: &VertexBuffer);
    fn clear_view(&mut self, colour: Colour, alpha: f64);
    fn clear_zbuf(&mut self);

    fn get_3d(&self) -> bool;
    fn set_3d(&mut self, use_3d: bool);
    fn get_depth(&self) -> f32;
    fn set_depth(&mut self, depth: f32);
    fn get_depth_test(&self) -> bool;
    fn set_depth_test(&mut self, depth_test: bool);
    fn get_write_depth(&self) -> bool;
    fn set_write_depth(&mut self, write_depth: bool);
    fn get_culling(&self) -> bool;
    fn set_culling(&mut self, culling: bool);
    fn get_perspective(&self) -> bool;
    fn set_perspective(&mut self, perspective: bool);
    fn get_fog(&self) -> Option<Fog>;
    fn set_fog(&mut self, fog: Option<Fog>);
    fn get_gouraud(&self) -> bool;
    fn set_gouraud(&mut self, gouraud: bool);
    fn get_lighting_enabled(&self) -> bool;
    fn set_lighting_enabled(&mut self, enabled: bool);
    fn get_ambient_colour(&self) -> i32;
    fn set_ambient_colour(&mut self, colour: i32);
    fn get_lights(&self) -> [(bool, Light); 8];
    fn set_light_enabled(&mut self, id: usize, enabled: bool);
    fn set_light(&mut self, id: usize, light: Light);
}

pub struct RendererOptions {
    pub size: (u32, u32),
    pub vsync: bool,
    pub interpolate_pixels: bool,
    pub normalize_normals: bool,
    pub zbuf_24: bool,
}

impl Default for RendererOptions {
    fn default() -> Self {
        RendererOptions {
            size: (8, 8),
            vsync: true,
            interpolate_pixels: false,
            normalize_normals: false,
            zbuf_24: false,
        }
    }
}

impl Renderer {
    pub fn new(backend: (), options: &RendererOptions, window: &Window, clear_colour: Colour) -> Result<Self, String> {
        Ok(Self(Box::new(match backend {
            () => opengl::RendererImpl::new(options, window, clear_colour)?,
        })))
    }

    pub fn max_texture_size(&self) -> u32 {
        self.0.max_texture_size()
    }

    pub fn push_atlases(&mut self, atl: AtlasBuilder) -> Result<(), String> {
        self.0.push_atlases(atl)
    }

    pub fn upload_sprite(
        &mut self,
        data: Box<[u8]>,
        width: i32,
        height: i32,
        origin_x: i32,
        origin_y: i32,
    ) -> Result<AtlasRef, String> {
        self.0.upload_sprite(data, width, height, origin_x, origin_y)
    }

    pub fn duplicate_sprite(&mut self, atlas_ref: &AtlasRef) -> Result<AtlasRef, String> {
        self.0.duplicate_sprite(atlas_ref)
    }

    pub fn delete_sprite(&mut self, atlas_ref: AtlasRef) {
        self.0.delete_sprite(atlas_ref)
    }

    pub fn set_vsync(&self, vsync: bool) {
        self.0.set_vsync(vsync)
    }

    pub fn get_vsync(&self) -> bool {
        self.0.get_vsync()
    }

    pub fn wait_vsync(&self) {
        self.0.wait_vsync()
    }

    pub fn draw_sprite(
        &mut self,
        texture: &AtlasRef,
        x: f64,
        y: f64,
        xscale: f64,
        yscale: f64,
        angle: f64,
        colour: i32,
        alpha: f64,
    ) {
        self.0.draw_sprite(texture, x, y, xscale, yscale, angle, colour, alpha)
    }

    pub fn draw_sprite_colour(
        &mut self,
        tex: &AtlasRef,
        x: f64,
        y: f64,
        xs: f64,
        ys: f64,
        ang: f64,
        col1: i32,
        col2: i32,
        col3: i32,
        col4: i32,
        alpha: f64,
    ) {
        self.0.draw_sprite_colour(tex, x, y, xs, ys, ang, col1, col2, col3, col4, alpha)
    }

    pub fn draw_sprite_general(
        &mut self,
        texture: &AtlasRef,
        part_x: f64,
        part_y: f64,
        part_w: f64,
        part_h: f64,
        x: f64,
        y: f64,
        xscale: f64,
        yscale: f64,
        angle: f64,
        col1: i32,
        col2: i32,
        col3: i32,
        col4: i32,
        alpha: f64,
        use_origin: bool,
    ) {
        self.0.draw_sprite_general(
            texture, part_x, part_y, part_w, part_h, x, y, xscale, yscale, angle, col1, col2, col3, col4, alpha,
            use_origin,
        )
    }

    pub fn set_view_matrix(&mut self, view: [f32; 16]) {
        self.0.set_view_matrix(view)
    }

    pub fn set_viewproj_matrix(&mut self, view: [f32; 16], proj: [f32; 16]) {
        self.0.set_viewproj_matrix(view, proj)
    }

    pub fn get_model_matrix(&self) -> [f32; 16] {
        self.0.get_model_matrix()
    }

    pub fn set_model_matrix(&mut self, model: [f32; 16]) {
        self.0.set_model_matrix(model)
    }

    pub fn mult_model_matrix(&mut self, model: [f32; 16]) {
        self.0.mult_model_matrix(model)
    }

    pub fn set_projection_ortho(&mut self, x: f64, y: f64, w: f64, h: f64, angle: f64) {
        self.0.set_projection_ortho(x, y, w, h, angle)
    }

    pub fn set_projection_perspective(&mut self, x: f64, y: f64, w: f64, h: f64, angle: f64) {
        self.0.set_projection_perspective(x, y, w, h, angle)
    }

    pub fn set_view(
        &mut self,
        src_x: i32,
        src_y: i32,
        src_w: i32,
        src_h: i32,
        src_angle: f64,
        port_x: i32,
        port_y: i32,
        port_w: i32,
        port_h: i32,
    ) {
        self.0.set_view(src_x, src_y, src_w, src_h, src_angle, port_x, port_y, port_w, port_h)
    }

    pub fn draw_sprite_partial(
        &mut self,
        texture: &AtlasRef,
        part_x: f64,
        part_y: f64,
        part_w: f64,
        part_h: f64,
        x: f64,
        y: f64,
        xscale: f64,
        yscale: f64,
        angle: f64,
        colour: i32,
        alpha: f64,
    ) {
        self.0.draw_sprite_partial(texture, part_x, part_y, part_w, part_h, x, y, xscale, yscale, angle, colour, alpha)
    }

    pub fn draw_sprite_tiled(
        &mut self,
        texture: &AtlasRef,
        x: f64,
        y: f64,
        xscale: f64,
        yscale: f64,
        colour: i32,
        alpha: f64,
        tile_end_x: Option<f64>,
        tile_end_y: Option<f64>,
    ) {
        self.0.draw_sprite_tiled(texture, x, y, xscale, yscale, colour, alpha, tile_end_x, tile_end_y)
    }

    pub fn draw_rectangle(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, colour: i32, alpha: f64) {
        self.0.draw_rectangle(x1, y1, x2, y2, colour, alpha)
    }

    pub fn draw_rectangle_outline(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, colour: i32, alpha: f64) {
        self.0.draw_rectangle_outline(x1, y1, x2, y2, colour, alpha)
    }

    pub fn draw_rectangle_gradient(
        &mut self,
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        c1: i32,
        c2: i32,
        c3: i32,
        c4: i32,
        alpha: f64,
        outline: bool,
    ) {
        self.0.draw_rectangle_gradient(x1, y1, x2, y2, c1, c2, c3, c4, alpha, outline)
    }

    pub fn draw_point(&mut self, x: f64, y: f64, colour: i32, alpha: f64) {
        self.0.draw_point(x, y, colour, alpha)
    }

    pub fn draw_line(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, width: Option<f64>, c1: i32, c2: i32, alpha: f64) {
        self.0.draw_line(x1, y1, x2, y2, width, c1, c2, alpha)
    }

    pub fn draw_triangle(
        &mut self,
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        x3: f64,
        y3: f64,
        c1: i32,
        c2: i32,
        c3: i32,
        alpha: f64,
        outline: bool,
    ) {
        self.0.draw_triangle(x1, y1, x2, y2, x3, y3, c1, c2, c3, alpha, outline)
    }

    pub fn draw_ellipse(
        &mut self,
        x: f64,
        y: f64,
        rad_x: f64,
        rad_y: f64,
        c1: i32,
        c2: i32,
        alpha: f64,
        outline: bool,
    ) {
        self.0.draw_ellipse(x, y, rad_x, rad_y, c1, c2, alpha, outline)
    }

    pub fn draw_roundrect(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, c1: i32, c2: i32, alpha: f64, outline: bool) {
        self.0.draw_roundrect(x1, y1, x2, y2, c1, c2, alpha, outline)
    }

    pub fn set_circle_precision(&mut self, prec: i32) {
        self.0.set_circle_precision(prec)
    }

    pub fn get_circle_precision(&self) -> i32 {
        self.0.get_circle_precision()
    }

    pub fn reset_primitive_2d(&mut self, ptype: PrimitiveType, atlas_ref: Option<AtlasRef>) {
        self.0.reset_primitive_2d(ptype, atlas_ref)
    }

    pub fn vertex_2d(&mut self, x: f64, y: f64, xtex: f64, ytex: f64, col: i32, alpha: f64) {
        self.0.vertex_2d(x, y, xtex, ytex, col, alpha)
    }

    pub fn draw_primitive_2d(&mut self) {
        self.0.draw_primitive_2d()
    }

    pub fn get_primitive_2d(&self) -> PrimitiveBuilder {
        self.0.get_primitive_2d()
    }

    pub fn set_primitive_2d(&mut self, prim: PrimitiveBuilder) {
        self.0.set_primitive_2d(prim)
    }

    pub fn reset_primitive_3d(&mut self, ptype: PrimitiveType, atlas_ref: Option<AtlasRef>) {
        self.0.reset_primitive_3d(ptype, atlas_ref)
    }

    pub fn vertex_3d(
        &mut self,
        x: f64,
        y: f64,
        z: f64,
        nx: f64,
        ny: f64,
        nz: f64,
        xtex: f64,
        ytex: f64,
        col: i32,
        alpha: f64,
    ) {
        self.0.vertex_3d(x, y, z, nx, ny, nz, xtex, ytex, col, alpha)
    }

    pub fn draw_primitive_3d(&mut self) {
        self.0.draw_primitive_3d()
    }

    pub fn get_primitive_3d(&self) -> PrimitiveBuilder {
        self.0.get_primitive_3d()
    }

    pub fn set_primitive_3d(&mut self, prim: PrimitiveBuilder) {
        self.0.set_primitive_3d(prim)
    }

    pub fn extend_buffers(&self, buf: &mut VertexBuffer) {
        self.0.extend_buffers(buf)
    }

    pub fn draw_buffers(&mut self, atlas_ref: Option<AtlasRef>, buf: &VertexBuffer) {
        self.0.draw_buffers(atlas_ref, buf)
    }

    pub fn dump_sprite(&self, atlas_ref: &AtlasRef) -> Box<[u8]> {
        self.0.dump_sprite(atlas_ref)
    }

    pub fn dump_sprite_part(
        &self,
        texture: &AtlasRef,
        part_x: i32,
        part_y: i32,
        part_w: i32,
        part_h: i32,
    ) -> Box<[u8]> {
        self.0.dump_sprite_part(texture, part_x, part_y, part_w, part_h)
    }

    pub fn resize_framebuffer(&mut self, width: u32, height: u32) {
        self.0.resize_framebuffer(width, height)
    }

    pub fn get_pixels(&self, x: i32, y: i32, w: i32, h: i32) -> Box<[u8]> {
        self.0.get_pixels(x, y, w, h)
    }

    pub fn dump_zbuffer(&self) -> Box<[f32]> {
        self.0.dump_zbuffer()
    }

    pub fn draw_raw_frame(
        &mut self,
        rgba: Box<[u8]>,
        zbuf: Box<[f32]>,
        fb_w: i32,
        fb_h: i32,
        window_w: u32,
        window_h: u32,
        scaling: Scaling,
    ) {
        self.0.draw_raw_frame(rgba, zbuf, fb_w, fb_h, window_w, window_h, scaling)
    }

    pub fn dump_dynamic_textures(&self) -> Vec<Option<SavedTexture>> {
        self.0.dump_dynamic_textures()
    }

    pub fn upload_dynamic_textures(&mut self, textures: &[Option<SavedTexture>]) {
        self.0.upload_dynamic_textures(textures)
    }

    pub fn create_sprite_colour(&mut self, width: i32, height: i32, col: Colour) -> Result<AtlasRef, String> {
        self.0.create_sprite_colour(width, height, col)
    }

    pub fn create_surface(&mut self, w: i32, h: i32, has_zbuffer: bool) -> Result<AtlasRef, String> {
        self.0.create_surface(w, h, has_zbuffer)
    }

    pub fn set_target(&mut self, atlas_ref: &AtlasRef) {
        self.0.set_target(atlas_ref)
    }

    pub fn reset_target(&mut self) {
        self.0.reset_target()
    }

    pub fn get_texture_id(&mut self, atl_ref: &AtlasRef) -> i32 {
        self.0.get_texture_id(atl_ref)
    }

    pub fn get_texture_from_id(&self, id: i32) -> Option<&AtlasRef> {
        self.0.get_texture_from_id(id)
    }

    pub fn get_sprite_count(&self) -> i32 {
        self.0.get_sprite_count()
    }

    pub fn set_sprite_count(&mut self, sprite_count: i32) {
        self.0.set_sprite_count(sprite_count)
    }

    pub fn get_blend_mode(&self) -> (BlendType, BlendType) {
        self.0.get_blend_mode()
    }

    pub fn set_blend_mode(&mut self, src: BlendType, dst: BlendType) {
        self.0.set_blend_mode(src, dst)
    }

    pub fn get_pixel_interpolation(&self) -> bool {
        self.0.get_pixel_interpolation()
    }

    pub fn set_pixel_interpolation(&mut self, lerping: bool) {
        self.0.set_pixel_interpolation(lerping)
    }

    pub fn get_texture_repeat(&self) -> bool {
        self.0.get_texture_repeat()
    }

    pub fn set_texture_repeat(&mut self, repeat: bool) {
        self.0.set_texture_repeat(repeat)
    }

    pub fn flush_queue(&mut self) {
        self.0.flush_queue()
    }

    pub fn clear_view(&mut self, colour: Colour, alpha: f64) {
        self.0.clear_view(colour, alpha)
    }

    pub fn clear_zbuf(&mut self) {
        self.0.clear_zbuf()
    }

    pub fn get_3d(&self) -> bool {
        self.0.get_3d()
    }

    pub fn set_3d(&mut self, use_3d: bool) {
        self.0.set_3d(use_3d)
    }

    pub fn get_depth(&self) -> f32 {
        self.0.get_depth()
    }

    pub fn set_depth(&mut self, depth: f32) {
        self.0.set_depth(depth)
    }

    pub fn get_depth_test(&self) -> bool {
        self.0.get_depth_test()
    }

    pub fn set_depth_test(&mut self, depth_test: bool) {
        self.0.set_depth_test(depth_test)
    }

    pub fn get_write_depth(&self) -> bool {
        self.0.get_write_depth()
    }

    pub fn set_write_depth(&mut self, write_depth: bool) {
        self.0.set_write_depth(write_depth)
    }

    pub fn get_culling(&self) -> bool {
        self.0.get_culling()
    }

    pub fn set_culling(&mut self, culling: bool) {
        self.0.set_culling(culling)
    }

    pub fn get_perspective(&self) -> bool {
        self.0.get_perspective()
    }

    pub fn set_perspective(&mut self, perspective: bool) {
        self.0.set_perspective(perspective)
    }

    pub fn get_fog(&self) -> Option<Fog> {
        self.0.get_fog()
    }

    pub fn set_fog(&mut self, fog: Option<Fog>) {
        self.0.set_fog(fog)
    }

    pub fn get_gouraud(&self) -> bool {
        self.0.get_gouraud()
    }

    pub fn set_gouraud(&mut self, gouraud: bool) {
        self.0.set_gouraud(gouraud)
    }

    pub fn get_lighting_enabled(&self) -> bool {
        self.0.get_lighting_enabled()
    }

    pub fn set_lighting_enabled(&mut self, enabled: bool) {
        self.0.set_lighting_enabled(enabled)
    }

    pub fn get_ambient_colour(&self) -> i32 {
        self.0.get_ambient_colour()
    }

    pub fn set_ambient_colour(&mut self, colour: i32) {
        self.0.set_ambient_colour(colour)
    }

    pub fn get_lights(&self) -> [(bool, Light); 8] {
        self.0.get_lights()
    }

    pub fn set_light_enabled(&mut self, id: usize, enabled: bool) {
        self.0.set_light_enabled(id, enabled)
    }

    pub fn set_light(&mut self, id: usize, light: Light) {
        self.0.set_light(id, light)
    }

    pub fn present(&mut self, window_width: u32, window_height: u32, scaling: Scaling) {
        self.0.present(window_width, window_height, scaling)
    }

    pub fn finish(&mut self, window_width: u32, window_height: u32, clear_colour: Colour) {
        self.0.finish(window_width, window_height, clear_colour)
    }
}

/// Multiply two mat4's together
fn mat4mult(m1: [f32; 16], m2: [f32; 16]) -> [f32; 16] {
    [
        (m1[0] * m2[0]) + (m1[1] * m2[4]) + (m1[2] * m2[8]) + (m1[3] * m2[12]),
        (m1[0] * m2[1]) + (m1[1] * m2[5]) + (m1[2] * m2[9]) + (m1[3] * m2[13]),
        (m1[0] * m2[2]) + (m1[1] * m2[6]) + (m1[2] * m2[10]) + (m1[3] * m2[14]),
        (m1[0] * m2[3]) + (m1[1] * m2[7]) + (m1[2] * m2[11]) + (m1[3] * m2[15]),
        (m1[4] * m2[0]) + (m1[5] * m2[4]) + (m1[6] * m2[8]) + (m1[7] * m2[12]),
        (m1[4] * m2[1]) + (m1[5] * m2[5]) + (m1[6] * m2[9]) + (m1[7] * m2[13]),
        (m1[4] * m2[2]) + (m1[5] * m2[6]) + (m1[6] * m2[10]) + (m1[7] * m2[14]),
        (m1[4] * m2[3]) + (m1[5] * m2[7]) + (m1[6] * m2[11]) + (m1[7] * m2[15]),
        (m1[8] * m2[0]) + (m1[9] * m2[4]) + (m1[10] * m2[8]) + (m1[11] * m2[12]),
        (m1[8] * m2[1]) + (m1[9] * m2[5]) + (m1[10] * m2[9]) + (m1[11] * m2[13]),
        (m1[8] * m2[2]) + (m1[9] * m2[6]) + (m1[10] * m2[10]) + (m1[11] * m2[14]),
        (m1[8] * m2[3]) + (m1[9] * m2[7]) + (m1[10] * m2[11]) + (m1[11] * m2[15]),
        (m1[12] * m2[0]) + (m1[13] * m2[4]) + (m1[14] * m2[8]) + (m1[15] * m2[12]),
        (m1[12] * m2[1]) + (m1[13] * m2[5]) + (m1[14] * m2[9]) + (m1[15] * m2[13]),
        (m1[12] * m2[2]) + (m1[13] * m2[6]) + (m1[14] * m2[10]) + (m1[15] * m2[14]),
        (m1[12] * m2[3]) + (m1[13] * m2[7]) + (m1[14] * m2[11]) + (m1[15] * m2[15]),
    ]
}
