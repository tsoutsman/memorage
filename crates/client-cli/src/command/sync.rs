use crate::sleep_till;

use std::{net::IpAddr, path::PathBuf};

use memorage_client::{
    fs::index::Index,
    net::{protocol::request::Complete, Client, PeerConnection},
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
    debug!(?initiator);

    let backup_path_clone = config.backup_path.clone();
    let new_index_handle = tokio::spawn(async move {
        if no_send {
            Ok(Index::new())
        } else {
            // TODO: Race conditions?
            Index::from_directory(backup_path_clone).await
        }
    });

    sleep_till(time).await?;
    let mut peer_connection = client.connect_to_peer(initiator).await?;

    async fn indefinite_ping(peer: &mut PeerConnection<'_, '_>) -> ! {
        loop {
            // TODO: The select statement could drop indefinite_ping during the
            // ping, which may result in a write error on the peer if
            // indefinite_ping gets dropped after transmitting a ping but before
            // receiving a response.
            let result = peer.ping().await;
            debug!(?result, "pinged peer");
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        }
    }
    let new_index = tokio::select! {
        // Biased mode first checks if the new index has already been created
        // before beginning to ping.
        biased;
        new_index = new_index_handle => new_index??,
        // indefinite_ping will keep pinging the peer to keep the connection
        // open until the local index has been created. Index::new() is just
        // there to satisfy the type checker.
        _ = indefinite_ping(&mut peer_connection) => Index::new(),
    };
    debug!("new index created");

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

    Ok(())
}
