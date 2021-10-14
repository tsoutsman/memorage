use std::net::SocketAddr;

use hashbrown::HashMap;
use lib::cs::PublicKey;
use tokio::sync::{mpsc, oneshot};

#[allow(clippy::large_enum_variant)]
#[allow(dead_code)]
pub enum Command {
    RequestConnection {
        initiator: PublicKey,
        target: PublicKey,
        resp: oneshot::Sender<Option<String>>,
    },
    CheckConnection {
        // TODO
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

pub async fn connection_map_manager(mut rx: mpsc::Receiver<Command>) {
    let _map: HashMap<HashablePublicKey, SocketAddr> = HashMap::new();

    while let Some(cmd) = rx.recv().await {
        match cmd {
            #[allow(unused_variables)]
            Command::RequestConnection {
                initiator,
                target,
                resp,
            } => {
                //
            }
            Command::CheckConnection {} => {
                todo!();
            }
        }
    }
}
