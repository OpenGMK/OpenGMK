use crate::math::Real;
use gmio::render::{PrimitiveType, VertexBuffer};
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

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Model {
    pub old_draw_colour: Option<(i32, f64)>,
    pub commands: Vec<Command>,
    pub cache: Option<VertexBuffer>,
}
