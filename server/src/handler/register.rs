use crate::{manager::code_map, setup::Channels};

use lib::cs::{
    key::PublicKey,
    protocol::{error::Error, response::Register},
};

#[inline]
pub async fn register(channels: Channels, key: PublicKey) -> Result<Register, Error> {
    let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();

    channels
        .code
        .send(code_map::Command::Generate { key, resp: resp_tx })
        .await?;

    let code = resp_rx.await?;
    Ok(Register(code))
}
