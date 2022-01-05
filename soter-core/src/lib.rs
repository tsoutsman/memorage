#![deny(non_ascii_idents, rustdoc::broken_intra_doc_links)]
#![warn(
    // missing_docs,
    rust_2018_idioms,
    rust_2021_compatibility,
    future_incompatible,
    missing_debug_implementations,
    missing_copy_implementations,
    nonstandard_style,
    clippy::missing_panics_doc,
    clippy::missing_safety_doc
)]

pub use rand;
use serde::{Deserialize, Serialize};

pub const PORT: u16 = 1117;

#[derive(Debug)]
pub struct KeyPair {
    pub public: PublicKey,
    pub private: PrivateKey,
}

impl KeyPair {
    pub fn generate<T>(rng: &mut T) -> Self
    where
        T: rand::CryptoRng + rand::RngCore,
    {
        // Key Pair is two words
        let kp = ed25519_dalek::Keypair::generate(rng);
        Self {
            public: PublicKey(kp.public),
            private: PrivateKey(kp.secret),
        }
    }

    pub fn from_entropy() -> Self {
        // God help us all with the fifteen different fucking rand versions please it's 1 AM why
        // are my local builds working but CI builds failing.
        Self::generate(&mut rand::thread_rng())
    }

    pub fn to_pkcs8(&self) -> [u8; 85] {
        // Adapted from https://github.com/briansmith/ring/blob/main/src/pkcs8.rs
        // Poor man's DER encoding.
        const BEFORE_PRIVATE_KEY: [u8; 16] = [
            0x30, 0x43, 0x02, 0x01, 0x01, 0x30, 0x04, 0x06, 0x03, 0x2b, 0x65, 0x70, 0x04, 0x22,
            0x04, 0x20,
        ];
        const AFTER_PRIVATE_KEY: [u8; 5] = [0xa1, 0x23, 0x03, 0x21, 0x00];

        let mut bytes = [0; 85];
        bytes[..16].copy_from_slice(&BEFORE_PRIVATE_KEY);
        bytes[16..(16 + 32)].copy_from_slice(self.private.as_ref());
        bytes[(16 + 32)..(16 + 32 + 5)].copy_from_slice(&AFTER_PRIVATE_KEY);
        bytes[(16 + 32 + 5)..].copy_from_slice(self.public.as_ref());
        bytes
    }
}

#[derive(Copy, Clone, Debug, Eq, Serialize, Deserialize)]
pub struct PublicKey(ed25519_dalek::PublicKey);

impl AsRef<[u8]> for PublicKey {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl From<&PrivateKey> for PublicKey {
    fn from(pk: &PrivateKey) -> Self {
        Self((&pk.0).into())
    }
}

impl TryFrom<&[u8]> for PublicKey {
    type Error = KeyGenerationError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Ok(Self(
            ed25519_dalek::PublicKey::from_bytes(value).map_err(|_| KeyGenerationError)?,
        ))
    }
}

impl std::hash::Hash for PublicKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_ref().hash(state);
    }
}

impl std::cmp::PartialEq for PublicKey {
    fn eq(&self, other: &Self) -> bool {
        self.as_ref() == other.as_ref()
    }
}

#[derive(Debug)]
pub struct PrivateKey(ed25519_dalek::SecretKey);

impl PrivateKey {
    pub fn public(&self) -> PublicKey {
        PublicKey::from(self)
    }
}

impl AsRef<[u8]> for PrivateKey {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl TryFrom<&[u8]> for PrivateKey {
    type Error = KeyGenerationError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Ok(Self(
            ed25519_dalek::SecretKey::from_bytes(value).map_err(|_| KeyGenerationError)?,
        ))
    }
}

#[derive(Copy, Clone, Debug)]
pub struct KeyGenerationError;

impl std::fmt::Display for KeyGenerationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "error generating key")
    }
}

impl std::error::Error for KeyGenerationError {}
