mod app;
mod command;
mod io;

use crate::app::{Args, Command};

use memorage_core::time::OffsetDateTime;

use clap::Parser;

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
        } => command::setup(config_output, data_output),
        Command::Pair {
            code,
            config,
            data,
            server,
        } => command::pair(code, config, data, server).await,
        Command::Connect {
            config,
            data,
            server,
        } => command::connect(config, data, server).await,
        Command::Check {
            config,
            data,
            server,
        } => command::check(config, data, server).await,
    }
}

async fn sleep_till(time: OffsetDateTime) -> memorage_client::Result<()> {
    let delay = time - OffsetDateTime::now_utc();
    tracing::info!(?time, ?delay, "waiting for synchronisation");
    // TODO: Create index while sleeping?
    tokio::time::sleep(
        delay
            .try_into()
            .map_err(|_| memorage_client::Error::MissedSynchronisation)?,
    )
    .await;
    Ok(())
}
