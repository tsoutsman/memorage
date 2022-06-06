use crate::io;

use std::path::PathBuf;

use memorage_client::{
    mnemonic::MnemonicPhrase,
    persistent::{
        data::{Data, DataWithoutPeer},
        Persistent, CONFIG_PATH,
    },
    Error, Result,
};

use tracing::info;

pub async fn setup(config_output: Option<PathBuf>, data_output: Option<PathBuf>) -> Result<()> {
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

    let data = DataWithoutPeer::from_key_pair(phrase.into());
    info!("Generated public key: {}", data.key_pair.public);

    println!();

    let config = io::setup_config().await?;

    match Data::from_disk(data_output.as_ref()).await {
        // Match config read error in case DataWithoutPeer was saved. This may result in false
        // positives but they are better than false negatives.
        Ok(_) | Err(Error::ConfigRead(_)) => {
            io::prompt_continue("Logging in will log out the current user")?;
            data.to_disk(data_output).await?;
        }
        _ => data.to_disk(data_output).await?,
    }
    config.to_disk(config_output).await?;

    println!("\nSetup successful!\n");
    println!("You can modify these configuration values at any time in the ");
    println!("config file located at {}", CONFIG_PATH.display());

    Ok(())
}
