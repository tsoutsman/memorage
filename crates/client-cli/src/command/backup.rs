use std::{net::IpAddr, path::PathBuf};

use memorage_client::{
    fs::index::Index,
    net::{
        peer::{sleep_till, OutgoingConnection},
        Client,
    },
    persistent::{config::Config, data::Data, Persistent},
    Result,
};

use tracing::{debug, trace};

pub async fn backup(
    config: Option<PathBuf>,
    data: Option<PathBuf>,
    server: Option<IpAddr>,
) -> Result<()> {
    let config = Config::from_disk(config).await?;
    let data = Data::from_disk(data).await?;
    debug!("loaded config and data files");
    if let Some(server) = server {
        let server_address = &mut config.lock().server_address;
        *server_address = vec![server];
    }

    let client = Client::new(data, config.clone()).await?;

    let time = client.schedule_outgoing_connection().await?;

    let backup_path_clone = config.lock().backup_path.clone();
    let new_index_handle = tokio::spawn(async move {
        // TODO: Race conditions?
        Index::from_directory(backup_path_clone).await
    });

    sleep_till(time).await?;
    let mut outgoing_connection = client.create_outgoing_connection().await?;

    async fn indefinite_ping(connection: &mut OutgoingConnection) -> ! {
        loop {
            // TODO: The select statement could drop indefinite_ping during the
            // ping, which may result in a write error on the peer if
            // indefinite_ping gets dropped after transmitting a ping but before
            // receiving a response.
            let result = connection.ping().await;

            trace!(?result, "pinged peer");
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
        _ = indefinite_ping(&mut outgoing_connection) => Index::new(),
    };
    debug!("new index created");

    outgoing_connection.backup(&new_index).await?;

    Ok(())
}
