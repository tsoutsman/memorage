use crate::sleep_till;

use std::{net::IpAddr, path::PathBuf};

use memorage_client::{
    fs::index::Index,
    net::{protocol::request::Complete, Client},
    persistent::{config::Config, data::Data, Persistent},
    Error, Result,
};

use tracing::{debug, warn};

pub async fn sync(
    config: Option<PathBuf>,
    data: Option<PathBuf>,
    server: Option<IpAddr>,
    no_send: bool,
    no_receive: bool,
) -> Result<()> {
    let mut config = Config::from_disk(config).await?;
    let data = Data::from_disk(data).await?;
    debug!("loaded config and data files");
    if let Some(server) = server {
        config.server_address = vec![server];
    }

    // TODO: Race conditions
    // TODO: Do while waiting for connection
    let new_index = if no_send {
        Index::new()
    } else {
        Index::from_directory(&config.backup_path).await?
    };

    let client = Client::new(&data, &config).await?;
    debug!(public_key=?data.key_pair.public, target_key=?data.peer, "trying to establish connection");
    let initiator;

    let time = match client.check_peer_connection().await {
        Ok(t) => {
            initiator = false;
            t
        }
        Err(Error::Server(memorage_cs::Error::NoData)) => {
            initiator = true;
            client.establish_peer_connection().await?
        }
        Err(e) => return Err(e),
    };
    sleep_till(time).await?;

    let mut peer_connection = client.connect_to_peer(true).await?;

    if initiator {
        if no_send {
            peer_connection
                .send_data(&Index::new(), &Index::new(), !no_receive)
                .await?;
        } else {
            let old_index = peer_connection.get_index().await?;
            peer_connection
                .send_data(&old_index, &new_index, !no_receive)
                .await?;
        }
        if !no_receive {
            peer_connection.receive_commands().await?;
        }
    } else {
        // TODO: no_receive has no effect if host is not initiator.
        match peer_connection.receive_commands().await? {
            Complete::Continue => {
                let old_index = peer_connection.get_index().await?;
                peer_connection
                    .send_data(&old_index, &new_index, false)
                    .await?;
            }
            Complete::Close => {
                warn!("peer refused to receive commands");
            }
        };
    }

    todo!();
}
