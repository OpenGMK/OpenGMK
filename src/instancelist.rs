use crate::{
    gml,
    instance::{Instance, InstanceState},
    tile::Tile,
    types::ID,
};
use std::{
    alloc,
    cell::RefCell,
    cmp::Ordering,
    collections::{HashMap, HashSet},
    ptr,
    rc::Rc,
};

/// Elements per Chunk (fixed size).
const CHUNK_SIZE: usize = 256;

/// Typedef to not have to write `[Option<T>; CHUNK_SIZE]` everywhere.
/// Array of CHUNK_SIZE with either vacant or occupied (T) slots.
type ChunkArray<T> = [Option<T>; CHUNK_SIZE];

/// Slab-like fixed size memory chunk with standard vacant/occupied system.
struct Chunk<T> {
    slots: Box<ChunkArray<T>>,
    vacant: usize,
}

/// How many chunks ChunkList preallocates (16 + 102400 bytes each for instances).
static CHUNKS_PREALLOCATED: usize = 8;

/// Growable container managing allocated Chunks.
struct ChunkList<T>(Vec<Chunk<T>>);

impl<T> Chunk<T> {
    pub fn new() -> Self {
        Self {
            slots: unsafe {
                // manual alloc since T isn't Copy... but it's all None anyway
                let memory = alloc::alloc(alloc::Layout::new::<ChunkArray<T>>()) as *mut ChunkArray<T>;
                let mut slots = Box::from_raw(memory);

                // initialize memory, using ptr::write to avoid Drop running on uninitialized memory
                slots.iter_mut().for_each(|slot| ptr::write(slot, None));
                slots
            },
            vacant: CHUNK_SIZE,
        }
    }
}

impl<T> ChunkList<T> {
    fn new() -> Self {
        Self({
            let mut chunks = Vec::with_capacity(CHUNKS_PREALLOCATED);
            for _ in 0..CHUNKS_PREALLOCATED {
                chunks.push(Chunk::new());
            }
            chunks
        })
    }

    fn get(&self, idx: usize) -> Option<&T> {
        // Calculating these right next to each other guarantees they'll be optimized to a single div op.
        // Using [] in chunk.slots won't be bounds checked since LLVM will see %CHUNK_SIZE.
        let idx_div = idx / CHUNK_SIZE;
        let idx_mod = idx % CHUNK_SIZE;
        self.0.get(idx_div).and_then(|chunk| chunk.slots[idx_mod].as_ref())
    }

    fn insert(&mut self, t: T) -> usize {
        match self.0.iter_mut().enumerate().find(|(_, chunk)| chunk.vacant != 0) {
            Some((idx, chunk)) => {
                chunk.vacant -= 1;
                match chunk.slots.iter_mut().enumerate().find(|(_, slot)| slot.is_none()) {
                    Some((slot_idx, slot @ None)) => {
                        *slot = Some(t);
                        (idx * CHUNK_SIZE) + slot_idx
                    },
                    _ => unreachable!(),
                }
            },
            None => {
                let mut chunk = Chunk::new();
                chunk.vacant -= 1;
                chunk.slots[0] = Some(t);
                self.0.push(chunk);
                (self.0.len() - 1) * CHUNK_SIZE
            },
        }
    }

    fn iter(&self) -> impl Iterator<Item = &Chunk<T>> {
        self.0.iter()
    }

    fn iter_mut(&mut self) -> impl Iterator<Item = &mut Chunk<T>> {
        self.0.iter_mut()
    }

    fn remove_with(&mut self, mut f: impl FnMut(&T) -> bool) {
        for chunk in self.iter_mut() {
            for slot in chunk.slots.iter_mut() {
                if let Some(t) = slot {
                    if f(&*t) {
                        *slot = None;
                        chunk.vacant += 1;
                    }
                }
            }
        }
    }

    fn clear(&mut self) {
        for chunk in self.iter_mut() {
            for slot in chunk.slots.iter_mut() {
                *slot = None;
            }
            chunk.vacant = CHUNK_SIZE;
        }
    }
}

// non-borrowing instancelist iterator things
fn nb_il_iter(coll: &[usize], idx: &mut usize, list: &InstanceList) -> Option<usize> {
    coll.get(*idx..)?
        .iter()
        .enumerate()
        .find(|(_, &inst_idx)| {
            list.get(inst_idx).map(|inst| inst.state.get() == InstanceState::Active).unwrap_or_default()
        })
        .map(|(idx_offset, val)| {
            *idx += idx_offset + 1;
            *val
        })
}

// the function above but more generic
fn nb_coll_iter_advance<T: Copy>(coll: &[T], idx: &mut usize) -> Option<T> {
    coll.get(*idx).map(|val| {
        *idx += 1;
        *val
    })
}

pub struct InstanceList {
    chunks: ChunkList<Instance>,
    insert_order: Vec<usize>,
    draw_order: Vec<usize>,
    id_map: HashMap<ID, usize>, // Object ID <-> Count
}

// generic purpose non-borrowing iterators
pub struct ILIterDrawOrder(usize);
pub struct ILIterInsertOrder(usize);
impl ILIterDrawOrder {
    pub fn next(&mut self, list: &InstanceList) -> Option<usize> {
        nb_il_iter(&list.draw_order, &mut self.0, &list)
    }
}
impl ILIterInsertOrder {
    pub fn next(&mut self, list: &InstanceList) -> Option<usize> {
        nb_il_iter(&list.insert_order, &mut self.0, &list)
    }
}

// iteration by identity (each object or each object that parents said object)
pub struct IdentityIter {
    count: usize,
    position: usize,
    children: Rc<RefCell<HashSet<ID>>>,
}
impl IdentityIter {
    pub fn next(&mut self, list: &InstanceList) -> Option<usize> {
        if self.count > 0 {
            for (idx, &instance) in list.insert_order.get(self.position..)?.iter().enumerate() {
                let inst = list.get(instance)?;
                if inst.state.get() == InstanceState::Active {
                    let oidx = inst.object_index.get();
                    if self.children.borrow().contains(&oidx) {
                        self.count -= 1;
                        self.position += idx + 1;
                        return Some(instance)
                    }
                }
            }
        }
        None
    }
}

/// iteration, filtering by object id, in insertion order (does NOT follow parents)
pub struct ObjectIter {
    // count of objects (stored to optimize and match GM8 weird behaviour)
    count: usize,
    // position in the insert-order vec
    position: usize,
    // object index
    object_index: ID,
}
impl ObjectIter {
    pub fn next(&mut self, list: &InstanceList) -> Option<usize> {
        if self.count > 0 {
            for (idx, &instance) in list.insert_order.get(self.position..)?.iter().enumerate() {
                let inst = list.get(instance)?;
                if inst.state.get() == InstanceState::Active {
                    if inst.object_index.get() == self.object_index {
                        self.count -= 1;
                        self.position += idx + 1;
                        return Some(instance)
                    }
                }
            }
        }
        None
    }
}

impl InstanceList {
    pub fn new() -> Self {
        Self { chunks: ChunkList::new(), insert_order: Vec::new(), draw_order: Vec::new(), id_map: HashMap::new() }
    }

    pub fn get(&self, idx: usize) -> Option<&Instance> {
        self.chunks.get(idx)
    }

    pub fn get_by_instid(&self, instance_index: ID) -> Option<usize> {
        self.insert_order
            .iter()
            .copied()
            .find(|&inst| self.get(inst).map(|x| x.id.get() == instance_index).unwrap_or_default())
    }

    pub fn count(&self, object_index: ID) -> usize {
        self.id_map.get(&object_index).copied().unwrap_or_default()
    }

    pub fn count_all(&self) -> usize {
        self.insert_order
            .iter()
            .filter(|&&inst_idx| {
                self.get(inst_idx).map(|inst| inst.state.get() != InstanceState::Inactive).unwrap_or_default()
            })
            .count()
    }

    pub fn instance_at(&self, n: usize) -> ID {
        self.insert_order
            .iter()
            .filter(|&&inst_idx| {
                self.get(inst_idx).map(|inst| inst.state.get() != InstanceState::Inactive).unwrap_or_default()
            })
            .nth(n)
            .and_then(|inst_idx| self.get(*inst_idx).map(|inst| inst.id.get()))
            .unwrap_or(gml::NOONE)
    }

    pub fn draw_sort(&mut self) {
        let chunks = &self.chunks; // borrowck :)
        self.draw_order.sort_by(move |&idx1, &idx2| {
            // TODO: Bench if this is faster with unreachable_unchecked...
            let left = chunks.get(idx1).unwrap();
            let right = chunks.get(idx2).unwrap();

            // First, draw order is sorted by depth (higher is lowest...)
            match right.depth.get().cmp(&left.depth.get()) {
                Ordering::Equal => {
                    // If they're equal then it's the ordering of object index.
                    // If those are equal it's in insertion order (aka, equal, this is stablesort).
                    left.object_index.get().cmp(&right.object_index.get())
                },
                other => other,
            }
        })
    }

    pub const fn iter_by_drawing(&self) -> ILIterDrawOrder {
        ILIterDrawOrder(0)
    }

    pub const fn iter_by_insertion(&self) -> ILIterInsertOrder {
        ILIterInsertOrder(0)
    }

    pub fn iter_by_identity(&self, identities: Rc<RefCell<HashSet<ID>>>) -> IdentityIter {
        let count = identities.borrow().iter().fold(0, |acc, x| acc + self.id_map.get(x).copied().unwrap_or_default());
        IdentityIter { count, position: 0, children: identities }
    }

    pub fn iter_by_object(&self, object_index: ID) -> ObjectIter {
        ObjectIter {
            count: self.id_map.get(&object_index).copied().unwrap_or(0),
            position: 0,
            object_index: object_index,
        }
    }

    pub fn insert(&mut self, el: Instance) -> usize {
        let object_id = el.object_index.get();
        let value = self.chunks.insert(el);
        self.insert_order.push(value);
        self.draw_order.push(value);
        self.id_map.entry(object_id).and_modify(|n| *n += 1).or_insert(1);
        value
    }

    pub fn mark_deleted(&mut self, instance: usize) {
        let object_id = self.get(instance).and_then(|instance| {
            if instance.state.get() != InstanceState::Deleted {
                instance.state.set(InstanceState::Deleted);
                Some(instance.object_index.get())
            } else {
                None
            }
        });
        if let Some(o) = object_id {
            let entry = self.id_map.entry(o).and_modify(|n| *n -= 1);
            if let std::collections::hash_map::Entry::Occupied(occupied) = entry {
                if *occupied.get() == 0 {
                    occupied.remove_entry();
                }
            }
        }
    }

    pub fn obj_count_hint(&mut self, n: usize) {
        self.id_map.reserve((n as isize - self.id_map.len() as isize).max(0) as usize)
    }

    pub fn remove_with(&mut self, f: impl Fn(&Instance) -> bool) {
        let id_map = &mut self.id_map; // borrowck :)
        self.chunks.remove_with(|x| {
            let remove = f(x);
            if remove {
                if x.state.get() == InstanceState::Active {
                    let entry = id_map.entry(x.object_index.get()).and_modify(|n| *n -= 1);
                    if let std::collections::hash_map::Entry::Occupied(occupied) = entry {
                        if *occupied.get() == 0 {
                            occupied.remove_entry();
                        }
                    }
                }
            }
            remove
        });
        let chunks = &self.chunks;
        self.draw_order.retain(|idx| chunks.get(*idx).is_some());
        self.insert_order.retain(|idx| chunks.get(*idx).is_some());
    }
}

pub struct TileList {
    chunks: ChunkList<Tile>,
    insert_order: Vec<usize>,
    draw_order: Vec<usize>,
}

// generic purpose non-borrowing iterators
pub struct TLIterDrawOrder(usize);
pub struct TLIterInsertOrder(usize);
impl TLIterDrawOrder {
    pub fn next(&mut self, list: &TileList) -> Option<usize> {
        nb_coll_iter_advance(&list.draw_order, &mut self.0)
    }
}
impl TLIterInsertOrder {
    pub fn next(&mut self, list: &TileList) -> Option<usize> {
        nb_coll_iter_advance(&list.insert_order, &mut self.0)
    }
}

impl TileList {
    pub fn new() -> Self {
        Self { chunks: ChunkList::new(), insert_order: Vec::new(), draw_order: Vec::new() }
    }

    pub fn get(&self, idx: usize) -> Option<&Tile> {
        self.chunks.get(idx)
    }

    pub const fn iter_by_drawing(&self) -> TLIterDrawOrder {
        TLIterDrawOrder(0)
    }

    pub const fn iter_by_insertion(&self) -> TLIterInsertOrder {
        TLIterInsertOrder(0)
    }

    pub fn draw_sort(&mut self) {
        let chunks = &self.chunks; // borrowck :)
        self.draw_order.sort_by(move |&idx1, &idx2| {
            // TODO: (dupe) Bench if this is faster with unreachable_unchecked...
            let left = chunks.get(idx1).unwrap();
            let right = chunks.get(idx2).unwrap();

            right.depth.cmp(&left.depth)
        })
    }

    pub fn insert(&mut self, el: Tile) -> usize {
        let value = self.chunks.insert(el);
        self.insert_order.push(value);
        self.draw_order.push(value);
        value
    }

    pub fn clear(&mut self) {
        self.chunks.clear();
        self.insert_order.clear();
        self.draw_order.clear();
    }
}

// TODO: Maybe preallocating order/draw_order would increase perf - test this!
