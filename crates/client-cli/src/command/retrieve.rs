use crate::sleep_till;

use std::{net::IpAddr, path::PathBuf};

use memorage_client::{
    net::Client,
    persistent::{config::Config, data::Data, Persistent},
    Result,
};

use tracing::{debug, info};

pub async fn retrieve(
    output: Option<PathBuf>,
    config: Option<PathBuf>,
    data: Option<PathBuf>,
    server: Option<IpAddr>,
) -> Result<()> {
    let output = match output {
        Some(p) => p,
        None => std::env::current_dir()?.join("memorage_backup"),
    };
    let mut config = Config::from_disk(config).await?;
    let data = Data::from_disk(data).await?;
    debug!("loaded config and data files");
    if let Some(server) = server {
        config.server_address = vec![server];
    }

    let client = Client::new(&data, &config).await?;
    info!(public_key=?data.key_pair.public, target_key=?data.peer, "trying to establish connection");
    let time = client.establish_peer_connection().await?;
    sleep_till(time).await?;

    let mut peer_connection = client.connect_to_peer(true).await?;

    let index = peer_connection.get_index().await?;

    peer_connection
        .retrieve_backup_data(&index, &output)
        .await?;

    println!("Retrieval succesful");
    Ok(())
}
