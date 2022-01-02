use crate::{manager::code_map, setup::Channels};

use soter_cs::{request, response, Error, Result};

#[inline]
#[tracing::instrument(skip(channels))]
pub async fn get_key(
    channels: Channels,
    request::GetKey(code): request::GetKey,
) -> Result<response::GetKey> {
    let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();

    channels
        .code
        .send(code_map::Command::Get {
            code,
            resp: resp_tx,
        })
        .await
        .map_err(|_| Error::Generic)?;
    let key = resp_rx
        .await
        .map_err(|_| Error::Generic)?
        .ok_or(Error::InvalidPairingCode)?;
    Ok(response::GetKey(key))
}
