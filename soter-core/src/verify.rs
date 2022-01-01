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
    pub fn verify<B>(self, bytes: &B, public_key: &PublicKey) -> Result<T, ()>
    where
        B: AsRef<[u8]>,
    {
        let public_key =
            ring::signature::UnparsedPublicKey::new(&ring::signature::ED25519, public_key.as_ref());
        match public_key.verify(bytes.as_ref(), self.signature.as_ref()) {
            Ok(_) => Ok(self.inner),
            Err(_) => Err(()),
        }
    }
}

impl Verifiable<PublicKey> {
    #[allow(clippy::result_unit_err)]
    pub fn into_key<B>(self, bytes: &B) -> Result<PublicKey, ()>
    where
        B: AsRef<[u8]>,
    {
        let public_key =
            ring::signature::UnparsedPublicKey::new(&ring::signature::ED25519, self.inner.as_ref());
        match public_key.verify(bytes.as_ref(), self.signature.as_ref()) {
            Ok(_) => Ok(self.inner),
            Err(_) => Err(()),
        }
    }
}
