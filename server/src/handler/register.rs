use crate::{manager::code_map, setup::Channels};

use lib::cs::protocol::{error::Result, request, response};

#[inline]
pub async fn register(
    channels: Channels,
    request::Register(key): request::Register,
) -> Result<response::Register> {
    let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();

    channels
        .code
        .send(code_map::Command::Generate { key, resp: resp_tx })
        .await?;

    let code = resp_rx.await?;
    Ok(response::Register(code))
}
