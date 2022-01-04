use soter_client::Config;

#[tokio::main]
async fn main() {
    human_panic::setup_panic!();
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info")
    }
    tracing_subscriber::fmt::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let keypair = soter_core::KeyPair::generate(&soter_core::rand::SystemRandom::new()).unwrap();
    let target_key = soter_core::KeyPair::generate(&soter_core::rand::SystemRandom::new())
        .unwrap()
        .public_key();
    let config = Config::default();

    tracing::info!(?keypair, ?target_key, "trying to establish connection");
    soter_client::establish_connection(std::sync::Arc::new(keypair), &target_key, &config)
        .await
        .unwrap();
}
