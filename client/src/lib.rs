mod config;
mod conn;
mod error;

use crate::{conn::Connection, error::Result};

use lib::cs::{
    key::{Keypair, PublicKey, VerifiablePublicKey},
    protocol::request,
};

const SERVER_ADDRESS: &str = "some address";

pub use config::Config;

pub async fn establish_connection(
    keypair: &Keypair,
    // TODO can we take ref
    target_key: PublicKey,
    config: &Config,
) -> Result<Connection> {
    let mut server = Connection::try_to(SERVER_ADDRESS).await?;

    let signing_bytes = server.request(request::GetSigningBytes).await?.0;
    let initiator_key = VerifiablePublicKey::new(keypair, &signing_bytes);

    server
        .request(request::RequestConnection {
            initiator_key,
            target_key,
        })
        .await?;

    loop {
        // TODO configure sleep time
        tokio::time::sleep(config.server_ping_delay).await;
        if let Some(target_address) = server.request(request::Ping).await?.0 {
            // TODO i don't think this is how u do nat traversal :)
            return Connection::try_to(target_address).await;
        }
    }
}
