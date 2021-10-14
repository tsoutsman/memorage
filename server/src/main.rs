mod code_map;

use code_map::code_map_manager;

use lib::cs::protocol::{ClientRequest, Error, ServerResponse, SuccesfulResponse};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::{mpsc, oneshot},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (tx, rx) = mpsc::channel(32);

    let _map_manager = tokio::spawn(code_map_manager(rx));

    let listener = TcpListener::bind("0.0.0.0:8389").await?;

    loop {
        let (socket, _addr) = listener.accept().await?;
        tokio::spawn(handle_request(socket, tx.clone()));
    }

    #[allow(unreachable_code)]
    {
        _map_manager.await?;
        Ok(())
    }
}

trait SendError: std::error::Error + Send {}

async fn handle_request(
    mut socket: TcpStream,
    mm_tx: mpsc::Sender<code_map::Command>,
) -> Result<(), Box<dyn SendError>> {
    let mut buf = vec![0; 2048];

    // This is inside an async block so I can easily propagate errors that should be sent.
    let resp: ServerResponse = async {
        socket.read_to_end(&mut buf).await?;
        let request: ClientRequest = bincode::deserialize(&buf)?;

        match request {
            ClientRequest::Register(key) => {
                let (resp_tx, resp_rx) = oneshot::channel();

                mm_tx
                    .send(code_map::Command::Generate { key, resp: resp_tx })
                    .await?;
                let code = resp_rx.await?;

                Ok(SuccesfulResponse::Register(code))
            }
            ClientRequest::GetKey(code) => {
                let (resp_tx, resp_rx) = oneshot::channel();

                mm_tx
                    .send(code_map::Command::Get {
                        code,
                        resp: resp_tx,
                    })
                    .await?;
                let key = resp_rx.await?.ok_or(Error::InvalidCode)?;

                Ok(SuccesfulResponse::GetKey(key))
            }
            ClientRequest::EstablishConnection(_) => todo!(),
            ClientRequest::Ping => todo!(),
        }
    }
    .await;

    let resp = bincode::serialize(&resp).unwrap();

    socket.write(&resp).await.unwrap();
    Ok(())
}
