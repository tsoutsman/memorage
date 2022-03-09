use crate::Result;

use std::path::Path;

use memorage_core::PrivateKey;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EncryptedFile {
    nonce: [u8; 24],
    file: Vec<u8>,
}

impl EncryptedFile {
    pub fn serialize(&self) -> Result<Vec<u8>> {
        bincode::serialize(&self).map_err(|e| e.into())
    }
}

impl EncryptedFile {
    pub fn from_disk(path: &Path, key: &PrivateKey) -> Result<EncryptedFile> {
        let bytes = std::fs::read(path)?;
        Self::encrypt(&bytes, key)
    }

    pub fn encrypt(bytes: &[u8], key: &PrivateKey) -> Result<EncryptedFile> {
        let (nonce, file) = crate::crypto::encrypt(key, bytes)?;
        Ok(Self { nonce, file })
    }

    pub fn decrypt(&self, key: &PrivateKey) -> Result<Vec<u8>> {
        crate::crypto::decrypt(key, &self.nonce, &self.file)
    }
}
