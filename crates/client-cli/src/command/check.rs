use crate::sleep_till;

use std::{net::IpAddr, path::PathBuf};

use memorage_client::{
    net::Client,
    persistent::{config::Config, data::Data, Persistent},
    Result,
};
use tracing::debug;

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

    let client = Client::new(&data, &config).await?;
    let time = match client.check_incoming_connection().await? {
        Some(t) => t,
        None => return Ok(()),
    };
    sleep_till(time).await?;
    let incoming_connection = client.receive_incoming_connection().await?;
    incoming_connection.handle().await?;

    Ok(())
}
