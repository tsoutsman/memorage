use memorage_client::{
    persistent::{data::Data, Persistent},
    Error, Result,
};

use std::{io::Write, path::Path};

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
