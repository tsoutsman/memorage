use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct EncryptedPath(String);

impl<P> From<P> for EncryptedPath
where
    P: AsRef<Path>,
{
    fn from(p: P) -> Self {
        let mut result = String::new();
        // TODO: to_string_lossy?
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
