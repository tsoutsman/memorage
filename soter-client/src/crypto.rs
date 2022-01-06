use crate::error::{Error, Result};

use chacha20poly1305::{
    aead::{Aead, NewAead},
    XChaCha20Poly1305, XNonce,
};
use soter_core::rand::{thread_rng, RngCore};
use soter_core::PrivateKey;

pub fn encrypt(key: &PrivateKey, bytes: &[u8]) -> Result<([u8; 24], Vec<u8>)> {
    let aed = XChaCha20Poly1305::new(chacha20poly1305::Key::from_slice(key.as_ref()));

    let mut rng = thread_rng();
    let nonce_value = &mut [0u8; 24];
    rng.fill_bytes(&mut nonce_value[..]);

    let nonce = XNonce::from_slice(nonce_value);

    let encrypted = match aed.encrypt(nonce, bytes) {
        Ok(c) => c,
        Err(_) => return Err(Error::Encryption),
    };

    Ok((nonce_value.to_owned(), encrypted))
}

pub fn decrypt(key: &PrivateKey, nonce: &[u8], bytes: &[u8]) -> Result<Vec<u8>> {
    let aed = XChaCha20Poly1305::new(chacha20poly1305::Key::from_slice(key.as_ref()));
    let nonce = XNonce::from_slice(nonce);

    let decrypted = match aed.decrypt(nonce, bytes) {
        Ok(c) => c,
        Err(_) => return Err(Error::Decryption),
    };

    Ok(decrypted)
}

#[cfg(test)]
mod tests {
    use super::*;
    use soter_core::KeyPair;

    #[test]
    fn test_encrypt_correctly() {
        let key = KeyPair::from_entropy().private;
        let message = b"super secret message pls don't steal";

        let (nonce, encrypted) = encrypt(&key, message).unwrap();
        let decrypted = decrypt(&key, &nonce, &encrypted).unwrap();

        assert_eq!(&decrypted, message);
    }

    #[test]
    fn test_decrypt_incorrect_key() {
        let key = KeyPair::from_entropy().private;
        let message = b"super secret message pls don't steal";

        let (nonce, encrypted) = encrypt(&key, message).unwrap();

        let incorrect_key = KeyPair::from_entropy().private;
        let decrypted = decrypt(&incorrect_key, &nonce, &encrypted);
        assert!(matches!(decrypted, Err(Error::Decryption)));
    }
}
