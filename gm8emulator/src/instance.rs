use crate::{
    asset::{Object, Sprite},
    gml::{InstanceVariable, Value},
    math::Real,
    types::ID,
    util,
};
use serde::{Deserialize, Serialize};
use std::{
    cell::{Cell, RefCell},
    collections::{HashMap, HashSet},
    f64,
    rc::Rc,
};

// Default in GameMaker 8
const BBOX_DEFAULT: i32 = -100000;
// Rust can't represent this many decimal places yet I think. In GM8 it's a TBYTE definition
const PI: f64 = 3.1415926535897932380;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum InstanceState {
    Active,
    Inactive,
    Deleted,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Instance {
    pub state: Cell<InstanceState>,
    pub id: Cell<ID>,
    pub object_index: Cell<ID>,
    pub solid: Cell<bool>,
    pub visible: Cell<bool>,
    pub persistent: Cell<bool>,
    pub depth: Cell<Real>,
    pub sprite_index: Cell<i32>,
    pub image_alpha: Cell<Real>,
    pub image_blend: Cell<i32>,
    pub image_index: Cell<Real>,
    pub image_speed: Cell<Real>,
    pub image_xscale: Cell<Real>,
    pub image_yscale: Cell<Real>,
    pub image_angle: Cell<Real>,
    pub mask_index: Cell<i32>,
    pub direction: Cell<Real>,
    pub friction: Cell<Real>,
    pub gravity: Cell<Real>,
    pub gravity_direction: Cell<Real>,
    pub hspeed: Cell<Real>,
    pub vspeed: Cell<Real>,
    pub speed: Cell<Real>,
    pub x: Cell<Real>,
    pub y: Cell<Real>,
    pub xprevious: Cell<Real>,
    pub yprevious: Cell<Real>,
    pub xstart: Cell<Real>,
    pub ystart: Cell<Real>,
    pub path_index: Cell<i32>,
    pub path_position: Cell<Real>, // Normalized from 0 to 1
    pub path_positionprevious: Cell<Real>,
    pub path_speed: Cell<Real>,
    pub path_scale: Cell<Real>,
    pub path_orientation: Cell<Real>,
    pub path_endaction: Cell<i32>, // https://docs.yoyogames.com/source/dadiospice/002_reference/paths/path_start.html
    pub path_xstart: Cell<Real>,
    pub path_ystart: Cell<Real>,
    pub path_pointspeed: Cell<Real>, // TODO remove
    pub timeline_index: Cell<i32>,
    pub timeline_running: Cell<bool>,
    pub timeline_speed: Cell<Real>,
    pub timeline_position: Cell<Real>,
    pub timeline_loop: Cell<bool>,

    pub bbox_top: Cell<i32>,
    pub bbox_left: Cell<i32>,
    pub bbox_right: Cell<i32>,
    pub bbox_bottom: Cell<i32>,
    pub bbox_is_stale: Cell<bool>,

    pub fields: RefCell<HashMap<usize, Field>>,
    pub alarms: RefCell<HashMap<u32, i32>>,

    pub parents: Rc<RefCell<HashSet<i32>>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Field {
    Single(Value),
    Array(HashMap<u32, Value>),
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DummyFieldHolder {
    pub fields: HashMap<usize, Field>,
    pub vars: HashMap<InstanceVariable, Field>,
}

impl Instance {
    pub fn new(id: ID, x: Real, y: Real, object_index: i32, object: &Object) -> Self {
        Self::new_ext(
            id,
            x,
            y,
            object_index,
            Some(object),
            Real::from(1),
            Real::from(1),
            0xFFFFFF,
            Real::from(1),
            Real::from(0),
        )
    }

    pub fn new_dummy(object: Option<&Object>) -> Self {
        Self::new_ext(
            0,
            Real::from(0),
            Real::from(0),
            0,
            object,
            Real::from(1),
            Real::from(1),
            0xFFFFFF,
            Real::from(1),
            Real::from(0),
        )
    }

    pub fn new_ext(
        id: ID,
        x: Real,
        y: Real,
        object_index: i32,
        object: Option<&Object>,
        xscale: Real,
        yscale: Real,
        blend: i32,
        alpha: Real,
        angle: Real,
    ) -> Self {
        Self {
            id: Cell::new(id),
            x: Cell::new(x),
            y: Cell::new(y),
            object_index: Cell::new(object_index),
            solid: Cell::new(object.map(|x| x.solid).unwrap_or(false)),
            visible: Cell::new(object.map(|x| x.visible).unwrap_or(false)),
            persistent: Cell::new(object.map(|x| x.persistent).unwrap_or(false)),
            depth: Cell::new(object.map(|x| x.depth.into()).unwrap_or(Real::from(0.0))),
            sprite_index: Cell::new(object.map(|x| x.sprite_index).unwrap_or(0)),
            image_blend: Cell::new(blend),
            image_alpha: Cell::new(alpha),
            image_angle: Cell::new(angle),
            mask_index: Cell::new(object.map(|x| x.mask_index).unwrap_or(0)),
            parents: object.map(|x| x.parents.clone()).unwrap_or_default(),
            xprevious: Cell::new(x),
            yprevious: Cell::new(y),
            xstart: Cell::new(x),
            ystart: Cell::new(y),
            state: Cell::new(InstanceState::Active),
            image_index: Cell::new(Real::from(0.0)),
            image_speed: Cell::new(Real::from(1.0)),
            image_xscale: Cell::new(xscale),
            image_yscale: Cell::new(yscale),
            direction: Cell::new(Real::from(0.0)),
            gravity: Cell::new(Real::from(0.0)),
            gravity_direction: Cell::new(Real::from(270.0)),
            hspeed: Cell::new(Real::from(0.0)),
            vspeed: Cell::new(Real::from(0.0)),
            speed: Cell::new(Real::from(0.0)),
            friction: Cell::new(Real::from(0.0)),
            path_index: Cell::new(-1),
            path_position: Cell::new(Real::from(0.0)),
            path_positionprevious: Cell::new(Real::from(0.0)),
            path_speed: Cell::new(Real::from(0.0)),
            path_scale: Cell::new(Real::from(1.0)),
            path_orientation: Cell::new(Real::from(0.0)),
            path_endaction: Cell::new(0),
            path_xstart: Cell::new(Real::from(0.0)),
            path_ystart: Cell::new(Real::from(0.0)),
            path_pointspeed: Cell::new(Real::from(0.0)),
            timeline_index: Cell::new(-1),
            timeline_running: Cell::new(false),
            timeline_speed: Cell::new(Real::from(1.0)),
            timeline_position: Cell::new(Real::from(0.0)),
            timeline_loop: Cell::new(false),
            bbox_top: Cell::new(BBOX_DEFAULT),
            bbox_left: Cell::new(BBOX_DEFAULT),
            bbox_right: Cell::new(BBOX_DEFAULT),
            bbox_bottom: Cell::new(BBOX_DEFAULT),
            bbox_is_stale: Cell::new(true),
            fields: RefCell::new(HashMap::new()),
            alarms: RefCell::new(HashMap::new()),
        }
    }

    // Sets direction, also updating hspeed and vspeed
    pub fn set_direction(&self, direction: Real) {
        self.direction.set(direction.rem_euclid(Real::from(360.0)));
        self.update_hvspeed()
    }

    // Sets speed, also updating hspeed and vspeed
    pub fn set_speed(&self, speed: Real) {
        self.speed.set(speed);
        self.update_hvspeed()
    }

    // Sets speed and direction, also updating hspeed and vspeed
    pub fn set_speed_direction(&self, speed: Real, direction: Real) {
        self.speed.set(speed);
        self.direction.set(direction.rem_euclid(Real::from(360.0)));
        self.update_hvspeed();
    }

    // Sets hspeed, also updating direction and speed
    pub fn set_hspeed(&self, hspeed: Real) {
        if self.hspeed.get() != hspeed {
            self.hspeed.set(hspeed);
            self.update_speed_direction()
        }
    }

    // Sets vspeed, also updating direction and speed
    pub fn set_vspeed(&self, vspeed: Real) {
        if self.vspeed.get() != vspeed {
            self.vspeed.set(vspeed);
            self.update_speed_direction()
        }
    }

    // Sets hspeed and vspeed, also updating direction and speed
    pub fn set_hvspeed(&self, hspeed: Real, vspeed: Real) {
        self.hspeed.set(hspeed);
        self.vspeed.set(vspeed);
        self.update_speed_direction()
    }

    // Sets hspeed and vspeed based on direction and speed
    fn update_hvspeed(&self) {
        let round_threshold = Real::from(0.0001); // The fudge-factor used by GM8
        let hspeed = self.direction.get().to_radians().cos() * self.speed.get();
        let vspeed = -self.direction.get().to_radians().sin() * self.speed.get();
        let rounded_hspeed = Real::from(hspeed.round());
        let rounded_vspeed = Real::from(vspeed.round());

        if (rounded_hspeed - hspeed).abs() < round_threshold {
            self.hspeed.set(rounded_hspeed);
        } else {
            self.hspeed.set(hspeed);
        }

        if (rounded_vspeed - vspeed).abs() < round_threshold {
            self.vspeed.set(rounded_vspeed);
        } else {
            self.vspeed.set(vspeed);
        }
    }

    // Sets direction and speed based on hspeed and vspeed
    fn update_speed_direction(&self) {
        self.direction.set((-self.vspeed.get()).arctan2(self.hspeed.get()).to_degrees().rem_euclid(Real::from(360.0)));
        self.speed.set((self.hspeed.get() * self.hspeed.get() + self.vspeed.get() * self.vspeed.get()).sqrt());
    }

    // Updates the bbox variables if they're stale, otherwise does nothing
    pub fn update_bbox(&self, sprite: Option<&Sprite>) {
        // Do nothing if bbox isn't stale
        if self.bbox_is_stale.get() {
            // Also do nothing if the given Sprite is None
            if let Some(sprite) = sprite {
                // Get coordinates of top-left and bottom-right corners of the collider at self's x and y,
                // taking image scale (but not angle) into account
                let x = self.x.get();
                let y = self.y.get();
                let xscale = self.image_xscale.get();
                let yscale = self.image_yscale.get();
                let mut top_left_x =
                    (x - (Real::from(sprite.origin_x) * xscale)) + (Real::from(sprite.bbox_left) * xscale);
                let mut top_left_y =
                    (y - (Real::from(sprite.origin_y) * yscale)) + (Real::from(sprite.bbox_top) * yscale);
                let mut bottom_right_x =
                    top_left_x + (Real::from(sprite.bbox_right + 1 - sprite.bbox_left) * xscale) - Real::from(1.0);
                let mut bottom_right_y =
                    top_left_y + (Real::from(sprite.bbox_bottom + 1 - sprite.bbox_top) * yscale) - Real::from(1.0);

                // Make sure left/right and top/bottom are the right way around
                if xscale <= Real::from(0.0) {
                    std::mem::swap(&mut top_left_x, &mut bottom_right_x);
                }
                if yscale <= Real::from(0.0) {
                    std::mem::swap(&mut top_left_y, &mut bottom_right_y);
                }

                // Copy values for the other two corners (top-right, bottom-left)...
                let mut top_right_x = bottom_right_x;
                let mut top_right_y = top_left_y;
                let mut bottom_left_x = top_left_x;
                let mut bottom_left_y = bottom_right_y;

                // Rotate these points
                let angle = -self.image_angle.get().to_radians();
                let sin = angle.sin().into_inner();
                let cos = angle.cos().into_inner();
                util::rotate_around(top_left_x.as_mut_ref(), top_left_y.as_mut_ref(), x.into(), y.into(), sin, cos);
                util::rotate_around(top_right_x.as_mut_ref(), top_right_y.as_mut_ref(), x.into(), y.into(), sin, cos);
                util::rotate_around(
                    bottom_left_x.as_mut_ref(),
                    bottom_left_y.as_mut_ref(),
                    x.into(),
                    y.into(),
                    sin,
                    cos,
                );
                util::rotate_around(
                    bottom_right_x.as_mut_ref(),
                    bottom_right_y.as_mut_ref(),
                    x.into(),
                    y.into(),
                    sin,
                    cos,
                );

                // Set left to whichever x is lowest, right to whichever x is highest,
                // top to whichever y is lowest, and bottom to whichever y is highest.
                self.bbox_left.set(top_left_x.min(top_right_x.min(bottom_left_x.min(bottom_right_x))).round().to_i32());
                self.bbox_right
                    .set(top_left_x.max(top_right_x.max(bottom_left_x.max(bottom_right_x))).round().to_i32());
                self.bbox_top.set(top_left_y.min(top_right_y.min(bottom_left_y.min(bottom_right_y))).round().to_i32());
                self.bbox_bottom
                    .set(top_left_y.max(top_right_y.max(bottom_left_y.max(bottom_right_y))).round().to_i32());
            } else {
                // No valid collider provided - set default values and return
                self.bbox_top.set(BBOX_DEFAULT);
                self.bbox_left.set(BBOX_DEFAULT);
                self.bbox_right.set(BBOX_DEFAULT);
                self.bbox_bottom.set(BBOX_DEFAULT);
            }

            // Indicate bbox is no longer stale
            self.bbox_is_stale.set(false);
        }
    }

    #[inline]
    pub fn is_active(&self) -> bool {
        self.state.get() == InstanceState::Active
    }
}

impl Field {
    pub fn new(index: u32, value: Value) -> Self {
        match index {
            0 => Self::Single(value),
            i => {
                let mut array = HashMap::new();
                array.insert(i, value);
                Self::Array(array)
            },
        }
    }

    pub fn get(&self, index: u32) -> Option<Value> {
        match (self, index) {
            (Self::Single(v), 0) => Some(v.clone()),
            (Self::Array(m), i) => match m.get(&i) {
                Some(v) => Some(v.clone()),
                None => {
                    if m.iter().any(|(k, _)| *k > i && *k < (((i / 32000) + 1) * 32000)) {
                        Some(Value::Real(Real::from(0.0)))
                    } else {
                        None
                    }
                },
            },
            _ => None,
        }
    }

    pub fn set(&mut self, index: u32, value: Value) {
        match self {
            Self::Single(v) => match index {
                0 => {
                    *v = value;
                },
                i => {
                    let mut array = HashMap::with_capacity(2);
                    array.insert(0, v.clone());
                    array.insert(i, value);
                    *self = Self::Array(array);
                },
            },
            Self::Array(m) => {
                m.insert(index, value);
            },
        }
    }
}

impl DummyFieldHolder {
    pub fn new() -> Self {
        Self { fields: HashMap::new(), vars: HashMap::new() }
    }
}
