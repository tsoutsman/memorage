use crate::manager::{code_map, connection_map, signing_bytes};

use lib::cs::key::SigningBytes;
use tokio::{
    sync::{mpsc, oneshot},
    task,
};

#[derive(Clone)]
pub struct Channels {
    pub code: mpsc::Sender<code_map::Command>,
    pub conn: mpsc::Sender<connection_map::Command>,
    pub sign: mpsc::Sender<oneshot::Sender<SigningBytes>>,
}

pub struct Handles {
    code: task::JoinHandle<()>,
    conn: task::JoinHandle<()>,
    sign: task::JoinHandle<()>,
}

impl Handles {
    pub async fn join(self) -> Result<(), task::JoinError> {
        self.code.await?;
        self.conn.await?;
        self.sign.await
    }
}

pub fn setup() -> (Channels, Handles) {
    let (code_tx, code_rx) = mpsc::channel(32);
    let (conn_tx, conn_rx) = mpsc::channel(32);
    let (sign_tx, sign_rx) = mpsc::channel(32);

    let code_map_manager = tokio::spawn(code_map::manager(code_rx));
    let connection_map_manager = tokio::spawn(connection_map::manager(conn_rx));
    let signing_bytes_manager = tokio::spawn(signing_bytes::manager(sign_rx));

    (
        Channels {
            code: code_tx,
            conn: conn_tx,
            sign: sign_tx,
        },
        Handles {
            code: code_map_manager,
            conn: connection_map_manager,
            sign: signing_bytes_manager,
        },
    )
}
