#![deny(non_ascii_idents, rustdoc::broken_intra_doc_links)]
#![warn(
    // missing_docs,
    rust_2018_idioms,
    rust_2021_compatibility,
    future_incompatible,
    missing_debug_implementations,
    missing_copy_implementations,
    nonstandard_style,
    clippy::missing_panics_doc,
    clippy::missing_safety_doc
)]
#![feature(int_roundings, concat_idents)]

mod config;
mod error;

pub use config::Config;
pub use error::{Error, Result};

pub mod crypto;
pub mod fs;
pub mod net;
