// This module implements the handle indices allocation as in the original GM.

use serde::{Deserialize, Serialize};
use std::{error, result};

// Required because handle initialization closures must be able to return any errors.
// TODO: Do we also need "+ Send + Sync + 'static" for BoxedStdError?
type BoxedStdError = Box<dyn error::Error>;
type InitResult<T> = result::Result<T, BoxedStdError>;

#[derive(Debug)]
pub enum Error {
    OutOfSlots,
    InitError(BoxedStdError),
}

pub type Result<T> = result::Result<T, Error>;

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::OutOfSlots => write!(f, "handle limit reached"),
            Self::InitError(e) => e.fmt(f),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::InitError(e) => Some(e.as_ref()),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandleList<T>(Vec<Option<T>>);

#[derive(Debug)]
pub struct HandleArray<T, const LEN: usize>([Option<T>; LEN]);

impl<T> HandleList<T> {
    pub fn new() -> Self {
        Self(Default::default())
    }

    pub fn get(&self, index: i32) -> Option<&T> {
        self.0.get(usize::try_from(index).ok()?)?.as_ref()
    }

    pub fn get_mut(&mut self, index: i32) -> Option<&mut T> {
        self.0.get_mut(usize::try_from(index).ok()?)?.as_mut()
    }

    pub fn put(&mut self, handle: T) -> i32 {
        self.add(handle).unwrap()
    }
}

impl<T, const LEN: usize> HandleArray<T, LEN> {
    // TODO: This is somewhat annoying.
    // https://github.com/rust-lang/rust/issues/44796
    // https://stackoverflow.com/a/66776497
    const NONE_INIT: Option<T> = None;

    pub fn new() -> Self {
        Self([Self::NONE_INIT; LEN])
    }

    pub fn get(&self, index: i32) -> Option<&T> {
        self.0.get(usize::try_from(index).ok()?)?.as_ref()
    }

    pub fn get_mut(&mut self, index: i32) -> Option<&mut T> {
        self.0.get_mut(usize::try_from(index).ok()?)?.as_mut()
    }

    pub fn capacity(&self) -> i32 {
        LEN.try_into().unwrap()
    }
}

#[inline]
fn cvt<T>(i: InitResult<T>) -> Result<T> {
    match i {
        Ok(r) => Ok(r),
        Err(e) => Err(Error::InitError(e)),
    }
}

pub trait HandleManager<T>: private::HandleStorage<T> {
    fn add(&mut self, handle: T) -> Option<i32> {
        self.add_from(|| Ok(handle)).ok()
    }

    fn add_from<F>(&mut self, init_handle: F) -> Result<i32>
    where
        F: FnOnce() -> InitResult<T>,
    {
        for (index, slot) in self.iter_mut().enumerate() {
            if slot.is_none() {
                slot.replace(cvt(init_handle())?);
                return Ok(index.try_into().unwrap())
            }
        }
        self.push_from(init_handle)
    }

    fn delete(&mut self, index: i32) -> bool {
        usize::try_from(index).ok().and_then(|x| self.iter_mut().nth(x)).and_then(|x| x.take()).is_some()
    }
}

impl<T> HandleManager<T> for HandleList<T> {}
impl<T, const LEN: usize> HandleManager<T> for HandleArray<T, LEN> {}

mod private {
    use super::*;

    pub trait HandleStorage<T> {
        fn iter_mut(&mut self) -> std::slice::IterMut<Option<T>>;
        fn push_from<F>(&mut self, init_handle: F) -> Result<i32>
        where
            F: FnOnce() -> InitResult<T>;
    }

    impl<T> HandleStorage<T> for HandleList<T> {
        fn iter_mut(&mut self) -> std::slice::IterMut<Option<T>> {
            self.0.iter_mut()
        }

        fn push_from<F>(&mut self, init_handle: F) -> Result<i32>
        where
            F: FnOnce() -> InitResult<T>,
        {
            // init will occur before push but there it's pretty legit
            self.0.push(cvt(init_handle())?.into());
            Ok((self.0.len() - 1).try_into().unwrap())
        }
    }

    impl<T, const LEN: usize> HandleStorage<T> for HandleArray<T, LEN> {
        fn iter_mut(&mut self) -> std::slice::IterMut<Option<T>> {
            self.0.iter_mut()
        }

        fn push_from<F>(&mut self, _init_handle: F) -> Result<i32>
        where
            F: FnOnce() -> InitResult<T>,
        {
            Err(Error::OutOfSlots)
        }
    }
}
