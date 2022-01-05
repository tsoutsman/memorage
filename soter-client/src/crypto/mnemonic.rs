use soter_core::{rand::seq::SliceRandom, KeyPair, PrivateKey};

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

const WORD_LIST: [&str; 2048] = soter_macros::word_list!("english.txt");

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
    pub fn generate(num_words: usize, password: Option<String>) -> Self {
        Self {
            words: WORD_LIST
                .choose_multiple(&mut soter_core::rand::thread_rng(), num_words)
                .copied()
                .collect(),
            password,
        }
    }
}

impl From<MnemonicPhrase<'_>> for KeyPair {
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
            .hash_password_into(string.as_bytes(), b"very secure salt :)", &mut salt)
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
