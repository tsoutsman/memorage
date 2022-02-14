#![deny(
    non_ascii_idents,
    // missing_docs,
    rust_2018_idioms,
    rust_2021_compatibility,
    future_incompatible,
    missing_debug_implementations,
    missing_copy_implementations,
    nonstandard_style,
    unreachable_pub,
    clippy::missing_panics_doc,
    clippy::missing_safety_doc,
    rustdoc::broken_intra_doc_links
)]

mod code;
mod error;
pub mod serde;

pub use crate::serde::{deserialize, serialize, Deserialize, Serialize};
pub use code::PairingCode;
pub use error::{Error, Result};

pub mod request;
pub mod response;

mod private {
    pub trait Sealed {}
}
