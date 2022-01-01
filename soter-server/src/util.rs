use std::net::SocketAddr;

use soter_core::{PublicKey, Verifiable};
use soter_cs::{Error, SigningBytes};
use tokio::{
    net::UdpSocket,
    sync::{mpsc, oneshot},
};

// TODO error type
pub async fn public_address() -> Result<SocketAddr, ()> {
    // TODO don't hardcode the address of a single STUN server.
    let addr = "172.253.59.127:19302";

    let mut message = soter_stun::Message::new(soter_stun::Type {
        class: soter_stun::Class::Request,
        method: soter_stun::Method::Binding,
    });
    message.push(soter_stun::attribute::Attribute::Software(
        // Generates `Software` with a description something like "oxalis v0.1.0"
        soter_stun::attribute::Software::try_from(concat!("soter v", env!("CARGO_PKG_VERSION")))
            .map_err(|_| ())?,
    ));

    let socket = UdpSocket::bind("0.0.0.0:0").await.map_err(|_| ())?;
    socket
        .send_to(&<Vec<u8>>::from(message)[..], addr)
        .await
        .map_err(|_| ())?;

    let mut buf = [0u8; 32];
    socket.recv_from(&mut buf).await.map_err(|_| ())?;

    let received = soter_stun::Message::try_from(&buf[..]).map_err(|_| ())?;

    for attr in received.attrs() {
        // TODO add non xor mapped address
        if let soter_stun::attribute::Attribute::XorMappedAddress(a) = attr {
            return Ok(a.into());
        }
    }

    // TODO more specific error?
    Err(())
}

pub async fn signing_bytes(
    sign_tx: mpsc::Sender<oneshot::Sender<SigningBytes>>,
) -> Result<SigningBytes, Error> {
    let (resp_tx, resp_rx) = oneshot::channel();
    sign_tx.send(resp_tx).await.map_err(|_| Error::Generic)?;
    resp_rx.await.map_err(|_| Error::Generic)
}

pub async fn verify_key(
    key: Verifiable<PublicKey>,
    sign_tx: mpsc::Sender<oneshot::Sender<SigningBytes>>,
) -> Result<PublicKey, Error> {
    key.into_verifier(&signing_bytes(sign_tx).await?)
        .map_err(|e| e.into())
}
