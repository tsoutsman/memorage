use std::net::SocketAddr;

use memorage_core::PublicKey;
use tokio::sync::{mpsc, oneshot};
use tracing::{info, info_span, warn};

use crate::{hash_map::MaxSizeBiMap, REQUEST_MAP_SIZE, SOCKET_MAP_SIZE};

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

#[tracing::instrument]
pub async fn manager(mut rx: mpsc::Receiver<Command>) {
    let mut sockets = MaxSizeBiMap::<PublicKey, SocketAddr, SOCKET_MAP_SIZE>::new();
    let mut requests = MaxSizeBiMap::<PublicKey, PublicKey, REQUEST_MAP_SIZE>::new();

    while let Some(cmd) = rx.recv().await {
        let span = info_span!("received command", ?cmd).entered();
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
                        let initiator_socket = sockets.get_by_left(&(*initiator_key)).cloned();
                        info!(?initiator_key, ?initiator_socket, "check connection some");
                        initiator_socket
                    }
                    None => {
                        info!("check connection none");
                        None
                    }
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
                        Some(target_key) => {
                            info!(?target_key, "ping result");
                            sockets.get_by_left(target_key).cloned()
                        }
                        None => None,
                    },
                    None => {
                        // TODO return err?
                        warn!("ping from unknown address");
                        None
                    }
                };
                let _ = resp.send(result);
            }
        }
        drop(span);
    }
}
