// This module implements the handle indices allocation as in the original GM.

use serde::{Serialize, Deserialize};

// TODO: Replace with 'const generics' when stabilized.
const ARRAY_SIZE: usize = 32;  // as limited in GM file API

// Required because handle initialization closures must be able to return unknown errors.
// https://doc.rust-lang.org/rust-by-example/error/multiple_error_types/boxing_errors.html
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, Clone)]
pub struct OutOfHandleSlotsError;

impl std::fmt::Display for OutOfHandleSlotsError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "handle limit reached")
    }
}

impl std::error::Error for OutOfHandleSlotsError {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandleList<T> (Vec<Option<T>>,);

#[derive(Debug)]
pub struct HandleArray<T> ([Option<T>; ARRAY_SIZE],);

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

impl<T> HandleArray<T> {
    pub fn new() -> Self {
        Self ( Default::default() )
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.0.get(index)?.as_ref()
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.0.get_mut(index)?.as_mut()
    }
}

pub trait HandleManager<T>: private::HandleStorage<T> {
    fn add(&mut self, handle: T) -> Option<usize> {
        self.add_from(|| Ok(handle)).ok()
    }

    fn add_from<F>(&mut self, init_handle: F) -> Result<usize>
        where F: FnOnce() -> Result<T>,
    {
        for (index, slot) in self.iter_mut().enumerate() {
            if slot.is_none() {
                slot.replace(init_handle()?);
                return Ok(index);
            }
        }
        self.push_from(init_handle)
    }

    fn delete(&mut self, index: usize) -> bool {
        self.iter_mut()
            .nth(index)
            .and_then(|x| x.take())
            .is_some()
    }
}

impl<T> HandleManager<T> for HandleList<T> {}
impl<T> HandleManager<T> for HandleArray<T> {}

mod private {
    use super::*;

    pub trait HandleStorage<T> {
        fn iter_mut(&mut self) -> std::slice::IterMut<Option<T>>;
        fn push_from<F>(&mut self, init_handle: F) -> Result<usize>
            where F: FnOnce() -> Result<T>;
    }

    impl<T> HandleStorage<T> for HandleList<T> {
        fn iter_mut(&mut self) -> std::slice::IterMut<Option<T>> {
            self.0.iter_mut()
        }

        fn push_from<F>(&mut self, init_handle: F) -> Result<usize>
            where F: FnOnce() -> Result<T>
        {
            // init will occur before push but there it's pretty legit
            self.0.push(init_handle()?.into());
            Ok(self.0.len() - 1)
        }
    }

    impl<T> HandleStorage<T> for HandleArray<T> {
        fn iter_mut(&mut self) -> std::slice::IterMut<Option<T>> {
            self.0.iter_mut()
        }

        fn push_from<F>(&mut self, _init_handle: F) -> Result<usize>
            where F: FnOnce() -> Result<T>
        {
            Err(OutOfHandleSlotsError.into())
        }
    }
}
