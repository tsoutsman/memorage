use crate::Signature;

use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicKey([u8; 32]);

impl AsRef<[u8]> for PublicKey {
    fn as_ref(&self) -> &[u8] {
        &self.0[..]
    }
}

impl PublicKey {
    #[allow(clippy::result_unit_err)]
    pub fn verify<B>(&self, bytes: B, signature: Signature) -> Result<(), ()>
    where
        B: AsRef<[u8]>,
    {
        let public_key =
            ring::signature::UnparsedPublicKey::new(&ring::signature::ED25519, self.as_ref());
        public_key
            .verify(bytes.as_ref(), signature.as_ref())
            .map_err(|_| ())
    }
}
