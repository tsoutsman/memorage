use std::net::SocketAddr;

use crate::{manager::connection_map, setup::Channels, util::verify_key};

use soter_cs::{request, response, Error, Result};

#[inline]
#[tracing::instrument(skip(channels))]
pub async fn check_connection(
    channels: Channels,
    request::CheckConnection(target_key): request::CheckConnection,
    target_address: SocketAddr,
) -> Result<response::CheckConnection> {
    let target_key = verify_key(target_key, channels.sign).await?;
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
