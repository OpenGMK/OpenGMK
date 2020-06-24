use gmio::atlas::AtlasRef;

#[derive(Clone, Debug)]
pub struct Font(Box<[Character]>);

#[derive(Clone, Copy, Debug)]
pub struct Character {
    pub atlas_ref: AtlasRef,
    pub advance_width: f64,
    pub left_side_bearing: f64,
}

impl From<Box<[Character]>> for Font {
    fn from(b: Box<[Character]>) -> Self {
        Self(b)
    }
}

impl Font {
    pub fn get(&self, index: u8) -> Option<&Character> {
        self.0.get(index as usize)
    }
}
