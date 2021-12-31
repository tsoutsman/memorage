use lib::{
    cs::{
        key::{PublicKey, SigningBytes},
        protocol::error::Error,
    },
    Verifiable,
};
use tokio::sync::{mpsc, oneshot};

pub async fn signing_bytes(
    sign_tx: mpsc::Sender<oneshot::Sender<SigningBytes>>,
) -> Result<SigningBytes, Error> {
    let (resp_tx, resp_rx) = oneshot::channel();
    sign_tx.send(resp_tx).await.map_err(|_| Error::Generic)?;
    resp_rx.await.map_err(|_| Error::Generic)
}

pub async fn verify_key(
    key: Verifiable<PublicKey>,
    sign_tx: mpsc::Sender<oneshot::Sender<SigningBytes>>,
) -> Result<PublicKey, Error> {
    key.into_verifier(&signing_bytes(sign_tx).await?)
        .map_err(|e| e.into())
}

pub fn serialize<T>(o: T) -> Result<Vec<u8>, Error>
where
    T: serde::Serialize,
{
    // TODO unwrap
    Ok(lib::bincode::serialize(&o).unwrap())
}
