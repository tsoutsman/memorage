#![deny(
    non_ascii_idents,
    // missing_docs,
    rust_2018_idioms,
    // rust_2021_compatibility,
    future_incompatible,
    missing_debug_implementations,
    missing_copy_implementations,
    nonstandard_style,
    unreachable_pub,
    clippy::missing_panics_doc,
    clippy::missing_safety_doc,
    rustdoc::broken_intra_doc_links
)]
#![feature(int_roundings, concat_idents, try_blocks)]

mod error;

pub use error::{Error, Result};

pub mod crypto;
pub mod fs;
pub mod io;
pub mod mnemonic;
pub mod net;
pub mod persistent;
