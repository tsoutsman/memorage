use crate::{fs::hash, Result};

use std::path::{Path, PathBuf};

use bimap::BiMap;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use serde::{Deserialize, Serialize};
use tokio::fs::File;

#[derive(Clone, Default, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Index(BiMap<PathBuf, [u8; 32]>);

impl Index {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn from_directory<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
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

        let handle = tokio::runtime::Handle::current();
        let paths = paths
            .into_par_iter()
            .map(|file_path| -> Result<(PathBuf, [u8; 32])> {
                handle.block_on(async {
                    let hash = hash(File::open(&file_path).await?)?;
                    Ok((file_path, hash))
                })
            });

        let mut index = Self::new();

        // TODO: Don't collect
        for result in paths.collect::<Vec<_>>() {
            let (path, hash) = result?;
            index.0.insert(path, hash);
        }

        Ok(index)
    }

    // TODO: Clones or consume self (or serialize refs?)?
    pub fn difference(&self, other: &Index) -> Vec<IndexDifference> {
        let mut diff = Vec::new();

        for (path, hash) in &self.0 {
            match (other.0.get_by_left(path), other.0.get_by_right(hash)) {
                (Some(_), Some(_)) => {}
                (None, Some(old_path)) => diff.push(IndexDifference::Rename {
                    from: old_path.clone(),
                    to: path.clone(),
                }),
                (Some(_), None) | (None, None) => diff.push(IndexDifference::Write(path.clone())),
            }
        }

        for (path, hash) in &other.0 {
            if !self.0.contains_left(path) && !self.0.contains_right(hash) {
                diff.push(IndexDifference::Delete(path.clone()))
            }
        }

        diff
    }
}

impl<'a> IntoIterator for &'a Index {
    type Item = (&'a PathBuf, &'a [u8; 32]);
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Iter<'a> {
        self.0.iter()
    }
}

pub type Iter<'a> = bimap::hash::Iter<'a, PathBuf, [u8; 32]>;

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum IndexDifference {
    Write(PathBuf),
    Rename { from: PathBuf, to: PathBuf },
    Delete(PathBuf),
}
