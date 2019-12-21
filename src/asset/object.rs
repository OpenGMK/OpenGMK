pub struct Object {
    pub name: String,
    pub solid: bool,
    pub visible: bool,
    pub persistent: bool,
    pub depth: i32,
    pub sprite_index: i32,
    pub mask_index: i32,
    // todo
}

impl From<gm8exe::asset::Object> for Object {
    fn from(exe_object: gm8exe::asset::Object) -> Self {
        Self {
            name: exe_object.name,
            solid: exe_object.solid,
            visible: exe_object.visible,
            persistent: exe_object.persistent,
            depth: exe_object.depth,
            sprite_index: exe_object.sprite_index,
            mask_index: exe_object.mask_index,
        }
    }
}