use crate::{
    crypto::Key,
    error::Result,
    fs::file::{EncryptedFile, File},
};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Backup {
    pub version: u16,
    path_hashes: Vec<[u8; 32]>,
    files: Vec<EncryptedFile>,
}

impl Backup {
    pub fn with_version(v: u16) -> Self {
        Self {
            version: v,
            path_hashes: Vec::new(),
            files: Vec::new(),
        }
    }

    pub fn deserialise_hashes(bytes: &[u8]) -> Result<Vec<[u8; 32]>> {
        #[derive(Deserialize)]
        struct BackupFileHashes {
            version: u16,
            path_hashes: Vec<[u8; 32]>,
        }
        // TODO does this deserialisation load the entire file into memory?
        let temp: BackupFileHashes = bincode::deserialize(bytes)?;
        Ok(temp.path_hashes)
    }

    pub fn encrypt_and_add(&mut self, key: &Key, file: File) -> Result<()> {
        let encrypted = file.encrypt(key)?;
        self.path_hashes.push(encrypted.0);
        self.files.push(encrypted.1);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fs::file::FileContents;
    use std::path::PathBuf;

    #[test]
    fn test_backup_file_deserialise_hashes() {
        let p1: PathBuf = "/example/path/file_1".into();
        let f1 = File {
            path: p1.clone(),
            contents: FileContents::Uncompressed(vec![1, 2, 3, 4, 5]),
        };

        let p2: PathBuf = "/example/path/file_2".into();
        let f2 = File {
            path: p2.clone(),
            contents: FileContents::Uncompressed(vec![5, 6, 7]),
        };

        let mut bf = Backup::with_version(1);
        let key = Key::from("hello");
        bf.encrypt_and_add(&key, f1).unwrap();
        bf.encrypt_and_add(&key, f2).unwrap();

        let bf_serialised = bincode::serialize(&bf).unwrap();
        let result = Backup::deserialise_hashes(&bf_serialised).unwrap();

        let expected: Vec<[u8; 32]> = vec![
            blake3::hash(p1.to_string_lossy().as_bytes()).into(),
            blake3::hash(p2.to_string_lossy().as_bytes()).into(),
        ];

        assert_eq!(result, expected);
    }
}
