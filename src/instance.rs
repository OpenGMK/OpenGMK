use crate::{
    asset::Object,
    gml::{InstanceVariable, Value},
};
use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    f64,
};

pub struct Instance {
    pub exists: Cell<bool>,
    pub id: Cell<usize>,
    pub object_index: Cell<i32>,
    pub solid: Cell<bool>,
    pub visible: Cell<bool>,
    pub persistent: Cell<bool>,
    pub depth: Cell<i32>,
    pub sprite_index: Cell<i32>,
    pub image_alpha: Cell<f64>,
    pub image_blend: Cell<i32>,
    pub image_index: Cell<f64>,
    pub image_speed: Cell<f64>,
    pub image_xscale: Cell<f64>,
    pub image_yscale: Cell<f64>,
    pub image_angle: Cell<f64>,
    pub mask_index: Cell<i32>,
    pub direction: Cell<f64>,
    pub friction: Cell<f64>,
    pub gravity: Cell<f64>,
    pub gravity_direction: Cell<f64>,
    pub hspeed: Cell<f64>,
    pub vspeed: Cell<f64>,
    pub speed: Cell<f64>,
    pub x: Cell<f64>,
    pub y: Cell<f64>,
    pub xprevious: Cell<f64>,
    pub yprevious: Cell<f64>,
    pub xstart: Cell<f64>,
    pub ystart: Cell<f64>,
    pub path_index: Cell<i32>,
    pub path_position: Cell<f64>, // Normalized from 0 to 1
    pub path_positionprevious: Cell<f64>,
    pub path_speed: Cell<f64>,
    pub path_scale: Cell<f64>,
    pub path_orientation: Cell<f64>,
    pub path_endaction: Cell<i32>, // https://docs.yoyogames.com/source/dadiospice/002_reference/paths/path_start.html
    pub timeline_index: Cell<i32>,
    pub timeline_running: Cell<bool>,
    pub timeline_speed: Cell<f64>,
    pub timeline_position: Cell<f64>,
    pub timeline_loop: Cell<bool>,

    pub bbox_top: Cell<i32>,
    pub bbox_left: Cell<i32>,
    pub bbox_right: Cell<i32>,
    pub bbox_bottom: Cell<i32>,
    pub bbox_is_stale: Cell<bool>,

    pub fields: RefCell<HashMap<usize, Field>>,
    pub alarms: RefCell<HashMap<u32, Value>>,
}

#[derive(Debug)]
pub enum Field {
    Single(Value),
    Array(HashMap<u32, Value>),
}

#[derive(Debug, Default)]
pub struct DummyFieldHolder {
    pub fields: HashMap<usize, Field>,
    pub vars: HashMap<InstanceVariable, Field>,
}

impl Instance {
    pub fn new(id: usize, x: f64, y: f64, object_index: i32, object: &Object) -> Self {
        Self {
            exists: Cell::new(true),
            id: Cell::new(id),
            object_index: Cell::new(object_index),
            solid: Cell::new(object.solid),
            visible: Cell::new(object.visible),
            persistent: Cell::new(object.persistent),
            depth: Cell::new(object.depth),
            sprite_index: Cell::new(object.sprite_index),
            image_alpha: Cell::new(1.0),
            image_blend: Cell::new(0xFFFFFF),
            image_index: Cell::new(0.0),
            image_speed: Cell::new(1.0),
            image_xscale: Cell::new(1.0),
            image_yscale: Cell::new(1.0),
            image_angle: Cell::new(0.0),
            mask_index: Cell::new(object.mask_index),
            direction: Cell::new(0.0),
            gravity: Cell::new(0.0),
            gravity_direction: Cell::new(0.0),
            hspeed: Cell::new(0.0),
            vspeed: Cell::new(0.0),
            speed: Cell::new(0.0),
            friction: Cell::new(0.0),
            x: Cell::new(x),
            y: Cell::new(y),
            xprevious: Cell::new(x),
            yprevious: Cell::new(y),
            xstart: Cell::new(x),
            ystart: Cell::new(y),
            path_index: Cell::new(-1),
            path_position: Cell::new(0.0),
            path_positionprevious: Cell::new(0.0),
            path_speed: Cell::new(0.0),
            path_scale: Cell::new(1.0),
            path_orientation: Cell::new(0.0),
            path_endaction: Cell::new(0),
            timeline_index: Cell::new(-1),
            timeline_running: Cell::new(false),
            timeline_speed: Cell::new(1.0),
            timeline_position: Cell::new(0.0),
            timeline_loop: Cell::new(false),
            bbox_top: Cell::new(-100000),
            bbox_left: Cell::new(-100000),
            bbox_right: Cell::new(-100000),
            bbox_bottom: Cell::new(-100000),
            bbox_is_stale: Cell::new(true),
            fields: RefCell::new(HashMap::new()),
            alarms: RefCell::new(HashMap::new()),
        }
    }

    // Sets direction, also updating hspeed and vspeed
    pub fn set_direction(&self, direction: f64) {
        self.direction.set(direction);
        self.update_hvspeed()
    }

    // Sets speed, also updating hspeed and vspeed
    pub fn set_speed(&self, speed: f64) {
        self.speed.set(speed);
        self.update_hvspeed()
    }

    // Sets hspeed, also updating direction and speed
    pub fn set_hspeed(&self, hspeed: f64) {
        self.hspeed.set(hspeed);
        self.update_speed_direction()
    }

    // Sets vspeed, also updating direction and speed
    pub fn set_vspeed(&self, vspeed: f64) {
        self.vspeed.set(vspeed);
        self.update_speed_direction()
    }

    // Sets hspeed and vspeed based on direction and speed
    fn update_hvspeed(&self) {
        self.hspeed.set(-self.direction.get().cos() * self.speed.get());
        self.vspeed.set(self.direction.get().sin() * self.speed.get());
    }

    // Sets direction and speed based on hspeed and vspeed
    fn update_speed_direction(&self) {
        self.direction.set((-self.vspeed.get()).atan2(self.hspeed.get()));
        self.speed.set((self.hspeed.get().powi(2) + self.vspeed.get().powi(2)).sqrt());
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
                        Some(Value::Real(0.0))
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
        Self {
            fields: HashMap::new(),
            vars: HashMap::new(),
        }
    }
}