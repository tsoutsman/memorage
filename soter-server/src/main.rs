use std::net::IpAddr;

#[inline]
fn gen_crypto(public_address: IpAddr) -> rustls::ServerConfig {
    let mut cert_params = rcgen::CertificateParams::default();
    cert_params.subject_alt_names = vec![rcgen::SanType::IpAddress(public_address)];
    let cert = rcgen::Certificate::from_params(cert_params).unwrap();

    let key = cert.serialize_private_key_der();
    let cert = cert.serialize_der().unwrap();

    let key = rustls::PrivateKey(key);
    let cert = rustls::Certificate(cert);

    rustls::ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(vec![cert], key)
        .unwrap()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (channels, handles) = soter_server::setup();

    let public_address = soter_server::public_address().await.unwrap().ip();
    let crypto = gen_crypto(public_address);
    let server_config = quinn::ServerConfig::with_crypto(std::sync::Arc::new(crypto));

    let (_endpoint, mut incoming) =
        quinn::Endpoint::server(server_config, "0.0.0.0:1117".parse().unwrap())?;

    while let Some(conn) = incoming.next().await {
        tokio::spawn(soter_server::handle_connection(conn, channels.clone()));
    }

    handles.join().await?;
    Ok(())
}
