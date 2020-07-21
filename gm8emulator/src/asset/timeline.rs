use crate::{action::Tree, game::string::RCStr};
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

#[derive(Clone, Serialize, Deserialize)]
pub struct Timeline {
    pub name: RCStr,
    pub moments: Rc<RefCell<BTreeMap<i32, Rc<RefCell<Tree>>>>>,
}
