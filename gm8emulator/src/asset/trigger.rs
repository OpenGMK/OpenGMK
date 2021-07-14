use crate::gml::{self, runtime::Instruction};
use gm8exe::asset::trigger::TriggerKind;
use serde::{Deserialize, Serialize};
use std::rc::Rc;

#[derive(Clone, Serialize, Deserialize)]
pub struct Trigger {
    pub name: gml::String,
    pub condition: Rc<[Instruction]>,
    pub moment: TriggerTime,
}

#[derive(PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum TriggerTime {
    Step,
    BeginStep,
    EndStep,
}

impl From<TriggerKind> for TriggerTime {
    fn from(tk: TriggerKind) -> Self {
        match tk {
            TriggerKind::BeginStep => TriggerTime::BeginStep,
            TriggerKind::Step => TriggerTime::Step,
            TriggerKind::EndStep => TriggerTime::EndStep,
        }
    }
}
