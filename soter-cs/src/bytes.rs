use ed25519_dalek::{ExpandedSecretKey, Keypair, PublicKey, Signature, Signer};
use rand::RngCore;
use rand_chacha::{rand_core::SeedableRng, ChaCha20Rng};
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;
use soter_core::Verifiable;

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

    pub fn bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn create_verifiable<S, T>(
        &self,
        signer: S,
        inner: T,
    ) -> Result<Verifiable<T>, ed25519_dalek::ed25519::Error>
    where
        S: Signer<Signature>,
    {
        let signature = signer.try_sign(self.bytes())?;
        Ok(Verifiable::new(signature, inner))
    }

    pub fn create_verifiable_key(&self, keypair: &Keypair) -> Verifiable<PublicKey> {
        let signature =
            ExpandedSecretKey::from(&keypair.secret).sign(self.bytes(), &keypair.public);
        let inner = keypair.public;

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
