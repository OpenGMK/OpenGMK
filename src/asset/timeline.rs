use crate::action::Tree;
use std::collections::HashMap;

pub struct Timeline {
    pub name: String,
    pub moments: HashMap<u32, Tree>,
}