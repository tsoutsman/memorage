use crate::sleep_till;

use std::{net::IpAddr, path::PathBuf};

use memorage_client::{
    fs::index::Index,
    net::{protocol::request::Complete, Client},
    persistent::{config::Config, data::Data, Persistent},
    Error, Result,
};

use tracing::{debug, warn};

pub async fn check(
    config: Option<PathBuf>,
    data: Option<PathBuf>,
    server: Option<IpAddr>,
) -> Result<()> {
    let mut config = Config::from_disk(config).await?;
    let data = Data::from_disk(data).await?;
    debug!("loaded config and data files");
    if let Some(server) = server {
        config.server_address = vec![server];
    }

    let new_index = Index::from_directory(&config.backup_path).await?;

    let client = Client::new(&data, &config).await?;
    let time = match client.check_peer_connection().await {
        Ok(t) => t,
        Err(Error::Server(memorage_cs::Error::NoData)) => {
            println!("No connection requested by peer");
            return Ok(());
        }
        Err(e) => return Err(e),
    };
    sleep_till(time).await?;

    let mut peer_connection = client.connect_to_peer(false).await?;
    let old_index = peer_connection.receive_commands().await?;

    match old_index {
        Complete::Continue { index: old_index } => {
            let old_index = match old_index {
                Some(i) => i.decrypt(&data.key_pair.private)?,
                None => Index::new(),
            };
            peer_connection
                .send_data(&old_index, &new_index, false)
                .await?;
        }
        Complete::Close => {
            warn!("peer refused to receive commands");
        }
    }

    println!("Backup succesful");

    Ok(())
}
