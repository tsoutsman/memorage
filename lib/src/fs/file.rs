use std::path::PathBuf;

use crate::{
    crypto::{encrypt, Key},
    error::Result,
};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(super) struct EncryptedFile {
    nonce: [u8; 24],
    file: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct File {
    pub path: PathBuf,
    pub contents: FileContents,
}

impl File {
    pub(super) fn encrypt(&self, key: &Key) -> Result<([u8; 32], EncryptedFile)> {
        let path_hash = blake3::hash(self.path.to_string_lossy().as_bytes()).into();
        let serialised = bincode::serialize(&self)?;
        let (nonce, encrypted_self) = encrypt(key, &serialised)?;
        let encrypted = EncryptedFile {
            nonce,
            file: encrypted_self,
        };
        Ok((path_hash, encrypted))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum FileContents {
    Uncompressed(Vec<u8>),
    Zstd { level: u8, contents: Vec<u8> },
}
