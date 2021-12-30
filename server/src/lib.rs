mod handler;
mod manager;
pub mod setup;
mod util;

use std::net::SocketAddr;

use util::serialize;

use lib::{
    bincode,
    cs::protocol::{error::Error, request::RequestType, response::GetSigningBytes},
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub use setup::setup;

pub async fn handle_request<T>(mut socket: T, address: SocketAddr, channels: setup::Channels)
where
    // TODO buffered read and write?
    T: tokio::io::AsyncRead + tokio::io::AsyncWrite + std::marker::Unpin,
{
    // TODO buf length
    let mut buf = vec![0; 1024];

    let resp: Result<Vec<u8>, Error> = async {
        socket.read(&mut buf).await?;
        let request: RequestType = bincode::deserialize(&buf)?;

        // TODO remove serialize from every branch
        match request {
            RequestType::Register(r) => serialize(handler::register(channels, r).await),
            RequestType::GetKey(r) => serialize(handler::get_key(channels, r).await),
            RequestType::GetSigningBytes(_) => {
                let signing_bytes = util::signing_bytes(channels.sign).await?;
                serialize(Result::<_, Error>::Ok(GetSigningBytes(signing_bytes)))
            }
            RequestType::RequestConnection(r) => {
                serialize(handler::request_connection(channels, r, address).await)
            }
            RequestType::CheckConnection(r) => {
                serialize(handler::check_connection(channels, r, address).await)
            }
            // initiator address
            RequestType::Ping(_) => serialize(handler::ping(channels, address).await),
        }
    }
    .await;

    let resp = match resp {
        Ok(b) => b,
        // TODO unwrap
        Err(e) => bincode::serialize::<Result<RequestType, Error>>(&Err(e)).unwrap(),
    };

    // TODO unwrap
    socket.write(&resp).await.unwrap();
}
