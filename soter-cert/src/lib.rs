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
mod config;
mod error;
mod stun;
mod verifier;

use std::net::SocketAddr;

use tokio::net::UdpSocket;

pub use config::{gen_client_config, gen_server_config};
pub use error::{Error, Result};

pub const DEFAULT_STUN_SERVER: &str = "172.253.59.127:19302";

#[inline]
pub async fn public_address(addr: &str) -> crate::Result<SocketAddr> {
    let mut message = stun::Message::new(stun::Type {
        class: stun::Class::Request,
        method: stun::Method::Binding,
    });
    message.push(stun::attribute::Attribute::Software(
        stun::attribute::Software::try_from(concat!("soter v", env!("CARGO_PKG_VERSION")))?,
    ));

    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    socket.send_to(&<Vec<u8>>::from(message)[..], addr).await?;

    let mut buf = [0u8; 32];
    socket.recv_from(&mut buf).await?;

    let received = stun::Message::try_from(&buf[..])?;

    for attr in received.attrs() {
        // TODO add non xor mapped address
        if let stun::attribute::Attribute::XorMappedAddress(a) = attr {
            return Ok(a.into());
        }
    }

    Err(Error::Stun(stun::Error::NoAddress))
}
