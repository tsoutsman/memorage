use hashbrown::HashMap;
use soter_core::PublicKey;
use soter_cs::PairingCode;
use tokio::sync::{mpsc, oneshot};

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

pub async fn manager(mut rx: mpsc::Receiver<Command>) {
    let mut map: HashMap<PairingCode, PublicKey> = HashMap::new();

    while let Some(cmd) = rx.recv().await {
        match cmd {
            Command::Get { code, resp } => {
                let _ = resp.send(map.remove(&code));
            }
            Command::Generate { key, resp } => {
                let mut code = PairingCode::new();

                // TODO exploitable
                while map.contains_key(&code) {
                    code = PairingCode::new()
                }

                map.insert(code.clone(), key);
                let _ = resp.send(code);
            }
        }
    }
}
