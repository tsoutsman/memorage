use std::{net::IpAddr, sync::Arc};

use soter_core::{KeyPair, PublicKey};

use crate::{verifier::ServerCertVerifier, Result};

#[inline]
fn gen_cert(
    public_address: IpAddr,
    key_pair: &KeyPair,
) -> Result<(rustls::Certificate, rustls::PrivateKey)> {
    let mut cert_params = rcgen::CertificateParams::default();
    cert_params.alg = &rcgen::PKCS_ED25519;
    cert_params.subject_alt_names = vec![rcgen::SanType::IpAddress(public_address)];
    cert_params.key_pair = Some(rcgen::KeyPair::from_der(key_pair.as_ref())?);
    let cert = rcgen::Certificate::from_params(cert_params)?;

    let serialized_cert = cert.serialize_der()?;
    let serialized_key = cert.serialize_private_key_der();

    let cert = rustls::Certificate(serialized_cert);
    let key = rustls::PrivateKey(serialized_key);
    Ok((cert, key))
}

#[inline]
pub fn gen_server_config(
    public_address: IpAddr,
    key_pair: &KeyPair,
    // TODO check validity of incoming connection certificates
) -> Result<quinn::ServerConfig> {
    let (cert, key) = gen_cert(public_address, key_pair)?;

    let rustls_config = rustls::ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(vec![cert], key)?;
    Ok(quinn::ServerConfig::with_crypto(Arc::new(rustls_config)))
}

#[inline]
pub fn gen_client_config(
    public_address: IpAddr,
    initiator_key_pair: &KeyPair,
    _target_key: Option<&PublicKey>,
) -> Result<quinn::ClientConfig> {
    let (cert, key) = gen_cert(public_address, initiator_key_pair)?;

    let rustls_config = rustls::ClientConfig::builder()
        .with_safe_default_cipher_suites()
        .with_safe_default_kx_groups()
        .with_safe_default_protocol_versions()?
        // TODO
        .with_custom_certificate_verifier(Arc::new(ServerCertVerifier(|| {
            Ok(rustls::client::ServerCertVerified::assertion())
        })))
        .with_single_cert(vec![cert], key)?;
    Ok(quinn::ClientConfig::new(Arc::new(rustls_config)))
}
