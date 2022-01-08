use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use crate::{Config, Result};

use soter_core::PublicKey;
use soter_cs::{
    request::{self, Request},
    PairingCode,
};
use tracing::info;

#[derive(Debug)]
pub struct Client<'a> {
    config: &'a Config,
    public_address: IpAddr,
}

impl Client<'_> {
    async fn server_connection(&self) -> Result<ServerConnection> {
        let (send_config, recv_config) =
            soter_cert::gen_configs(self.public_address, &self.config.key_pair, None)?;
        let (endpoint, incoming) = quinn::Endpoint::server(
            recv_config,
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0),
        )?;
        Ok(ServerConnection {
            send_config,
            server_address: self.config.server_socket_address(),
            endpoint,
            incoming,
        })
    }

    async fn peer_connection(
        &self,
        peer_address: SocketAddr,
        target_key: PublicKey,
    ) -> Result<PeerConnection> {
        let (send_config, recv_config) =
            soter_cert::gen_configs(self.public_address, &self.config.key_pair, Some(target_key))?;
        let (endpoint, incoming) = quinn::Endpoint::server(
            recv_config,
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0),
        )?;
        Ok(PeerConnection {
            send_config,
            peer_address,
            endpoint,
            incoming,
        })
    }
}

impl<'a> Client<'a> {
    pub async fn new(config: &'a Config) -> Result<Client<'a>> {
        let public_address = soter_stun::public_address(soter_stun::DEFAULT_STUN_SERVER).await?;
        info!(%public_address, "received public address");
        let public_address = public_address.ip();
        Ok(Self {
            config,
            public_address,
        })
    }

    pub async fn register(&self) -> Result<PairingCode> {
        let server = self.server_connection().await?;
        Ok(server.request(request::Register).await?.0)
    }

    pub async fn get_key(&self, code: PairingCode) -> Result<PublicKey> {
        let server = self.server_connection().await?;
        Ok(server.request(request::GetKey(code)).await?.0)
    }

    /// Establish a connection to a peer.
    #[allow(clippy::missing_panics_doc)]
    pub async fn establish_peer_connection(&self) -> Result<PeerConnection> {
        let server = self.server_connection().await?;
        // TODO
        let target_key = self.config.peer.unwrap();

        server
            .request(request::RequestConnection(target_key))
            .await?;

        loop {
            tokio::time::sleep(self.config.server_ping_delay).await;
            if let Some(target_address) = server.request(request::Ping).await?.0 {
                return self.peer_connection(target_address, target_key).await;
            }
        }
    }
}

#[derive(Debug)]
pub struct ServerConnection {
    send_config: quinn::ClientConfig,
    server_address: SocketAddr,
    endpoint: quinn::Endpoint,
    #[allow(dead_code)]
    incoming: quinn::Incoming,
}

impl ServerConnection {
    pub async fn request<T>(&self, request: T) -> Result<T::Response>
    where
        T: soter_cs::Serialize + Request,
    {
        let (mut send, recv) = self
            .endpoint
            .connect_with(self.send_config.clone(), self.server_address, "ooga.com")?
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
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct PeerConnection {
    send_config: quinn::ClientConfig,
    peer_address: SocketAddr,
    endpoint: quinn::Endpoint,
    #[allow(dead_code)]
    incoming: quinn::Incoming,
}
