use rand::RngCore;
use rand_chacha::{rand_core::SeedableRng, ChaCha20Rng};
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;
use soter_core::{KeyPair, PublicKey, Verifiable};

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

    pub fn create_verifiable<T>(&self, kp: &KeyPair, inner: T) -> Verifiable<T> {
        let signature = kp.sign(self);
        Verifiable::new(signature, inner)
    }

    pub fn create_verifiable_key(&self, kp: &KeyPair) -> Verifiable<PublicKey> {
        let signature = kp.sign(self);
        let inner = kp.public_key();

        Verifiable::new(signature, inner)
    }
}

impl AsRef<[u8]> for SigningBytes {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Default for SigningBytes {
    fn default() -> Self {
        Self::new()
    }
}
