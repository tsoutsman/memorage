use crate::{
    net::PeerConnection,
    persistent::{
        config::Config,
        data::{Data, KeyPairData},
    },
    Error, Result,
};

use std::net::IpAddr;

use memorage_core::{time::OffsetDateTime, PublicKey};
use memorage_cs::{
    request::{self, Request},
    PairingCode,
};

use quinn::{Endpoint, EndpointConfig, Incoming};
use tokio::net::UdpSocket;
use tracing::{debug, info, warn};

#[derive(Debug)]
pub struct Client<'a, 'b, T>
where
    T: KeyPairData,
{
    data: &'a T,
    config: &'b Config,
    send_config: quinn::ClientConfig,
    public_address: IpAddr,
    endpoint: Endpoint,
    incoming: Incoming,
    socket: UdpSocket,
}

impl<'a, 'b, T> Client<'a, 'b, T>
where
    T: KeyPairData,
{
    pub async fn new(data: &'a T, config: &'b Config) -> Result<Client<'a, 'b, T>> {
        let mut socket = UdpSocket::bind("0.0.0.0:0").await?;
        let public_address =
            memorage_stun::public_address(&mut socket, memorage_stun::DEFAULT_STUN_SERVER).await?;
        info!(%public_address, "received public address");
        let public_address = public_address.ip();

        let (send_config, recv_config) =
            memorage_cert::gen_configs(public_address, data.key_pair(), None)?;

        let socket = socket.into_std()?;
        let cloned_socket = socket.try_clone()?;
        let (endpoint, incoming) =
            quinn::Endpoint::new(EndpointConfig::default(), Some(recv_config), socket)?;

        Ok(Self {
            data,
            config,
            send_config,
            public_address,
            endpoint,
            incoming,
            socket: UdpSocket::from_std(cloned_socket)?,
        })
    }

    pub async fn register(&self) -> Result<PairingCode> {
        Ok(self.request(request::Register).await?.0)
    }

    pub async fn get_key(&self, code: PairingCode) -> Result<PublicKey> {
        Ok(self.request(request::GetKey(code)).await?.0)
    }

    pub async fn register_response(&self) -> Result<PublicKey> {
        let mut counter: usize = 0;
        loop {
            tokio::time::sleep(self.config.register_response.ping_delay).await;

            match self.request(request::GetRegisterResponse).await {
                Ok(pk) => return Ok(pk.0),
                Err(Error::Server(memorage_cs::Error::NoData)) => {
                    counter += 1;
                }
                Err(e) => return Err(e),
            }

            if counter == self.config.register_response.tries {
                return Err(Error::PeerNoResponse);
            }
        }
    }

    async fn request<R>(&self, request: R) -> Result<R::Response>
    where
        R: memorage_cs::Serialize + Request + std::fmt::Debug,
    {
        debug!(?request, "sending request");

        let (mut send, recv) = self
            .endpoint
            .connect_with(
                self.send_config.clone(),
                // TODO: Iterate over all supplied addresses until one connects.
                self.config.server_socket_addresses()[0],
                "ooga.com",
            )?
            .await?
            .connection
            .open_bi()
            .await?;

        let encoded = memorage_cs::serialize(request)?;
        send.write_all(&encoded).await?;
        send.finish().await?;

        let buffer = recv.read_to_end(1024).await?;
        let response = memorage_cs::deserialize::<_, memorage_cs::Result<R::Response>>(&buffer)?
            .map_err(|e| e.into());
        debug!(?response, "received response");
        response
    }
}

impl<'a, 'b> Client<'a, 'b, Data> {
    /// Establish a connection to a peer.
    #[allow(clippy::missing_panics_doc)]
    pub async fn establish_peer_connection(self, initiator: bool) -> Result<PeerConnection<'a>> {
        let target = self.data.peer;
        let time = OffsetDateTime::now_utc() + self.config.peer_connection_schedule_delay;

        self.request(request::RequestConnection { target, time })
            .await?;

        let delay = time - OffsetDateTime::now_utc();
        info!(?delay, "sleeping");
        // TODO: Unwrap fails if delay is negative i.e. time < now
        tokio::time::sleep(delay.try_into().unwrap()).await;

        self.connect_to_peer(initiator).await
    }

    pub async fn check_connection(&self) -> Result<OffsetDateTime> {
        let peer = self.data.peer;

        let response = self.request(request::CheckConnection).await?;

        if response.initiator == peer {
            Ok(response.time)
        } else {
            Err(Error::UnauthorisedConnectionRequest)
        }
    }

    pub async fn connect_to_peer(mut self, initiator: bool) -> Result<PeerConnection<'a>> {
        let peer_key = self.data.peer;

        let mut counter: usize = 0;

        debug!("sending first packet");

        let _temp = self.request(request::Ping(peer_key)).await;
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;

        debug!(?_temp, "looping");

        loop {
            match self.request(request::Ping(peer_key)).await {
                Ok(memorage_cs::response::Ping(peer_address)) => {
                    info!(%peer_address, "received peer address");
                    let (send_config, recv_config) = memorage_cert::gen_configs(
                        self.public_address,
                        &self.data.key_pair,
                        Some(peer_key),
                    )?;
                    self.endpoint.set_server_config(Some(recv_config));

                    self.socket.connect(peer_address).await?;
                    debug!("connected to peer");

                    for _ in 0..10 {
                        let result = self.socket.send(&[15, 96, 13]).await;
                        debug!(?result, "punching");
                        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    }

                    let connection;
                    if initiator {
                        // TODO: Is this needed?
                        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                        connection = self
                            .endpoint
                            .connect_with(send_config, peer_address, "ooga.com")?
                            .await?;
                    } else {
                        connection = self
                            .incoming
                            .next()
                            .await
                            .ok_or(Error::FailedConnection)?
                            .await?;
                    }

                    return Ok(PeerConnection {
                        data: self.data,
                        connection,
                        socket: self.socket,
                    });
                }
                Err(Error::Server(memorage_cs::Error::NoData)) => {
                    counter += 1;
                    info!(%counter, "no data on server");
                }
                Err(e) => {
                    warn!(?e, "server error");
                    return Err(e);
                }
            }

            if counter == self.config.request_connection.tries {
                warn!("no peer response");
                return Err(Error::PeerNoResponse);
            }

            tokio::time::sleep(self.config.request_connection.ping_delay).await;
        }
    }
}
