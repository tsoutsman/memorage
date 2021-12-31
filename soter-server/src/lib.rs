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

mod handler;
mod manager;
pub mod setup;
mod util;

use std::net::SocketAddr;

use soter_cs::{request::RequestType, response, serialize, Error};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub use setup::setup;

pub async fn handle_request<T>(
    mut socket: T,
    address: SocketAddr,
    channels: setup::Channels,
    // TODO error type
) -> Result<(), ()>
where
    // TODO buffered read and write?
    T: tokio::io::AsyncRead + tokio::io::AsyncWrite + std::marker::Unpin,
{
    // TODO buf length
    let mut buf = vec![0; 1024];

    let request: Result<RequestType, Error> = async {
        socket.read(&mut buf).await.map_err(|_| Error::Generic)?;
        soter_cs::deserialize(&buf).map_err(|_| Error::Generic)
    }
    .await;

    let resp = match request {
        Ok(request) => {
            match request {
                RequestType::Register(r) => serialize(handler::register(channels, r).await),
                RequestType::GetKey(r) => serialize(handler::get_key(channels, r).await),
                RequestType::GetSigningBytes(_) => match util::signing_bytes(channels.sign).await {
                    Ok(signing_bytes) => serialize(Result::<_, Error>::Ok(
                        response::GetSigningBytes(signing_bytes),
                    )),
                    Err(e) => serialize(Result::<soter_cs::response::GetSigningBytes, _>::Err(e)),
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
        Err(e) => serialize(Result::<soter_cs::response::Register, _>::Err(e)),
    };

    // TODO clean up all these maps
    socket
        .write(&resp.map_err(|_| ())?)
        .await
        .map(|_| ())
        .map_err(|_| ())
}
