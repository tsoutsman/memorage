use std::net::SocketAddr;

use crate::{manager::connection_map, setup::Channels};

use soter_core::PublicKey;
use soter_cs::{response, Error, Result};

#[inline]
#[tracing::instrument(skip(channels))]
pub async fn check_connection(
    channels: Channels,
    target_key: PublicKey,
    target_address: SocketAddr,
) -> Result<response::CheckConnection> {
    let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();

    channels
        .conn
        .send(connection_map::Command::CheckConnection {
            target_key,
            target_address,
            resp: resp_tx,
        })
        .await
        .map_err(|_| Error::Generic)?;

    let initiator_socket = resp_rx.await.map_err(|_| Error::Generic)?;
    Ok(response::CheckConnection(initiator_socket))
}
