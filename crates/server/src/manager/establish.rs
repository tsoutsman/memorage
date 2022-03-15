use std::net::SocketAddr;

use crate::{collections::MaxSizeHashMap, ADDRESS_MAP_SIZE};

use memorage_core::PublicKey;
use memorage_cs::{response::Ping, Error, Result};

use tokio::sync::{mpsc, oneshot};
use tracing::info_span;

#[derive(Debug)]
pub enum Command {
    Ping {
        initiator_key: PublicKey,
        initiator_address: SocketAddr,
        target: PublicKey,
        resp: oneshot::Sender<Result<Ping>>,
    },
}

#[tracing::instrument]
pub async fn manager(mut rx: mpsc::Receiver<Command>) {
    let mut addresses = MaxSizeHashMap::<PublicKey, SocketAddr, ADDRESS_MAP_SIZE>::new();
    while let Some(cmd) = rx.recv().await {
        let span = info_span!("received command", ?cmd).entered();
        match cmd {
            Command::Ping {
                initiator_key,
                initiator_address,
                target,
                resp,
            } => {
                addresses.insert(initiator_key, initiator_address);
                let _ = resp.send(match addresses.remove(&target) {
                    // TODO: Only reveal socket address to valid initiators?
                    Some(a) => Ok(Ping(a)),
                    None => Err(Error::NoData),
                });
            }
        }
        drop(span);
    }
}
