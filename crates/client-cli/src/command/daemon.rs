use std::{net::IpAddr, path::PathBuf};

use memorage_client::{
    persistent::{config::Config, Persistent},
    Result,
};

pub async fn daemon(
    config: Option<PathBuf>,
    data: Option<PathBuf>,
    server: Option<IpAddr>,
) -> Result<()> {
    loop {
        let daemon_sync_interval = Config::from_disk(config.as_ref())
            .await?
            .daemon_sync_interval;
        crate::command::sync(config.clone(), data.clone(), server, false, false).await?;
        println!("Sleeping for {}", daemon_sync_interval.as_secs());
        tokio::time::sleep(daemon_sync_interval).await;
    }
}
