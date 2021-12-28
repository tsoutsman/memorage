mod code_map;
mod connection_map;
mod signing_bytes;

use code_map::code_map_manager;
use connection_map::connection_map_manager;
use signing_bytes::signing_bytes_manager;

use lib::cs::{
    key::{PublicKey, SigningBytes, VerifiablePublicKey},
    protocol::{
        error::Error,
        request::Request,
        response::{CheckConnection, GetKey, GetSigningBytes, Ping, Register, RequestConnection},
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
impl<T> SendError for T where T: std::error::Error + Send {}

macro_rules! serialize {
    ($e:expr) => {
        // TODO unwrap
        Ok(
            ::bincode::serialize(
                &::std::result::Result::<_, ::lib::cs::protocol::error::Error>::Ok($e)
            )
            .unwrap(),
        )
    };
}

async fn signing_bytes(
    sign_tx: mpsc::Sender<oneshot::Sender<SigningBytes>>,
) -> Result<SigningBytes, Error> {
    let (resp_tx, resp_rx) = oneshot::channel();
    sign_tx.send(resp_tx).await.map_err(|_| Error::Generic)?;
    resp_rx.await.map_err(|_| Error::Generic)
}

async fn verify_key(
    key: VerifiablePublicKey,
    sign_tx: mpsc::Sender<oneshot::Sender<SigningBytes>>,
) -> Result<PublicKey, Error> {
    key.into_key(&signing_bytes(sign_tx).await?)
}

async fn handle_request(
    mut socket: TcpStream,
    code_tx: mpsc::Sender<code_map::Command>,
    conn_tx: mpsc::Sender<connection_map::Command>,
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

                serialize!(Register(code))
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

                serialize!(GetKey(key))
            }
            Request::GetSigningBytes => {
                let signing_bytes = signing_bytes(sign_tx).await?;
                serialize!(GetSigningBytes(signing_bytes))
            }
            Request::RequestConnection {
                initiator_key,
                target_key,
            } => {
                let signing_bytes = signing_bytes(sign_tx).await?;
                let initiator_key = initiator_key.into_key(&signing_bytes)?;
                let initiator_socket = socket.peer_addr()?;
                let (resp_tx, resp_rx) = oneshot::channel();

                conn_tx
                    .send(connection_map::Command::RequestConnection {
                        initiator_key,
                        initiator_socket,
                        target_key,
                        resp: resp_tx,
                    })
                    .await?;

                let _ = resp_rx.await?;
                serialize!(RequestConnection)
            }
            Request::CheckConnection(target_key) => {
                let target_key = verify_key(target_key, sign_tx).await?;
                let target_socket = socket.peer_addr()?;
                let (resp_tx, resp_rx) = oneshot::channel();

                conn_tx
                    .send(connection_map::Command::CheckConnection {
                        target_key,
                        target_socket,
                        resp: resp_tx,
                    })
                    .await?;

                let initiator_socket = resp_rx.await?;
                serialize!(CheckConnection(initiator_socket))
            }
            Request::Ping => {
                let initiator_socket = socket.peer_addr()?;
                let (resp_tx, resp_rx) = oneshot::channel();

                conn_tx
                    .send(connection_map::Command::Ping {
                        initiator_socket,
                        resp: resp_tx,
                    })
                    .await?;

                let target_socket = resp_rx.await?;
                serialize!(Ping(target_socket))
            }
        }
    }
    .await;

    let resp = match resp {
        Ok(b) => b,
        Err(e) => bincode::serialize(&e).unwrap(),
    };

    socket.write(&resp).await.unwrap();

    Ok(())
}
