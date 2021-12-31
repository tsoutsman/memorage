use crate::cs::key::SigningBytes;

use ed25519_dalek::{Signature, SignatureError, Verifier};
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Verifiable<T> {
    pub(crate) signature: Signature,
    pub(crate) inner: T,
}

impl<T> Verifiable<T>
where
    T: Verifier<Signature>,
{
    pub fn into_verifier(self, signing_bytes: &SigningBytes) -> Result<T, SignatureError> {
        self.inner.verify(signing_bytes.bytes(), &self.signature)?;
        Ok(self.inner)
    }
}

impl<T> Verifiable<T> {
    pub fn verify<V>(self, signing_bytes: &SigningBytes, verifier: &V) -> Result<T, SignatureError>
    where
        V: Verifier<Signature>,
    {
        verifier.verify(signing_bytes.bytes(), &self.signature)?;
        Ok(self.inner)
    }
}
