use crate::{collections::MaxSizeHashMap, CODE_MAP_SIZE};

use memorage_core::PublicKey;
use memorage_cs::{
    response::{GetKey, GetRegisterResponse, Register},
    Error, PairingCode, Result,
};

use tokio::sync::{mpsc, oneshot};
use tracing::{info, info_span};

#[derive(Debug)]
pub enum Command {
    Register {
        key: PublicKey,
        resp: oneshot::Sender<Register>,
    },
    GetKey {
        code: PairingCode,
        requestor: PublicKey,
        resp: oneshot::Sender<Result<GetKey>>,
    },
    GetRegisterResponse {
        initiator: PublicKey,
        resp: oneshot::Sender<Result<GetRegisterResponse>>,
    },
}

#[tracing::instrument]
pub async fn manager(mut rx: mpsc::Receiver<Command>) {
    let mut code_map = MaxSizeHashMap::<_, _, CODE_MAP_SIZE>::new();
    let mut requestor_map = MaxSizeHashMap::<_, _, CODE_MAP_SIZE>::new();

    while let Some(cmd) = rx.recv().await {
        let span = info_span!("received command", ?cmd).entered();
        match cmd {
            Command::Register { key, resp } => {
                let mut code = PairingCode::new();

                // TODO: Exploitable?
                while code_map.contains_key(&code) {
                    code = PairingCode::new()
                }
                code_map.insert(code.clone(), key);

                info!(?code, "generated pairing code");
                let _ = resp.send(Register(code));
            }
            Command::GetKey {
                code,
                resp,
                requestor,
            } => {
                let _ = resp.send(match code_map.remove(&code) {
                    Some(initiator) => {
                        requestor_map.insert(initiator, requestor);
                        Ok(GetKey(initiator))
                    }
                    None => Err(Error::NoData),
                });
            }
            Command::GetRegisterResponse { initiator, resp } => {
                let _ = resp.send(match requestor_map.remove(&initiator) {
                    Some(requestor) => Ok(GetRegisterResponse(requestor)),
                    None => Err(Error::NoData),
                });
            }
        }
        drop(span);
    }
}
