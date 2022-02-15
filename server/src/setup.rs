use crate::manager::{establish, pair, request};

use tokio::{sync::mpsc, task};

#[derive(Clone, Debug)]
pub struct Channels {
    pub pair: mpsc::Sender<pair::Command>,
    pub request: mpsc::Sender<request::Command>,
    pub establish: mpsc::Sender<establish::Command>,
}

#[derive(Debug)]
pub struct Handles {
    pair: task::JoinHandle<()>,
    request: task::JoinHandle<()>,
    establish: task::JoinHandle<()>,
}

impl Handles {
    pub async fn join(self) -> Result<(), task::JoinError> {
        self.pair.await?;
        self.request.await?;
        self.establish.await
    }
}

pub fn setup() -> (Channels, Handles) {
    let (pair_tx, pair_rx) = mpsc::channel(16);
    let (request_tx, request_rx) = mpsc::channel(32);
    let (establish_tx, establish_rx) = mpsc::channel(32);

    let pair_manager = tokio::spawn(pair::manager(pair_rx));
    let request_manager = tokio::spawn(request::manager(request_rx));
    let establish_manager = tokio::spawn(establish::manager(establish_rx));

    (
        Channels {
            pair: pair_tx,
            request: request_tx,
            establish: establish_tx,
        },
        Handles {
            pair: pair_manager,
            request: request_manager,
            establish: establish_manager,
        },
    )
}
