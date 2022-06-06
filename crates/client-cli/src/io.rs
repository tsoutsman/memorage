use memorage_client::{
    persistent::{config::Config, data::Data, Persistent},
    Error, Result,
};

use std::{
    io::Write,
    path::{Path, PathBuf},
};

use memorage_core::{KeyPair, PublicKey};

#[inline]
pub fn prompt<S>(s: S) -> Result<String>
where
    S: AsRef<str>,
{
    let mut stdout = std::io::stdout();
    stdout.write_all(s.as_ref().as_bytes())?;
    stdout.flush()?;

    let mut input = String::with_capacity(0);
    std::io::stdin().read_line(&mut input)?;

    if input.ends_with('\n') {
        input.pop();
    }
    if input.ends_with('\r') {
        input.pop();
    }

    Ok(input)
}

#[inline]
pub fn securely_prompt<S>(s: S) -> Result<String>
where
    S: AsRef<str>,
{
    let mut stdout = std::io::stdout();
    stdout.write_all(s.as_ref().as_bytes())?;
    stdout.flush()?;

    Ok(rpassword::read_password()?)
}

#[inline]
pub fn prompt_continue<S>(reason: S) -> Result<()>
where
    S: AsRef<str>,
{
    println!("{}", reason.as_ref());
    loop {
        match prompt("Do you wish to proceed [y/n]? ")?
            .to_lowercase()
            .as_ref()
        {
            "y" | "yes" => return Ok(()),
            "n" | "no" => return Err(Error::UserCancelled),
            _ => eprintln!("Invalid choice"),
        }
    }
}

#[inline]
pub async fn setup_config() -> Result<Config> {
    let mut config = Config::default();

    let backup_path = loop {
        let input = prompt("Backup path: ")?;
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
            let input = prompt("> ")?;
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

        let backup_path = prompt(&format!("Backup path [{}]: ", config.backup_path.display()))?;
        if backup_path.as_str() != "" {
            config.backup_path = PathBuf::from(backup_path).canonicalize()?;
        }

        println!();

        let peer_storage_path = prompt(&format!(
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

    Ok(config)
}

#[inline]
pub async fn verify_peer(key_pair: KeyPair, peer: PublicKey, initiator: bool) -> Result<()> {
    let (key_1, key_2);
    if initiator {
        (key_1, key_2) = (key_pair.public, peer);
    } else {
        (key_1, key_2) = (peer, key_pair.public);
    }
    println!("Key 1: {}", key_1);
    println!("Key 2: {}", key_2);

    let input = prompt("Does your peer see the exact same keys? [y/n] ")?
        .trim()
        .to_lowercase();

    if input == "y" || input == "yes" {
        let data = Data { key_pair, peer };
        println!("Saving peer");
        data.to_disk(Option::<&Path>::None).await?;
        println!("Pairing successful");
        Ok(())
    } else {
        eprintln!("Aborting pairing process");
        Err(Error::IncorrectPeer)
    }
}
