use std::net::SocketAddr;

use crate::{manager::connection_map, setup::Channels};

use memorage_cs::{response::Ping, Error};

#[inline]
#[tracing::instrument(skip(channels))]
pub async fn ping(channels: Channels, initiator_address: SocketAddr) -> Result<Ping, Error> {
    let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();

    channels
        .conn
        .send(connection_map::Command::Ping {
            initiator_address,
            resp: resp_tx,
        })
        .await
        .map_err(|_| Error::Generic)?;

    let target_socket = resp_rx.await.map_err(|_| Error::Generic)?;
    Ok(Ping(target_socket))
}
