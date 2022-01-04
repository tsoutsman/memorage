use std::net::SocketAddr;

use crate::{manager::connection_map, setup::Channels};

use soter_core::PublicKey;
use soter_cs::{request, response, Error, Result};

#[inline]
#[tracing::instrument(skip(channels))]
pub async fn request_connection(
    channels: Channels,
    request::RequestConnection(target_key): request::RequestConnection,
    initiator_key: PublicKey,
    initiator_address: SocketAddr,
) -> Result<response::RequestConnection> {
    let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();

    channels
        .conn
        .send(connection_map::Command::RequestConnection {
            initiator_key,
            initiator_address,
            target_key,
            resp: resp_tx,
        })
        .await
        .map_err(|_| Error::Generic)?;

    let _ = resp_rx.await.map_err(|_| Error::Generic)?;
    Ok(response::RequestConnection)
}
