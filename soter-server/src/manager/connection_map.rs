use std::net::SocketAddr;

use soter_core::PublicKey;
use tokio::sync::{mpsc, oneshot};

#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum Command {
    RequestConnection {
        initiator_key: PublicKey,
        initiator_address: SocketAddr,
        target_key: PublicKey,
        resp: oneshot::Sender<()>,
    },
    CheckConnection {
        target_key: PublicKey,
        target_address: SocketAddr,
        // TODO also initiator_key
        resp: oneshot::Sender<Option<SocketAddr>>,
    },
    Ping {
        initiator_address: SocketAddr,
        resp: oneshot::Sender<Option<SocketAddr>>,
    },
}

pub async fn manager(mut rx: mpsc::Receiver<Command>) {
    let mut sockets: bimap::BiMap<PublicKey, SocketAddr> = bimap::BiMap::new();
    let mut requests: bimap::BiMap<PublicKey, PublicKey> = bimap::BiMap::new();

    while let Some(cmd) = rx.recv().await {
        match cmd {
            Command::RequestConnection {
                initiator_key,
                initiator_address,
                target_key,
                resp,
            } => {
                // TODO multiple people want to connect to same target (hashmap value is vec)
                // TODO only connections approved by initiator
                sockets.insert(initiator_key, initiator_address);
                requests.insert(initiator_key, target_key);
                // TODO do we even need resp
                let _ = resp.send(());
            }
            Command::CheckConnection {
                target_key,
                target_address,
                resp,
            } => {
                // TODO only accept connections from trusted keys
                let result = match requests.get_by_right(&target_key) {
                    Some(initiator_key) => {
                        // TODO add socket here or in both branches?
                        sockets.insert(target_key, target_address);
                        sockets.get_by_left(&(*initiator_key)).cloned()
                    }
                    None => None,
                };
                let _ = resp.send(result);
            }
            Command::Ping {
                initiator_address,
                resp,
            } => {
                let result = match sockets.get_by_right(&initiator_address) {
                    Some(initiator_key) => match requests.get_by_left(initiator_key) {
                        // TODO delete request?
                        Some(target_key) => sockets.get_by_left(target_key).cloned(),
                        None => None,
                    },
                    None => None,
                };
                let _ = resp.send(result);
            }
        }
    }
}
