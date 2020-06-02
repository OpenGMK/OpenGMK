use crate::{gml::Value, math::Real};
use std::{cmp::Ordering, collections};
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

    pub fn get(&self, id: i32) -> Result<&T> {
        self.map.get(&id).ok_or(Error::NonexistentStructure(id))
    }

    pub fn get_mut(&mut self, id: i32) -> Result<&mut T> {
        self.map.get_mut(&id).ok_or(Error::NonexistentStructure(id))
    }

    pub fn destroy(&mut self, id: i32) -> Result<()> {
        match self.map.remove(&id) {
            Some(_) => Ok(()),
            None => Err(Error::NonexistentStructure(id)),
        }
    }
}

impl Map {
    // Returns the index associated with the given key, or None if there is none.
    pub fn get_index(&self, key: &Value, precision: Real) -> Option<usize> {
        match self.keys.binary_search_by(|x| cmp(x, key, precision)) {
            Ok(mut index) => {
                while index > 0 && eq(&self.keys[index - 1], key, precision) {
                    index -= 1;
                }
                Some(index)
            },
            Err(_) => None,
        }
    }

    // Returns the index of the given key, or if there is none, that of its successor.
    pub fn get_index_unchecked(&self, key: &Value, precision: Real) -> usize {
        match self.keys.binary_search_by(|x| cmp(x, key, precision)) {
            Ok(mut index) => {
                while index > 0 && eq(&self.keys[index - 1], key, precision) {
                    index -= 1;
                }
                index
            },
            Err(index) => index,
        }
    }

    // Returns the index of the key following the given key.
    pub fn get_next_index(&self, key: &Value, precision: Real) -> usize {
        match self.keys.binary_search_by(|x| cmp(x, key, precision)) {
            Ok(mut index) => {
                while index < self.keys.len() && eq(&self.keys[index], key, precision) {
                    index += 1;
                }
                index
            },
            Err(index) => index,
        }
    }

    pub fn contains_key(&self, key: &Value, precision: Real) -> bool {
        self.keys.binary_search_by(|x| cmp(x, key, precision)).is_ok()
    }
}

pub fn eq(v1: &Value, v2: &Value, precision: Real) -> bool {
    match (v1, v2) {
        (Value::Real(x), Value::Real(y)) => (*x - *y).abs() <= precision,
        (Value::Str(x), Value::Str(y)) => x == y,
        _ => false,
    }
}

pub fn cmp(v1: &Value, v2: &Value, precision: Real) -> Ordering {
    match (v1, v2) {
        (Value::Real(x), Value::Real(y)) => {
            if (*x - *y).abs() <= precision {
                Ordering::Equal
            } else {
                x.partial_cmp(&y).unwrap()
            }
        },
        (Value::Str(x), Value::Str(y)) => x.cmp(y),
        (Value::Real(_), Value::Str(_)) => Ordering::Less,
        (Value::Str(_), Value::Real(_)) => Ordering::Greater,
    }
}
