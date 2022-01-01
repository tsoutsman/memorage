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

#[derive(Copy, Clone, Debug, Eq)]
pub struct HashablePublicKey(pub PublicKey);

// TODO https://github.com/dalek-cryptography/ed25519-dalek/issues/52
impl std::hash::Hash for HashablePublicKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.as_ref().hash(state);
    }
}

// TODO https://github.com/dalek-cryptography/ed25519-dalek/issues/52
impl std::cmp::PartialEq for HashablePublicKey {
    fn eq(&self, other: &Self) -> bool {
        self.0.as_ref() == other.0.as_ref()
    }
}

impl From<PublicKey> for HashablePublicKey {
    fn from(k: PublicKey) -> Self {
        Self(k)
    }
}

impl From<HashablePublicKey> for PublicKey {
    fn from(k: HashablePublicKey) -> Self {
        k.0
    }
}

pub async fn manager(mut rx: mpsc::Receiver<Command>) {
    let mut sockets: bimap::BiMap<HashablePublicKey, SocketAddr> = bimap::BiMap::new();
    let mut requests: bimap::BiMap<HashablePublicKey, HashablePublicKey> = bimap::BiMap::new();

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
                sockets.insert(initiator_key.into(), initiator_address);
                requests.insert(initiator_key.into(), target_key.into());
                // TODO do we even need resp
                let _ = resp.send(());
            }
            Command::CheckConnection {
                target_key,
                target_address,
                resp,
            } => {
                // TODO only accept connections from trusted keys
                let result = match requests.get_by_right(&target_key.into()) {
                    Some(initiator_key) => {
                        // TODO add socket here or in both branches?
                        sockets.insert(target_key.into(), target_address);
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
