use crate::{Config, Result};

use std::io::Write;

use memorage_core::PublicKey;

#[inline]
pub fn verify_peer(peer: &PublicKey, config: &mut Config) -> Result<bool> {
    println!("Peer: {}", peer);
    print!("Is this correct? [y/n] ");

    std::io::stdout().flush()?;
    let mut input = String::with_capacity(2);
    std::io::stdin().read_line(&mut input)?;
    input = input.trim().to_lowercase();

    if input == "y" || input == "yes" {
        config.peer = Some(*peer);
        println!("Saving peer");
        config.save_to_disk(&crate::config::CONFIG_PATH)?;
        Ok(true)
    } else {
        println!("Aborting pairing process");
        Ok(false)
    }
}
