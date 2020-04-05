use crate::action::Tree;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

pub struct Timeline {
    pub name: String,
    pub moments: HashMap<u32, Rc<RefCell<Tree>>>,
}
