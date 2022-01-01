use std::net::SocketAddr;

use crate::{manager::connection_map, setup::Channels, util::signing_bytes};

use soter_cs::{request, response, Error, Result};

#[inline]
pub async fn request_connection(
    channels: Channels,
    request::RequestConnection {
        initiator_key,
        target_key,
    }: request::RequestConnection,
    initiator_address: SocketAddr,
) -> Result<response::RequestConnection> {
    let signing_bytes = signing_bytes(channels.sign).await?;
    let initiator_key = initiator_key.into_key(&signing_bytes)?;
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
