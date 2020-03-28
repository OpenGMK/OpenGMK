use crate::action::Tree;
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

pub struct Object {
    pub name: String,
    pub solid: bool,
    pub visible: bool,
    pub persistent: bool,
    pub depth: i32,
    pub sprite_index: i32,
    pub mask_index: i32,
    pub parent_index: i32,

    pub events: [HashMap<u32, Tree>; 12],
    pub identities: Rc<RefCell<HashSet<i32>>>,
    pub children: Rc<RefCell<HashSet<i32>>>,
}
