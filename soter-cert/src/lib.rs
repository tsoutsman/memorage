#![deny(non_ascii_idents, rustdoc::broken_intra_doc_links)]
#![warn(
    // missing_docs,
    rust_2018_idioms,
    // rust_2021_compatibility,
    future_incompatible,
    missing_debug_implementations,
    missing_copy_implementations,
    nonstandard_style,
    clippy::missing_panics_doc,
    clippy::missing_safety_doc
)]
mod config;
mod error;
mod verify;

pub use config::{gen_recv_config, gen_send_config};
pub use error::{Error, Result};
pub use verify::get_key_unchecked;
