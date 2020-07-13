use crate::game::string::RCStr;
use gmio::render::AtlasRef;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Sprite {
    pub name: RCStr,
    pub frames: Vec<Frame>,
    pub colliders: Vec<Collider>,
    pub width: u32,
    pub height: u32,
    pub origin_x: i32,
    pub origin_y: i32,
    pub per_frame_colliders: bool,
    pub bbox_left: u32,
    pub bbox_right: u32,
    pub bbox_top: u32,
    pub bbox_bottom: u32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Frame {
    pub width: u32,
    pub height: u32,
    pub atlas_ref: AtlasRef,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Collider {
    pub width: u32,
    pub height: u32,
    pub bbox_left: u32,
    pub bbox_right: u32,
    pub bbox_top: u32,
    pub bbox_bottom: u32,
    pub data: Box<[bool]>,
}

pub fn make_colliders(frames: &[image::RgbaImage]) -> Vec<Collider> {
    // only supports non-separated precise colliders with 0 tolerance rn
    let tolerance = 0;
    let width = frames[0].width();
    let height = frames[0].height();
    let mut data = vec![false; (width * height) as usize];
    // extract pixels
    for f in frames {
        for (i, px) in f.pixels().enumerate() {
            if px[3] > tolerance {
                data[i] = true;
            }
        }
    }
    // calculate bbox values
    let mut bbox_left = width - 1;
    let mut bbox_right = 0;
    let mut bbox_top = height - 1;
    let mut bbox_bottom = 0;
    for x in 0..width {
        for y in 0..height {
            if data[(y * width + x) as usize] {
                if x < bbox_left {
                    bbox_left = x;
                }
                if x > bbox_right {
                    bbox_right = x;
                }
                if y < bbox_top {
                    bbox_top = y;
                }
                if y > bbox_bottom {
                    bbox_bottom = y;
                }
            }
        }
    }
    vec![Collider { width, height, bbox_left, bbox_right, bbox_top, bbox_bottom, data: data.into_boxed_slice() }]
}
