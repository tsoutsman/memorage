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
    let config = Config::from_disk(config).await?;
    let data = DataWithoutPeer::from_disk(data).await?;
    debug!("loaded config and data files");
    if let Some(server) = server {
        let server_address = &mut config.lock().server_address;
        *server_address = vec![server];
    }

    let client = Client::new(data.clone(), config).await?;

    let key_pair = data.lock().key_pair.clone();
    if let Some(code) = code {
        let peer = client.get_key(code).await?;
        io::verify_peer(key_pair, peer, false).await
    } else {
        let pairing_code = client.register().await?;
        println!("Pairing code: {}", pairing_code);

        let peer = client.register_response().await?;
        io::verify_peer(key_pair, peer, true).await
    }
}
