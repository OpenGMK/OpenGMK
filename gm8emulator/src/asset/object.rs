use crate::{action::Tree, game::string::RCStr};
use serde::{Deserialize, Serialize};
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

#[derive(Clone, Serialize, Deserialize)]
pub struct Object {
    pub name: RCStr,
    pub solid: bool,
    pub visible: bool,
    pub persistent: bool,
    pub depth: i32,
    pub sprite_index: i32,
    pub mask_index: i32,
    pub parent_index: i32,

    pub events: [HashMap<u32, Rc<RefCell<Tree>>>; 12],
    pub children: Rc<RefCell<HashSet<i32>>>,
    pub parents: Rc<RefCell<HashSet<i32>>>,
}
