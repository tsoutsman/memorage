use crate::Result;

use memorage_core::PrivateKey;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EncryptedFile {
    nonce: [u8; 24],
    file: Vec<u8>,
}

impl EncryptedFile {
    pub fn encrypt<T>(file: &T, key: &PrivateKey) -> Result<EncryptedFile>
    where
        T: serde::Serialize,
    {
        let serialised = bincode::serialize(file)?;
        let (nonce, encrypted_file) = crate::crypto::encrypt(key, &serialised)?;
        let encrypted = EncryptedFile {
            nonce,
            file: encrypted_file,
        };
        Ok(encrypted)
    }

    pub fn decrypt(&self, key: &PrivateKey) -> Result<Vec<u8>> {
        let decrypted = crate::crypto::decrypt(key, &self.nonce, &self.file)?;
        let deserialized = bincode::deserialize(&decrypted)?;
        Ok(deserialized)
    }
}
