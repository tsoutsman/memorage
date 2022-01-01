use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Signature(#[serde(with = "BigArray")] [u8; 64]);

impl AsRef<[u8]> for Signature {
    fn as_ref(&self) -> &[u8] {
        &self.0[..]
    }
}

impl From<ring::signature::Signature> for Signature {
    fn from(s: ring::signature::Signature) -> Self {
        Self(<[u8; 64]>::try_from(s.as_ref()).unwrap())
    }
}
