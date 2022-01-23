use std::net::{IpAddr, SocketAddr};

use crate::{Config, Error, Result};

use quinn::{Endpoint, EndpointConfig, Incoming};
use soter_core::PublicKey;
use soter_cs::{
    request::{self, Request},
    PairingCode,
};
use tokio::net::UdpSocket;
use tracing::info;

#[derive(Debug)]
pub struct Client<'a> {
    config: &'a Config,
    send_config: quinn::ClientConfig,
    public_address: IpAddr,
    endpoint: Endpoint,
    incoming: Incoming,
}

impl<'a> Client<'a> {
    pub async fn new(config: &'a Config) -> Result<Client<'a>> {
        let mut socket = UdpSocket::bind("0.0.0.0:0").await?;
        let public_address =
            soter_stun::public_address(&mut socket, soter_stun::DEFAULT_STUN_SERVER).await?;
        info!(%public_address, "received public address");
        let public_address = public_address.ip();

        let (send_config, recv_config) =
            soter_cert::gen_configs(public_address, &config.key_pair, None)?;

        let (endpoint, incoming) = quinn::Endpoint::new(
            EndpointConfig::default(),
            Some(recv_config),
            socket.into_std()?,
        )?;

        Ok(Self {
            config,
            send_config,
            public_address,
            endpoint,
            incoming,
        })
    }

    pub async fn register(&self) -> Result<PairingCode> {
        Ok(self.request(request::Register).await?.0)
    }

    pub async fn get_key(&self, code: PairingCode) -> Result<PublicKey> {
        Ok(self.request(request::GetKey(code)).await?.0)
    }

    pub async fn request<T>(&self, request: T) -> Result<T::Response>
    where
        T: soter_cs::Serialize + Request,
    {
        let (mut send, recv) = self
            .endpoint
            .connect_with(
                self.send_config.clone(),
                self.config.server_socket_address(),
                "ooga.com",
            )?
            .await?
            .connection
            .open_bi()
            .await?;

        let encoded = soter_cs::serialize(request)?;
        send.write_all(&encoded).await?;
        send.finish().await?;

        let buffer = recv.read_to_end(1024).await?;
        soter_cs::deserialize::<_, soter_cs::Result<T::Response>>(&buffer)?.map_err(|e| e.into())
    }

    /// Establish a connection to a peer.
    #[allow(clippy::missing_panics_doc)]
    pub async fn establish_peer_connection(self) -> Result<PeerConnection> {
        let target_key = match self.config.peer {
            Some(k) => k,
            None => return Err(Error::PeerNotSet),
        };
        self.request(request::RequestConnection(target_key)).await?;

        loop {
            tokio::time::sleep(self.config.server_ping_delay).await;
            if let Some(peer_address) = self.request(request::Ping).await?.0 {
                // We have to use the same endpoint for
                let (send_config, recv_config) = soter_cert::gen_configs(
                    self.public_address,
                    &self.config.key_pair,
                    Some(target_key),
                )?;
                self.endpoint.set_server_config(Some(recv_config));
                return Ok(PeerConnection {
                    send_config,
                    peer_address,
                    endpoint: self.endpoint,
                    incoming: self.incoming,
                });
            }
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct PeerConnection {
    send_config: quinn::ClientConfig,
    peer_address: SocketAddr,
    endpoint: Endpoint,
    incoming: Incoming,
}
