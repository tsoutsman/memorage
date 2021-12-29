use crate::{manager::code_map, setup::Channels};

use lib::cs::{
    protocol::{error::Error, response::GetKey},
    Code,
};

#[inline]
pub async fn get_key(channels: Channels, code: Code) -> Result<GetKey, Error> {
    let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();

    channels
        .code
        .send(code_map::Command::Get {
            code,
            resp: resp_tx,
        })
        .await?;
    let key = resp_rx.await?.ok_or(Error::InvalidCode)?;
    Ok(GetKey(key))
}
