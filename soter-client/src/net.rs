use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
};

use crate::Result;

use quinn::{RecvStream, SendStream};
use soter_core::{KeyPair, PublicKey};
use soter_cs::request::Request;
use tracing::info;

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
        let public_address = soter_cert::public_address(soter_cert::DEFAULT_STUN_SERVER).await?;
        info!(%public_address, "received public address");
        let public_address = public_address.ip();
        let server_config = soter_cert::gen_server_config(public_address, &key_pair)?;
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

    pub async fn connect_to_server(&self, address: SocketAddr) -> Result<ServerConnection> {
        let (send, recv) = self
            .endpoint
            .connect_with(
                soter_cert::gen_client_config(self.public_address, &self.key_pair, None)?,
                address,
                "ooga.com",
            )?
            .await?
            .connection
            .open_bi()
            .await?;
        Ok(ServerConnection { send, recv })
    }

    pub async fn connect_to_peer(
        &self,
        address: SocketAddr,
        target_key: &PublicKey,
    ) -> Result<PeerConnection> {
        let (send, recv) = self
            .endpoint
            .connect_with(
                soter_cert::gen_client_config(
                    self.public_address,
                    &self.key_pair,
                    Some(target_key),
                )?,
                address,
                "ooga.com",
            )?
            .await?
            .connection
            .open_bi()
            .await?;
        Ok(PeerConnection { send, recv })
    }
}

#[derive(Debug)]
pub struct ServerConnection {
    send: SendStream,
    recv: RecvStream,
}

impl ServerConnection {
    pub async fn request<T>(&mut self, request: T) -> Result<T::Response>
    where
        T: soter_cs::Serialize + Request,
    {
        let encoded = soter_cs::serialize(request)?;
        self.send.write_all(&encoded).await?;
        let mut buffer = vec![0; 1024];
        self.recv.read(&mut buffer[..]).await?;
        soter_cs::deserialize::<_, soter_cs::Result<T::Response>>(&buffer)?.map_err(|e| e.into())
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct PeerConnection {
    send: SendStream,
    recv: RecvStream,
}

// impl PeerConnection {
//     pub async fn request<T>(&mut self, request: T) -> Result<T::Response>
//     where
//         T: soter_p2p::Serialize + Request,
//     {
//         let encoded = soter_cs::serialize(request)?;
//         self.send.write_all(&encoded).await?;
//         let mut buffer = vec![0; 1024];
//         self.recv.read(&mut buffer[..]).await?;
//         soter_cs::deserialize::<_, T::Response>(&buffer).map_err(|e| e.into())
//     }
// }
