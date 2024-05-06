use crate::{gml, math::Real, render::atlas::AtlasRef, util};
use image::{Pixel, RgbaImage};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Sprite {
    pub name: gml::String,
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

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct BoundingBox {
    pub left: u32,
    pub right: u32,
    pub top: u32,
    pub bottom: u32,
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

pub enum ColliderShape {
    Rectangle,
    Ellipse,
    Diamond,
}

pub fn process_image(image: &mut RgbaImage, removeback: bool, smooth: bool, fill_transparent: bool) {
    if fill_transparent {
        // if the image is completely transparent, make it completely opaque
        if image.pixels().all(|p| p[3] == 0) {
            for px in image.pixels_mut() {
                px[3] = 255;
            }
        }
    }
    if removeback {
        // remove background colour
        let bottom_left = image.get_pixel(0, image.height() - 1).to_rgb();
        for px in image.pixels_mut() {
            if px.to_rgb() == bottom_left {
                px[3] = 0;
            }
        }
    }
    if smooth {
        // smooth
        for y in 0..image.height() {
            for x in 0..image.width() {
                // if pixel is transparent
                if image.get_pixel(x, y)[3] == 0 {
                    // for all surrounding pixels
                    for y in y.saturating_sub(1)..(y + 2).min(image.height()) {
                        for x in x.saturating_sub(1)..(x + 2).min(image.width()) {
                            // subtract 32 if possible
                            if image.get_pixel(x, y)[3] >= 32 {
                                image.get_pixel_mut(x, y)[3] -= 32;
                            }
                        }
                    }
                }
            }
        }
    }
    if removeback {
        // make lerping less ugly
        for y in 0..image.height() {
            for x in 0..image.width() {
                if image.get_pixel(x, y)[3] == 0 {
                    let (sx, sy) = if x > 0 && image.get_pixel(x - 1, y)[3] != 0 {
                        (x - 1, y)
                    } else if x < image.width() - 1 && image.get_pixel(x + 1, y)[3] != 0 {
                        (x + 1, y)
                    } else if y > 0 && image.get_pixel(x, y - 1)[3] != 0 {
                        (x, y - 1)
                    } else if y < image.height() - 1 && image.get_pixel(x, y + 1)[3] != 0 {
                        (x, y + 1)
                    } else {
                        continue
                    };
                    let src = *image.get_pixel(sx, sy);
                    let dst = image.get_pixel_mut(x, y);
                    dst[0] = src[0];
                    dst[1] = src[1];
                    dst[2] = src[2];
                }
            }
        }
    }
}

/// Calculates bounding box values for a given frame.
/// The algorithm doesn't check more pixels than it needs to.
fn make_bbox(coll: impl Fn(u32, u32) -> bool, frame_width: u32, frame_height: u32) -> BoundingBox {
    let mut left = frame_width - 1;
    let mut right = 0;
    let mut top = frame_height - 1;
    let mut bottom = 0;
    // Set bbox_left and bbox_top to the leftmost column with collision, and the highest pixel within that column.
    for x in 0..frame_width {
        if let Some(y) = (0..frame_height).find(|&y| coll(x, y)) {
            left = x;
            top = y;
            break
        }
    }
    // Set bbox_top to the highest pixel in the remaining columns, if there's one above the one we already found.
    if let Some(y) = (0..top).find(|&y| ((left + 1)..frame_width).any(|x| coll(x, y))) {
        top = y;
    }
    // Set bbox_right and bbox_bottom to the rightmost column with collision, and the lowest pixel within that column,
    // ignoring the rows and columns which are known to be empty.
    for x in (left..frame_width).rev() {
        if let Some(y) = (top..frame_height).rfind(|&y| coll(x, y)) {
            right = x;
            bottom = y;
            break
        }
    }
    // Set bbox_bottom to the lowest pixel between bbox_left and bbox_right, if there's one below the one we found.
    if let Some(y) = ((bottom + 1)..frame_height).rev().find(|&y| (left..(right + 1)).any(|x| coll(x, y))) {
        bottom = y;
    }
    BoundingBox { left, right, top, bottom }
}

/// Creates a collider from the given collision data and dimensions, giving it an appropriate bounding box.
fn complete_bbox(data: Box<[bool]>, width: u32, height: u32) -> Collider {
    let bbox = make_bbox(|x, y| data[(y * width + x) as usize], width, height);
    Collider {
        width,
        height,
        bbox_left: bbox.left,
        bbox_right: bbox.right,
        bbox_top: bbox.top,
        bbox_bottom: bbox.bottom,
        data,
    }
}

pub fn make_colliders_precise(frames: &[RgbaImage], tolerance: u8, sepmasks: bool) -> Vec<Collider> {
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

pub fn make_colliders_shaped(
    frames: &[RgbaImage],
    tolerance: u8,
    sepmasks: bool,
    bbox: Option<BoundingBox>,
    shape: Option<ColliderShape>,
) -> Vec<Collider> {
    let width = frames[0].width();
    let height = frames[0].height();
    let bbox_iterator: Box<dyn Iterator<Item = BoundingBox>> = if let Some(bbox) = bbox {
        Box::new(std::iter::once(bbox))
    } else {
        let bbox_iterator = frames.iter().map(|f| make_bbox(|x, y| f.get_pixel(x, y)[3] > tolerance, width, height));
        if sepmasks {
            Box::new(bbox_iterator)
        } else {
            Box::new(std::iter::once(bbox_iterator.fold(
                BoundingBox { left: width - 1, right: 0, top: height - 1, bottom: 0 },
                |acc, new| BoundingBox {
                    left: acc.left.min(new.left),
                    right: acc.right.max(new.right),
                    top: acc.top.min(new.top),
                    bottom: acc.bottom.max(new.bottom),
                },
            )))
        }
    };
    bbox_iterator
        .map(|bbox| {
            let mut data = vec![false; (width * height) as usize].into_boxed_slice();
            match shape {
                None => (),
                Some(ColliderShape::Rectangle) => {
                    for y in bbox.top..bbox.bottom + 1 {
                        for x in bbox.left..bbox.right + 1 {
                            data[(y * width + x) as usize] = true;
                        }
                    }
                },
                Some(ColliderShape::Ellipse) => {
                    let xcenter = f64::from(bbox.right - bbox.left) / 2.0;
                    let xrad = xcenter - f64::from(bbox.left) + 0.5; // GM8 adds 0.5, no idea why
                    let ycenter = f64::from(bbox.bottom - bbox.top) / 2.0;
                    let yrad = xcenter - f64::from(bbox.top) + 0.5;
                    for y in bbox.top..bbox.bottom + 1 {
                        for x in bbox.left..bbox.right + 1 {
                            let x_scaled: f64 = (f64::from(x) - xcenter) / xrad;
                            let y_scaled: f64 = (f64::from(y) - ycenter) / yrad;
                            data[(y * width + x) as usize] = x_scaled * x_scaled + y_scaled * y_scaled < 1.0;
                        }
                    }
                },
                Some(ColliderShape::Diamond) => {
                    let xcenter = f64::from(bbox.right - bbox.left) / 2.0;
                    let xrad = xcenter - f64::from(bbox.left) + 0.5;
                    let ycenter = f64::from(bbox.bottom - bbox.top) / 2.0;
                    let yrad = xcenter - f64::from(bbox.top) + 0.5;
                    for y in bbox.top..bbox.bottom + 1 {
                        for x in bbox.left..bbox.right + 1 {
                            let x_scaled: f64 = (f64::from(x) - xcenter) / xrad;
                            let y_scaled: f64 = (f64::from(y) - ycenter) / yrad;
                            // the IDE uses <= here (only for diamonds)
                            data[(y * width + x) as usize] = x_scaled.abs() + y_scaled.abs() < 1.0;
                        }
                    }
                },
            }
            Collider {
                width,
                height,
                bbox_left: bbox.left,
                bbox_right: bbox.right,
                bbox_top: bbox.top,
                bbox_bottom: bbox.bottom,
                data,
            }
        })
        .collect()
}

// used for adding frames to sprites
pub fn scale(input: &mut RgbaImage, width: u32, height: u32) {
    if input.dimensions() != (width, height) {
        let xscale = Real::from(width) / input.width().into();
        let yscale = Real::from(height) / input.height().into();
        let mut output_vec = Vec::with_capacity((width * height * 4) as usize);
        for y in 0..height {
            for x in 0..width {
                let px = input
                    .get_pixel((Real::from(x) / xscale).floor().to_u32(), (Real::from(y) / yscale).floor().to_u32());
                // this makes lerping uglier but it's accurate to GM8
                if px[3] > 0 {
                    output_vec.extend_from_slice(px.channels());
                } else {
                    output_vec.extend_from_slice(&[0, 0, 0, 0]);
                }
            }
        }
        *input = RgbaImage::from_vec(width, height, output_vec).unwrap();
    }
}

impl Sprite {
    fn get_frame_index(&self, image_idx: isize) -> Option<usize> {
        image_idx.checked_rem_euclid(self.frames.len() as isize).map(|x| x as usize)
    }

    pub fn get_frame(&self, image_index: i32) -> Option<&Frame> {
        match self.get_frame_index(image_index as isize) {
            Some(frame_idx) => self.frames.get(frame_idx),
            None => None,
        }
    }

    pub fn get_atlas_ref(&self, image_index: i32) -> Option<AtlasRef> {
        Some(self.get_frame(image_index)?.atlas_ref)
    }
}

impl Collider {
    // collision_point but it checks the collider's precise hitbox only, no AABB
    pub fn check_collision_point_precise(&self, x: i32, y: i32, inst_x: i32, inst_y: i32, origin_x: i32, origin_y: i32, xscale: Real, yscale: Real, angle_sin: f64, angle_cos: f64) -> bool {
        let mut x = Real::from(x - inst_x);
        let mut y = Real::from(y - inst_y);
        util::rotate_around_center(x.as_mut_ref(), y.as_mut_ref(), angle_sin, angle_cos);
        let x = (Real::from(origin_x) + (x / xscale)).floor().to_i32();
        let y = (Real::from(origin_y) + (y / yscale)).floor().to_i32();
        x >= self.bbox_left as i32
            && y >= self.bbox_top as i32
            && x <= self.bbox_right as i32
            && y <= self.bbox_bottom as i32
            && self.data.get((y as usize * self.width as usize) + x as usize).copied().unwrap_or(false)
    }
}
