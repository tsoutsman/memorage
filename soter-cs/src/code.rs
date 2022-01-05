use serde::{Deserialize, Serialize};
use soter_core::rand::{distributions::Alphanumeric, thread_rng, Rng};

/// Struct representing the code used during pairing.
///
/// The code consists of [`LEN`](Self::LEN) alphanumeric characters.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PairingCode(String);

impl PairingCode {
    /// The length of the code in characters.
    pub const LEN: usize = 6;

    /// Create a random code.
    pub fn new() -> Self {
        Self(
            thread_rng()
                .sample_iter(&Alphanumeric)
                .take(Self::LEN)
                .map(char::from)
                .collect(),
        )
    }
}

impl Default for PairingCode {
    fn default() -> Self {
        Self::new()
    }
}

// TODO specialisation
impl std::convert::TryFrom<String> for PairingCode {
    type Error = ();

    /// Returns an error if the [`String`] is of incorrect length.
    fn try_from(c: String) -> Result<Self, Self::Error> {
        if c.len() == Self::LEN {
            Ok(Self(c))
        } else {
            Err(())
        }
    }
}
