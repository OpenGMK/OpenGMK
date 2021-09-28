use crate::{gml::Value, math::Real};
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, collections};

pub type Result<T> = std::result::Result<T, Error>;

pub type Stack = Vec<Value>;
pub type Queue = collections::VecDeque<Value>;
pub type List = Vec<Value>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Map {
    pub keys: Vec<Value>, // should be pre-sorted
    pub values: Vec<Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Priority {
    pub priorities: Vec<Value>,
    pub values: Vec<Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Grid {
    grid: Vec<Vec<Value>>,
    height: usize, // if width is 0, this is inaccessible otherwise
}

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

impl Priority {
    fn extremity(&self, precision: Real, diff: Ordering) -> Option<usize> {
        if self.priorities.is_empty() {
            return None
        }
        let mut ext = 0;
        for i in 1..self.priorities.len() {
            if cmp(&self.priorities[i], &self.priorities[ext], precision) == diff {
                ext = i;
            }
        }
        Some(ext)
    }

    pub fn min_id(&self, precision: Real) -> Option<usize> {
        self.extremity(precision, Ordering::Less)
    }

    pub fn max_id(&self, precision: Real) -> Option<usize> {
        self.extremity(precision, Ordering::Greater)
    }
}

impl Grid {
    pub fn new(width: usize, height: usize) -> Self {
        let grid = vec![vec![Value::from(0); height]; width];
        Self { grid, height }
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        self.height = height;
        self.grid.resize_with(width, || vec![Value::from(0); height]);
        for column in &mut self.grid {
            column.resize_with(height, Default::default);
        }
    }

    // This will panic on OOB, so make sure you check bounds before calling
    pub fn get(&self, x: usize, y: usize) -> &Value {
        &self.grid[x][y]
    }

    // This will panic on OOB, so make sure you check bounds before calling
    pub fn get_mut(&mut self, x: usize, y: usize) -> &mut Value {
        &mut self.grid[x][y]
    }

    /// Goes through each column
    pub fn all_mut(&mut self) -> impl Iterator<Item = &mut Value> {
        self.grid.iter_mut().flatten()
    }

    pub fn width(&self) -> usize {
        self.grid.len()
    }

    pub fn height(&self) -> usize {
        self.height
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
