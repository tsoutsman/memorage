use crate::{hash_map::MaxSizeHashMap, CODE_MAP_SIZE};

use soter_core::PublicKey;
use soter_cs::PairingCode;
use tokio::sync::{mpsc, oneshot};
use tracing::{info, info_span};

#[derive(Debug)]
pub enum Command {
    Get {
        code: PairingCode,
        resp: oneshot::Sender<Option<PublicKey>>,
    },
    Generate {
        key: PublicKey,
        resp: oneshot::Sender<PairingCode>,
    },
}

#[tracing::instrument]
pub async fn manager(mut rx: mpsc::Receiver<Command>) {
    let mut map = MaxSizeHashMap::<_, _, CODE_MAP_SIZE>::new();

    while let Some(cmd) = rx.recv().await {
        let span = info_span!("received command", ?cmd).entered();
        match cmd {
            Command::Get { code, resp } => {
                let _ = resp.send(map.remove(&code));
            }
            Command::Generate { key, resp } => {
                let mut code = PairingCode::new();

                // TODO: Exploitable?
                while map.contains_key(&code) {
                    code = PairingCode::new()
                }
                map.insert(code.clone(), key);

                info!(?code, "generated pairing code");
                let _ = resp.send(code);
            }
        }
        drop(span);
    }
}
