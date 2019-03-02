use crate::types::*;

pub struct GMBackground {
    pub name: String,
    pub size: Dimensions,
    pub data: Option<Box<[u8]>>,
}

pub struct GMPath {
    pub name: String,
    pub kind: u32, // TODO: enumify
    pub closed: bool,
    pub precision: u32, // TOOD: why is this an int
    pub points: Vec<GMPathPoint>,
}

pub struct GMPathPoint {
    pub x: f64,
    pub y: f64,
    pub speed: f64,
}

pub struct GMSound {
    /// Asset name
    pub name: String,

    /// Any of: normal, background, 3d, use multimedia player
    /// I should make this an enum eventually. TODO
    pub kind: u32,

    pub file_type: String,
    pub file_name: String,

    /// This is optional because the associated data doesn't need to exist. Fantastic.
    pub file_data: Option<Box<[u8]>>,

    /// Volume - Between 0 and 1, although the editor only allows as low as 0.3
    pub volume: f64,

    /// 3D Pan - Between -1 and 1 (L <-> R)
    pub pan: f64,

    /// TODO: I have no idea what this does.
    pub preload: bool,
}

pub struct GMSprite {
    pub name: String,
    pub size: Dimensions,
    pub origin: Point,
    pub frame_count: u32,
    pub frames: Option<Vec<Box<[u8]>>>,
    pub colliders: Option<Vec<CollisionMap>>,
    pub per_frame_colliders: bool,
}
