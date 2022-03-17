use crate::io;

use std::{net::IpAddr, path::PathBuf};

use memorage_client::{
    net::Client,
    persistent::{config::Config, data::DataWithoutPeer, Persistent},
    Result,
};
use memorage_cs::PairingCode;

use tracing::debug;

pub async fn pair(
    code: Option<PairingCode>,
    config: Option<PathBuf>,
    data: Option<PathBuf>,
    server: Option<IpAddr>,
) -> Result<()> {
    let mut config = Config::from_disk(config).await?;
    let data = DataWithoutPeer::from_disk(data).await?;
    debug!("loaded config and data files");
    if let Some(server) = server {
        config.server_address = vec![server];
    }

    let client = Client::new(&data, &config).await?;

    if let Some(code) = code {
        let peer = client.get_key(code).await?;
        io::verify_peer(data.key_pair, peer, false).await
    } else {
        let pairing_code = client.register().await?;
        println!("Pairing code: {}", pairing_code);

        let peer = client.register_response().await?;
        io::verify_peer(data.key_pair, peer, true).await
    }
}