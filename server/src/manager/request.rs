use crate::{collections::MaxSizeHashMap, REQUEST_MAP_SIZE};

use memorage_core::{time::OffsetDateTime, PublicKey};
use memorage_cs::{
    response::{CheckConnection, RequestConnection},
    Error, Result,
};

use tokio::sync::{mpsc, oneshot};
use tracing::info_span;

#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum Command {
    RequestConnection {
        initiator: PublicKey,
        target: PublicKey,
        time: OffsetDateTime,
        resp: oneshot::Sender<RequestConnection>,
    },
    CheckConnection {
        target: PublicKey,
        resp: oneshot::Sender<Result<CheckConnection>>,
    },
}

#[tracing::instrument]
pub async fn manager(mut rx: mpsc::Receiver<Command>) {
    let mut requests = MaxSizeHashMap::<PublicKey, CheckConnection, REQUEST_MAP_SIZE>::new();

    while let Some(cmd) = rx.recv().await {
        let span = info_span!("received command", ?cmd).entered();
        match cmd {
            Command::RequestConnection {
                initiator,
                target,
                time,
                resp,
            } => {
                // TODO multiple people want to connect to same target (hashmap value is vec)
                requests.insert(target, CheckConnection { initiator, time });
                // TODO do we even need resp
                let _ = resp.send(RequestConnection);
            }
            Command::CheckConnection { target, resp } => {
                let _ = resp.send(requests.remove(&target).ok_or(Error::NoData));
            }
        }
        drop(span);
    }
}
