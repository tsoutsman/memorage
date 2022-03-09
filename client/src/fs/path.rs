use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct EncryptedPath(PathBuf);

// TODO: Specialisation
impl From<&Path> for EncryptedPath {
    fn from(path: &Path) -> Self {
        let mut result = String::new();
        // TODO: to_string_lossy?
        let hash: [u8; 32] = blake3::hash(path.to_string_lossy().as_bytes()).into();
        for x in hash {
            result.push_str(&format!("{:02x?}", x));
        }
        Self(result.into())
    }
}

impl AsRef<Path> for EncryptedPath {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}
