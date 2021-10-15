use argon2::{Algorithm, Argon2, ParamsBuilder, Version};
use lazy_static::lazy_static;

lazy_static! {
    static ref ARGON2: Argon2<'static> = {
        let mut params = ParamsBuilder::new();
        params
            .m_cost(4096)
            .unwrap()
            .t_cost(24)
            .unwrap()
            .p_cost(8)
            .unwrap()
            .output_len(32)
            .unwrap();
        let params = params.params().unwrap();
        Argon2::new(Algorithm::Argon2id, Version::V0x13, params)
    };
}

/// A key used to encrypt data.
#[derive(Copy, Clone)]
#[allow(missing_debug_implementations)]
pub struct Key([u8; 32]);

impl Key {
    #[inline]
    pub fn hash(&self) -> [u8; 32] {
        self.0
    }
}

impl std::convert::From<Key> for chacha20poly1305::Key {
    #[inline]
    fn from(key: Key) -> Self {
        chacha20poly1305::Key::from(key.0)
    }
}

impl<T: AsRef<str>> std::convert::From<T> for Key {
    // TODO maybe make lazy but cached?
    fn from(p: T) -> Self {
        // TODO unwraps
        let p = p.as_ref().as_bytes();

        let mut salt = [0; 32];
        ARGON2
            .hash_password_into(p, b"very secure salt :)", &mut salt)
            .unwrap();
        let mut output = [0; 32];
        ARGON2.hash_password_into(p, &salt, &mut output).unwrap();

        Self(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{distributions::Alphanumeric, Rng};
    use rand_chacha::{rand_core::SeedableRng, ChaCha20Rng};

    #[test]
    fn test_key_hash() {
        for i in 0..10 {
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
