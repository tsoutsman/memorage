use crate::{
    fs::{hash, EncryptedPath},
    Result,
};

use std::{
    collections::HashMap,
    fs::File,
    path::{Path, PathBuf},
};

use bimap::{BiMap, Overwritten};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Index(BiMap<EncryptedPath, [u8; 32]>);

impl std::default::Default for Index {
    fn default() -> Self {
        Self(BiMap::new())
    }
}

impl Index {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_directory(path: &Path) -> Result<(Self, HashMap<EncryptedPath, PathBuf>)> {
        let mut paths = Vec::new();

        for entry in jwalk::WalkDir::new(path) {
            // TODO: Document that empty folders are not backed up.
            // TODO: Symbolic links
            let entry = entry?;

            if entry.file_type().is_file() {
                let file_path = entry.path();
                paths.push(file_path);
            }
            // TODO: Do we return error if it isn't.
        }

        let paths =
            paths
                .into_par_iter()
                .map(|file_path| -> Result<(PathBuf, EncryptedPath, [u8; 32])> {
                    let encrypted_file_path = EncryptedPath::from(file_path.clone());
                    let hash = hash(File::open(&file_path)?)?;
                    Ok((file_path, encrypted_file_path, hash))
                });

        let mut paths_map = HashMap::new();
        let mut index = Self::new();

        // TODO: Don't collect
        for result in paths.collect::<Vec<_>>() {
            let (path, encrypted_path, hash) = result?;
            paths_map.insert(encrypted_path.clone(), path);
            index.insert(encrypted_path, hash);
        }

        Ok((index, paths_map))
    }

    pub fn from_disk(path: &Path) -> Result<Self> {
        // TODO: Buffered?
        bincode::deserialize_from(File::open(path)?).map_err(|e| e.into())
    }

    pub fn to_disk(&self, path: &Path) -> Result<()> {
        // TODO: Buffered?
        bincode::serialize_into(File::create(path)?, &self).map_err(|e| e.into())
    }

    /// Returns the changes necessary to convert `other` into `self`.
    // TODO: Clones or consume self (or serialize refs?)?
    pub fn difference(&self, other: &Self) -> Vec<IndexDifference> {
        let mut diff = Vec::new();

        for (path, hash) in &self.0 {
            match (other.0.get_by_left(path), other.0.get_by_right(hash)) {
                (Some(_), Some(_)) => {}
                (Some(_), None) => diff.push(IndexDifference::Edit(path.clone())),
                (None, Some(old_path)) => diff.push(IndexDifference::Rename {
                    from: old_path.clone(),
                    to: path.clone(),
                }),
                (None, None) => diff.push(IndexDifference::Add(path.clone())),
            }
        }

        for (path, hash) in &other.0 {
            if !self.0.contains_left(path) && !self.0.contains_right(hash) {
                diff.push(IndexDifference::Delete(path.clone()))
            }
        }

        diff
    }

    fn insert(
        &mut self,
        path: EncryptedPath,
        hash: [u8; 32],
    ) -> Overwritten<EncryptedPath, [u8; 32]> {
        self.0.insert(path, hash)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum IndexDifference {
    Add(EncryptedPath),
    Edit(EncryptedPath),
    Rename {
        from: EncryptedPath,
        to: EncryptedPath,
    },
    Delete(EncryptedPath),
}
