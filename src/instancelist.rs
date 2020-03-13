use crate::instance::Instance;
use std::{alloc, cmp::Ordering, ptr};

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

/// How many chunks ChunkList preallocates (16 + 102400 bytes each).
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
        let idx_div = idx / CHUNK_SIZE;
        let idx_mod = idx % CHUNK_SIZE;
        self.0.get(idx_div).and_then(|chunk| chunk.slots.get(idx_mod)).and_then(|slot| slot.as_ref())
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

    fn remove_with(&mut self, f: impl Fn(&T) -> bool) {
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
}

pub struct InstanceList {
    chunks: ChunkList<Instance>,
    order: Vec<usize>,
    draw_order: Vec<usize>,
}

pub struct Iter {
    order_idx: usize,
    draw: bool, // whether it's draw or insert ordering
}

impl Iter {
    pub fn next(&mut self, list: &InstanceList) -> Option<usize> {
        self.order_idx += 1;
        if self.draw { list.draw_order.get(self.order_idx).copied() } else { list.order.get(self.order_idx).copied() }
    }
}

impl InstanceList {
    pub fn new() -> Self {
        Self { chunks: ChunkList::new(), order: Vec::new(), draw_order: Vec::new() }
    }

    pub fn get(&self, idx: usize) -> Option<&Instance> {
        self.chunks.get(idx)
    }

    pub fn iter(&self) -> Iter {
        Iter { order_idx: 0, draw: false }
    }

    pub fn iter_draw(&self) -> Iter {
        Iter { order_idx: 0, draw: true }
    }

    pub fn draw_sort(&mut self) {
        let chunks = &self.chunks; // borrowck :)
        self.draw_order.sort_by(move |&idx1, &idx2| {
            // TODO: Bench if this is faster with unreachable_unchecked...
            let left = chunks.get(idx1).unwrap();
            let right = chunks.get(idx2).unwrap();

            // First, draw order is sorted by depth...
            match left.depth.get().cmp(&right.depth.get()) {
                Ordering::Equal => {
                    // If they're equal then it's the ordering of object index.
                    // If those are equal it's in insertion order (aka, equal, this is stablesort).
                    left.object_index.get().cmp(&right.object_index.get())
                },
                other => other,
            }
        })
    }

    pub fn insert(&mut self, instance: Instance) -> usize {
        let value = self.chunks.insert(instance);
        self.order.push(value);
        self.draw_order.push(value);
        value
    }

    pub fn remove_with(&mut self, f: impl Fn(&Instance) -> bool) {
        for chunk in self.chunks.iter_mut() {
            for slot in chunk.slots.iter_mut() {
                if let Some(instance) = slot {
                    if f(&*instance) {
                        *slot = None;
                        chunk.vacant += 1;
                    }
                }
            }
        }

        let chunks = &self.chunks;
        self.order.retain(|idx| chunks.get(*idx).is_some());
    }
}
