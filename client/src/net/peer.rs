use crate::net::protocol;

use std::net::SocketAddr;

use quinn::{ClientConfig, Endpoint, Incoming};
use tokio::net::UdpSocket;
use tracing::debug;

#[derive(Debug)]
pub struct PeerConnection {
    pub(super) send_config: ClientConfig,
    pub(super) peer_address: SocketAddr,
    pub(super) endpoint: Endpoint,
    pub(super) incoming: Incoming,
    #[allow(dead_code)]
    pub(super) socket: UdpSocket,
}

impl PeerConnection {
    pub async fn next(&mut self) -> Option<quinn::Connecting> {
        self.incoming.next().await
    }

    pub async fn send<T>(&self, request: T) -> crate::Result<T::Response>
    where
        T: protocol::Serialize + protocol::request::Request + std::fmt::Debug,
    {
        debug!(?request, "sending request");

        let (mut send, recv) = self
            .endpoint
            .connect_with(self.send_config.clone(), self.peer_address, "ooga.com")?
            .await?
            .connection
            .open_bi()
            .await?;

        let encoded = protocol::serialize(request)?;
        send.write_all(&encoded).await?;
        send.finish().await?;

        // TODO: Not large enough.
        let buffer = recv.read_to_end(1024).await?;
        let response = protocol::deserialize::<_, protocol::Result<T::Response>>(&buffer)?
            .map_err(|e| e.into());
        debug!(?response, "received response");
        response
    }
}
