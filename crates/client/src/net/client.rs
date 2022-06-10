use crate::{
    net::peer::{IncomingConnection, OutgoingConnection},
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

use futures_util::StreamExt;
use quinn::{Endpoint, EndpointConfig, Incoming, NewConnection};
use tokio::net::UdpSocket;
use tracing::{debug, info, trace, warn};

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

    pub async fn register(&self) -> Result<PairingCode> {
        Ok(self.request(request::Register).await?.0)
    }

    pub async fn register_response(&self) -> Result<PublicKey> {
        let mut counter = 0;
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

    pub async fn get_key(&self, code: PairingCode) -> Result<PublicKey> {
        Ok(self.request(request::GetKey(code)).await?.0)
    }
}

impl<'a, 'b> Client<'a, 'b, Data> {
    // TODO: Add type enforcement to creat_outgoing_connection and
    // receive_incoming_connection.

    /// Establish a connection to a peer.
    pub async fn schedule_outgoing_connection(&self) -> Result<OffsetDateTime> {
        debug!(
            public_key=?self.data.key_pair.public,
            target_key=?self.data.peer,
            "trying to establish connection"
        );
        let target = self.data.peer;
        let time = OffsetDateTime::now_utc() + self.config.peer_connection_schedule_delay;

        self.request(request::RequestConnection { target, time })
            .await?;

        Ok(time)
    }

    pub async fn create_outgoing_connection(self) -> Result<OutgoingConnection<'a, 'b>> {
        let data = self.data;
        let config = self.config;
        let connection = self.connect_to_peer(true).await?.connection;

        Ok(OutgoingConnection {
            data,
            config,
            connection,
        })
    }

    pub async fn check_incoming_connection(&self) -> Result<Option<OffsetDateTime>> {
        debug!(
            public_key=?self.data.key_pair.public,
            target_key=?self.data.peer,
            "checking for peer connections"
        );

        let peer = self.data.peer;
        let response = match self.request(request::CheckConnection).await {
            Ok(r) => r,
            Err(Error::Server(memorage_cs::Error::NoData)) => return Ok(None),
            Err(e) => return Err(e),
        };

        if response.initiator == peer {
            Ok(Some(response.time))
        } else {
            Err(Error::UnauthorisedConnectionRequest)
        }
    }

    pub async fn receive_incoming_connection(self) -> Result<IncomingConnection<'a, 'b>> {
        let data = self.data;
        let config = self.config;
        let bi_streams = self.connect_to_peer(false).await?.bi_streams;

        Ok(IncomingConnection {
            data,
            config,
            bi_streams,
        })
    }

    async fn connect_to_peer(mut self, initiator: bool) -> Result<NewConnection> {
        let peer_key = self.data.peer;

        let mut counter = 0;

        debug!("sending identification ping");

        let _temp = self.request(request::Ping(peer_key)).await;
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;

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
                        trace!(?result, "punching");
                        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    }

                    return if initiator {
                        self.endpoint
                            .connect_with(send_config, peer_address, "ooga.com")?
                            .await
                            .map_err(|e| e.into())
                    } else {
                        self.incoming
                            .next()
                            .await
                            .ok_or(Error::FailedConnection)?
                            .await
                            .map_err(|e| e.into())
                    };
                }
                Err(Error::Server(memorage_cs::Error::NoData)) => {
                    counter += 1;
                    debug!(%counter, "peer address not on server");
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
