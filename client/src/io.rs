use crate::{Config, Result};

use std::io::Write;

use memorage_core::PublicKey;

use rpassword::read_password;

#[inline]
pub fn prompt_secure<S>(s: S) -> Result<String>
where
    S: AsRef<str>,
{
    let mut stdout = std::io::stdout();
    stdout.write_all(s.as_ref().as_bytes())?;
    stdout.flush()?;

    Ok(read_password()?)
}

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
pub fn verify_peer(peer: &PublicKey, config: &mut Config) -> Result<bool> {
    let input = prompt("Does your peer see the exact same keys? [y/n] ")?
        .trim()
        .to_lowercase();

    if input == "y" || input == "yes" {
        config.peer = Some(*peer);
        println!("Saving peer");
        config.save_to_disk(&crate::config::CONFIG_PATH)?;
        Ok(true)
    } else {
        eprintln!("Aborting pairing process");
        Ok(false)
    }
}