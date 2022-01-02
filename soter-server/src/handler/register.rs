use crate::{manager::code_map, setup::Channels};

use soter_cs::{request, response, Error, Result};

#[inline]
#[tracing::instrument(skip(channels))]
pub async fn register(
    channels: Channels,
    request::Register(key): request::Register,
) -> Result<response::Register> {
    let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();

    channels
        .code
        .send(code_map::Command::Generate { key, resp: resp_tx })
        .await
        .map_err(|_| Error::Generic)?;

    let code = resp_rx.await.map_err(|_| Error::Generic)?;
    Ok(response::Register(code))
}
