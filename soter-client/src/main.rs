use std::sync::Arc;

use soter_client::{net::establish_connection, Config};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    human_panic::setup_panic!();
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info")
    }
    tracing_subscriber::fmt::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let key_pair = Arc::new(soter_core::KeyPair::from_entropy());
    let target_key = Arc::new(soter_core::KeyPair::from_entropy().public);
    let config = Config::default();

    tracing::info!(public_key=?key_pair.public, ?target_key, "trying to establish connection");

    let _peer_connection = establish_connection(key_pair, target_key, &config).await?;

    Ok(())
}
