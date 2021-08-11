use crate::error::{Error, Result};

use std::{
    fs,
    io::{ErrorKind, Read},
    path::Path,
};

use chacha20poly1305::{
    aead::{Aead, NewAead},
    Key, XChaCha20Poly1305, XNonce,
};
use rand_chacha::{
    rand_core::{RngCore, SeedableRng},
    ChaCha20Rng,
};

#[allow(dead_code)]
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

    let aed = XChaCha20Poly1305::new(key);

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

#[allow(dead_code)]
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

    let aed = XChaCha20Poly1305::new(key);

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
    use chacha20poly1305::Key;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_encrypt_contents() {
        let root_path = tempdir().unwrap().into_path();

        let message = b"super secret message pls don't steal";

        let file_path = root_path.join("foo");
        let mut file = fs::File::create(&file_path).unwrap();
        file.write_all(message.as_ref()).unwrap();

        let key = Key::from_slice(b"secret keyec.tuhjbunetonec'lec.t");
        let encrypted = encrypt_contents(&file_path, key).unwrap();

        let encrypted_file_path = root_path.join("encrypted_foo");
        let mut encrypted_file = fs::File::create(encrypted_file_path.clone()).unwrap();
        encrypted_file.write_all(&encrypted).unwrap();

        let decrypted = decrypt_contents(&encrypted_file_path, key).unwrap();

        assert_eq!(&decrypted, message);
    }
}
