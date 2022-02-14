use crate::manager::{code_map, connection_map};

use tokio::{sync::mpsc, task};

#[derive(Clone, Debug)]
pub struct Channels {
    pub code: mpsc::Sender<code_map::Command>,
    pub conn: mpsc::Sender<connection_map::Command>,
}

#[derive(Debug)]
pub struct Handles {
    code: task::JoinHandle<()>,
    conn: task::JoinHandle<()>,
}

impl Handles {
    pub async fn join(self) -> Result<(), task::JoinError> {
        self.code.await?;
        self.conn.await
    }
}

pub fn setup() -> (Channels, Handles) {
    let (code_tx, code_rx) = mpsc::channel(32);
    let (conn_tx, conn_rx) = mpsc::channel(32);

    let code_map_manager = tokio::spawn(code_map::manager(code_rx));
    let connection_map_manager = tokio::spawn(connection_map::manager(conn_rx));

    (
        Channels {
            code: code_tx,
            conn: conn_tx,
        },
        Handles {
            code: code_map_manager,
            conn: connection_map_manager,
        },
    )
}
