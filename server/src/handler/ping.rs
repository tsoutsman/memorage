use std::net::SocketAddr;

use crate::{manager::connection_map, setup::Channels};

use lib::cs::protocol::{error::Error, response::Ping};

#[inline]
pub async fn ping(channels: Channels, initiator_address: SocketAddr) -> Result<Ping, Error> {
    let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();

    channels
        .conn
        .send(connection_map::Command::Ping {
            initiator_address,
            resp: resp_tx,
        })
        .await?;

    let target_socket = resp_rx.await?;
    Ok(Ping(target_socket))
}
