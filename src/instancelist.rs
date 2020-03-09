use crate::instance::Instance;
use generational_arena::{Arena, Index};

pub struct InstanceList {
    pub arena: Arena<Instance>,
}

impl InstanceList {
    pub fn new() -> Self {
        Self { arena: Arena::with_capacity(1024) }
    }

    pub fn insert(&mut self, instance: Instance) -> Index {
        self.arena.insert(instance)
    }

    pub fn remove(&mut self, index: Index) -> Option<Instance> {
        self.arena.remove(index)
    }

    pub fn remove_deleted(&mut self) {
        self.arena.retain(|_, x| x.exists.get())
    }

    pub fn remove_non_persistent(&mut self) {
        self.arena.retain(|_, x| x.persistent.get())
    }

    pub fn get(&self, index: Index) -> Option<&Instance> {
        self.arena.get(index)
    }

    pub fn iter(&self) -> generational_arena::Iter<Instance> {
        self.arena.iter()
    }
}
