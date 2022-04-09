use crate::{
    fs::index::Index,
    net::protocol::{self, request},
    persistent::{config::Config, data::Data},
    Result,
};

use quinn::{NewConnection, RecvStream, SendStream};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UdpSocket,
};
use tracing::debug;

mod receieve;
mod retrieve;
mod send;

#[derive(Debug)]
pub struct PeerConnection<'a, 'b> {
    pub(super) data: &'a Data,
    pub(super) config: &'b Config,
    pub(super) connection: NewConnection,
    #[allow(dead_code)]
    pub(super) socket: UdpSocket,
}

impl<'a, 'b> PeerConnection<'a, 'b> {
    pub async fn get_index(&self) -> Result<Index> {
        Ok(match self.send_request(&request::GetIndex).await?.0.index {
            Some(i) => i.decrypt(&self.data.key_pair.private)?,
            None => Index::new(),
        })
    }
}

impl<'a, 'b> PeerConnection<'a, 'b> {
    async fn send_request<T>(&self, request: &T) -> Result<(T::Response, (SendStream, RecvStream))>
    where
        T: protocol::Serialize + request::Request + std::fmt::Debug,
    {
        let (mut send, mut recv) = self.send_request_without_response(request).await?;
        send.finish().await?;
        let response = receive_from_stream::<protocol::Result<_>>(&mut recv).await??;
        debug!(?response, "received response");
        Ok((response, (send, recv)))
    }

    async fn send_request_without_response<T>(
        &self,
        request: &T,
    ) -> Result<(SendStream, RecvStream)>
    where
        T: protocol::Serialize + request::Request + std::fmt::Debug,
    {
        debug!(?request, "sending request");
        let (mut send, recv) = self.connection.connection.open_bi().await?;
        send_with_stream(&mut send, request).await?;
        Ok((send, recv))
    }
}

async fn send_with_stream<T>(send: &mut SendStream, packet: &T) -> Result<()>
where
    T: protocol::Serialize,
{
    let encoded = protocol::serialize(packet)?;

    debug!("encoded length: {}", encoded.len());
    send.write_u16(encoded.len() as u16).await?;
    send.write_all(&encoded).await?;

    Ok(())
}

async fn receive_from_stream<T>(recv: &mut RecvStream) -> Result<T>
where
    T: protocol::Deserialize,
{
    // TODO: Make sure not too long.
    let request_length = usize::from(recv.read_u16().await?);
    let mut buf = vec![0; request_length];
    recv.read_exact(&mut buf).await?;
    protocol::deserialize::<_, T>(&buf).map_err(|e| e.into())
}
