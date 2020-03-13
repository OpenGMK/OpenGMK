use crate::instance::Instance;

const CHUNK_SIZE: usize = 256;

struct ChunkList<T>(Vec<Chunk<T>>);

struct Chunk<T> {
    slots: Box<[Option<T>; CHUNK_SIZE]>,
    vacant: usize,
}

impl<T> Chunk<T> {
    pub fn new() -> Self {
        Self {
            // TODO: fix this, please
            slots: Box::new([
                None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None,
            ]),
            vacant: CHUNK_SIZE,
        }
    }
}

impl<T> ChunkList<T> {
    fn new() -> Self {
        Self(vec![Chunk::new()])
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
}

pub struct InstanceList {
    chunks: ChunkList<Instance>,
    order: Vec<usize>,
    order_depth: Vec<usize>,
}

pub struct Iter {
    order_idx: usize,
}

impl Iter {
    pub fn next(&mut self, list: &InstanceList) -> Option<usize> {
        self.order_idx += 1;
        list.order.get(self.order_idx).copied()
    }
}

impl InstanceList {
    pub fn new() -> Self {
        Self { chunks: ChunkList::new(), order: Vec::new(), order_depth: Vec::new() }
    }

    pub fn get(&self, idx: usize) -> Option<&Instance> {
        self.chunks.get(idx)
    }

    pub fn iter(&self) -> Iter {
        Iter { order_idx: 0 }
    }

    pub fn insert(&mut self, instance: Instance) -> usize {
        let value = self.chunks.insert(instance);
        self.order.push(value);
        self.order_depth.push(value);
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
