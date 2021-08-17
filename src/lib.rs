#![warn(
    // missing_docs,
    rust_2018_idioms,
    rust_2021_compatibility,
    future_incompatible,
    missing_debug_implementations,
    missing_copy_implementations,
    rustdoc::broken_intra_doc_links
)]
#![allow(dead_code)]
#![allow(clippy::len_without_is_empty)]

pub mod crypto;
pub mod error;
pub mod fs;
pub mod net;
pub mod stun;
pub mod test;
