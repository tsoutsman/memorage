use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    human_panic::setup_panic!();
    memorage_server::setup_logger();

    let (channels, handles) = memorage_server::setup();

    let mut socket = tokio::net::UdpSocket::bind("0.0.0.0:0").await?;
    let public_address =
        memorage_stun::public_address(&mut socket, memorage_stun::DEFAULT_STUN_SERVER).await?;
    info!(%public_address, "received public address");

    let key_pair = memorage_core::KeyPair::from_entropy();
    let server_config = memorage_cert::gen_recv_config(public_address.ip(), &key_pair, None)?;

    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), memorage_core::PORT);
    let (_endpoint, mut incoming) = quinn::Endpoint::server(server_config, addr)?;

    while let Some(conn) = incoming.next().await {
        tokio::spawn(memorage_server::handle_connection(conn, channels.clone()));
    }

    handles.join().await?;
    Ok(())
}
