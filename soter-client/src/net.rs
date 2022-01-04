use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
};

use crate::{Config, Result};

use soter_core::{KeyPair, PublicKey};
use soter_cs::request::{self, Request};
use tracing::info;

pub async fn establish_connection<'a>(
    client: &'a Client,
    target_key: &PublicKey,
    config: &Config,
) -> Result<PeerConnection<'a>> {
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
pub struct Client {
    key_pair: Arc<KeyPair>,
    public_address: IpAddr,
    endpoint: quinn::Endpoint,
    #[allow(dead_code)]
    incoming: quinn::Incoming,
}

impl Client {
    // TODO: Take ref instead of Arc
    pub async fn new(key_pair: Arc<KeyPair>) -> Result<Self> {
        let public_address = soter_stun::public_address(soter_stun::DEFAULT_STUN_SERVER).await?;
        info!(%public_address, "received public address");
        let public_address = public_address.ip();
        let server_config = soter_cert::gen_recv_config(public_address, &key_pair)?;
        let (endpoint, incoming) = quinn::Endpoint::server(
            server_config,
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), soter_core::PORT),
        )?;
        Ok(Self {
            key_pair,
            public_address,
            endpoint,
            incoming,
        })
    }

    pub fn public_key(&self) -> PublicKey {
        self.key_pair.public_key()
    }

    pub async fn server_connection<'a>(
        &'_ self,
        address: SocketAddr,
    ) -> Result<ServerConnection<'_>> {
        let config = soter_cert::gen_send_config(self.public_address, &self.key_pair, None)?;
        let endpoint = &self.endpoint;
        Ok(ServerConnection {
            address,
            endpoint,
            config,
        })
    }

    pub async fn peer_connection<'a>(
        &'_ self,
        address: SocketAddr,
        target_key: &PublicKey,
    ) -> Result<PeerConnection<'_>> {
        let config =
            soter_cert::gen_send_config(self.public_address, &self.key_pair, Some(target_key))?;
        let endpoint = &self.endpoint;
        Ok(PeerConnection {
            address,
            endpoint,
            config,
        })
    }
}

#[derive(Debug)]
pub struct ServerConnection<'a> {
    address: SocketAddr,
    config: quinn::ClientConfig,
    endpoint: &'a quinn::Endpoint,
}

impl<'a> ServerConnection<'a> {
    pub async fn request<T>(&self, request: T) -> Result<T::Response>
    where
        T: soter_cs::Serialize + Request,
    {
        let (mut send, recv) = self
            .endpoint
            // I don't think there is a way around this clone
            .connect_with(self.config.clone(), self.address, "ooga.com")?
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
pub struct PeerConnection<'a> {
    address: SocketAddr,
    config: quinn::ClientConfig,
    endpoint: &'a quinn::Endpoint,
}
