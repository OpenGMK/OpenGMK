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

fn complete_bbox(data: Box<[bool]>, width: u32, height: u32) -> Collider {
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
    Collider { width, height, bbox_left, bbox_right, bbox_top, bbox_bottom, data }
}

pub fn make_colliders(frames: &[image::RgbaImage], sepmasks: bool) -> Vec<Collider> {
    // only supports precise colliders with 0 tolerance rn
    let tolerance = 0;
    let width = frames[0].width();
    let height = frames[0].height();
    if sepmasks {
        frames
            .iter()
            .map(|f| {
                complete_bbox(
                    f.pixels().map(|p| p[3] > tolerance).collect::<Vec<_>>().into_boxed_slice(),
                    width,
                    height,
                )
            })
            .collect()
    } else {
        let mut data = vec![false; (width * height) as usize];
        // merge pixels
        for f in frames {
            for y in 0..height.min(f.height()) {
                for x in 0..width.min(f.width()) {
                    if f.get_pixel(x, y)[3] > tolerance {
                        data[(y * width + x) as usize] = true;
                    }
                }
            }
        }
        vec![complete_bbox(data.into_boxed_slice(), width, height)]
    }
}
