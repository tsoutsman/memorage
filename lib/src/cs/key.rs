pub use crate::cs::protocol::error::Result;

use ed25519_dalek::ExpandedSecretKey;
use rand::RngCore;
use rand_chacha::{rand_core::SeedableRng, ChaCha20Rng};
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

pub use ed25519_dalek::{Keypair, PublicKey, Signature, Verifier};

/// A struct representing a not yet verified key.
///
/// The struct consists of a [`Signature`] and a [`PublicKey`] whose corresponding
/// private key was allegedly used to create the signature.
///
/// The underlying [`PublicKey`] can only be extracted using the [`into_key`] function.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerifiablePublicKey {
    signature: Signature,
    key: PublicKey,
}

impl VerifiablePublicKey {
    pub fn new(keypair: &Keypair, signing_bytes: &SigningBytes) -> Self {
        let signature =
            ExpandedSecretKey::from(&keypair.secret).sign(&signing_bytes.0, &keypair.public);
        let key = keypair.public;

        Self { signature, key }
    }

    /// Verify that the [`Signature`] on the [`SigningBytes`] was generated with this key.
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
