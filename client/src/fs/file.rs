use std::path::PathBuf;

use crate::{
    crypto::{decrypt, encrypt},
    error::Result,
};

use serde::{Deserialize, Serialize};
use memorage_core::PrivateKey;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EncryptedFile {
    nonce: [u8; 24],
    file: Vec<u8>,
}

impl EncryptedFile {
    pub fn decrypt(&self, key: &PrivateKey) -> Result<File> {
        let decrypted = decrypt(key, &self.nonce, &self.file)?;
        let deserialized = bincode::deserialize(&decrypted)?;
        Ok(deserialized)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct File {
    pub path: PathBuf,
    pub contents: FileContents,
}

impl File {
    pub fn encrypt(&self, key: &PrivateKey) -> Result<(FileName, EncryptedFile)> {
        let path_hash: [u8; 32] = blake3::hash(self.path.to_string_lossy().as_bytes()).into();
        let serialised = bincode::serialize(&self)?;
        let (nonce, encrypted_self) = encrypt(key, &serialised)?;
        let encrypted = EncryptedFile {
            nonce,
            file: encrypted_self,
        };
        Ok((path_hash.as_ref().into(), encrypted))
    }
}

#[derive(Debug)]
pub struct FileName(String);

impl From<&[u8]> for FileName {
    fn from(a: &[u8]) -> Self {
        let mut result = String::with_capacity(a.len() * 2);
        for x in a {
            result.push_str(&format!("{:02x?}", x));
        }
        Self(result)
    }
}

impl From<FileName> for String {
    fn from(f: FileName) -> Self {
        f.0
    }
}

impl AsRef<str> for FileName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum FileContents {
    Uncompressed(Vec<u8>),
    Zstd { level: u8, contents: Vec<u8> },
}
