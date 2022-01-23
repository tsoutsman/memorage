#![deny(
    non_ascii_idents,
    // missing_docs,
    rust_2018_idioms,
    rust_2021_compatibility,
    future_incompatible,
    missing_debug_implementations,
    missing_copy_implementations,
    nonstandard_style,
    unreachable_pub,
    clippy::missing_panics_doc,
    clippy::missing_safety_doc,
    rustdoc::broken_intra_doc_links
)]

mod attribute;
mod error;
mod message;

pub use error::{Error, Result, StunError};
pub(crate) use message::*;

use std::net::SocketAddr;
use tokio::net::UdpSocket;

pub const DEFAULT_STUN_SERVER: &str = "172.253.59.127:19302";

#[inline]
pub async fn public_address(socket: &mut UdpSocket, addr: &str) -> crate::Result<SocketAddr> {
    let mut message = Message::new(Type {
        class: Class::Request,
        method: Method::Binding,
    });
    message.push(attribute::Attribute::Software(
        attribute::Software::try_from(concat!("soter v", env!("CARGO_PKG_VERSION")))?,
    ));

    socket.send_to(&<Vec<u8>>::from(message)[..], addr).await?;

    let mut buf = [0u8; 32];
    socket.recv_from(&mut buf).await?;

    let received = Message::try_from(&buf[..])?;

    for attr in received.attrs() {
        // TODO add non xor mapped address
        if let attribute::Attribute::XorMappedAddress(a) = attr {
            return Ok(a.into());
        }
    }

    Err(Error::Stun(StunError::NoAddress))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn same_socket_same_address() {
        let mut socket = UdpSocket::bind("0.0.0.0:0").await.unwrap();
        let addr_1 = public_address(&mut socket, DEFAULT_STUN_SERVER)
            .await
            .unwrap();
        let addr_2 = public_address(&mut socket, DEFAULT_STUN_SERVER)
            .await
            .unwrap();
        assert_eq!(addr_1, addr_2);
    }
}
