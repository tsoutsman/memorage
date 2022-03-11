use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// A path to an encrypted file.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct HashedPath(PathBuf);

impl From<&Path> for HashedPath {
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

// TODO: Specialisation

impl From<PathBuf> for HashedPath {
    fn from(path: PathBuf) -> Self {
        Self::from(path.as_path())
    }
}

impl AsRef<Path> for HashedPath {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}
