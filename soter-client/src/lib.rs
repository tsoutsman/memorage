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

mod config;
mod conn;
mod error;

use crate::conn::Connection;

use soter_core::{KeyPair, PublicKey};
use soter_cs::request;

const SERVER_ADDRESS: &str = "some address";

pub use config::Config;
pub use error::{Error, Result};

pub mod crypto;
pub mod fs;

pub async fn establish_connection(
    key_pair: &KeyPair,
    target_key: PublicKey,
    config: &Config,
) -> Result<Connection> {
    let public_address = soter_cert::public_address(soter_cert::DEFAULT_STUN_SERVER)
        .await?
        .ip();
    let _crypto = soter_cert::gen_crypto(public_address, Some(key_pair));

    let mut server = Connection::try_to(SERVER_ADDRESS).await?;

    let signing_bytes = server.request(request::GetSigningBytes).await?.0;
    let initiator_key = signing_bytes.create_verifiable_key(key_pair);

    server
        .request(request::RequestConnection {
            initiator_key,
            target_key,
        })
        .await?;

    loop {
        tokio::time::sleep(config.server_ping_delay).await;
        if let Some(target_address) = server.request(request::Ping).await?.0 {
            // TODO i don't think this is how u do nat traversal :)
            return Connection::try_to(target_address).await;
        }
    }
}
