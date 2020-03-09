use crate::{instance::Instance, tile::Tile};
use generational_arena::{Arena, Index};

pub struct InstanceList {
    pub instance_arena: Arena<Instance>,
    pub tile_arena: Arena<Tile>,
}

impl InstanceList {
    pub fn new() -> Self {
        Self { instance_arena: Arena::with_capacity(1024), tile_arena: Arena::with_capacity(1024) }
    }

    pub fn insert(&mut self, instance: Instance) -> Index {
        self.instance_arena.insert(instance)
    }

    pub fn insert_tile(&mut self, tile: Tile) -> Index {
        self.tile_arena.insert(tile)
    }

    pub fn remove(&mut self, index: Index) -> Option<Instance> {
        self.instance_arena.remove(index)
    }

    pub fn remove_tile(&mut self, index: Index) -> Option<Tile> {
        self.tile_arena.remove(index)
    }

    pub fn clear_tiles(&mut self) {
        self.tile_arena.clear()
    }

    pub fn remove_deleted(&mut self) {
        self.instance_arena.retain(|_, x| x.exists.get())
    }

    pub fn remove_non_persistent(&mut self) {
        self.instance_arena.retain(|_, x| x.persistent.get())
    }

    pub fn get(&self, index: Index) -> Option<&Instance> {
        self.instance_arena.get(index)
    }

    pub fn get_tile(&self, index: Index) -> Option<&Tile> {
        self.tile_arena.get(index)
    }

    pub fn get_tile_mut(&mut self, index: Index) -> Option<&mut Tile> {
        self.tile_arena.get_mut(index)
    }

    pub fn iter_instances(&self) -> generational_arena::Iter<Instance> {
        self.instance_arena.iter()
    }

    pub fn iter_tiles(&self) -> generational_arena::Iter<Tile> {
        self.tile_arena.iter()
    }
}
