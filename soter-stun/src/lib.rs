mod attribute;
mod error;
mod message;

pub use error::{Error, Result, StunError};
pub(crate) use message::*;

use std::net::SocketAddr;
use tokio::net::UdpSocket;

pub const DEFAULT_STUN_SERVER: &str = "172.253.59.127:19302";

#[inline]
pub async fn public_address(addr: &str) -> crate::Result<SocketAddr> {
    let mut message = Message::new(Type {
        class: Class::Request,
        method: Method::Binding,
    });
    message.push(attribute::Attribute::Software(
        attribute::Software::try_from(concat!("soter v", env!("CARGO_PKG_VERSION")))?,
    ));

    let socket = UdpSocket::bind("0.0.0.0:0").await?;
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
