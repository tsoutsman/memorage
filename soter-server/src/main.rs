use std::net::{IpAddr, Ipv4Addr, SocketAddr};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().init();

    let (channels, handles) = soter_server::setup();

    let public_address = soter_cert::public_address(soter_cert::DEFAULT_STUN_SERVER)
        .await?
        .ip();
    let crypto = soter_cert::gen_crypto(public_address, None);
    let server_config = quinn::ServerConfig::with_crypto(std::sync::Arc::new(crypto?));

    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 1117);
    let (_endpoint, mut incoming) = quinn::Endpoint::server(server_config, addr)?;

    while let Some(conn) = incoming.next().await {
        tokio::spawn(soter_server::handle_connection(conn, channels.clone()));
    }

    handles.join().await?;
    Ok(())
}
