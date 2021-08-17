use crate::{
    error::{Error, Result},
    stun,
};

use std::convert::TryFrom;

use tokio::net;

pub async fn stun_information() -> Result<std::net::SocketAddr> {
    // TODO don't hardcode the address of a single STUN server.
    let addr = "172.253.59.127:19302";

    let mut message = stun::Message::new(stun::Type {
        class: stun::Class::Request,
        method: stun::Method::Binding,
    });
    message.push(stun::attribute::Attribute::Software(
        // Generates `Software` with a description something like "oxalis v0.1.0"
        stun::attribute::Software::try_from(concat!("oxalis v", env!("CARGO_PKG_VERSION")))?,
    ));

    let socket = net::UdpSocket::bind("0.0.0.0:0").await?;
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

    // TODO more specific error?
    Err(Error::Decoding)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_stun_information() {
        let info = stun_information().await.unwrap();
        // TODO find a way to actually test this
        println!("STUN information: {:?}", info);
    }
}
