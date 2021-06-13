use crate::{
    gml,
    instance::{Instance, InstanceState},
    tile::Tile,
};
use serde::{
    de::{SeqAccess, Visitor},
    ser::{SerializeSeq, SerializeStruct},
    Deserialize, Deserializer, Serialize, Serializer,
};
use shared::types::ID;
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    fmt,
    rc::Rc,
};

/// Elements per Chunk (fixed size).
const CHUNK_SIZE: usize = 256;

/// Slab-like fixed size memory chunk with standard vacant/occupied system.
#[derive(Clone)]
struct Chunk<T> {
    slots: Box<[Option<T>; CHUNK_SIZE]>,
    vacant: usize,
}

/// How many chunks ChunkList preallocates (16 + 102400 bytes each for instances).
static CHUNKS_PREALLOCATED: usize = 8;

/// Growable container managing allocated Chunks.
#[derive(Clone)]
struct ChunkList<T>(Vec<Chunk<T>>);

impl<T> Chunk<T> {
    // TODO: This is somewhat annoying. See the similar comment in 'handleman'.
    const NONE_INIT: Option<T> = None;
    pub fn new() -> Self {
        Self {
            slots: Box::new([Self::NONE_INIT; CHUNK_SIZE]),
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

    fn remove_with(&mut self, mut f: impl FnMut(&T) -> bool) -> usize {
        let mut count = 0;
        for chunk in self.iter_mut() {
            for slot in chunk.slots.iter_mut() {
                if let Some(t) = slot {
                    if f(&*t) {
                        *slot = None;
                        chunk.vacant += 1;
                        count += 1;
                    }
                }
            }
        }
        count
    }

    fn remove_as_vec(&mut self, mut f: impl FnMut(&T) -> bool) -> Vec<T> {
        let mut output = Vec::new();
        for chunk in self.iter_mut() {
            for slot in chunk.slots.iter_mut() {
                if let Some(t) = slot {
                    if f(&*t) {
                        let instance = slot.take();
                        // SAFETY: we already checked that this instance is Some before taking it
                        output.push(instance.unwrap_or_else(|| unsafe { std::hint::unreachable_unchecked() }));
                        chunk.vacant += 1;
                    }
                }
            }
        }
        output
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
fn nb_il_iter(coll: &[usize], idx: &mut usize, list: &InstanceList, state: InstanceState) -> Option<usize> {
    coll.get(*idx..)?.iter().enumerate().find(|(_, &inst_idx)| list.get(inst_idx).state.get() == state).map(
        |(idx_offset, val)| {
            *idx += idx_offset + 1;
            *val
        },
    )
}

// the function above but more generic
fn nb_coll_iter_advance<T: Copy>(coll: &[T], idx: &mut usize) -> Option<T> {
    coll.get(*idx).map(|val| {
        *idx += 1;
        *val
    })
}

#[derive(Clone, Deserialize)]
pub struct InstanceList {
    chunks: ChunkList<Instance>,
    insert_order: Vec<usize>,
    draw_order: Vec<usize>,
    object_id_map: HashMap<ID, Vec<usize>>, // Object ID <-> Count
    object_id_map_inherit: HashMap<ID, Vec<usize>>,
    inactive_id_map: HashMap<ID, Vec<usize>>,
}

// generic purpose non-borrowing iterators
pub struct ILIterDrawOrder(usize);
pub struct ILIterInsertOrder(usize);
pub struct ILIterInactive(usize);
impl ILIterDrawOrder {
    pub fn next(&mut self, list: &InstanceList) -> Option<usize> {
        nb_il_iter(&list.draw_order, &mut self.0, &list, InstanceState::Active)
    }
}
impl ILIterInsertOrder {
    pub fn next(&mut self, list: &InstanceList) -> Option<usize> {
        nb_il_iter(&list.insert_order, &mut self.0, &list, InstanceState::Active)
    }
}
impl ILIterInactive {
    pub fn next(&mut self, list: &InstanceList) -> Option<usize> {
        nb_il_iter(&list.insert_order, &mut self.0, &list, InstanceState::Inactive)
    }
}

// iteration by identity (each object or each object that parents said object)
pub struct IdentityIter {
    count: usize,
    position: usize,
    children: Rc<RefCell<HashSet<ID>>>,
    state: InstanceState,
}
impl IdentityIter {
    pub fn next(&mut self, list: &InstanceList) -> Option<usize> {
        if self.count > 0 {
            for (idx, &instance) in list.insert_order.get(self.position..)?.iter().enumerate() {
                let inst = list.get(instance);
                if inst.state.get() == self.state {
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
        if self.position < self.count {
            list.object_id_map
                .get(&self.object_index)
                .and_then(|v| nb_il_iter(v, &mut self.position, list, InstanceState::Active))
        } else {
            None
        }
    }
}

impl InstanceList {
    pub fn new() -> Self {
        Self {
            chunks: ChunkList::new(),
            insert_order: Vec::new(),
            draw_order: Vec::new(),
            object_id_map: HashMap::new(),
            object_id_map_inherit: HashMap::new(),
            inactive_id_map: HashMap::new(),
        }
    }

    pub fn get(&self, idx: usize) -> &Instance {
        self.chunks.get(idx).unwrap_or_else(|| panic!("Invalid instance handle to InstanceList::get(): {}", idx))
    }

    pub fn get_by_instid(&self, instance_index: ID) -> Option<usize> {
        self.insert_order.iter().copied().find(|&inst| self.get(inst).id.get() == instance_index)
    }

    pub fn count(&self, object_index: ID) -> usize {
        self.object_id_map
            .get(&object_index)
            .map(|v| v.iter().filter(|&&inst_idx| self.get(inst_idx).state.get() == InstanceState::Active).count())
            .unwrap_or_default()
    }

    pub fn any_active(&self) -> bool {
        self.insert_order.iter().filter(|&&inst_idx| self.get(inst_idx).is_active()).next().is_some()
    }

    pub fn count_all_active(&self) -> usize {
        self.insert_order.iter().filter(|&&inst_idx| self.get(inst_idx).is_active()).count()
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
            right.depth.get().cmp_nan_first(&left.depth.get())
        })
    }

    pub const fn iter_by_drawing(&self) -> ILIterDrawOrder {
        ILIterDrawOrder(0)
    }

    pub const fn iter_by_insertion(&self) -> ILIterInsertOrder {
        ILIterInsertOrder(0)
    }

    pub const fn iter_inactive(&self) -> ILIterInactive {
        ILIterInactive(0)
    }

    pub fn iter_by_identity(&self, identities: Rc<RefCell<HashSet<ID>>>) -> IdentityIter {
        let count = identities
            .borrow()
            .iter()
            .fold(0, |acc, x| acc + self.object_id_map.get(x).map(|v| v.len()).unwrap_or_default());
        IdentityIter { count, position: 0, children: identities, state: InstanceState::Active }
    }

    pub fn iter_inactive_by_identity(&self, identities: Rc<RefCell<HashSet<ID>>>) -> IdentityIter {
        let count = identities
            .borrow()
            .iter()
            .fold(0, |acc, x| acc + self.inactive_id_map.get(x).map(|v| v.len()).unwrap_or_default());
        IdentityIter { count, position: 0, children: identities, state: InstanceState::Inactive }
    }

    pub fn iter_by_object(&self, object_index: ID) -> ObjectIter {
        ObjectIter {
            count: self.object_id_map.get(&object_index).map(|v| v.len()).unwrap_or(0),
            position: 0,
            object_index: object_index,
        }
    }

    pub fn insert(&mut self, el: Instance) -> usize {
        let object_id = el.object_index.get();
        let value = self.chunks.insert(el);
        self.insert_order.push(value);
        self.draw_order.push(value);
        self.object_id_map.entry(object_id).or_insert(Vec::new()).push(value);
        for &parent in self.get(value).parents.clone().borrow().iter() {
            self.object_id_map_inherit.entry(parent).or_insert(Vec::new()).push(value);
        }
        value
    }

    pub fn insert_dummy(&mut self, el: Instance) -> usize {
        self.chunks.insert(el)
    }

    pub fn remove_dummy(&mut self, instance: usize) {
        self.chunks.remove(instance)
    }

    pub fn deactivate(&mut self, handle: usize) {
        let instance = self.get(handle);
        if instance.state.get() == InstanceState::Active {
            instance.state.set(InstanceState::Inactive);

            let object_id = instance.object_index.get();
            let parents = instance.parents.clone();
            // Add to inactive
            self.inactive_id_map.entry(object_id).or_insert(Vec::new()).push(handle);
            // Remove from active
            let entry = self.object_id_map.entry(object_id).and_modify(|v| v.retain(|h| *h != handle));
            if let std::collections::hash_map::Entry::Occupied(occupied) = entry {
                if occupied.get().is_empty() {
                    occupied.remove_entry();
                }
            }
            for &parent in parents.borrow().iter() {
                let entry = self.object_id_map_inherit.entry(parent).and_modify(|v| v.retain(|h| *h != handle));
                if let std::collections::hash_map::Entry::Occupied(occupied) = entry {
                    if occupied.get().is_empty() {
                        occupied.remove_entry();
                    }
                }
            }
        }
    }

    pub fn activate(&mut self, handle: usize) {
        let instance = self.get(handle);
        if instance.state.get() == InstanceState::Inactive {
            instance.state.set(InstanceState::Active);

            let object_id = instance.object_index.get();
            let parents = instance.parents.clone();
            // Add to active
            self.object_id_map.entry(object_id).or_insert(Vec::new()).push(handle);
            for &parent in parents.borrow().iter() {
                self.object_id_map_inherit.entry(parent).or_insert(Vec::new()).push(handle);
            }
            // Remove from inactive
            let entry = self.inactive_id_map.entry(object_id).and_modify(|v| v.retain(|h| *h != handle));
            if let std::collections::hash_map::Entry::Occupied(occupied) = entry {
                if occupied.get().is_empty() {
                    occupied.remove_entry();
                }
            }
        }
    }

    pub fn mark_deleted(&mut self, handle: usize) {
        let instance = self.get(handle);
        if instance.state.get() != InstanceState::Deleted {
            instance.state.set(InstanceState::Deleted);
        }
    }

    pub fn obj_count_hint(&mut self, n: usize) {
        self.object_id_map.reserve((n as isize - self.object_id_map.len() as isize).max(0) as usize);
    }

    pub fn remove_with(&mut self, f: impl Fn(&Instance) -> bool) {
        if self.chunks.remove_with(f) > 0 {
            let chunks = &self.chunks;
            self.draw_order.retain(|idx| chunks.get(*idx).is_some());
            self.insert_order.retain(|idx| chunks.get(*idx).is_some());
            for instances in self.object_id_map.values_mut() {
                instances.retain(|idx| chunks.get(*idx).is_some());
            }
            self.object_id_map.retain(|_, list| !list.is_empty());
            for instances in self.object_id_map_inherit.values_mut() {
                instances.retain(|idx| chunks.get(*idx).is_some());
            }
            self.object_id_map_inherit.retain(|_, list| !list.is_empty());
        }
    }

    pub fn remove_as_vec(&mut self, f: impl Fn(&Instance) -> bool) -> Vec<Instance> {
        let instances = self.chunks.remove_as_vec(f);
        if instances.len() > 0 {
            let chunks = &self.chunks;
            self.draw_order.retain(|idx| chunks.get(*idx).is_some());
            self.insert_order.retain(|idx| chunks.get(*idx).is_some());
            for instances in self.object_id_map.values_mut() {
                instances.retain(|idx| chunks.get(*idx).is_some());
            }
            self.object_id_map.retain(|_, list| !list.is_empty());
            for instances in self.object_id_map_inherit.values_mut() {
                instances.retain(|idx| chunks.get(*idx).is_some());
            }
            self.object_id_map_inherit.retain(|_, list| !list.is_empty());
        }
        instances
    }
}

#[derive(Clone, Deserialize)]
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
        self.chunks.get(idx).unwrap_or_else(|| panic!("Invalid instance handle to TileList::get(): {}", idx))
    }

    pub fn get_by_tileid(&self, tile_id: ID) -> Option<usize> {
        self.insert_order.iter().copied().find(|&inst| self.get(inst).id.get() == tile_id)
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

            right.depth.get().cmp_nan_first(&left.depth.get())
        })
    }

    pub fn insert(&mut self, el: Tile) -> usize {
        let value = self.chunks.insert(el);
        self.insert_order.push(value);
        self.draw_order.push(value);
        value
    }

    pub fn remove(&mut self, idx: usize) {
        self.chunks.remove(idx);
        self.insert_order.retain(|&i| i != idx);
        self.draw_order.retain(|&i| i != idx);
    }

    pub fn remove_with(&mut self, f: impl Fn(&Tile) -> bool) {
        let mut removed_any = false;
        self.chunks.remove_with(|x| {
            let remove = f(x);
            if remove {
                removed_any = true;
            }
            remove
        });
        if removed_any {
            let chunks = &self.chunks;
            self.draw_order.retain(|idx| chunks.get(*idx).is_some());
            self.insert_order.retain(|idx| chunks.get(*idx).is_some());
        }
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
        let count = self.0.iter().map(|x| x.slots.iter().flatten().count()).sum();
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
        }

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

impl Serialize for InstanceList {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut list = serializer.serialize_struct("InstanceList", 5)?;
        list.serialize_field("chunks", &self.chunks)?;
        list.serialize_field("insert_order", &defrag(&self.insert_order))?;
        list.serialize_field("draw_order", &defrag(&self.draw_order))?;
        list.serialize_field("object_id_map", &defrag_map(&self.object_id_map, &self.insert_order))?;
        list.serialize_field("object_id_map_inherit", &defrag_map(&self.object_id_map_inherit, &self.insert_order))?;
        list.serialize_field("inactive_id_map", &defrag_map(&self.inactive_id_map, &self.insert_order))?;
        list.end()
    }
}

impl Serialize for TileList {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut list = serializer.serialize_struct("TileList", 3)?;
        list.serialize_field("chunks", &self.chunks)?;
        list.serialize_field("insert_order", &defrag(&self.insert_order))?;
        list.serialize_field("draw_order", &defrag(&self.draw_order))?;
        list.end()
    }
}

fn defrag(list: &[usize]) -> Vec<usize> {
    let mut output = Vec::with_capacity(list.len());
    for i in list.iter() {
        output.push(list.iter().copied().filter(|x| x < i).count())
    }
    output
}

fn defrag_map(map: &HashMap<i32, Vec<usize>>, all_insts: &[usize]) -> HashMap<i32, Vec<usize>> {
    let mut output = HashMap::new();
    for (obj_id, in_vec) in map.iter() {
        let mut out_vec = Vec::with_capacity(in_vec.len());
        for i in in_vec.iter() {
            out_vec.push(all_insts.iter().copied().filter(|x| x < i).count())
        }
        output.insert(*obj_id, out_vec);
    }
    output
}

// TODO: Maybe preallocating order/draw_order would increase perf - test this!
