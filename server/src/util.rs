use lib::cs::{
    key::{PublicKey, SigningBytes, VerifiablePublicKey},
    protocol::error::Error,
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
    key: VerifiablePublicKey,
    sign_tx: mpsc::Sender<oneshot::Sender<SigningBytes>>,
) -> Result<PublicKey, Error> {
    key.into_key(&signing_bytes(sign_tx).await?)
}

pub fn serialize<T>(o: T) -> Result<Vec<u8>, Error>
where
    T: serde::Serialize,
{
    // TODO unwrap
    Ok(bincode::serialize(&Result::<_, ::lib::cs::protocol::error::Error>::Ok(o)).unwrap())
}
