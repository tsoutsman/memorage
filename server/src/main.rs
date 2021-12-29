mod handler;
mod manager;
mod setup;
mod util;

use std::net::SocketAddr;

use util::serialize;

use lib::cs::protocol::{error::Error, request::Request, response::GetSigningBytes};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (channels, _handles) = setup::setup();

    let listener = TcpListener::bind("0.0.0.0:1117").await?;

    loop {
        let (socket, addr) = listener.accept().await?;
        tokio::spawn(handle_request(socket, addr, channels.clone()));
    }

    #[allow(unreachable_code)]
    {
        _handles.join().await?;
        Ok(())
    }
}

async fn handle_request<T>(mut socket: T, address: SocketAddr, channels: setup::Channels)
where
    // TODO buffered read and write?
    T: tokio::io::AsyncRead + tokio::io::AsyncWrite + std::marker::Unpin,
{
    // TODO buf length
    let mut buf = vec![0; 1024];

    let resp: Result<Vec<u8>, Error> = async {
        socket.read_to_end(&mut buf).await?;
        let request: Request = bincode::deserialize(&buf)?;

        match request {
            Request::Register(key) => serialize(handler::register(channels, key).await),
            Request::GetKey(code) => serialize(handler::get_key(channels, code).await),
            Request::GetSigningBytes => {
                let signing_bytes = util::signing_bytes(channels.sign).await?;
                serialize(GetSigningBytes(signing_bytes))
            }
            Request::RequestConnection {
                initiator_key,
                target_key,
            } => {
                serialize(
                    handler::request_connection(
                        channels,
                        initiator_key,
                        target_key,
                        // initiator address
                        address,
                    )
                    .await,
                )
            }
            Request::CheckConnection(target_key) => {
                // target address
                serialize(handler::check_connection(channels, target_key, address).await)
            }
            Request::Ping => {
                // initiator address
                serialize(handler::ping(channels, address).await)
            }
        }
    }
    .await;

    let resp = match resp {
        Ok(b) => b,
        Err(e) => bincode::serialize(&e).unwrap(),
    };

    // TODO unwrap
    socket.write(&resp).await.unwrap();
}
