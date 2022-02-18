use std::path::{Path, PathBuf};

use crate::{
    crypto::{decrypt, encrypt},
    error::Result,
};

use memorage_core::PrivateKey;
use serde::{Deserialize, Serialize};

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
    #[allow(clippy::missing_panics_doc)]
    pub fn from_disk(_path: &Path) -> Result<Self> {
        todo!();
    }

    pub fn encrypt(&self, key: &PrivateKey) -> Result<(EncryptedPath, EncryptedFile)> {
        let serialised = bincode::serialize(&self)?;
        let (nonce, encrypted_self) = encrypt(key, &serialised)?;
        let encrypted = EncryptedFile {
            nonce,
            file: encrypted_self,
        };
        // NOTE: https://github.com/rust-lang/rust-clippy/pull/8355
        #[allow(clippy::needless_borrow)]
        Ok(((&self.path).into(), encrypted))
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct EncryptedPath(String);

impl<P> From<P> for EncryptedPath
where
    P: AsRef<Path>,
{
    fn from(p: P) -> Self {
        let mut result = String::new();
        let hash: [u8; 32] = blake3::hash(p.as_ref().to_string_lossy().as_bytes()).into();
        for x in hash {
            result.push_str(&format!("{:02x?}", x));
        }
        Self(result)
    }
}

impl AsRef<str> for EncryptedPath {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum FileContents {
    Uncompressed(Vec<u8>),
    Zstd { level: u8, contents: Vec<u8> },
}
