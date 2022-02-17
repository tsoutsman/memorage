use crate::fs::file::FileName;

use std::{
    collections::{hash_map::RandomState, hash_set::SymmetricDifference, HashSet},
    time::SystemTime,
};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Index(HashSet<IndexEntry>);

impl Index {
    pub fn diff<'a>(
        &'a self,
        other: &'a Index,
    ) -> SymmetricDifference<'a, IndexEntry, RandomState> {
        self.0.symmetric_difference(&other.0)
    }

    pub fn insert(&mut self, value: IndexEntry) -> bool {
        self.0.insert(value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct IndexEntry {
    pub name: FileName,
    pub modified: SystemTime,
}
