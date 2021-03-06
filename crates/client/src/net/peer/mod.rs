use crate::{net::protocol, Error, Result};

use memorage_core::time::OffsetDateTime;
use quinn::{RecvStream, SendStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::trace;

mod incoming;
mod outgoing;
mod stream;

pub use incoming::IncomingConnection;
pub use outgoing::OutgoingConnection;

pub async fn sleep_till(time: OffsetDateTime) -> Result<()> {
    let delay = time - OffsetDateTime::now_utc();
    tracing::info!(%time, %delay, "waiting for synchronisation");
    tokio::time::sleep(delay.try_into().map_err(|_| Error::MissedSynchronisation)?).await;
    Ok(())
}

async fn send_packet<T>(send: &mut SendStream, packet: &T) -> Result<()>
where
    T: protocol::Serialize + std::fmt::Debug,
{
    let encoded = protocol::serialize(packet)?;
    trace!(
        length = ?encoded.len(),
        ?packet,
        "sending packet"
    );
    send.write_u16(encoded.len() as u16).await?;
    send.write_all(&encoded).await?;
    Ok(())
}

async fn receive_packet<T>(recv: &mut RecvStream) -> Result<T>
where
    T: protocol::Deserialize + std::fmt::Debug,
{
    // TODO: Make sure not too long.
    let length = usize::from(recv.read_u16().await?);
    let mut buf = vec![0; length];
    recv.read_exact(&mut buf).await?;
    let packet = protocol::deserialize::<_, T>(&buf).map_err(|e| e.into());
    trace!(
        ?length,
        ?packet,
        expected = std::any::type_name::<T>(),
        "incoming packet"
    );
    packet
}
