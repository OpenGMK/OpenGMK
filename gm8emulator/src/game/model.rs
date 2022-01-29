use crate::{
    math::Real,
    render::{atlas::AtlasRef, PrimitiveType, Renderer, VertexBuffer},
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Command {
    Begin(PrimitiveType),
    End,
    Vertex { pos: [Real; 3], normal: [Real; 3], tex_coord: [Real; 2] },
    VertexColour { pos: [Real; 3], normal: [Real; 3], tex_coord: [Real; 2], col: (i32, Real) },
    Block { pos1: [Real; 3], pos2: [Real; 3], tex_repeat: [Real; 2] },
    Cylinder { pos1: [Real; 3], pos2: [Real; 3], tex_repeat: [Real; 2], closed: bool, steps: i32 },
    Cone { pos1: [Real; 3], pos2: [Real; 3], tex_repeat: [Real; 2], closed: bool, steps: i32 },
    Ellipsoid { pos1: [Real; 3], pos2: [Real; 3], tex_repeat: [Real; 2], steps: i32 },
    Wall { pos1: [Real; 3], pos2: [Real; 3], tex_repeat: [Real; 2] },
    Floor { pos1: [Real; 3], pos2: [Real; 3], tex_repeat: [Real; 2] },
}

impl Command {
    // for use with d3d_model_save
    pub fn to_line(&self) -> (i32, [Real; 10]) {
        let mut args = [Real::from(0); 10];
        let cmd = match self {
            Command::Begin(pt) => {
                args[0] = match pt {
                    PrimitiveType::PointList => 1,
                    PrimitiveType::LineList => 2,
                    PrimitiveType::LineStrip => 3,
                    PrimitiveType::TriList => 4,
                    PrimitiveType::TriStrip => 5,
                    PrimitiveType::TriFan => 6,
                }
                .into();
                0
            },
            Command::End => 1,
            Command::Vertex { pos, normal, tex_coord } => {
                args[..3].copy_from_slice(pos);
                args[3..6].copy_from_slice(normal);
                args[6..8].copy_from_slice(tex_coord);
                8
            },
            Command::VertexColour { pos, normal, tex_coord, col: (col, alpha) } => {
                args[..3].copy_from_slice(pos);
                args[3..6].copy_from_slice(normal);
                args[6..8].copy_from_slice(tex_coord);
                args[8] = (*col).into();
                args[9] = *alpha;
                9
            },
            Command::Block { pos1, pos2, tex_repeat } => {
                args[..3].copy_from_slice(pos1);
                args[3..6].copy_from_slice(pos2);
                args[6..8].copy_from_slice(tex_repeat);
                10
            },
            Command::Cylinder { pos1, pos2, tex_repeat, closed, steps } => {
                args[..3].copy_from_slice(pos1);
                args[3..6].copy_from_slice(pos2);
                args[6..8].copy_from_slice(tex_repeat);
                args[8] = (*closed as i32).into();
                args[9] = (*steps).into();
                11
            },
            Command::Cone { pos1, pos2, tex_repeat, closed, steps } => {
                args[..3].copy_from_slice(pos1);
                args[3..6].copy_from_slice(pos2);
                args[6..8].copy_from_slice(tex_repeat);
                args[8] = (*closed as i32).into();
                args[9] = (*steps).into();
                12
            },
            Command::Ellipsoid { pos1, pos2, tex_repeat, steps } => {
                args[..3].copy_from_slice(pos1);
                args[3..6].copy_from_slice(pos2);
                args[6..8].copy_from_slice(tex_repeat);
                args[8] = (*steps).into();
                13
            },
            Command::Wall { pos1, pos2, tex_repeat } => {
                args[..3].copy_from_slice(pos1);
                args[3..6].copy_from_slice(pos2);
                args[6..8].copy_from_slice(tex_repeat);
                14
            },
            Command::Floor { pos1, pos2, tex_repeat } => {
                args[..3].copy_from_slice(pos1);
                args[3..6].copy_from_slice(pos2);
                args[6..8].copy_from_slice(tex_repeat);
                15
            },
        };
        (cmd, args)
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Model {
    pub old_draw_colour: Option<(i32, f64)>,
    pub commands: Vec<Command>,
    pub cache: Option<VertexBuffer>,
}

pub fn draw_block<F>(
    renderer: &mut Renderer,
    atlas_ref: Option<AtlasRef>,
    primitive_draw: &mut F,
    x1: f64,
    y1: f64,
    z1: f64,
    x2: f64,
    y2: f64,
    z2: f64,
    hrepeat: f64,
    vrepeat: f64,
    col: i32,
    alpha: f64,
) where
    F: FnMut(&mut Renderer),
{
    let old_repeat = renderer.get_texture_repeat();
    renderer.set_texture_repeat(true);
    renderer.reset_primitive_3d(PrimitiveType::TriFan, atlas_ref);
    renderer.vertex_3d(x1, y1, z1, 0.0, 0.0, -1.0, 0.0, 0.0, col, alpha);
    renderer.vertex_3d(x1, y2, z1, 0.0, 0.0, -1.0, 0.0, vrepeat, col, alpha);
    renderer.vertex_3d(x2, y2, z1, 0.0, 0.0, -1.0, hrepeat, vrepeat, col, alpha);
    renderer.vertex_3d(x2, y1, z1, 0.0, 0.0, -1.0, hrepeat, 0.0, col, alpha);
    primitive_draw(renderer);
    renderer.reset_primitive_3d(PrimitiveType::TriFan, atlas_ref);
    renderer.vertex_3d(x1, y1, z2, 0.0, 0.0, 1.0, 0.0, 0.0, col, alpha);
    renderer.vertex_3d(x2, y1, z2, 0.0, 0.0, 1.0, hrepeat, 0.0, col, alpha);
    renderer.vertex_3d(x2, y2, z2, 0.0, 0.0, 1.0, hrepeat, vrepeat, col, alpha);
    renderer.vertex_3d(x1, y2, z2, 0.0, 0.0, 1.0, 0.0, vrepeat, col, alpha);
    primitive_draw(renderer);
    renderer.reset_primitive_3d(PrimitiveType::TriFan, atlas_ref);
    renderer.vertex_3d(x1, y2, z1, 0.0, 1.0, 0.0, 0.0, 0.0, col, alpha);
    renderer.vertex_3d(x1, y2, z2, 0.0, 1.0, 0.0, 0.0, vrepeat, col, alpha);
    renderer.vertex_3d(x2, y2, z2, 0.0, 1.0, 0.0, hrepeat, vrepeat, col, alpha);
    renderer.vertex_3d(x2, y2, z1, 0.0, 1.0, 0.0, hrepeat, 0.0, col, alpha);
    primitive_draw(renderer);
    renderer.reset_primitive_3d(PrimitiveType::TriFan, atlas_ref);
    renderer.vertex_3d(x2, y2, z1, 1.0, 0.0, 0.0, 0.0, 0.0, col, alpha);
    renderer.vertex_3d(x2, y2, z2, 1.0, 0.0, 0.0, 0.0, vrepeat, col, alpha);
    renderer.vertex_3d(x2, y1, z2, 1.0, 0.0, 0.0, hrepeat, vrepeat, col, alpha);
    renderer.vertex_3d(x2, y1, z1, 1.0, 0.0, 0.0, hrepeat, 0.0, col, alpha);
    primitive_draw(renderer);
    renderer.reset_primitive_3d(PrimitiveType::TriFan, atlas_ref);
    renderer.vertex_3d(x2, y1, z1, 0.0, -1.0, 0.0, 0.0, 0.0, col, alpha);
    renderer.vertex_3d(x2, y1, z2, 0.0, -1.0, 0.0, 0.0, vrepeat, col, alpha);
    renderer.vertex_3d(x1, y1, z2, 0.0, -1.0, 0.0, hrepeat, vrepeat, col, alpha);
    renderer.vertex_3d(x1, y1, z1, 0.0, -1.0, 0.0, hrepeat, 0.0, col, alpha);
    primitive_draw(renderer);
    renderer.reset_primitive_3d(PrimitiveType::TriFan, atlas_ref);
    renderer.vertex_3d(x1, y1, z1, -1.0, 0.0, 0.0, 0.0, 0.0, col, alpha);
    renderer.vertex_3d(x1, y1, z2, -1.0, 0.0, 0.0, 0.0, vrepeat, col, alpha);
    renderer.vertex_3d(x1, y2, z2, -1.0, 0.0, 0.0, hrepeat, vrepeat, col, alpha);
    renderer.vertex_3d(x1, y2, z1, -1.0, 0.0, 0.0, hrepeat, 0.0, col, alpha);
    primitive_draw(renderer);
    renderer.set_texture_repeat(old_repeat);
}

pub fn draw_cylinder<F>(
    renderer: &mut Renderer,
    atlas_ref: Option<AtlasRef>,
    primitive_draw: &mut F,
    x1: f64,
    y1: f64,
    z1: f64,
    x2: f64,
    y2: f64,
    z2: f64,
    hrepeat: f64,
    vrepeat: f64,
    closed: bool,
    steps: i32,
    col: i32,
    alpha: f64,
) where
    F: FnMut(&mut Renderer),
{
    let steps = steps.max(3).min(128);
    let trigs = (0..=steps)
        .map(|i| {
            let angle = Real::from(i * 360).to_radians() / steps.into();
            (angle.cos().into_inner(), angle.sin().into_inner())
        })
        .collect::<Vec<_>>();
    let xcenter = (x2 + x1) / 2.0;
    let ycenter = (y2 + y1) / 2.0;
    let xrad = (x2 - x1) / 2.0;
    let yrad = (y2 - y1) / 2.0;
    let hrepeat_step = hrepeat / f64::from(steps);
    let old_repeat = renderer.get_texture_repeat();
    renderer.set_texture_repeat(true);
    if closed {
        renderer.reset_primitive_3d(PrimitiveType::TriFan, atlas_ref);
        renderer.vertex_3d(xcenter, ycenter, z2, 0.0, 0.0, 1.0, 0.0, vrepeat, col, alpha);
        for (cos, sin) in &trigs {
            renderer.vertex_3d(xcenter + xrad * cos, ycenter + yrad * sin, z2, 0.0, 0.0, 1.0, 0.0, vrepeat, col, alpha);
        }
        primitive_draw(renderer);
    }
    renderer.reset_primitive_3d(PrimitiveType::TriStrip, atlas_ref);
    for (i, (cos, sin)) in trigs.iter().copied().enumerate() {
        renderer.vertex_3d(
            xcenter + xrad * cos,
            ycenter + yrad * sin,
            z2,
            cos,
            sin,
            0.0,
            hrepeat_step * i as f64,
            vrepeat,
            col,
            alpha,
        );
        renderer.vertex_3d(
            xcenter + xrad * cos,
            ycenter + yrad * sin,
            z1,
            cos,
            sin,
            0.0,
            hrepeat_step * i as f64,
            0.0,
            col,
            alpha,
        );
    }
    primitive_draw(renderer);
    if closed {
        renderer.reset_primitive_3d(PrimitiveType::TriFan, atlas_ref);
        renderer.vertex_3d(xcenter, ycenter, z1, 0.0, 0.0, -1.0, 0.0, 0.0, col, alpha);
        for (cos, sin) in trigs.iter().rev() {
            renderer.vertex_3d(xcenter + xrad * cos, ycenter + yrad * sin, z1, 0.0, 0.0, -1.0, 0.0, 0.0, col, alpha);
        }
        primitive_draw(renderer);
    }
    renderer.set_texture_repeat(old_repeat);
}

pub fn draw_cone<F>(
    renderer: &mut Renderer,
    atlas_ref: Option<AtlasRef>,
    primitive_draw: &mut F,
    x1: f64,
    y1: f64,
    z1: f64,
    x2: f64,
    y2: f64,
    z2: f64,
    hrepeat: f64,
    vrepeat: f64,
    closed: bool,
    steps: i32,
    col: i32,
    alpha: f64,
) where
    F: FnMut(&mut Renderer),
{
    let steps = steps.max(3).min(128);
    let trigs = (0..=steps)
        .map(|i| {
            let angle = Real::from(i * 360).to_radians() / steps.into();
            (angle.cos().into_inner(), angle.sin().into_inner())
        })
        .collect::<Vec<_>>();
    let xcenter = (x2 + x1) / 2.0;
    let ycenter = (y2 + y1) / 2.0;
    let xrad = (x2 - x1) / 2.0;
    let yrad = (y2 - y1) / 2.0;
    let hrepeat_step = hrepeat / f64::from(steps);
    let old_repeat = renderer.get_texture_repeat();
    renderer.set_texture_repeat(true);
    renderer.reset_primitive_3d(PrimitiveType::TriStrip, atlas_ref);
    for (i, (cos, sin)) in trigs.iter().copied().enumerate() {
        renderer.vertex_3d(xcenter, ycenter, z2, 0.0, 0.0, 1.0, hrepeat_step * i as f64, vrepeat, col, alpha);
        renderer.vertex_3d(
            xcenter + xrad * cos,
            ycenter + yrad * sin,
            z1,
            cos,
            sin,
            0.0,
            hrepeat_step * i as f64,
            0.0,
            col,
            alpha,
        );
    }
    primitive_draw(renderer);
    if closed {
        renderer.reset_primitive_3d(PrimitiveType::TriFan, atlas_ref);
        renderer.vertex_3d(xcenter, ycenter, z1, 0.0, 0.0, -1.0, 0.0, 0.0, col, alpha);
        for (cos, sin) in trigs.iter().rev() {
            renderer.vertex_3d(xcenter + xrad * cos, ycenter + yrad * sin, z1, 0.0, 0.0, -1.0, 0.0, 0.0, col, alpha);
        }
        primitive_draw(renderer);
    }
    renderer.set_texture_repeat(old_repeat);
}

pub fn draw_ellipsoid<F>(
    renderer: &mut Renderer,
    atlas_ref: Option<AtlasRef>,
    primitive_draw: &mut F,
    x1: f64,
    y1: f64,
    z1: f64,
    x2: f64,
    y2: f64,
    z2: f64,
    hrepeat: f64,
    vrepeat: f64,
    steps: i32,
    col: i32,
    alpha: f64,
) where
    F: FnMut(&mut Renderer),
{
    let steps = steps.max(3).min(128);
    let trigs = (0..=steps)
        .map(|i| {
            let angle = Real::from(i * 360).to_radians() / steps.into();
            (angle.cos().into_inner(), angle.sin().into_inner())
        })
        .collect::<Vec<_>>();
    let xcenter = (x2 + x1) / 2.0;
    let ycenter = (y2 + y1) / 2.0;
    let zcenter = (z2 + z1) / 2.0;
    let xrad = (x2 - x1) / 2.0;
    let yrad = (y2 - y1) / 2.0;
    let zrad = (z2 - z1) / 2.0;
    let hrepeat_step = hrepeat / f64::from(steps);
    let old_repeat = renderer.get_texture_repeat();
    renderer.set_texture_repeat(true);
    let row_count = (steps + 1) / 2;
    let vrepeat_step = vrepeat / f64::from(row_count);
    for row in 0..row_count {
        let row1_angle = Real::from(row * 360).to_radians() / steps.into();
        let row2_angle = Real::from((row + 1) * 360).to_radians() / steps.into();
        let (row1_cos, row1_sin) = (row1_angle.cos().into_inner(), row1_angle.sin().into_inner());
        let (row2_cos, row2_sin) = (row2_angle.cos().into_inner(), row2_angle.sin().into_inner());
        renderer.reset_primitive_3d(PrimitiveType::TriStrip, atlas_ref);
        for (i, (cos, sin)) in trigs.iter().copied().enumerate() {
            renderer.vertex_3d(
                xcenter + xrad * cos * row1_sin,
                ycenter + yrad * sin * row1_sin,
                zcenter + zrad * row1_cos,
                cos * row1_sin,
                sin * row1_sin,
                row1_cos,
                hrepeat_step * i as f64,
                vrepeat_step * f64::from(row),
                col,
                alpha,
            );
            renderer.vertex_3d(
                xcenter + xrad * cos * row2_sin,
                ycenter + yrad * sin * row2_sin,
                zcenter + zrad * row2_cos,
                cos * row2_sin,
                sin * row2_sin,
                row2_cos,
                hrepeat_step * i as f64,
                vrepeat_step * f64::from(row + 1),
                col,
                alpha,
            );
        }
        primitive_draw(renderer);
    }
    renderer.set_texture_repeat(old_repeat);
}

pub fn draw_wall<F>(
    renderer: &mut Renderer,
    atlas_ref: Option<AtlasRef>,
    primitive_draw: &mut F,
    x1: f64,
    y1: f64,
    z1: f64,
    x2: f64,
    y2: f64,
    z2: f64,
    hrepeat: f64,
    vrepeat: f64,
    col: i32,
    alpha: f64,
) where
    F: FnMut(&mut Renderer),
{
    // set texture repeat outside the if block so it gets set to true if ny is 0 (yes, really)
    let old_repeat = renderer.get_texture_repeat();
    renderer.set_texture_repeat(true);
    let diag_length = (x2 - x1).hypot(y2 - y1);
    if diag_length != 0.0 {
        let nx = (y2 - y1) / diag_length;
        let ny = -(x2 - x1) / diag_length;
        renderer.reset_primitive_3d(PrimitiveType::TriFan, atlas_ref);
        renderer.vertex_3d(x1, y1, z1, nx, ny, 0.0, 0.0, 0.0, col, alpha);
        renderer.vertex_3d(x2, y2, z1, nx, ny, 0.0, hrepeat, 0.0, col, alpha);
        renderer.vertex_3d(x2, y2, z2, nx, ny, 0.0, hrepeat, vrepeat, col, alpha);
        renderer.vertex_3d(x1, y1, z2, nx, ny, 0.0, 0.0, vrepeat, col, alpha);
        primitive_draw(renderer);
        renderer.set_texture_repeat(old_repeat);
    }
}

pub fn draw_floor<F>(
    renderer: &mut Renderer,
    atlas_ref: Option<AtlasRef>,
    primitive_draw: &mut F,
    x1: f64,
    y1: f64,
    z1: f64,
    x2: f64,
    y2: f64,
    z2: f64,
    hrepeat: f64,
    vrepeat: f64,
    col: i32,
    alpha: f64,
) where
    F: FnMut(&mut Renderer),
{
    let old_repeat = renderer.get_texture_repeat();
    renderer.set_texture_repeat(true);
    renderer.reset_primitive_3d(PrimitiveType::TriFan, atlas_ref);
    renderer.vertex_3d(x1, y1, z1, 0.0, 0.0, 1.0, 0.0, 0.0, col, alpha);
    renderer.vertex_3d(x1, y2, z1, 0.0, 0.0, 1.0, 0.0, vrepeat, col, alpha);
    renderer.vertex_3d(x2, y2, z2, 0.0, 0.0, 1.0, hrepeat, vrepeat, col, alpha);
    renderer.vertex_3d(x2, y1, z2, 0.0, 0.0, 1.0, hrepeat, 0.0, col, alpha);
    primitive_draw(renderer);
    renderer.set_texture_repeat(old_repeat);
}
