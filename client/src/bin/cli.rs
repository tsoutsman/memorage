use std::{net::IpAddr, path::PathBuf};

use clap::{Parser, Subcommand};
use memorage_client::{config, net, Config};
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    human_panic::setup_panic!();
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info")
    }
    tracing_subscriber::fmt::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let args = Args::parse();

    if let Command::NewKey = args.command {
        let config = Config::with_key_pair(memorage_core::KeyPair::from_entropy());
        info!("Public Key: {}", config.key_pair.public);
        config.save_to_disk(&config::CONFIG_PATH)?;
        return Ok(());
    }

    let mut config = Config::from_disk(&config::CONFIG_PATH)?;

    match args.command {
        Command::Register => {
            let client = net::Client::new(&config).await?;
            let pairing_code = client.register().await?;
            info!("Pairing Code: {}", pairing_code);
            let peer = client.register_response().await?;
            info!("Peer Public Key: {}", peer);
            config.peer = Some(peer);
        }
        Command::Pair { code } => {
            let client = net::Client::new(&config).await?;
            let peer = client.get_key(code).await?;
            config.peer = Some(peer);
            info!("Peer Public Key: {}", peer);
        }
        Command::Connect { server } => {
            if let Some(server) = server {
                config.server_address = server;
            }
            let client = net::Client::new(&config).await?;

            let target_key = memorage_core::KeyPair::from_entropy().public;
            info!(public_key=?config.key_pair.public, ?target_key, "trying to establish connection");
            let mut peer_connection = client.establish_peer_connection().await?;

            if let Some(conn) = peer_connection.next().await {
                let quinn::NewConnection {
                    connection: _connection,
                    mut bi_streams,
                    ..
                } = conn.await?;
                while let Some(stream) = bi_streams.next().await {
                    let (_send, recv) = stream?;
                    let buf = recv.read_to_end(1024).await;
                    eprintln!("buf: {:#?}", buf);
                }
            }
        }
        Command::Check => {
            // TODO: Implement
            // TODO: check if person requesting == config.peer
        }
        Command::Watch { directory } => {
            let mut rx = memorage_client::fs::changed_files(directory)?;
            while let Some(event) = rx.recv().await {
                println!("Event: {:#?}", event);
            }
        }
        Command::NewKey => unreachable!(),
    }

    config.save_to_disk(&config::CONFIG_PATH)?;
    Ok(())
}

#[derive(Parser, Debug)]
struct Args {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    NewKey,
    Register,
    Pair {
        code: memorage_cs::PairingCode,
    },
    /// Attempt to connect to a peer
    Connect {
        /// Address of the coordinating server
        #[clap(short, long)]
        server: Option<IpAddr>,
    },
    Check,
    /// Watch over a directory
    Watch {
        /// Directory to watch over
        directory: PathBuf,
    },
}