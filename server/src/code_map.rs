use hashbrown::HashMap;
use lib::cs::{Code, PublicKey};
use tokio::sync::{mpsc, oneshot};

pub enum Command {
    Get {
        code: Code,
        resp: oneshot::Sender<Option<PublicKey>>,
    },
    Generate {
        key: PublicKey,
        resp: oneshot::Sender<Code>,
    },
}

pub async fn code_map_manager(mut rx: mpsc::Receiver<Command>) {
    let mut map: HashMap<Code, PublicKey> = HashMap::new();

    while let Some(cmd) = rx.recv().await {
        match cmd {
            Command::Get { code, resp } => {
                let _ = resp.send(map.remove(&code));
            }
            Command::Generate { key, resp } => {
                let mut code = Code::new();
                while !map.contains_key(&code) {
                    code = Code::new()
                }

                map.insert(code.clone(), key);
                let _ = resp.send(code);
            }
        }
    }
}
