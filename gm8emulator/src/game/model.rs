use crate::math::Real;
use gmio::render::{PrimitiveType, VertexBuffer};

#[derive(Clone, Debug, Serialize, Deserialize)]
enum Command {
    Begin(PrimitiveType),
    End,
    Vertex { pos: [Real; 3], normal: [Real; 3], tex_coord: [Real; 2] },
    VertexColour { pos: [Real; 3], normal: [Real; 3], tex_coord: [Real; 2], col: Option<(i32, Real)> },
    Block { pos1: [Real; 3], pos2: [Real; 3], tex_repeat: [Real; 2] },
    Cylinder { pos1: [Real; 3], pos2: [Real; 3], tex_repeat: [Real; 2], closed: bool, steps: i32 },
    Cone { pos1: [Real; 3], pos2: [Real; 3], tex_repeat: [Real; 2], closed: bool, steps: i32 },
    Ellipsoid { pos1: [Real; 3], pos2: [Real; 3], tex_repeat: [Real; 2], steps: i32 },
    Wall { pos1: [Real; 3], pos2: [Real; 3], tex_repeat: [Real; 2] },
    Floor { pos1: [Real; 3], pos2: [Real; 3], tex_repeat: [Real; 2] },
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Model {
    commands: Vec<Command>,
    cache: Option<VertexBuffer>,
}
