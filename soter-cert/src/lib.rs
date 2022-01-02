#![deny(non_ascii_idents, rustdoc::broken_intra_doc_links)]
#![warn(
    // missing_docs,
    rust_2018_idioms,
    // rust_2021_compatibility,
    future_incompatible,
    missing_debug_implementations,
    missing_copy_implementations,
    nonstandard_style,
    clippy::missing_panics_doc,
    clippy::missing_safety_doc
)]
mod attribute;
mod error;
mod message;

use std::net::{IpAddr, SocketAddr};

use message::*;
use tokio::net::UdpSocket;

pub use error::*;

pub const DEFAULT_STUN_SERVER: &str = "172.253.59.127:19302";

#[inline]
pub async fn public_address(addr: &str) -> crate::Result<SocketAddr> {
    let mut message = crate::Message::new(crate::Type {
        class: crate::Class::Request,
        method: crate::Method::Binding,
    });
    message.push(crate::attribute::Attribute::Software(
        // Generates `Software` with a description something like "oxalis v0.1.0"
        crate::attribute::Software::try_from(concat!("soter v", env!("CARGO_PKG_VERSION")))?,
    ));

    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    socket.send_to(&<Vec<u8>>::from(message)[..], addr).await?;

    let mut buf = [0u8; 32];
    socket.recv_from(&mut buf).await?;

    let received = crate::Message::try_from(&buf[..])?;

    for attr in received.attrs() {
        // TODO add non xor mapped address
        if let crate::attribute::Attribute::XorMappedAddress(a) = attr {
            return Ok(a.into());
        }
    }

    Err(Error::Stun(StunError::NoAddress))
}

#[inline]
pub fn gen_crypto(
    public_address: IpAddr,
    key_pair: Option<&soter_core::KeyPair>,
) -> crate::Result<rustls::ServerConfig> {
    let key_pair = match key_pair {
        Some(kp) => Some(rcgen::KeyPair::from_der(kp.as_ref())?),
        None => None,
    };
    let mut cert_params = rcgen::CertificateParams::default();
    cert_params.subject_alt_names = vec![rcgen::SanType::IpAddress(public_address)];
    cert_params.key_pair = key_pair;
    let cert = rcgen::Certificate::from_params(cert_params)?;

    let key = cert.serialize_private_key_der();
    let cert = cert.serialize_der()?;

    let key = rustls::PrivateKey(key);
    let cert = rustls::Certificate(cert);

    rustls::ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(vec![cert], key)
        .map_err(|e| e.into())
}
