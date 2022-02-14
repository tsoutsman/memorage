use serde::{Deserialize, Serialize};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Error {
    #[error("invalid pairing code")]
    InvalidPairingCode,
    #[error("unknown error")]
    Generic,
    #[error("error serializing response")]
    Serialization,
    #[error("invalid signature")]
    InvalidSignature,
    #[error("server has no data on requested key")]
    NoData,
}
