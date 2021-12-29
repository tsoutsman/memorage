use std::net::SocketAddr;

use crate::{manager::connection_map, setup::Channels, util::verify_key};

use lib::cs::{
    key::VerifiablePublicKey,
    protocol::{error::Error, response::CheckConnection},
};

#[inline]
pub async fn check_connection(
    channels: Channels,
    target_key: VerifiablePublicKey,
    target_address: SocketAddr,
) -> Result<CheckConnection, Error> {
    let target_key = verify_key(target_key, channels.sign).await?;
    let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();

    channels
        .conn
        .send(connection_map::Command::CheckConnection {
            target_key,
            target_address,
            resp: resp_tx,
        })
        .await?;

    let initiator_socket = resp_rx.await?;
    Ok(CheckConnection(initiator_socket))
}
