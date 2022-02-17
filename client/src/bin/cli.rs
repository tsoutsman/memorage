use std::{net::IpAddr, path::PathBuf};

use clap::{Parser, Subcommand};
use memorage_client::{config, io, mnemonic::MnemonicPhrase, net, Config};
use memorage_core::time::OffsetDateTime;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    human_panic::setup_panic!();
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "warn")
    }
    tracing_subscriber::fmt::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let args = Args::parse();

    if let Command::Setup = args.command {
        let config = Config::with_key_pair(memorage_core::KeyPair::from_entropy());

        let num_words = loop {
            match io::prompt("Mnemonic phrase length (18): ")?.parse::<usize>() {
                Ok(n) => break n,
                Err(_) => {
                    eprintln!("Mnemonic phrase length must be a number");
                }
            }
        };

        let password = io::prompt_secure("Enter password (empty for no password): ")?;
        let password = match &password[..] {
            "" => None,
            _ => Some(password),
        };

        let phrase = MnemonicPhrase::generate(num_words, password);
        println!("Mnemonic phrase: {}", phrase);

        info!("Generated public key: {}", config.key_pair.public);

        config.save_to_disk(&config::CONFIG_PATH)?;
        println!("Setup successful");
        return Ok(());
    }

    let mut config = Config::from_disk(&config::CONFIG_PATH)?;

    match args.command {
        Command::Pair { server, code } => {
            if let Some(server) = server {
                config.server_address = server;
            }

            let client = net::Client::new(&config).await?;

            if let Some(code) = code {
                let peer = client.get_key(code).await?;

                println!("Key 1: {}", peer);
                println!("Key 2: {}", config.key_pair.public);

                if io::verify_peer(&peer, &mut config)? {
                    return Ok(());
                } else {
                    std::process::exit(1);
                }
            } else {
                let pairing_code = client.register().await?;
                println!("Pairing Code: {}", pairing_code);

                let peer = client.register_response().await?;

                println!("Key 1: {}", config.key_pair.public);
                println!("Key 2: {}", peer);

                if io::verify_peer(&peer, &mut config)? {
                    println!("Pairing successful");
                    return Ok(());
                } else {
                    std::process::exit(1);
                }
            }
        }
        Command::Connect { server } => {
            if let Some(server) = server {
                config.server_address = server;
            }
            let client = net::Client::new(&config).await?;

            let target_key = memorage_core::KeyPair::from_entropy().public;
            info!(public_key=?config.key_pair.public, ?target_key, "trying to establish connection");
            let mut peer_connection = client.establish_peer_connection().await?;
            peer_connection.raw_recv().await?;

            // if let Some(conn) = peer_connection.next().await {
            //     let quinn::NewConnection {
            //         connection: _connection,
            //         mut bi_streams,
            //         ..
            //     } = conn.await?;
            //     while let Some(stream) = bi_streams.next().await {
            //         let (_send, recv) = stream?;
            //         let buf = recv.read_to_end(1024).await;
            //         info!("buf: {:#?}", buf);
            //     }
            // }
        }
        Command::Check { server } => {
            if let Some(server) = server {
                config.server_address = server;
            }
            let client = net::Client::new(&config).await?;
            let time = client.check_connection().await?;

            let delay = time - OffsetDateTime::now_utc();
            tokio::time::sleep(delay.try_into().unwrap()).await;

            let conn = client.connect_to_peer().await?;
            conn.send(&[1, 19, 1, 30, 0, 0, 1, 3]).await?;
        }
        Command::Watch { directory } => {
            let mut rx = memorage_client::fs::changed_files(directory)?;
            while let Some(event) = rx.recv().await {
                println!("Event: {:#?}", event);
            }
        }
        Command::Setup => unreachable!(),
    }

    Ok(())
}

#[derive(Parser, Debug)]
struct Args {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    Setup,
    Pair {
        /// Address of the coordination server
        #[clap(short, long)]
        server: Option<IpAddr>,
        code: Option<memorage_cs::PairingCode>,
    },
    /// Attempt to connect to a peer
    Connect {
        /// Address of the coordination server
        #[clap(short, long)]
        server: Option<IpAddr>,
    },
    Check {
        /// Address of the coordination server
        #[clap(short, long)]
        server: Option<IpAddr>,
    },
    /// Watch over a directory
    Watch {
        /// Directory to watch over
        directory: PathBuf,
    },
}
