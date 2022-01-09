#![deny(
    non_ascii_idents,
    // missing_docs,
    rust_2018_idioms,
    // rust_2021_compatibility,
    future_incompatible,
    missing_debug_implementations,
    missing_copy_implementations,
    nonstandard_style,
    unreachable_pub,
    clippy::missing_panics_doc,
    clippy::missing_safety_doc,
    rustdoc::broken_intra_doc_links
)]

const CODE_MAP_SIZE: usize = 256;
const SOCKET_MAP_SIZE: usize = 256;
const REQUEST_MAP_SIZE: usize = 256;

mod error;
mod handler;
mod hash_map;
mod manager;
pub mod setup;

use std::net::SocketAddr;

use soter_core::PublicKey;
use soter_cs::{request::RequestType, response, serialize};
use tracing::{info, warn};

pub use error::{Error, Result};
pub use setup::setup;

pub async fn handle_connection(conn: quinn::Connecting, channels: setup::Channels) -> Result<()> {
    // remote_address must be called before awaiting the connection
    let addr = conn.remote_address();
    let quinn::NewConnection {
        connection,
        mut bi_streams,
        ..
    } = conn.await?;
    let client_key = soter_cert::get_key_unchecked(&connection)?;
    info!(?client_key, "user key grabbed from connection");

    while let Some(stream) = bi_streams.next().await {
        let stream: (quinn::SendStream, quinn::RecvStream) = match stream {
            Ok(s) => s,
            Err(quinn::ConnectionError::ApplicationClosed { .. }) => return Ok(()),
            Err(e) => return Err(e.into()),
        };
        // TODO what happens if handle_request returns error
        tokio::spawn(handle_quinn_request(
            stream,
            client_key,
            addr,
            channels.clone(),
        ));
    }

    Ok(())
}

pub async fn handle_quinn_request(
    (mut send, recv): (quinn::SendStream, quinn::RecvStream),
    client_key: PublicKey,
    address: SocketAddr,
    channels: setup::Channels,
) -> Result<()> {
    let maybe_buf = recv
        .read_to_end(1024)
        .await
        .map_err(|_| soter_cs::Error::Generic);

    let result = handle_request(maybe_buf, client_key, address, channels).await?;

    match send.write_all(&result).await {
        Ok(_) => {
            info!("closing connection");
            send.finish().await?;
            Ok(())
        }
        Err(e) => {
            warn!(?e, "error sending response");
            Err(e.into())
        }
    }
}

#[doc(hidden)]
pub async fn __test_handle_request(
    maybe_buf: soter_cs::Result<Vec<u8>>,
    client_key: PublicKey,
    address: SocketAddr,
    channels: setup::Channels,
) -> Result<Vec<u8>> {
    handle_request(maybe_buf, client_key, address, channels).await
}

#[inline]
#[tracing::instrument(skip(maybe_buf, channels))]
async fn handle_request(
    maybe_buf: soter_cs::Result<Vec<u8>>,
    client_key: PublicKey,
    address: SocketAddr,
    channels: setup::Channels,
) -> Result<Vec<u8>> {
    info!("accepted connection");

    let request: soter_cs::Result<RequestType> =
        async { soter_cs::deserialize(&maybe_buf?).map_err(|_| soter_cs::Error::Generic) }.await;

    // TODO: Verify that client_key matches address stored in hashmap.
    let resp = match request {
        Ok(ty) => {
            info!(?ty, "decoded type");
            match ty {
                RequestType::Register(_) => {
                    serialize(handler::register(channels, client_key).await)
                }
                RequestType::GetKey(r) => serialize(handler::get_key(channels, r).await),
                RequestType::RequestConnection(r) => {
                    serialize(handler::request_connection(channels, r, client_key, address).await)
                }
                RequestType::CheckConnection(_) => {
                    serialize(handler::check_connection(channels, client_key, address).await)
                }
                // initiator address
                RequestType::Ping(_) => serialize(handler::ping(channels, address).await),
            }
        }
        // TODO not sure if this is sound. alternatively we can just ignore the request.
        Err(e) => serialize(soter_cs::Result::<response::Register>::Err(e)),
    }?;

    Ok(resp)
}
