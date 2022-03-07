use std::{net::IpAddr, path::PathBuf};

use clap::{Parser, Subcommand};
use memorage_client::{
    io,
    mnemonic::MnemonicPhrase,
    net,
    persistent::{Config, Data, Persistent},
};
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
        let data = Data::from_key_pair(memorage_core::KeyPair::from_entropy());

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
        if let Some(ref password) = password {
            let confirmed_password = io::prompt_secure("Confirm password: ")?;
            if &confirmed_password != password {
                eprintln!("Passwords didn't match");
                std::process::exit(1);
            }
        }

        let phrase = MnemonicPhrase::generate(num_words, password);
        println!("Mnemonic phrase: {}", phrase);

        info!("Generated public key: {}", data.key_pair.public);

        data.save_to_disk(None)?;
        println!("Setup successful");
        return Ok(());
    }

    let mut data = Data::from_disk(None)?;
    let mut config = Config::from_disk(None)?;

    match args.command {
        Command::Pair { server, code } => {
            if let Some(server) = server {
                config.server_address = vec![server];
            }

            let client = net::Client::new(&data, &config).await?;

            if let Some(code) = code {
                let peer = client.get_key(code).await?;

                println!("Key 1: {}", peer);
                println!("Key 2: {}", data.key_pair.public);

                if io::verify_peer(&peer, &mut data)? {
                    return Ok(());
                } else {
                    std::process::exit(1);
                }
            } else {
                let pairing_code = client.register().await?;
                println!("Pairing code: {}", pairing_code);

                let peer = client.register_response().await?;

                println!("Key 1: {}", data.key_pair.public);
                println!("Key 2: {}", peer);

                if io::verify_peer(&peer, &mut data)? {
                    println!("Pairing successful");
                    return Ok(());
                } else {
                    std::process::exit(1);
                }
            }
        }
        Command::Connect { server } => {
            if let Some(server) = server {
                config.server_address = vec![server];
            }
            let client = net::Client::new(&data, &config).await?;

            let target_key = memorage_core::KeyPair::from_entropy().public;
            info!(public_key=?data.key_pair.public, ?target_key, "trying to establish connection");
            let mut _peer_connection = client.establish_peer_connection().await?;

            todo!();
        }
        Command::Check { server } => {
            if let Some(server) = server {
                config.server_address = vec![server];
            }
            let client = net::Client::new(&data, &config).await?;
            let time = client.check_connection().await?;

            let delay = time - OffsetDateTime::now_utc();
            tokio::time::sleep(delay.try_into().unwrap()).await;

            let _conn = client.connect_to_peer().await?;

            todo!();
        }
        Command::Watch { .. } => {
            todo!();
        }
        Command::Setup => unreachable!(),
    }

    #[allow(unreachable_code)]
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
