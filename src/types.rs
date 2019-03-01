pub struct BoundingBox {
    pub width: u32,
    pub height: u32,

    pub top: u32,
    pub bottom: u32,
    pub left: u32,
    pub right: u32,
}

pub struct Dimensions {
    pub width: u32,
    pub height: u32,
}

pub struct CollisionMap {
    pub bounds: BoundingBox,
    pub data: Box<[u8]>,
    pub version: Version,
}

pub struct Point {
    pub x: u32,
    pub y: u32,
}

pub type Version = u32;
