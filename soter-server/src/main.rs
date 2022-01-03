use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    human_panic::setup_panic!();
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info")
    }
    tracing_subscriber::fmt::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let (channels, handles) = soter_server::setup();

    let public_address = soter_stun::public_address(soter_stun::DEFAULT_STUN_SERVER).await?;
    info!(%public_address, "received public address");
    let key_pair = soter_core::KeyPair::generate(&soter_core::rand::SystemRandom::new())?;
    let server_config = soter_cert::gen_recv_config(public_address.ip(), &key_pair)?;

    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), soter_core::PORT);
    let (_endpoint, mut incoming) = quinn::Endpoint::server(server_config, addr)?;

    while let Some(conn) = incoming.next().await {
        tokio::spawn(soter_server::handle_connection(conn, channels.clone()));
    }

    handles.join().await?;
    Ok(())
}
