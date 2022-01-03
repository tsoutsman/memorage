use crate::{PublicKey, Signature};

use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Verifiable<T> {
    signature: Signature,
    inner: T,
}

impl<T> Verifiable<T> {
    pub fn new(signature: Signature, inner: T) -> Verifiable<T> {
        Self { signature, inner }
    }

    #[allow(clippy::result_unit_err)]
    pub fn verify<B>(self, bytes: &B, public_key: &PublicKey) -> Result<T, VerificationError>
    where
        B: AsRef<[u8]>,
    {
        public_key
            .verify(bytes.as_ref(), self.signature)
            .map(|_| self.inner)
    }
}

impl Verifiable<PublicKey> {
    #[allow(clippy::result_unit_err)]
    pub fn into_key<B>(self, bytes: &B) -> Result<PublicKey, VerificationError>
    where
        B: AsRef<[u8]>,
    {
        let public_key =
            ring::signature::UnparsedPublicKey::new(&ring::signature::ED25519, self.inner.as_ref());
        match public_key.verify(bytes.as_ref(), self.signature.as_ref()) {
            Ok(_) => Ok(self.inner),
            Err(_) => Err(VerificationError),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct VerificationError;

impl std::fmt::Display for VerificationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "verification failed")
    }
}

impl std::error::Error for VerificationError {}
