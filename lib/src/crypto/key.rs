/// A key used to encrypt data.
#[derive(Clone)]
#[allow(missing_debug_implementations)]
pub struct Key(Vec<u8>);

impl Key {
    /// This function hashes the password using the blake3 hash.
    /// blake3 is very fast and not designed to be used as a password hashing algorithm, but we
    /// aren't using it as one. This is only used to turn a password with a variable length, into
    /// something that's always 32 bytes so that it can then be used by chacha. The hash is never
    /// stored anywhere, and a bad actor would only be able to see encrypted files which have
    /// been encrypted with this hash as the key. This reduces the effectiveness of brute
    /// force attacks as the decryption of the files would be the bottleneck.
    fn hash(&self) -> [u8; 32] {
        blake3::hash(&self.0).as_bytes().to_owned()
    }
}

impl std::convert::From<Key> for chacha20poly1305::Key {
    fn from(key: Key) -> Self {
        chacha20poly1305::Key::from(key.hash())
    }
}

impl<T: AsRef<str>> std::convert::From<T> for Key {
    fn from(p: T) -> Self {
        Self(p.as_ref().as_bytes().to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{distributions::Alphanumeric, Rng};
    use rand_chacha::{rand_core::SeedableRng, ChaCha20Rng};

    #[test]
    fn test_key_hash() {
        for i in 0..100 {
            let password: String = ChaCha20Rng::from_entropy()
                .sample_iter(&Alphanumeric)
                .take(i)
                .map(char::from)
                .collect();

            let hash = Key::from(password.clone()).hash();

            assert_ne!(hash, password.as_bytes());
            assert_eq!(hash.len(), 32);
        }
    }
}
