use ed25519_dalek::{Signature, SignatureError, Verifier};
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

    pub fn verify<B, V>(self, bytes: &B, verifier: &V) -> Result<T, SignatureError>
    where
        B: AsRef<[u8]>,
        V: Verifier<Signature>,
    {
        verifier.verify(bytes.as_ref(), &self.signature)?;
        Ok(self.inner)
    }
}

impl<V> Verifiable<V>
where
    V: Verifier<Signature>,
{
    pub fn into_verifier<B>(self, bytes: &B) -> Result<V, SignatureError>
    where
        B: AsRef<[u8]>,
    {
        self.inner.verify(bytes.as_ref(), &self.signature)?;
        Ok(self.inner)
    }
}
