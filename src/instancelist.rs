use crate::instance::Instance;

const INSTANCES_PER_CHUNK: usize = 256;

fn chunk_insert<'a, T>(chunks: &'a mut Vec<InstanceChunk<T>>, t: T) -> usize {
    match chunks.iter_mut().enumerate().find(|(_, chunk)| chunk.vacant != 0) {
        Some((idx, chunk)) => {
            chunk.vacant -= 1;
            match chunk.slots.iter_mut().enumerate().find(|(_, slot)| slot.is_none()) {
                Some((slot_idx, slot @ None)) => {
                    *slot = Some(t);
                    (idx * INSTANCES_PER_CHUNK) + slot_idx
                },
                _ => unreachable!(),
            }
        },
        None => {
            let mut chunk = InstanceChunk::new();
            chunk.vacant -= 1;
            chunk.slots[0] = Some(t);
            chunks.push(chunk);
            (chunks.len() - 1) * INSTANCES_PER_CHUNK
        },
    }
}

pub struct InstanceList {
    chunks: Vec<InstanceChunk<Instance>>,
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

struct InstanceChunk<T> {
    slots: Box<[Option<T>; INSTANCES_PER_CHUNK]>,
    vacant: usize,
}

impl<T> InstanceChunk<T> {
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
            vacant: INSTANCES_PER_CHUNK,
        }
    }
}

impl InstanceList {
    pub fn new() -> Self {
        Self { chunks: vec![InstanceChunk::new()], order: Vec::new(), order_depth: Vec::new() }
    }

    pub fn get(&self, idx: usize) -> Option<&Instance> {
        // TODO: does LLVM optimize out div/mod ?
        self.chunks
            .get(idx / INSTANCES_PER_CHUNK)
            .and_then(|chunk| chunk.slots.get(idx % INSTANCES_PER_CHUNK))
            .and_then(|slot| slot.as_ref())
    }

    pub fn iter(&self) -> Iter {
        Iter { order_idx: 0 }
    }

    pub fn insert(&mut self, instance: Instance) -> usize {
        let value = chunk_insert(&mut self.chunks, instance);
        self.order.push(value);
        self.order_depth.push(value);
        value
    }

    pub fn remove_with(&mut self, f: impl Fn(&Instance) -> bool) {
        for chunk in &mut self.chunks {
            for slot in chunk.slots.iter_mut() {
                if let Some(instance) = slot {
                    if f(&*instance) {
                        *slot = None;
                        chunk.vacant += 1;
                    }
                }
            }
        }
        let chunks = &mut self.chunks; // borrowck :)
        self.order.retain(|idx| chunks.get(*idx).is_some());
    }
}
