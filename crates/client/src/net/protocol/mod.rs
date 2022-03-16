mod error;
mod serde;

pub use crate::net::protocol::serde::{deserialize, serialize, Deserialize, Serialize};
pub use error::{Error, Result};

pub mod request;
pub mod response;

mod private {
    pub trait Sealed {}
}

pub(crate) const FILE_FRAME_SIZE: usize = 65536;
pub(crate) const NONCE_LENGTH: usize = 24;
pub(crate) const TAG_LENGTH: usize = 16;
pub(crate) const ENCRYPTED_FILE_FRAME_SIZE: usize = NONCE_LENGTH + FILE_FRAME_SIZE + TAG_LENGTH;
