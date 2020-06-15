use crate::{
    gml,
    instance::{Instance, InstanceState},
    tile::Tile,
    types::ID,
};
use serde::{
    de::{SeqAccess, Visitor},
    ser::SerializeSeq,
    Deserialize, Deserializer, Serialize, Serializer,
};
use std::{
    alloc,
    cell::RefCell,
    collections::{HashMap, HashSet},
    fmt, ptr,
    rc::Rc,
};

/// Elements per Chunk (fixed size).
const CHUNK_SIZE: usize = 256;

/// Typedef to not have to write `[Option<T>; CHUNK_SIZE]` everywhere.
/// Array of CHUNK_SIZE with either vacant or occupied (T) slots.
type ChunkArray<T> = [Option<T>; CHUNK_SIZE];

/// Slab-like fixed size memory chunk with standard vacant/occupied system.
#[derive(Clone)]
struct Chunk<T> {
    slots: Box<ChunkArray<T>>,
    vacant: usize,
}

/// How many chunks ChunkList preallocates (16 + 102400 bytes each for instances).
static CHUNKS_PREALLOCATED: usize = 8;

/// Growable container managing allocated Chunks.
#[derive(Clone)]
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

    fn remove(&mut self, idx: usize) {
        let idx_div = idx / CHUNK_SIZE;
        let idx_mod = idx % CHUNK_SIZE;
        self.0.get_mut(idx_div).map(|chunk| {
            chunk.slots[idx_mod] = None;
            chunk.vacant += 1;
        });
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
        .find(|(_, &inst_idx)| list.get(inst_idx).state.get() == InstanceState::Active)
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

#[derive(Clone, Serialize, Deserialize)]
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
                let inst = list.get(instance);
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
                let inst = list.get(instance);
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

    pub fn get(&self, idx: usize) -> &Instance {
        self.chunks
            .get(idx)
            .unwrap_or_else(|| panic!(format!("Invalid instance handle to InstanceList::get(): {}", idx)))
    }

    pub fn get_by_instid(&self, instance_index: ID) -> Option<usize> {
        self.insert_order.iter().copied().find(|&inst| self.get(inst).id.get() == instance_index)
    }

    pub fn count(&self, object_index: ID) -> usize {
        self.id_map.get(&object_index).copied().unwrap_or_default()
    }

    pub fn count_all(&self) -> usize {
        self.insert_order.iter().filter(|&&inst_idx| self.get(inst_idx).state.get() != InstanceState::Inactive).count()
    }

    pub fn instance_at(&self, n: usize) -> ID {
        self.insert_order
            .iter()
            .filter(|&&inst_idx| self.get(inst_idx).state.get() != InstanceState::Inactive)
            .nth(n)
            .map(|inst_idx| self.get(*inst_idx).id.get())
            .unwrap_or(gml::NOONE)
    }

    pub fn draw_sort(&mut self) {
        let chunks = &self.chunks; // borrowck :)
        self.draw_order.sort_by(move |&idx1, &idx2| {
            // TODO: Bench if this is faster with unreachable_unchecked...
            let left = chunks.get(idx1).unwrap();
            let right = chunks.get(idx2).unwrap();

            // Draw order is sorted by depth (higher is lowest...)
            right.depth.get().cmp(&left.depth.get())
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

    pub fn insert_dummy(&mut self, el: Instance) -> usize {
        self.chunks.insert(el)
    }

    pub fn remove_dummy(&mut self, instance: usize) {
        self.chunks.remove(instance)
    }

    pub fn mark_deleted(&mut self, instance: usize) {
        let instance = self.get(instance);
        if instance.state.get() != InstanceState::Deleted {
            instance.state.set(InstanceState::Deleted);

            let object_id = instance.object_index.get();
            let entry = self.id_map.entry(object_id).and_modify(|n| *n -= 1);
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

#[derive(Clone, Serialize, Deserialize)]
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

    pub fn get(&self, idx: usize) -> &Tile {
        self.chunks.get(idx).unwrap_or_else(|| panic!(format!("Invalid instance handle to TileList::get(): {}", idx)))
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

impl<T> Serialize for ChunkList<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let count = self.0.iter().map(|x| CHUNK_SIZE - x.vacant).sum();
        let mut seq = serializer.serialize_seq(Some(count))?;
        for element in self.0.iter().map(|x| x.slots.iter()).flatten() {
            if let Some(inst) = element {
                seq.serialize_element(inst)?;
            }
        }
        seq.end()
    }
}

impl<'de, T> Deserialize<'de> for ChunkList<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct InstanceVisitor<T> {
            phantom: std::marker::PhantomData<T>,
        };

        impl<'v, T> Visitor<'v> for InstanceVisitor<T>
        where
            T: Deserialize<'v>,
        {
            type Value = ChunkList<T>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a sequence")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
            where
                V: SeqAccess<'v>,
            {
                let mut list = ChunkList::new();

                while let Some(instance) = seq.next_element::<T>()? {
                    list.insert(instance);
                }

                Ok(list)
            }
        }

        deserializer.deserialize_seq(InstanceVisitor::<T> { phantom: Default::default() })
    }
}

// TODO: Maybe preallocating order/draw_order would increase perf - test this!
