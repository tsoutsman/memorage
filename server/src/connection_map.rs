use std::net::SocketAddr;

use hashbrown::HashMap;
use lib::cs::key::PublicKey;
use tokio::sync::{mpsc, oneshot};

#[allow(clippy::large_enum_variant)]
#[allow(dead_code)]
pub enum Command {
    RequestConnection {
        initiator_key: PublicKey,
        initiator_socket: SocketAddr,
        target_key: PublicKey,
        resp: oneshot::Sender<()>,
    },
    CheckConnection {
        target_key: PublicKey,
        target_socket: SocketAddr,
    },
}

#[derive(Clone, Debug, Eq)]
pub struct HashablePublicKey(pub PublicKey);

// TODO https://github.com/dalek-cryptography/ed25519-dalek/issues/52
impl std::hash::Hash for HashablePublicKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.as_bytes().hash(state);
    }
}

// TODO https://github.com/dalek-cryptography/ed25519-dalek/issues/52
impl std::cmp::PartialEq for HashablePublicKey {
    fn eq(&self, other: &Self) -> bool {
        self.0.as_bytes() == other.0.as_bytes()
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

pub async fn connection_map_manager(mut rx: mpsc::Receiver<Command>) {
    let mut sockets: HashMap<HashablePublicKey, SocketAddr> = HashMap::new();
    let mut requests: HashMap<HashablePublicKey, PublicKey> = HashMap::new();

    while let Some(cmd) = rx.recv().await {
        match cmd {
            Command::RequestConnection {
                initiator_key,
                initiator_socket,
                target_key,
                resp,
            } => {
                // TODO multiple people want to connect to same target (hashmap value is vec)
                // TODO only connections approved by initiator
                sockets.insert(initiator_key.into(), initiator_socket);
                requests.insert(target_key.into(), initiator_key);
                let _ = resp.send(());
            }
            #[allow(unused_variables)]
            Command::CheckConnection {
                target_key,
                target_socket,
            } => {
                // TODO only accept connections from trusted keys
            }
        }
    }
}
