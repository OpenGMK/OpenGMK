use crate::asset::Object;

pub struct Instance {
    pub exists: bool,
    pub id: usize,
    pub object_index: i32,
    pub solid: bool,
    pub visible: bool,
    pub persistent: bool,
    pub depth: i32,
    pub sprite_index: i32,
    pub image_alpha: f64,
    pub image_blend: i32,
    pub image_index: f64,
    pub image_speed: f64,
    pub image_xscale: f64,
    pub image_yscale: f64,
    pub image_angle: f64,
    pub mask_index: i32,
    pub direction: f64,
    pub friction: f64,
    pub gravity: f64,
    pub gravity_direction: f64,
    pub hspeed: f64,
    pub vspeed: f64,
    pub speed: f64,
    pub x: f64,
    pub y: f64,
    pub xprevious: f64,
    pub yprevious: f64,
    pub xstart: f64,
    pub ystart: f64,
    pub path_index: i32,
    pub path_position: f64, // Normalized from 0 to 1
    pub path_positionprevious: f64,
    pub path_speed: f64,
    pub path_scale: f64,
    pub path_orientation: f64,
    pub path_endaction: i32, // https://docs.yoyogames.com/source/dadiospice/002_reference/paths/path_start.html
    pub timeline_index: i32,
    pub timeline_running: bool,
    pub timeline_speed: f64,
    pub timeline_position: f64,
    pub timeline_loop: bool,

    pub bbox_top: i32,
    pub bbox_left: i32,
    pub bbox_right: i32,
    pub bbox_bottom: i32,
    pub bbox_is_stale: bool,
    // TODO: fields map
    // TODO: alarms map
}

impl Instance {
    pub fn new(id: usize, x: f64, y: f64, object_index: i32, object: &Object) -> Self {
        Self {
            exists: true,
            id,
            object_index,
            solid: object.solid,
            visible: object.visible,
            persistent: object.persistent,
            depth: object.depth,
            sprite_index: object.sprite_index,
            image_alpha: 1.0,
            image_blend: 0xFFFFFF,
            image_index: 0.0,
            image_speed: 1.0,
            image_xscale: 1.0,
            image_yscale: 1.0,
            image_angle: 0.0,
            mask_index: object.mask_index,
            direction: 0.0,
            gravity: 0.0,
            gravity_direction: 0.0,
            hspeed: 0.0,
            vspeed: 0.0,
            speed: 0.0,
            friction: 0.0,
            x: x,
            y: y,
            xprevious: x,
            yprevious: y,
            xstart: x,
            ystart: y,
            path_index: -1,
            path_position: 0.0,
            path_positionprevious: 0.0,
            path_speed: 0.0,
            path_scale: 1.0,
            path_orientation: 0.0,
            path_endaction: 0,
            timeline_index: -1,
            timeline_running: false,
            timeline_speed: 1.0,
            timeline_position: 0.0,
            timeline_loop: false,
            bbox_top: -100000,
            bbox_left: -100000,
            bbox_right: -100000,
            bbox_bottom: -100000,
            bbox_is_stale: true,
        }
    }
}
