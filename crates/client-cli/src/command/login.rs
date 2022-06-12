use crate::io;

use std::path::PathBuf;

use memorage_client::{
    mnemonic::MnemonicPhrase,
    persistent::{
        config::Config,
        data::{Data, DataWithoutPeer},
        Persistent,
    },
    Error, Result,
};
use memorage_core::KeyPair;

pub async fn login(config_output: Option<PathBuf>, data_output: Option<PathBuf>) -> Result<()> {
    let words_input = io::prompt("Mnemonic phrase: ")?;
    let words = words_input.split(' ').collect();
    let password = Option::from(io::securely_prompt("Password: ")?).filter(|s| s.as_str() != "");
    let key_pair: KeyPair = MnemonicPhrase::new(words, password)?.into();

    let config = Config::from_disk(config_output.as_ref()).await;
    let data = Data::from_disk(data_output.as_ref()).await;

    if let Ok(data) = data {
        if data.lock().key_pair == key_pair {
            println!("User already logged in");
            return Ok(());
        } else {
            io::prompt_continue("Logging in will log out the current user")?;
            let mut data = data.lock().clone();
            data.key_pair = key_pair;
            data.to_disk(data_output).await?;
        }
    } else {
        let data = DataWithoutPeer::from_key_pair(key_pair);
        data.to_disk(data_output).await?;
    }

    if let Err(Error::NotFound { .. }) = config {
        let config = io::setup_config().await?;
        config.to_disk(config_output).await?;
    }

    Ok(())
}
