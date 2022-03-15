use std::marker::PhantomData;

use crate::{Error, Result};

use chacha20poly1305::{
    aead::{Aead, NewAead},
    XChaCha20Poly1305, XNonce,
};
use memorage_core::rand::{thread_rng, RngCore};
use memorage_core::PrivateKey;

use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Encrypted<T>
where
    T: Serialize + DeserializeOwned,
{
    nonce: [u8; 24],
    value: Vec<u8>,
    _marker: PhantomData<T>,
}

impl<T> Encrypted<T>
where
    T: Serialize + DeserializeOwned,
{
    pub fn encrypt(key: &PrivateKey, value: &T) -> Result<Self> {
        let data = bincode::serialize(value)?;
        let aed = XChaCha20Poly1305::new(chacha20poly1305::Key::from_slice(key.as_ref()));

        let mut rng = thread_rng();
        let mut nonce = [0; 24];
        rng.fill_bytes(&mut nonce[..]);

        let xnonce = XNonce::from_slice(&nonce);

        let encrypted = match aed.encrypt(xnonce, data.as_ref()) {
            Ok(c) => c,
            Err(_) => return Err(Error::Encryption),
        };

        Ok(Self {
            nonce,
            value: encrypted,
            _marker: PhantomData,
        })
    }

    pub fn decrypt(&self, key: &PrivateKey) -> Result<T> {
        let aed = XChaCha20Poly1305::new(chacha20poly1305::Key::from_slice(key.as_ref()));
        let nonce = XNonce::from_slice(&self.nonce);

        let decrypted = match aed.decrypt(nonce, self.value.as_ref()) {
            Ok(c) => c,
            Err(_) => return Err(Error::Decryption),
        };
        bincode::deserialize(&decrypted).map_err(|e| e.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use memorage_core::KeyPair;

    #[test]
    fn encrypt_correctly() {
        let key = KeyPair::from_entropy().private;
        let message = b"super secret message pls don't steal".to_vec();

        let encrypted = Encrypted::encrypt(&key, &message).unwrap();
        let decrypted = encrypted.decrypt(&key).unwrap();

        assert_eq!(decrypted, message);
    }

    #[test]
    fn decrypt_incorrect_key() {
        let key = KeyPair::from_entropy().private;
        let message = b"super secret message pls don't steal".to_vec();

        let encrypted = Encrypted::encrypt(&key, &message).unwrap();

        let incorrect_key = KeyPair::from_entropy().private;
        let decrypted = encrypted.decrypt(&incorrect_key);
        assert!(matches!(decrypted, Err(Error::Decryption)));
    }
}
