use rect_packer::DensePacker;
use std::cmp::max;

pub struct AtlasBuilder {
    max_size: i32,
    packers: Vec<(DensePacker, i32, i32)>, // max_x, max_y
}

#[derive(Clone)]
pub struct AtlasRef {
    pub atlas_id: u32,

    pub w: i32,
    pub h: i32,
    pub x: i32,
    pub y: i32,
}

impl AtlasBuilder {
    pub fn new(max_wh: i32) -> Self {
        AtlasBuilder {
            max_size: max_wh,
            packers: Vec::with_capacity(1),
        }
    }

    pub fn add(&mut self, w: i32, h: i32) -> AtlasRef {
        for (i, (pk, max_x, max_y)) in self.packers.iter_mut().enumerate() {
            if let Some(rect) = pk.pack(w as _, h as _, false) {
                *max_x = max(*max_x, rect.x + rect.width);
                *max_y = max(*max_y, rect.y + rect.height);
                return AtlasRef {
                    atlas_id: i as _,
                    w: rect.width,
                    h: rect.height,
                    x: rect.x,
                    y: rect.y,
                };
            }
        }

        assert!(w <= self.max_size);
        assert!(h <= self.max_size);
        self.packers
            .push((DensePacker::new(self.max_size, self.max_size), 0, 0));
        self.add(w, h)
    }

    pub fn into_frames(self) -> Vec<(i32, i32)> {
        self.packers
            .iter()
            .map(|(_, x, y)| {
                let next_pow2 = |n: f32| 2i32.pow(n.log2().ceil() as _);
                (next_pow2(*x as _) as _, next_pow2(*y as _) as _)
            })
            .collect()
    }
}
