use std::{net::IpAddr, path::PathBuf};

use clap::{Parser, Subcommand};
use memorage_client::{
    crypto::Encrypted,
    fs::{read_bin, Index},
    io,
    mnemonic::MnemonicPhrase,
    net::{self, protocol::request},
    persistent::{
        config::Config,
        data::{Data, DataWithoutPeer},
        Persistent, CONFIG_PATH,
    },
};
use memorage_core::time::OffsetDateTime;
use tracing::{debug, info};

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

    match args.command {
        Command::Setup {
            config_output,
            data_output,
        } => {
            let data = DataWithoutPeer::from_key_pair(memorage_core::KeyPair::from_entropy());
            let mut config = Config::default();

            let num_words = loop {
                match io::prompt("Mnemonic phrase length (18): ")?.as_ref() {
                    "" => break 18,
                    s => match s.parse::<usize>() {
                        Ok(n) => break n,
                        Err(_) => {
                            eprintln!("Mnemonic phrase length must be a number");
                        }
                    },
                }
            };

            let password = io::securely_prompt("Enter password (empty for no password): ")?;
            let password = match &password[..] {
                "" => None,
                _ => Some(password),
            };
            if let Some(ref password) = password {
                let confirmed_password = io::securely_prompt("Confirm password: ")?;
                if &confirmed_password != password {
                    eprintln!("Passwords didn't match");
                    std::process::exit(1);
                }
            }

            let phrase = MnemonicPhrase::generate(num_words, password);
            println!("Mnemonic phrase: {}", phrase);

            info!("Generated public key: {}", data.key_pair.public);

            println!();

            let backup_path = loop {
                let input = io::prompt("Backup path: ")?;
                match input.as_str() {
                    "" => {
                        eprintln!("Backup path must be specified");
                    }
                    _ => break input.parse::<PathBuf>()?,
                }
            };
            config.backup_path = backup_path;

            loop {
                // Format taken from the rustup installer.

                println!("\nCurrent configuration options:\n");
                println!("        backup path: {}", config.backup_path.display());
                println!(
                    "  peer storage path: {}\n",
                    config.peer_storage_path.display()
                );

                println!("1) Proceed with installation (default)");
                println!("2) Customise installation");
                println!("3) Cancel installation");

                let mut proceed = false;

                loop {
                    let input = io::prompt("> ")?;
                    match input.as_str() {
                        "" => {
                            proceed = true;
                            break;
                        }
                        _ => match input.parse::<u8>() {
                            Ok(n) => match n {
                                1 => {
                                    proceed = true;
                                    break;
                                }
                                2 => break,
                                3 => std::process::exit(1),
                                _ => eprintln!("Invalid choice"),
                            },
                            Err(_) => eprintln!("Invalid choice"),
                        },
                    }
                }

                if proceed {
                    break;
                }

                println!("\nI'm going to ask you the value of each of these installation options.");
                println!("You may simply press the Enter key to leave unchanged.\n");

                let backup_path =
                    io::prompt(&format!("Backup path [{}]: ", config.backup_path.display()))?;
                if backup_path.as_str() != "" {
                    config.backup_path = backup_path.parse()?;
                }

                println!();

                let peer_storage_path = io::prompt(&format!(
                    "Peer storage path [{}]: ",
                    config.peer_storage_path.display()
                ))?;
                if peer_storage_path.as_str() != "" {
                    config.peer_storage_path = peer_storage_path.parse()?;
                }
            }

            data.to_disk(data_output)?;
            config.to_disk(config_output)?;

            println!("\nSetup successful!\n");
            println!("You can modify these configuration values at any time in the ");
            println!("config file located at {}", CONFIG_PATH.display());

            return Ok(());
        }
        Command::Pair {
            code,
            config,
            data,
            server,
        } => {
            let mut config = Config::from_disk(config)?;
            let data = DataWithoutPeer::from_disk(data)?;

            debug!("loaded config and data files");

            if let Some(server) = server {
                config.server_address = vec![server];
            }

            let client = net::Client::new(&data, &config).await?;

            if let Some(code) = code {
                let peer = client.get_key(code).await?;
                io::verify_peer(data.key_pair, peer, false)?;
            } else {
                let pairing_code = client.register().await?;
                println!("Pairing code: {}", pairing_code);

                let peer = client.register_response().await?;
                io::verify_peer(data.key_pair, peer, true)?;
            }
        }
        Command::Connect {
            config,
            data,
            server,
        } => {
            let mut config = Config::from_disk(config)?;
            let data = Data::from_disk(data)?;

            debug!("loaded config and data files");

            if let Some(server) = server {
                config.server_address = vec![server];
            }
            let client = net::Client::new(&data, &config).await?;

            let new_index = Index::from_directory(&config.backup_path)?;

            info!(public_key=?data.key_pair.public, target_key=?data.peer, "trying to establish connection");
            let mut peer_connection = client.establish_peer_connection(true).await?;

            let old_index = peer_connection
                .send(request::GetIndex)
                .await?
                .0
                .decrypt(&data.key_pair.private)?;

            peer_connection
                .send_difference(new_index.difference(&old_index))
                .await?;

            let complete = peer_connection
                .send(request::Complete(read_bin(&config.index_path())?))
                .await?
                .0;

            if complete {
                info!("peer completed connection");
                println!("Backup succesful");
                return Ok(());
            } else {
                peer_connection.receive_and_handle().await?;
            }
        }
        Command::Check {
            config,
            data,
            server,
        } => {
            let mut config = Config::from_disk(config)?;
            let data = Data::from_disk(data)?;

            debug!("loaded config and data files");

            if let Some(server) = server {
                config.server_address = vec![server];
            }
            let client = net::Client::new(&data, &config).await?;

            let time = client.check_connection().await?;
            let delay = time - OffsetDateTime::now_utc();
            info!(?time, ?delay, "waiting for synchronisation");
            // TODO: Create index while sleeping?
            tokio::time::sleep(delay.try_into().unwrap()).await;

            let mut peer_connection = client.connect_to_peer(false).await?;

            let new_index = Index::from_directory(&config.backup_path)?;
            let old_index = peer_connection.receive_and_handle().await?;

            peer_connection
                .send_difference(new_index.difference(&old_index))
                .await?;

            peer_connection
                // The index from request::Complete isn't used by the initiator
                // of the sync.
                .send(request::Complete(Encrypted::encrypt(
                    &data.key_pair.private,
                    &Index::new(),
                )?))
                .await?;
        }
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
    /// Generate the data and configuration files
    Setup {
        /// Save the generated config file to the specified path.
        #[clap(long)]
        config_output: Option<PathBuf>,
        /// Save the generated data file to the specified path.
        #[clap(long)]
        data_output: Option<PathBuf>,
    },
    /// Pair to a peer
    ///
    /// One peer runs the command without a code, generating a new code. The
    /// other peer runs the command with this newly generated code. Peers must
    /// use the same coordination server.
    Pair {
        code: Option<memorage_cs::PairingCode>,
        /// Use the specified configuration file
        #[clap(short, long)]
        config: Option<PathBuf>,
        /// Use the specified data file
        #[clap(short, long)]
        data: Option<PathBuf>,
        /// Use the specified coordination server
        ///
        /// The address can be IPv4 or IPv6.
        #[clap(short, long)]
        server: Option<IpAddr>,
    },
    /// Attempt to connect to a peer
    Connect {
        /// Use the specified configuration file
        #[clap(short, long)]
        config: Option<PathBuf>,
        /// Use the specified data file
        #[clap(short, long)]
        data: Option<PathBuf>,
        /// Use the specified coordination server
        ///
        /// The address can be IPv4 or IPv6.
        #[clap(short, long)]
        server: Option<IpAddr>,
    },
    /// Check for synchronisation requests
    Check {
        /// Use the specified configuration file
        #[clap(short, long)]
        config: Option<PathBuf>,
        /// Use the specified data file
        #[clap(short, long)]
        data: Option<PathBuf>,
        /// Use the specified coordination server
        ///
        /// The address can be IPv4 or IPv6.
        #[clap(short, long)]
        server: Option<IpAddr>,
    },
}
