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

const ADDRESS_MAP_SIZE: usize = 256;
const CODE_MAP_SIZE: usize = 256;
const REQUEST_MAP_SIZE: usize = 256;

mod collections;
mod error;
mod manager;
pub mod setup;

use std::net::SocketAddr;

use memorage_core::PublicKey;
use memorage_cs::{deserialize, request::RequestType, response, serialize};
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
    let client_key = memorage_cert::get_key_unchecked(&connection)?;
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
        .map_err(|_| memorage_cs::Error::Generic);

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
    maybe_buf: memorage_cs::Result<Vec<u8>>,
    client_key: PublicKey,
    address: SocketAddr,
    channels: setup::Channels,
) -> Result<Vec<u8>> {
    handle_request(maybe_buf, client_key, address, channels).await
}

#[doc(hidden)]
pub fn __setup_logger() {
    tracing_subscriber::fmt::fmt().init();
}

#[inline]
#[tracing::instrument(skip(maybe_buf, channels))]
async fn handle_request(
    maybe_buf: memorage_cs::Result<Vec<u8>>,
    client_key: PublicKey,
    client_address: SocketAddr,
    channels: setup::Channels,
) -> Result<Vec<u8>> {
    info!("accepted connection");

    let request: memorage_cs::Result<RequestType> =
        async { Ok(deserialize(&maybe_buf?).unwrap()) }.await;

    // TODO: Verify that client_key matches address stored in hashmap.
    let resp = match request {
        Ok(ty) => {
            info!(?ty, "decoded type");
            match ty {
                RequestType::Register(_) => {
                    let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
                    let cmd = manager::pair::Command::Register {
                        key: client_key,
                        resp: resp_tx,
                    };

                    let response: memorage_cs::Result<memorage_cs::response::Register> = async {
                        channels
                            .pair
                            .send(cmd)
                            .await
                            .map_err(|_| memorage_cs::Error::Generic)?;
                        resp_rx.await.map_err(|_| memorage_cs::Error::Generic)
                    }
                    .await;
                    serialize(response)
                }
                RequestType::GetKey(r) => {
                    let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
                    let cmd = manager::pair::Command::GetKey {
                        code: r.0,
                        requestor: client_key,
                        resp: resp_tx,
                    };

                    let response: memorage_cs::Result<memorage_cs::response::GetKey> = async {
                        channels
                            .pair
                            .send(cmd)
                            .await
                            .map_err(|_| memorage_cs::Error::Generic)?;
                        resp_rx.await.map_err(|_| memorage_cs::Error::Generic)?
                    }
                    .await;
                    serialize(response)
                }
                RequestType::GetRegisterResponse(_) => {
                    let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
                    let cmd = manager::pair::Command::GetRegisterResponse {
                        initiator: client_key,
                        resp: resp_tx,
                    };

                    let response: memorage_cs::Result<memorage_cs::response::GetRegisterResponse> =
                        async {
                            channels
                                .pair
                                .send(cmd)
                                .await
                                .map_err(|_| memorage_cs::Error::Generic)?;
                            resp_rx.await.map_err(|_| memorage_cs::Error::Generic)?
                        }
                        .await;
                    serialize(response)
                }
                RequestType::RequestConnection(r) => {
                    let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
                    let cmd = manager::request::Command::RequestConnection {
                        initiator: client_key,
                        target: r.target,
                        time: r.time,
                        resp: resp_tx,
                    };

                    let response: memorage_cs::Result<memorage_cs::response::RequestConnection> =
                        async {
                            channels
                                .request
                                .send(cmd)
                                .await
                                .map_err(|_| memorage_cs::Error::Generic)?;
                            resp_rx.await.map_err(|_| memorage_cs::Error::Generic)
                        }
                        .await;
                    serialize(response)
                }
                RequestType::CheckConnection(_) => {
                    let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
                    let cmd = manager::request::Command::CheckConnection {
                        target: client_key,
                        resp: resp_tx,
                    };

                    let response: memorage_cs::Result<memorage_cs::response::CheckConnection> =
                        async {
                            channels
                                .request
                                .send(cmd)
                                .await
                                .map_err(|_| memorage_cs::Error::Generic)?;
                            resp_rx.await.map_err(|_| memorage_cs::Error::Generic)?
                        }
                        .await;
                    serialize(response)
                }
                // initiator address
                RequestType::Ping(r) => {
                    let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
                    let cmd = manager::establish::Command::Ping {
                        initiator_key: client_key,
                        initiator_address: client_address,
                        target: r.0,
                        resp: resp_tx,
                    };

                    let response: memorage_cs::Result<memorage_cs::response::Ping> = async {
                        channels
                            .establish
                            .send(cmd)
                            .await
                            .map_err(|_| memorage_cs::Error::Generic)?;
                        resp_rx.await.map_err(|_| memorage_cs::Error::Generic)?
                    }
                    .await;
                    serialize(response)
                }
            }
        }
        // TODO not sure if this is sound. alternatively we can just ignore the request.
        Err(e) => serialize(memorage_cs::Result::<response::Register>::Err(e)),
    }?;

    Ok(resp)
}
