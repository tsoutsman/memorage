pub use crate::cs::protocol::error::Result;

pub use ed25519_dalek::{PublicKey, Signature, Verifier};
use rand::RngCore;
use rand_chacha::{rand_core::SeedableRng, ChaCha20Rng};
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerifiablePublicKey {
    signature: Signature,
    key: PublicKey,
}

impl VerifiablePublicKey {
    pub fn into_key(&self, b: &SigningBytes) -> Result<PublicKey> {
        self.key.verify(&b.0, &self.signature)?;
        Ok(self.key)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SigningBytes(#[serde(with = "BigArray")] [u8; Self::LEN]);

impl SigningBytes {
    pub const LEN: usize = 128;

    pub fn new() -> Self {
        let mut rng = ChaCha20Rng::from_entropy();
        let mut bytes = [0; Self::LEN];
        rng.fill_bytes(&mut bytes);

        Self(bytes)
    }
}

impl Default for SigningBytes {
    fn default() -> Self {
        Self::new()
    }
}
