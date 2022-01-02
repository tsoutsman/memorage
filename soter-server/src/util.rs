use soter_core::{PublicKey, Verifiable};
use soter_cs::{Error, SigningBytes};
use tokio::sync::{mpsc, oneshot};

pub async fn signing_bytes(
    sign_tx: mpsc::Sender<oneshot::Sender<SigningBytes>>,
) -> soter_cs::Result<SigningBytes> {
    let (resp_tx, resp_rx) = oneshot::channel();
    sign_tx
        .send(resp_tx)
        .await
        .map_err(|_| soter_cs::Error::Generic)?;
    resp_rx.await.map_err(|_| soter_cs::Error::Generic)
}

pub async fn verify_key(
    key: Verifiable<PublicKey>,
    sign_tx: mpsc::Sender<oneshot::Sender<SigningBytes>>,
) -> soter_cs::Result<PublicKey> {
    key.into_key(&signing_bytes(sign_tx).await?)
        .map_err(|_| Error::Generic)
}
