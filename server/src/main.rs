mod code_map;
mod connection_map;
mod signing_bytes;

use code_map::code_map_manager;
use connection_map::connection_map_manager;
use signing_bytes::signing_bytes_manager;

use bincode::serialize;
use lib::cs::{
    key::SigningBytes,
    protocol::{
        error::Error,
        request::Request,
        response::{GetKey, GetSigningBytes, Register},
    },
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::{mpsc, oneshot},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (code_tx, code_rx) = mpsc::channel(32);
    let (conn_tx, conn_rx) = mpsc::channel(32);
    let (sign_tx, sign_rx) = mpsc::channel(32);

    let _code_map_manager = tokio::spawn(code_map_manager(code_rx));
    let _connection_map_manager = tokio::spawn(connection_map_manager(conn_rx));
    let _signing_bytes_manager = tokio::spawn(signing_bytes_manager(sign_rx));

    let listener = TcpListener::bind("0.0.0.0:1117").await?;

    loop {
        let (socket, _addr) = listener.accept().await?;
        tokio::spawn(handle_request(
            socket,
            code_tx.clone(),
            conn_tx.clone(),
            sign_tx.clone(),
        ));
    }

    #[allow(unreachable_code)]
    {
        _code_map_manager.await?;
        _connection_map_manager.await?;
        _signing_bytes_manager.await?;
        Ok(())
    }
}

trait SendError: std::error::Error + Send {}

async fn handle_request(
    mut socket: TcpStream,
    code_tx: mpsc::Sender<code_map::Command>,
    _conn_tx: mpsc::Sender<connection_map::Command>,
    sign_tx: mpsc::Sender<oneshot::Sender<SigningBytes>>,
) -> Result<(), Box<dyn SendError>> {
    let mut buf = vec![0; 1024];

    // This is inside an async block so I can easily propagate errors that should be sent.
    // TODO return types and dat
    let resp: Result<Vec<u8>, Error> = async {
        socket.read_to_end(&mut buf).await?;
        let request: Request = bincode::deserialize(&buf)?;

        match request {
            Request::Register(key) => {
                let (resp_tx, resp_rx) = oneshot::channel();

                code_tx
                    .send(code_map::Command::Generate { key, resp: resp_tx })
                    .await?;
                let code = resp_rx.await?;

                Ok(serialize(&Result::<Register, Error>::Ok(Register(code))).unwrap())
            }
            Request::GetKey(code) => {
                let (resp_tx, resp_rx) = oneshot::channel();

                code_tx
                    .send(code_map::Command::Get {
                        code,
                        resp: resp_tx,
                    })
                    .await?;
                let key = resp_rx.await?.ok_or(Error::InvalidCode)?;

                Ok(serialize(&Result::<GetKey, Error>::Ok(GetKey(key))).unwrap())
            }
            Request::GetSigningBytes => {
                let (resp_tx, resp_rx) = oneshot::channel();

                sign_tx.send(resp_tx).await?;
                let signing_bytes = resp_rx.await?;

                Ok(
                    serialize(&Result::<GetSigningBytes, Error>::Ok(GetSigningBytes(
                        signing_bytes,
                    )))
                    .unwrap(),
                )
            }
            Request::RequestConnection { .. } => {
                todo!();
            }
            Request::CheckConnection(_) => {
                todo!();
            }
            Request::Ping { .. } => {
                todo!();
            }
        }
    }
    .await;

    let resp = match resp {
        Ok(b) => b,
        Err(e) => serialize(&e).unwrap(),
    };

    socket.write(&resp).await.unwrap();

    Ok(())
}
