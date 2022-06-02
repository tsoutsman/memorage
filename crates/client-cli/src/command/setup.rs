use crate::io;

use std::path::PathBuf;

use memorage_client::{
    mnemonic::MnemonicPhrase,
    persistent::{config::Config, data::DataWithoutPeer, Persistent, CONFIG_PATH},
    Result,
};

use tracing::info;

pub async fn setup(config_output: Option<PathBuf>, data_output: Option<PathBuf>) -> Result<()> {
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
                eprintln!("Backup path must be specified\n");
            }
            _ => {
                let path = PathBuf::from(input).canonicalize()?;
                if path.exists() {
                    break path;
                } else {
                    eprintln!("Backup path does not exist\n");
                }
            }
        }
    };
    config.backup_path = backup_path;

    loop {
        // Format taken from the rustup installer.

        println!("\nCurrent configuration options:\n");
        println!("        backup path: {}", config.backup_path.display());
        println!("  peer storage path: {}\n", config.peer_storage_path);

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

        let backup_path = io::prompt(&format!("Backup path [{}]: ", config.backup_path.display()))?;
        if backup_path.as_str() != "" {
            config.backup_path = PathBuf::from(backup_path).canonicalize()?;
        }

        println!();

        let peer_storage_path = io::prompt(&format!(
            "Peer storage path [{}]: ",
            config.peer_storage_path
        ))?;
        if peer_storage_path.as_str() != "" {
            config.peer_storage_path = PathBuf::from(peer_storage_path).canonicalize()?.into();
        }
    }

    // TODO: Move to client common?
    match tokio::fs::create_dir_all(&config.peer_storage_path).await {
        Ok(_) => {}
        Err(e) => match e.kind() {
            std::io::ErrorKind::AlreadyExists => {}
            _ => return Err(e.into()),
        },
    }

    data.to_disk(data_output).await?;
    config.to_disk(config_output).await?;

    println!("\nSetup successful!\n");
    println!("You can modify these configuration values at any time in the ");
    println!("config file located at {}", CONFIG_PATH.display());

    Ok(())
}
