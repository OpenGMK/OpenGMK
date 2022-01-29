use gm8exe::asset::sprite::{Frame, Sprite};

pub struct GmkCollision {
    pub bbox_top: u32,
    pub bbox_bottom: u32,
    pub bbox_left: u32,
    pub bbox_right: u32,
    pub shape: Shape,
    pub alpha_tolerance: u32,
}

pub enum Shape {
    Precise = 0,
    Rectangle = 1,
    Disk = 2,
    Diamond = 3,
}

// Resolves an exe-format sprite's collision map to a GmkCollision struct
// Returns None if the provided list is empty
pub fn resolve_map(sprite: &Sprite) -> Option<GmkCollision> {
    let maps = &sprite.colliders;
    // Return None if empty
    if sprite.frames.is_empty() || maps.is_empty() {
        return None
    }

    // Return None if there are less colliders than there should be
    let n_colliders = if sprite.per_frame_colliders { sprite.frames.len() } else { 1 };
    if sprite.colliders.len() < n_colliders {
        return None
    }

    // Each map has its own bounds, so we need to find out the outmost bounds
    let left = maps.iter().min_by(|x, y| x.bbox_left.cmp(&y.bbox_left))?.bbox_left;
    let right = maps.iter().max_by(|x, y| x.bbox_right.cmp(&y.bbox_right))?.bbox_right;
    let bottom = maps.iter().max_by(|x, y| x.bbox_bottom.cmp(&y.bbox_bottom))?.bbox_bottom;
    let top = maps.iter().min_by(|x, y| x.bbox_top.cmp(&y.bbox_top))?.bbox_top;

    // Little helper function for later
    fn alpha_at(frame: &Frame, x: u32, y: u32) -> Option<u8> {
        frame.data.get(((y * frame.width + x) * 4 + 3) as usize).copied()
    }

    // The various bits of data we want to collect:
    let mut all_have_collision = true;
    let mut lowest_alpha_with_col = 255u8;
    let mut highest_alpha_no_col = 0u8;

    // Iterate through each pixel in the collision rectangle
    for y in top..=bottom {
        for x in left..=right {
            // Check if there are any pixels with no collision
            for map in maps {
                all_have_collision &= map.data.get((y * map.width + x) as usize)?;
            }

            // Check relationships between collision and pixel alpha
            if sprite.per_frame_colliders {
                // Per frame colliders - easy to check each map against each frame one-by-one
                for (frame, map) in sprite.frames.iter().zip(sprite.colliders.iter()) {
                    let has_collision = *map.data.get((y * map.width + x) as usize)?;
                    let alpha = alpha_at(&frame, x, y)?;
                    if has_collision && (alpha < lowest_alpha_with_col) {
                        lowest_alpha_with_col = alpha;
                    } else if !has_collision && (alpha > highest_alpha_no_col) {
                        highest_alpha_no_col = alpha;
                    }
                }
            } else {
                // Not per-frame colliders - the highest alpha value from each pixel is used
                let map = sprite.colliders.first()?;
                let alpha = sprite
                    .frames
                    .iter()
                    .map(|f| alpha_at(f, x, y).map(|a| (f, a)))
                    .flatten()
                    .max_by(|(_, a1), (_, a2)| a1.cmp(&a2))?
                    .1;

                let has_collision = *map.data.get((y * map.width + x) as usize)?;
                if has_collision && (alpha < lowest_alpha_with_col) {
                    lowest_alpha_with_col = alpha;
                } else if !has_collision && (alpha > highest_alpha_no_col) {
                    highest_alpha_no_col = alpha;
                }
            }
        }
    }

    // Decide on shape
    let (shape, alpha_tolerance) = if all_have_collision {
        (Shape::Rectangle, 0)
    } else if lowest_alpha_with_col > highest_alpha_no_col {
        (Shape::Precise, highest_alpha_no_col)
    } else {
        // Decide between circle or diamond using the % of pixels which have collision
        // Note: I use the first map here because all maps are guaranteed to be the same
        // for all shapes except Precise.
        let map = maps.first()?;
        let collision_count = map.data.iter().filter(|x| **x).count();
        let ratio = (collision_count as f64)
            / (((map.bbox_right + 1 - map.bbox_left) * (map.bbox_bottom + 1 - map.bbox_top)) as f64);
        // Highest diamond ratio I've seen is 0.5454.. Lowest disk ratio I've seen is 0.777..
        if ratio < 0.65 { (Shape::Diamond, 0) } else { (Shape::Disk, 0) }
    };

    Some(GmkCollision {
        bbox_top: top,
        bbox_bottom: bottom,
        bbox_left: left,
        bbox_right: right,
        shape,
        alpha_tolerance: u32::from(alpha_tolerance),
    })
}
