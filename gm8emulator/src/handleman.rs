// This module implements the handle indices allocation as in the original GM.

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandleList<T> (Vec<Option<T>>,);

impl<T> HandleList<T> {
    pub fn new() -> Self {
        Self ( Default::default() )
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.0.get(index)?.as_ref()
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.0.get_mut(index)?.as_mut()
    }

    pub fn put(&mut self, handle: T) -> usize {
        self.add(handle).unwrap()
    }
}

pub trait HandleManager<T>: private::HandleStorage<T> {
    fn add(&mut self, handle: T) -> Option<usize> {
        for (index, slot) in self.iter_mut().enumerate() {
            if slot.is_none() {
                slot.replace(handle);
                return Some(index);
            }
        }
        self.push(handle)
    }

    fn delete(&mut self, index: usize) -> bool {
        self.iter_mut()
            .nth(index)
            .and_then(|x| x.take())
            .is_some()
    }
}

impl<T> HandleManager<T> for HandleList<T> {}

mod private {
    use super::*;

    pub trait HandleStorage<T> {
        fn iter_mut(&mut self) -> std::slice::IterMut<Option<T>>;
        fn push(&mut self, handle: T) -> Option<usize>;
    }

    impl<T> HandleStorage<T> for HandleList<T> {
        fn iter_mut(&mut self) -> std::slice::IterMut<Option<T>> {
            self.0.iter_mut()
        }

        fn push(&mut self, handle: T) -> Option<usize> {
            self.0.push(handle.into());
            Some(self.0.len() - 1)
        }
    }
}
