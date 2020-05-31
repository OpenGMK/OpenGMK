use crate::gml::Value;
use std::collections;
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct DataStructureManager<T> {
    map: collections::HashMap<i32, T>,
}

pub type Stack = Vec<Value>;
pub type Queue = collections::VecDeque<Value>;
pub type List = Vec<Value>;
#[derive(Debug)]
pub struct Map {
    pub keys: Vec<Value>, // should be pre-sorted
    pub values: Vec<Value>,
}
#[derive(Debug)]
pub struct Priority {
    pub priorities: Vec<Value>,
    pub values: Vec<Value>,
}
pub type Grid = Vec<Vec<Value>>;

#[derive(Debug)]
pub enum Error {
    NonexistentStructure(i32),
}

impl From<Error> for String {
    fn from(e: Error) -> Self {
        match e {
            Error::NonexistentStructure(id) => format!("data structure with index {} does not exist", id),
        }
    }
}

impl<T> DataStructureManager<T> {
    pub fn new() -> Self {
        Self { map: collections::HashMap::new() }
    }

    pub fn add(&mut self, to_add: T) -> i32 {
        for id in 0..self.map.len() as i32 + 1 {
            if !self.map.contains_key(&id) {
                self.map.insert(id, to_add);
                return id as i32
            }
        }
        unreachable!()
    }

    pub fn get(&mut self, id: i32) -> Result<&mut T> {
        self.map.get_mut(&id).ok_or(Error::NonexistentStructure(id))
    }

    pub fn destroy(&mut self, id: i32) -> Result<()> {
        match self.map.remove(&id) {
            Some(_) => Ok(()),
            None => Err(Error::NonexistentStructure(id)),
        }
    }
}