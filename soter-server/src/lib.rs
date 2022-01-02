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

mod error;
mod handler;
mod manager;
pub mod setup;
mod util;

use std::net::SocketAddr;

use soter_cs::{request::RequestType, response, serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub use error::{Error, Result};
pub use setup::setup;

pub async fn handle_connection(conn: quinn::Connecting, channels: setup::Channels) -> Result<()> {
    // remote_address must be called before awaiting the connection
    let addr = conn.remote_address();
    let quinn::NewConnection { mut bi_streams, .. } = conn.await?;
    while let Some(stream) = bi_streams.next().await {
        let stream: (quinn::SendStream, quinn::RecvStream) = match stream {
            Ok(s) => s,
            Err(quinn::ConnectionError::ApplicationClosed { .. }) => return Ok(()),
            Err(e) => return Err(e.into()),
        };
        // TODO what happens if handle_request returns error
        tokio::spawn(handle_request(stream, addr, channels.clone()));
    }

    Ok(())
}

#[inline]
pub async fn handle_request<S, R>(
    (mut send, mut recv): (S, R),
    address: SocketAddr,
    channels: setup::Channels,
) -> Result<()>
where
    S: tokio::io::AsyncWrite + std::marker::Unpin,
    R: tokio::io::AsyncRead + std::marker::Unpin,
{
    // TODO buf length
    let mut buf = vec![0; 1024];

    let request: soter_cs::Result<RequestType> = async {
        recv.read(&mut buf)
            .await
            .map_err(|_| soter_cs::Error::Generic)?;
        soter_cs::deserialize(&buf).map_err(|_| soter_cs::Error::Generic)
    }
    .await;

    let resp = match request {
        Ok(request) => {
            match request {
                RequestType::Register(r) => serialize(handler::register(channels, r).await),
                RequestType::GetKey(r) => serialize(handler::get_key(channels, r).await),
                RequestType::GetSigningBytes(_) => match util::signing_bytes(channels.sign).await {
                    Ok(signing_bytes) => serialize(Ok(response::GetSigningBytes(signing_bytes))),
                    Err(e) => serialize(soter_cs::Result::<response::GetSigningBytes>::Err(e)),
                },
                RequestType::RequestConnection(r) => {
                    serialize(handler::request_connection(channels, r, address).await)
                }
                RequestType::CheckConnection(r) => {
                    serialize(handler::check_connection(channels, r, address).await)
                }
                // initiator address
                RequestType::Ping(_) => serialize(handler::ping(channels, address).await),
            }
        }
        // TODO not sure if this is sound. alternatively we can just ignore the request.
        Err(e) => serialize(soter_cs::Result::<response::Register>::Err(e)),
    }?;

    match send.write(&resp).await {
        Ok(_) => Ok(()),
        Err(e) => Err(e.into()),
    }
}
