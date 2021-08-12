use crate::error::{Error, Result};

use std::{
    fs,
    io::{ErrorKind, Read},
    path::Path,
};

use chacha20poly1305::{
    aead::{Aead, NewAead},
    XChaCha20Poly1305, XNonce,
};
use rand_chacha::{
    rand_core::{RngCore, SeedableRng},
    ChaCha20Rng,
};

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct Key(Vec<u8>);

impl Key {
    pub fn from<S: AsRef<str>>(p: S) -> Self {
        p.into()
    }

    // IDK why this method is necessary; slices and vecs are complicated.
    pub fn from_slice(p: &[u8]) -> Self {
        Self(p.to_owned())
    }

    /// This function hashes the password using the blake3 hash.
    /// blake3 is very fast and not designed to be used as a password hashing algorithm, but we
    /// aren't using it as one. This is only used to turn a password with a variable length, into
    /// something that's always 32 bytes so that it can then be used by chacha. The hash is never
    /// stored anywhere, and a bad actor would only be able to see encrypted files which have
    /// been encrypted with this hash as the key. This reduces the effectiveness of brute
    /// force attacks as the decryption of the files would be the bottleneck.
    pub fn hash(&self) -> [u8; 32] {
        blake3::hash(&self.0).as_bytes().to_owned()
    }
}

impl std::convert::From<&Key> for chacha20poly1305::Key {
    fn from(key: &Key) -> Self {
        chacha20poly1305::Key::clone_from_slice(&key.hash())
    }
}

impl<T: AsRef<str>> std::convert::From<T> for Key {
    fn from(p: T) -> Self {
        Self(p.as_ref().to_owned().as_bytes().to_owned())
    }
}

pub fn encrypt_contents<P: AsRef<Path>>(path: &P, key: &Key) -> Result<Vec<u8>> {
    let path = path.as_ref();

    let mut file = match fs::File::open(path) {
        Ok(f) => f,
        Err(e) => match e.kind() {
            ErrorKind::NotFound => return Err(Error::FileNotFound(path.to_owned())),
            ErrorKind::PermissionDenied => return Err(Error::PermissionDenied(path.to_owned())),
            _ => return Err(e.into()),
        },
    };

    let size = file.metadata()?.len() as usize;
    let mut contents = vec![0; size];
    file.read_exact(&mut contents)?;

    let aed = XChaCha20Poly1305::new(&key.into());

    let mut rng = ChaCha20Rng::from_entropy();
    let nonce_value = &mut [0u8; 24];
    rng.fill_bytes(&mut nonce_value[..]);

    let nonce = XNonce::from_slice(nonce_value);

    let mut encrypted = match aed.encrypt(nonce, contents.as_ref()) {
        Ok(e) => e,
        Err(_) => return Err(Error::Encryption(path.to_owned())),
    };

    // Append the nonce value to the end of the file so it can then be decrypted.
    encrypted.extend(nonce_value.to_owned());

    Ok(encrypted)
}

pub fn decrypt_contents<P: AsRef<Path>>(path: &P, key: &Key) -> Result<Vec<u8>> {
    let path = path.as_ref();

    let mut file = match fs::File::open(path) {
        Ok(f) => f,
        Err(e) => match e.kind() {
            ErrorKind::NotFound => return Err(Error::FileNotFound(path.to_owned())),
            ErrorKind::PermissionDenied => return Err(Error::PermissionDenied(path.to_owned())),
            _ => return Err(e.into()),
        },
    };

    let size = file.metadata()?.len() as usize;
    let mut encrypted: Vec<u8> = vec![0; size];
    file.read_exact(&mut encrypted)?;

    let aed = XChaCha20Poly1305::new(&key.into());

    // The nonce is stored in the last 24 bytes of the file.
    let nonce_value = &encrypted[size - 24..size];
    let nonce = XNonce::from_slice(nonce_value);

    // The file minus the nonce value.
    let encrypted_contents = &encrypted[0..size - 24];

    let decrypted = match aed.decrypt(nonce, encrypted_contents.as_ref()) {
        Ok(d) => d,
        Err(_) => return Err(Error::Encryption(path.to_owned())),
    };

    Ok(decrypted)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    // This is more of a sanity check than anything.
    #[test]
    fn test_key_hash() {
        let mut rng = ChaCha20Rng::from_entropy();
        for i in 1..100 {
            let mut password = vec![0u8; i];
            rng.fill_bytes(&mut password);
            let hash = Key::from_slice(&password).hash();

            assert_ne!(hash, &password[..]);
            assert_eq!(hash.len(), 32);
        }
    }

    #[test]
    fn test_encrypt_correctly() {
        let root_path = tempdir().unwrap().into_path();

        let message = b"super secret message pls don't steal";

        let file_path = root_path.join("foo");
        let mut file = fs::File::create(&file_path).unwrap();
        file.write_all(message.as_ref()).unwrap();

        let key = Key::from("super secret key");
        let encrypted = encrypt_contents(&file_path, &key).unwrap();

        let encrypted_file_path = root_path.join("encrypted_foo");
        let mut encrypted_file = fs::File::create(encrypted_file_path.clone()).unwrap();
        encrypted_file.write_all(&encrypted).unwrap();

        let decrypted = decrypt_contents(&encrypted_file_path, &key).expect("UEOE: ");

        assert_eq!(&decrypted, message);
    }

    #[test]
    #[should_panic]
    fn test_decrypt_incorrect_key() {
        let root_path = tempdir().unwrap().into_path();

        let message = b"super secret message pls don't steal";

        let file_path = root_path.join("foo");
        let mut file = fs::File::create(&file_path).unwrap();
        file.write_all(message.as_ref()).unwrap();

        let key = Key::from("super secret key");
        let encrypted = encrypt_contents(&file_path, &key).unwrap();

        let encrypted_file_path = root_path.join("encrypted_foo");
        let mut encrypted_file = fs::File::create(encrypted_file_path.clone()).unwrap();
        encrypted_file.write_all(&encrypted).unwrap();

        // Note the changed key
        let key = Key::from("secret key");
        // This unwrap should fail as the key is incorrect.
        decrypt_contents(&encrypted_file_path, &key).unwrap();
    }
}
