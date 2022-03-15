use crate::sleep_till;

use std::{net::IpAddr, path::PathBuf};

use memorage_client::{
    fs::Index,
    net::{self, protocol::request},
    persistent::{config::Config, data::Data, Persistent},
    Result,
};

use tracing::{debug, info};

pub async fn connect(
    config: Option<PathBuf>,
    data: Option<PathBuf>,
    server: Option<IpAddr>,
) -> Result<()> {
    let mut config = Config::from_disk(config)?;
    let data = Data::from_disk(data)?;
    debug!("loaded config and data files");
    if let Some(server) = server {
        config.server_address = vec![server];
    }

    let new_index = Index::from_directory(&config.backup_path)?;

    let client = net::Client::new(&data, &config).await?;
    info!(public_key=?data.key_pair.public, target_key=?data.peer, "trying to establish connection");
    let time = client.establish_peer_connection().await?;
    sleep_till(time).await?;

    let mut peer_connection = client.connect_to_peer(true).await?;

    let old_index = match peer_connection.send(request::GetIndex).await?.0 {
        Some(i) => i.decrypt(&data.key_pair.private)?,
        None => Index::new(),
    };

    peer_connection
        .send_backup_data(&old_index, &new_index, true)
        .await?;
    peer_connection.receive_backup_data().await?;

    println!("Backup succesful");

    Ok(())
}
