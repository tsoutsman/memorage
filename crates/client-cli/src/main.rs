mod app;
mod command;
mod io;

use crate::app::{Args, Command};

use memorage_core::time::OffsetDateTime;

use clap::Parser;
use tracing::info;

#[tokio::main]
async fn main() -> memorage_client::Result<()> {
    human_panic::setup_panic!();
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "warn")
    }
    tracing_subscriber::fmt::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let args = Args::parse();

    match args.command {
        Command::Setup {
            config_output,
            data_output,
        } => command::setup(config_output, data_output).await,
        Command::Pair {
            code,
            config,
            data,
            server,
        } => command::pair(code, config, data, server).await,
        Command::Sync {
            config,
            data,
            server,
            no_send,
            no_receive,
        } => command::sync(config, data, server, no_send, no_receive).await,
        Command::Daemon {
            config,
            data,
            server,
        } => command::daemon(config, data, server).await,
        Command::Retrieve {
            output,
            config,
            data,
            server,
        } => command::retrieve(output, config, data, server).await,
    }
}

async fn sleep_till(time: OffsetDateTime) -> memorage_client::Result<()> {
    let delay = time - OffsetDateTime::now_utc();
    info!(%time, %delay, "waiting for synchronisation");
    tokio::time::sleep(
        delay
            .try_into()
            .map_err(|_| memorage_client::Error::MissedSynchronisation)?,
    )
    .await;
    Ok(())
}
