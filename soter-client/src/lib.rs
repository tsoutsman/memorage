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
mod error;
mod net;

use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use net::{Client, PeerConnection};
use soter_core::{KeyPair, PublicKey};

pub use config::Config;
pub use error::{Error, Result};
use soter_cs::request;

pub mod crypto;
pub mod fs;

pub async fn establish_connection(
    key_pair: std::sync::Arc<KeyPair>,
    target_key: &PublicKey,
    config: &Config,
) -> Result<PeerConnection> {
    let server_address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)), soter_core::PORT);
    let client = Client::new(key_pair.clone()).await?;
    let mut server = client.connect_to_server(server_address).await?;

    // TODO: Do we need signing bytes in the protocol?
    let signing_bytes = server.request(request::GetSigningBytes).await?.0;
    let initiator_key = signing_bytes.create_verifiable_key(&key_pair);
    server
        .request(request::RequestConnection {
            initiator_key,
            target_key: *target_key,
        })
        .await?;

    loop {
        tokio::time::sleep(config.server_ping_delay).await;
        if let Some(target_address) = server.request(request::Ping).await?.0 {
            return client.connect_to_peer(target_address, target_key).await;
        }
    }
}
