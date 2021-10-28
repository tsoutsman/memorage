mod code_map;
mod connection_map;

use code_map::code_map_manager;
use connection_map::connection_map_manager;

use lib::cs::protocol::{ClientRequest, Error, ServerResponse, SuccesfulResponse};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::{mpsc, oneshot},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (code_tx, code_rx) = mpsc::channel(32);
    let (conn_tx, conn_rx) = mpsc::channel(32);

    let _code_map_manager = tokio::spawn(code_map_manager(code_rx));
    let _connection_map_manager = tokio::spawn(connection_map_manager(conn_rx));

    let listener = TcpListener::bind("0.0.0.0:8389").await?;

    loop {
        let (socket, _addr) = listener.accept().await?;
        tokio::spawn(handle_request(socket, code_tx.clone(), conn_tx.clone()));
    }

    #[allow(unreachable_code)]
    {
        _code_map_manager.await?;
        _connection_map_manager.await?;
        Ok(())
    }
}

trait SendError: std::error::Error + Send {}

async fn handle_request(
    mut socket: TcpStream,
    code_tx: mpsc::Sender<code_map::Command>,
    _conn_tx: mpsc::Sender<connection_map::Command>,
) -> Result<(), Box<dyn SendError>> {
    let mut buf = vec![0; 1024];

    // This is inside an async block so I can easily propagate errors that should be sent.
    let resp: ServerResponse = async {
        socket.read_to_end(&mut buf).await?;
        let request: ClientRequest = bincode::deserialize(&buf)?;

        match request {
            ClientRequest::Register(key) => {
                let (resp_tx, resp_rx) = oneshot::channel();

                code_tx
                    .send(code_map::Command::Generate { key, resp: resp_tx })
                    .await?;
                let code = resp_rx.await?;

                Ok(SuccesfulResponse::Register(code))
            }
            ClientRequest::GetKey(code) => {
                let (resp_tx, resp_rx) = oneshot::channel();

                code_tx
                    .send(code_map::Command::Get {
                        code,
                        resp: resp_tx,
                    })
                    .await?;
                let key = resp_rx.await?.ok_or(Error::InvalidCode)?;

                Ok(SuccesfulResponse::GetKey(key))
            }
            ClientRequest::RequestConnection { .. } => {
                todo!();
            }
            ClientRequest::Ping { .. } => {
                todo!();
            }
            _ => {
                todo!();
            }
        }
    }
    .await;

    let resp = bincode::serialize(&resp).unwrap();
    socket.write(&resp).await.unwrap();

    Ok(())
}
