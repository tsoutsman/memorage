use crate::cs::protocol::error::Error;

use rand::{distributions::Alphanumeric, Rng};
use rand_chacha::{rand_core::SeedableRng, ChaCha20Rng};
use serde::{Deserialize, Serialize};

/// Struct representing the code used during pairing.
///
/// The code consists of [`LEN`](Self::LEN) alphanumeric characters.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Code(String);

impl Code {
    /// The length of the code in characters.
    pub const LEN: usize = 6;

    /// Create a random code.
    pub fn new() -> Self {
        Self(
            ChaCha20Rng::from_entropy()
                .sample_iter(&Alphanumeric)
                .take(Self::LEN)
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
    type Error = Error;

    /// Returns an error if the [`String`] is of incorrect length.
    fn try_from(c: String) -> Result<Self, Self::Error> {
        if c.len() == Self::LEN {
            Ok(Self(c))
        } else {
            Err(Error::InvalidCode)
        }
    }
}
