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
    fn from(other: gm8exe::asset::Object) -> Self {
        Self {
            name: other.name,
            solid: other.solid,
            visible: other.visible,
            persistent: other.persistent,
            depth: other.depth,
            sprite_index: other.sprite_index,
            mask_index: other.mask_index,
        }
    }
}
