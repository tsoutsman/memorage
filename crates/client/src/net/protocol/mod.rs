mod error;
mod serde;

pub use crate::net::protocol::serde::{deserialize, serialize, Deserialize, Serialize};
pub use error::{Error, Result};

pub mod request;
pub mod response;

mod private {
    pub trait Sealed {}
}
