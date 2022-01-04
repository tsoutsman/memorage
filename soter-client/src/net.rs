use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
};

use crate::{Config, Result};

use soter_core::{KeyPair, PublicKey};
use soter_cs::request::{self, Request};
use tracing::info;

pub async fn establish_connection(
    key_pair: Arc<KeyPair>,
    target_key: Arc<PublicKey>,
    config: &Config,
) -> Result<PeerConnection> {
    let client = Client::new(key_pair).await?;
    let server_address = SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(172, 105, 176, 37)),
        soter_core::PORT,
    );
    let server = client.server_connection(server_address).await?;

    server
        .request(request::RequestConnection(*target_key))
        .await?;

    loop {
        tokio::time::sleep(config.server_ping_delay).await;
        if let Some(target_address) = server.request(request::Ping).await?.0 {
            return client.peer_connection(target_address, target_key).await;
        }
    }
}

#[derive(Debug)]
struct Client {
    key_pair: Arc<KeyPair>,
    public_address: IpAddr,
}

impl Client {
    // TODO: Take ref instead of Arc
    pub async fn new(key_pair: Arc<KeyPair>) -> Result<Self> {
        let public_address = soter_stun::public_address(soter_stun::DEFAULT_STUN_SERVER).await?;
        info!(%public_address, "received public address");
        let public_address = public_address.ip();
        Ok(Self {
            key_pair,
            public_address,
        })
    }

    pub async fn server_connection(&self, server_address: SocketAddr) -> Result<ServerConnection> {
        let (send_config, recv_config) =
            soter_cert::gen_configs(self.public_address, &self.key_pair, None)?;
        let (endpoint, incoming) = quinn::Endpoint::server(
            recv_config,
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0),
        )?;
        Ok(ServerConnection {
            send_config,
            server_address,
            endpoint,
            incoming,
        })
    }

    pub async fn peer_connection(
        &self,
        peer_address: SocketAddr,
        target_key: Arc<PublicKey>,
    ) -> Result<PeerConnection> {
        let (send_config, recv_config) =
            soter_cert::gen_configs(self.public_address, &self.key_pair, Some(target_key))?;
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
            // I don't think there is a way around this clone
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
