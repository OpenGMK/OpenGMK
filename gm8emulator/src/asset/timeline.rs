use crate::{action::Tree, gml};
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

#[derive(Clone, Serialize, Deserialize)]
pub struct Timeline {
    pub name: gml::String,
    pub moments: Rc<RefCell<BTreeMap<i32, Rc<RefCell<Tree>>>>>,
}
