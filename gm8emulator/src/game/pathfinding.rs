use crate::{
    asset::path::{Path, Point},
    instance::Instance,
    math::Real,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct PotentialStepSettings {
    pub max_rotation: Real,
    pub rotate_step: Real,
    pub check_distance: Real,
    pub rotate_on_spot: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MpGrid {
    mpgrid: Vec<Vec<i32>>,
    pub(crate) left: i32,
    pub(crate) top: i32,
    pub(crate) hcells: usize,
    pub(crate) vcells: usize,
    pub(crate) cellwidth: i32,
    pub(crate) cellheight: i32,
}

#[derive(Debug)]
pub enum Error {
    NonexistentStructure(i32),
}

impl From<Error> for String {
    fn from(e: Error) -> Self {
        match e {
            Error::NonexistentStructure(id) => format!("mp_grid with index {} does not exist", id),
        }
    }
}

impl Default for PotentialStepSettings {
    fn default() -> Self {
        PotentialStepSettings {
            max_rotation: 30.0.into(),
            rotate_step: 10.0.into(),
            check_distance: 3.0.into(),
            rotate_on_spot: true,
        }
    }
}

impl MpGrid {
    pub fn new(left: i32, top: i32, hcells: usize, vcells: usize, cellwidth: i32, cellheight: i32) -> Self {
        let mpgrid = vec![vec![0; vcells]; hcells];
        Self { mpgrid, left, top, hcells, vcells, cellwidth, cellheight }
    }

    // This will panic on OOB, so make sure you check bounds before calling
    pub fn get(&self, x: usize, y: usize) -> i32 {
        self.mpgrid[x][y]
    }

    // This will panic on OOB, so make sure you check bounds before calling
    pub fn set(&mut self, x: usize, y: usize, val: i32) {
        self.mpgrid[x][y] = val;
    }
}

/// Performs a step straight towards the given destination, stopping when a wall is reached.
/// Returns true if the goal was reached, and false otherwise.
pub fn linear_step(x: Real, y: Real, step_size: Real, instance: &Instance, coll: impl Fn() -> bool) -> bool {
    let old_x = instance.x.get();
    let old_y = instance.y.get();
    if old_x == x && old_y == y {
        return true
    }
    let xdist = x - old_x;
    let ydist = y - old_y;
    let distance = xdist.into_inner().hypot(ydist.into());
    let (new_x, new_y) = if distance <= step_size.into() {
        (x, y)
    } else {
        (old_x + step_size * xdist / distance.into(), old_y + step_size * ydist / distance.into())
    };
    instance.x.set(new_x);
    instance.y.set(new_y);
    instance.bbox_is_stale.set(true);
    if coll() {
        instance.x.set(old_x);
        instance.y.set(old_y);
        instance.bbox_is_stale.set(true);
    } else {
        instance.set_direction(-ydist.arctan2(xdist).to_degrees());
    }
    distance <= step_size.into()
}

/// Performs a step towards the given destination. When a wall is reached, it will try to go around it.
/// Returns true if the goal was reached, and false otherwise.
pub fn potential_step(
    x: Real,
    y: Real,
    step_size: Real,
    settings: &PotentialStepSettings,
    instance: &Instance,
    coll: impl Fn() -> bool,
) -> bool {
    let old_x = instance.x.get();
    let old_y = instance.y.get();
    if old_x == x && old_y == y {
        return true
    }
    // try to move into the given position, if it's colliding move back, return true if successful
    let try_move = |x, y| {
        instance.x.set(x);
        instance.y.set(y);
        instance.bbox_is_stale.set(true);
        if coll() {
            instance.x.set(old_x);
            instance.y.set(old_y);
            instance.bbox_is_stale.set(true);
            false
        } else {
            true
        }
    };
    let distance = (old_x - x).into_inner().hypot((old_y - y).into());
    let direction = (old_y - y).arctan2(x - old_x).to_degrees().rem_euclid(360.into());
    if distance > step_size.into_inner() {
        let mut res = false;
        // start looking right at the destination, then check to the right and left in steps
        for angle_diff in std::iter::successors(Some(Real::from(0)), |&angle_diff| {
            if angle_diff <= 0.into() {
                Some(-angle_diff + settings.rotate_step).filter(|&x| x < 180.into())
            } else {
                Some(-angle_diff)
            }
        }) {
            let direction = (direction - angle_diff).rem_euclid(360.into());
            // ignore if this would cause a bigger rotation than the maximum
            let dir_change = (direction - instance.direction.get()).rem_euclid(360.into());
            if dir_change <= settings.max_rotation || dir_change >= Real::from(360) - settings.max_rotation {
                let dir_rad = direction.to_radians();
                let x_step = step_size * dir_rad.cos();
                let y_step = -step_size * dir_rad.sin();
                // check a bit further ahead before checking the position we will be moving into
                if try_move(old_x + x_step * settings.check_distance, old_y + y_step * settings.check_distance) {
                    if try_move(old_x + x_step, old_y + y_step) {
                        instance.direction.set(direction);
                        res = true;
                        break
                    } else {
                        // the position will be ahead, so put it back where it was
                        instance.x.set(old_x);
                        instance.y.set(old_y);
                        // no need to set bbox_stale as this was set in try_move already
                    }
                }
            }
        }
        if !res && settings.rotate_on_spot {
            instance.direction.set(instance.direction.get() + settings.max_rotation);
        }
        res
    } else {
        let res = try_move(x, y);
        if res {
            instance.direction.set(direction);
        }
        res
    }
}

#[derive(PartialEq, Eq)]
pub enum PathGenResult {
    Done,
    Failed,
    NotDone,
}

pub fn make_path(inst: &Instance, path: &mut Path, func: impl Fn(&Instance) -> PathGenResult) -> bool {
    let (old_x, old_y, old_direction) = (inst.x.get(), inst.y.get(), inst.direction.get());
    path.curve = false;
    path.closed = false;
    path.points.clear();
    let mut result = PathGenResult::NotDone;
    while result == PathGenResult::NotDone {
        result = func(inst);
        path.points.push(Point { x: inst.x.get(), y: inst.y.get(), speed: 100.into() });
    }
    path.update();
    inst.x.set(old_x);
    inst.y.set(old_y);
    inst.set_direction(old_direction);
    inst.bbox_is_stale.set(true);
    result == PathGenResult::Done
}
