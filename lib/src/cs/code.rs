use rand::{distributions::Alphanumeric, Rng};
use rand_chacha::{rand_core::SeedableRng, ChaCha20Rng};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Code(String);

impl Code {
    pub const LEN: usize = 6;

    pub fn new() -> Self {
        Self(
            ChaCha20Rng::from_entropy()
                .sample_iter(&Alphanumeric)
                .take(6)
                .map(char::from)
                .collect(),
        )
    }
}

impl Default for Code {
    fn default() -> Self {
        Self::new()
    }
}

// TODO specialisation
impl std::convert::TryFrom<String> for Code {
    type Error = crate::cs::error::Error;

    fn try_from(c: String) -> Result<Self, Self::Error> {
        if c.len() == Self::LEN {
            Ok(Self(c))
        } else {
            Err(crate::cs::error::Error::IncorrectCodeLength(c))
        }
    }
}
