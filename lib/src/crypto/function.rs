use crate::{
    crypto::Key,
    error::{Error, Result},
};
use chacha20poly1305::{
    aead::{Aead, NewAead},
    XChaCha20Poly1305, XNonce,
};
use rand_chacha::{
    rand_core::{RngCore, SeedableRng},
    ChaCha20Rng,
};

pub fn encrypt(key: &Key, bytes: &[u8]) -> Result<([u8; 24], Vec<u8>)> {
    let aed = XChaCha20Poly1305::new(&key.clone().into());

    let mut rng = ChaCha20Rng::from_entropy();
    let nonce_value = &mut [0u8; 24];
    rng.fill_bytes(&mut nonce_value[..]);

    let nonce = XNonce::from_slice(nonce_value);

    let encrypted = match aed.encrypt(nonce, bytes) {
        Ok(c) => c,
        Err(_) => return Err(Error::Encryption),
    };

    Ok((nonce_value.to_owned(), encrypted))
}

pub fn decrypt(key: &Key, nonce: &[u8; 24], bytes: &[u8]) -> Result<Vec<u8>> {
    let aed = XChaCha20Poly1305::new(&key.clone().into());
    let nonce = XNonce::from_slice(nonce);

    let decrypted = match aed.decrypt(nonce, bytes) {
        Ok(c) => c,
        Err(_) => return Err(Error::Encryption),
    };

    Ok(decrypted)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_correctly() {
        let key = Key::from("super secret key");
        let message = b"super secret message pls don't steal";

        let (nonce, encrypted) = encrypt(&key, message).unwrap();
        let decrypted = decrypt(&key, &nonce, &encrypted).unwrap();

        assert_eq!(&decrypted, message);
    }

    #[test]
    fn test_decrypt_incorrect_key() {
        let key = Key::from("super secret key");
        let message = b"super secret message pls don't steal";

        let (nonce, encrypted) = encrypt(&key, message).unwrap();

        let incorrect_key = Key::from("secret key");
        let decrypted = decrypt(&incorrect_key, &nonce, &encrypted);
        assert!(decrypted.is_err());
    }
}
