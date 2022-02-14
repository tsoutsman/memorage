use memorage_core::{rand::seq::SliceRandom, KeyPair, PrivateKey};

lazy_static::lazy_static! {
    static ref ARGON2: argon2::Argon2<'static> = {
        let mut params = argon2::ParamsBuilder::new();
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
        argon2::Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params)
    };
}

const WORD_LIST: [&str; 2048] = memorage_macros::word_list!("english.txt");

/// A mnemonic phrase consisting of a number of words and optionally a password.
///
/// The recommended length is at least 14 words.
#[derive(Debug, Eq, PartialEq)]
pub struct MnemonicPhrase<'a> {
    words: Vec<&'a str>,
    password: Option<String>,
}

impl<'a> MnemonicPhrase<'a> {
    pub fn new(words: Vec<&'a str>, password: Option<String>) -> Self {
        Self { words, password }
    }
}

impl MnemonicPhrase<'static> {
    /// Randomly select `num_words` words from the word list `english.txt`. Unlike BIP39,
    /// `MnemonicPhrase` does not use any checksums, and so each word encodes a full 11
    /// bits of entropy.
    pub fn generate(num_words: usize, password: Option<String>) -> Self {
        Self {
            words: WORD_LIST
                .choose_multiple(&mut memorage_core::rand::thread_rng(), num_words)
                .copied()
                .collect(),
            password,
        }
    }
}

impl Default for MnemonicPhrase<'static> {
    fn default() -> Self {
        Self::generate(14, None)
    }
}

impl From<MnemonicPhrase<'_>> for KeyPair {
    /// Convert a `MnemonicPhrase` into a [`KeyPair`].
    ///
    /// The algorithm is as follows:
    /// * Concatenate all the words in the mnemonic together.
    /// * If there is a password, append the string " password " followed by the
    ///   password.
    /// * Append the string " :)".
    /// * Hash the UTF-8 representation of the created string using the Argon2id hash function
    ///   with a tag length of 32, 24 iterations, and the UTF-8 representation of "very secure
    ///   salt" as the salt.
    /// * Hash the UTF-8 representation of the original string again using the Argon2id hash
    ///   function with the same parameters, but use the output of the previous hash as the salt.
    /// * Use the output of this hash function as the private key.
    ///
    /// The algorithm in pseudo-python:
    /// ```python
    /// string = "".join(words)
    /// if password != "":
    ///     string += f" password {password}"
    /// string += " :)"
    /// let salt = argon2id(string, salt="very secure salt", iterations=24)
    /// let private_key = argon2id(string, salt=salt, iterations=24)
    /// ```
    ///
    /// # Example
    /// ```rust
    /// # fn argon2id(password: &[u8], salt: &[u8]) -> [u8; 32] {
    /// # let mut params = argon2::ParamsBuilder::new();
    /// # params
    /// #     .m_cost(4096)
    /// #     .unwrap()
    /// #     .t_cost(24)
    /// #     .unwrap()
    /// #     .p_cost(8)
    /// #     .unwrap()
    /// #     .output_len(32)
    /// #     .unwrap();
    /// # let params = params.params().unwrap();
    /// # let argon2 = argon2::Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);
    /// # let mut output = [0; 32];
    /// # argon2
    /// #     .hash_password_into(password, salt, &mut output);
    /// # output.try_into().unwrap()
    /// # }
    /// use memorage_client::mnemonic::MnemonicPhrase;
    ///
    /// let key = memorage_core::KeyPair::from(MnemonicPhrase::new(
    ///     vec!["ahead", "blanket", "captain", "diamond"],
    ///     Some("hunter2".to_owned()),
    /// ))
    /// .private;
    ///
    /// let string = "aheadblanketcaptaindiamond password hunter2 :)";
    /// let salt = argon2id(string.as_bytes(), b"very secure salt");
    /// assert_eq!(
    ///     salt,
    ///     [
    ///         0xac, 0xad, 0x8e, 0xff, 0x8a, 0x04, 0xb4, 0x36, 0x8e, 0xa5, 0x9d, 0x06, 0xd7, 0xe6,
    ///         0x4e, 0x82, 0xc2, 0x3e, 0xa6, 0xf2, 0x4e, 0x7c, 0x51, 0xc6, 0x6f, 0x19, 0x1b, 0xe6,
    ///         0x4a, 0x39, 0x8a, 0x00,
    ///     ]
    /// );
    ///
    /// let output = argon2id(string.as_bytes(), &salt[..]);
    /// assert_eq!(
    ///     output,
    ///     [
    ///         0x45, 0x1e, 0x74, 0x61, 0x66, 0x13, 0x83, 0x30, 0xa1, 0x4f, 0x6e, 0xfb, 0xbd, 0xac,
    ///         0x0f, 0x7b, 0x7d, 0x9e, 0x30, 0x09, 0xc7, 0x72, 0x16, 0x9f, 0x0c, 0xd0, 0x33, 0x0e,
    ///         0xba, 0x71, 0x49, 0x09,
    ///     ]
    /// );
    /// assert_eq!(key.as_ref(), output);
    /// ```
    fn from(mp: MnemonicPhrase<'_>) -> Self {
        // TODO: Unwraps

        let mut string = String::with_capacity(mp.words.len() * 5);
        for word in mp.words {
            string.push_str(word);
        }
        if let Some(password) = mp.password {
            string.push_str(" password ");
            string.push_str(&password);
        }
        string.push_str(" :)");

        let mut salt = [0; 32];
        ARGON2
            .hash_password_into(string.as_bytes(), b"very secure salt", &mut salt)
            .unwrap();
        let mut output = [0; 32];
        ARGON2
            .hash_password_into(string.as_bytes(), &salt, &mut output)
            .unwrap();

        let private = PrivateKey::try_from(&output[..]).unwrap();
        let public = private.public();

        Self { public, private }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mnemonic_randomness() {
        let phrase_1 = MnemonicPhrase::generate(14, None);
        let phrase_2 = MnemonicPhrase::generate(14, None);
        assert_ne!(phrase_1, phrase_2);

        let key_1 = KeyPair::from(phrase_1).to_pkcs8();
        let key_2 = KeyPair::from(phrase_2).to_pkcs8();
        assert_ne!(key_1, key_2)
    }
}
