use soter_client::{net::establish_connection, net::Client, Config};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    human_panic::setup_panic!();
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info")
    }
    tracing_subscriber::fmt::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let key_pair = soter_core::KeyPair::generate(&soter_core::rand::SystemRandom::new()).unwrap();
    let target_key = soter_core::KeyPair::generate(&soter_core::rand::SystemRandom::new())
        .unwrap()
        .public_key();
    let config = Config::default();

    tracing::info!(public_key=?key_pair.public_key(), ?target_key, "trying to establish connection");

    let client = Client::new(std::sync::Arc::new(key_pair)).await?;
    let _peer_connection = establish_connection(&client, &target_key, &config).await?;

    Ok(())
}
