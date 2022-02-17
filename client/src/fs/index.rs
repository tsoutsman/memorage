use crate::{fs::file::EncryptedFileName, Result};

use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    time::SystemTime,
};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Default, Serialize, Deserialize)]
pub struct Index(HashSet<IndexEntry>);

impl Index {
    pub fn new() -> Self {
        Self(HashSet::new())
    }

    pub fn from_disk(path: &Path) -> Result<(Self, HashMap<EncryptedFileName, PathBuf>)> {
        let mut paths = HashMap::new();
        let mut index = Self::new();

        for entry in walkdir::WalkDir::new(path).into_iter() {
            // TODO: Document that empty folders are not backed up.
            // TODO: Symbolic links

            let entry = entry?;

            if entry.file_type().is_file() {
                let metadata = entry.metadata()?;

                let file_path = entry.path();
                let encrypted_file_path: EncryptedFileName = file_path.into();

                let index_entry = IndexEntry {
                    path: encrypted_file_path.clone(),
                    modified: metadata.modified()?,
                };

                paths.insert(encrypted_file_path, file_path.to_owned());
                index.0.insert(index_entry);
            }
        }

        Ok((index, paths))
    }

    pub fn changed_files(&self, path: &Path) -> Result<Vec<PathBuf>> {
        let (index, mut paths) = Self::from_disk(path)?;

        let mut result = Vec::new();

        for entry in index.0.symmetric_difference(&self.0) {
            // If a file was modified, it would be in the symmetric difference
            // twice, once with each modification time. After we add it to
            // result once, it is no longer in paths, and we don't need to add
            // it again.
            if let Some(path) = paths.remove(&entry.path) {
                result.push(path);
            }
        }

        Ok(result)
    }

    pub fn insert(&mut self, value: IndexEntry) -> bool {
        self.0.insert(value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct IndexEntry {
    pub path: EncryptedFileName,
    pub modified: SystemTime,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs::File, io::Write};

    #[test]
    fn index_changed_files() {
        let index_1 = Index::new();

        let dir = tempfile::tempdir().unwrap().into_path();
        efes::gen_fs!(dir => (dir_1: file_1 file_2) file_3);

        let files = index_1.changed_files(&dir).unwrap();
        assert_eq!(
            HashSet::<PathBuf>::from_iter(files),
            HashSet::from_iter(vec![
                dir.join("dir_1").join("file_1"),
                dir.join("dir_1").join("file_2"),
                dir.join("file_3"),
            ])
        );

        let (index_2, _) = Index::from_disk(&dir).unwrap();
        assert_eq!(index_2.changed_files(&dir).unwrap(), Vec::<PathBuf>::new());

        std::thread::sleep(std::time::Duration::from_millis(10));

        let mut file = File::options()
            .write(true)
            .open(dir.join("dir_1").join("file_2"))
            .unwrap();
        file.write_all(b"BEAR > USEC").unwrap();
        File::create(dir.join("file_4")).unwrap();

        let files = index_2.changed_files(&dir).unwrap();
        assert_eq!(
            HashSet::<PathBuf>::from_iter(files),
            HashSet::from_iter(vec![dir.join("dir_1").join("file_2"), dir.join("file_4"),])
        );
    }
}

// struct Index {
//     path_hashes: HashSet<EncryptedFileName>,
//     entries: Encrypted<HashSet<IndexEntry>>
// }
