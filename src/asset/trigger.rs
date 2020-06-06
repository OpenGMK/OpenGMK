use crate::gml::runtime::Instruction;
use gm8exe::asset::trigger::TriggerKind;
use std::rc::Rc;

#[derive(Clone)]
pub struct Trigger {
    pub name: Rc<str>,
    pub condition: Rc<[Instruction]>,
    pub moment: TriggerTime,
}

#[derive(PartialEq, Eq, Clone)]
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
