use memorage_core::rand::{distributions::Alphanumeric, thread_rng, Rng};
use serde::{Deserialize, Serialize};

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

impl std::fmt::Display for PairingCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

// TODO specialisation
impl std::convert::TryFrom<String> for PairingCode {
    type Error = PairingCodeError;

    /// Returns an error if the [`String`] is of incorrect length.
    fn try_from(s: String) -> Result<Self, Self::Error> {
        s.parse()
    }
}

impl std::str::FromStr for PairingCode {
    type Err = PairingCodeError;

    /// This implementation is only here for clap
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() == Self::LEN {
            Ok(Self(s.to_owned()))
        } else {
            Err(PairingCodeError)
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct PairingCodeError;

impl std::fmt::Display for PairingCodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid pairing code format")
    }
}

impl std::error::Error for PairingCodeError {}
