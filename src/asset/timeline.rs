use crate::{action::Tree, game::string::RCStr};
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

#[derive(Serialize, Deserialize)]
pub struct Timeline {
    pub name: RCStr,
    pub moments: Rc<RefCell<HashMap<u32, Rc<RefCell<Tree>>>>>,
}
