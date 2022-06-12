use crate::sleep_till;

use std::{net::IpAddr, path::PathBuf};

use memorage_client::{
    net::Client,
    persistent::{config::Config, data::Data, Persistent},
    Result,
};

use tracing::debug;

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
    let config = Config::from_disk(config).await?;
    let data = Data::from_disk(data).await?;
    debug!("loaded config and data files");
    if let Some(server) = server {
        let server_address = &mut config.lock().server_address;
        *server_address = vec![server];
    }

    let client = Client::new(data, config).await?;
    let time = client.schedule_outgoing_connection().await?;
    sleep_till(time).await?;
    let mut outgoing_connection = client.create_outgoing_connection().await?;
    outgoing_connection.retrieve(&output).await?;

    println!("Retrieval succesful");
    Ok(())
}
