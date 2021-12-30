use crate::{manager::code_map, setup::Channels};

use lib::cs::protocol::{
    error::{Error, Result},
    request, response,
};

#[inline]
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
        .await?;
    let key = resp_rx.await?.ok_or(Error::InvalidCode)?;
    Ok(response::GetKey(key))
}
