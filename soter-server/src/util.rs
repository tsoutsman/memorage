use std::net::SocketAddr;

use soter_core::{PublicKey, Verifiable};
use soter_cs::SigningBytes;
use tokio::{
    net::UdpSocket,
    sync::{mpsc, oneshot},
};

pub async fn public_address() -> crate::Result<SocketAddr> {
    // TODO don't hardcode the address of a single STUN server.
    let addr = "172.253.59.127:19302";

    let mut message = soter_stun::Message::new(soter_stun::Type {
        class: soter_stun::Class::Request,
        method: soter_stun::Method::Binding,
    });
    message.push(soter_stun::attribute::Attribute::Software(
        // Generates `Software` with a description something like "oxalis v0.1.0"
        soter_stun::attribute::Software::try_from(concat!("soter v", env!("CARGO_PKG_VERSION")))?,
    ));

    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    socket.send_to(&<Vec<u8>>::from(message)[..], addr).await?;

    let mut buf = [0u8; 32];
    socket.recv_from(&mut buf).await?;

    let received = soter_stun::Message::try_from(&buf[..])?;

    for attr in received.attrs() {
        // TODO add non xor mapped address
        if let soter_stun::attribute::Attribute::XorMappedAddress(a) = attr {
            return Ok(a.into());
        }
    }

    Err(crate::Error::NoAddress)
}

pub async fn signing_bytes(
    sign_tx: mpsc::Sender<oneshot::Sender<SigningBytes>>,
) -> soter_cs::Result<SigningBytes> {
    let (resp_tx, resp_rx) = oneshot::channel();
    sign_tx
        .send(resp_tx)
        .await
        .map_err(|_| soter_cs::Error::Generic)?;
    resp_rx.await.map_err(|_| soter_cs::Error::Generic)
}

pub async fn verify_key(
    key: Verifiable<PublicKey>,
    sign_tx: mpsc::Sender<oneshot::Sender<SigningBytes>>,
) -> soter_cs::Result<PublicKey> {
    key.into_key(&signing_bytes(sign_tx).await?)
        .map_err(|e| e.into())
}
