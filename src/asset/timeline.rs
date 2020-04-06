use crate::action::Tree;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

pub struct Timeline {
    pub name: Rc<str>,
    pub moments: HashMap<u32, Rc<RefCell<Tree>>>,
}
