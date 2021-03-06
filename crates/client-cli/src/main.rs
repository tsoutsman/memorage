#![feature(try_blocks, never_type)]

mod app;
mod command;
mod io;

use crate::app::{Args, Command};

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
        } => command::setup(config_output, data_output).await,
        Command::Login {
            config_output,
            data_output,
        } => command::login(config_output, data_output).await,
        Command::Pair {
            code,
            config,
            data,
            server,
        } => command::pair(code, config, data, server).await,
        Command::Backup {
            config,
            data,
            server,
        } => command::backup(config, data, server).await,
        Command::Check {
            config,
            data,
            server,
        } => command::check(config, data, server).await,
        Command::Retrieve {
            output,
            config,
            data,
            server,
        } => command::retrieve(output, config, data, server).await,
        Command::Daemon {
            config,
            data,
            server,
        } => command::daemon(config, data, server).await.map(|_| ()),
    }
}
