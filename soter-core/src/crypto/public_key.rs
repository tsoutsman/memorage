use crate::Signature;

use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PublicKey([u8; 32]);

impl AsRef<[u8]> for PublicKey {
    fn as_ref(&self) -> &[u8] {
        &self.0[..]
    }
}

impl TryFrom<&[u8]> for PublicKey {
    type Error = KeyLengthError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Ok(Self(
            <[u8; 32]>::try_from(value).map_err(|_| KeyLengthError)?,
        ))
    }
}

impl PublicKey {
    pub fn verify<B>(&self, bytes: B, signature: Signature) -> Result<(), VerificationError>
    where
        B: AsRef<[u8]>,
    {
        let public_key =
            ring::signature::UnparsedPublicKey::new(&ring::signature::ED25519, self.as_ref());
        public_key
            .verify(bytes.as_ref(), signature.as_ref())
            .map_err(|_| VerificationError)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct KeyLengthError;

impl std::fmt::Display for KeyLengthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "incorrect key length")
    }
}

impl std::error::Error for KeyLengthError {}

#[derive(Copy, Clone, Debug)]
pub struct VerificationError;

impl std::fmt::Display for VerificationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid verification signature")
    }
}

impl std::error::Error for VerificationError {}
