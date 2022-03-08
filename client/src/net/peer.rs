use crate::{
    fs::{
        index::{Index, IndexDifference},
        EncryptedFile, EncryptedPath,
    },
    net::protocol::{
        self,
        request::{self, RequestType},
    },
    persistent::data::Data,
    Error, Result,
};

use std::{collections::HashMap, path::PathBuf};

use quinn::{NewConnection, SendStream};
use tokio::net::UdpSocket;
use tracing::debug;

#[derive(Debug)]
pub struct PeerConnection<'a> {
    pub(super) data: &'a Data,
    pub(super) connection: NewConnection,
    #[allow(dead_code)]
    pub(super) socket: UdpSocket,
}

impl<'a> PeerConnection<'a> {
    #[allow(clippy::missing_panics_doc)]
    pub async fn send_difference(
        &self,
        difference: Vec<IndexDifference>,
        unencrypted_paths: HashMap<EncryptedPath, PathBuf>,
    ) -> Result<()> {
        // According to recv stream documentation, read_chunks cannot be used
        // for framing implying that new connections must be created for
        // framing.
        // TODO: Enforce unencrypted_paths containing path in type system.
        for d in difference {
            match d {
                IndexDifference::Add(name) => {
                    self.send(request::Add {
                        contents: EncryptedFile::from_disk(
                            unencrypted_paths.get(&name).unwrap(),
                            &self.data.key_pair.private,
                        )?,
                        name,
                    })
                    .await?;
                }
                IndexDifference::Edit(name) => {
                    self.send(request::Edit {
                        contents: EncryptedFile::from_disk(
                            unencrypted_paths.get(&name).unwrap(),
                            &self.data.key_pair.private,
                        )?,
                        name,
                    })
                    .await?;
                }
                IndexDifference::Rename { from, to } => {
                    self.send(request::Rename { from, to }).await?;
                }
                IndexDifference::Delete(path) => {
                    self.send(request::Delete(path)).await?;
                }
            }
        }
        Ok(())
    }

    pub async fn send<T>(&self, request: T) -> Result<T::Response>
    where
        T: protocol::Serialize + request::Request + std::fmt::Debug,
    {
        debug!(?request, "sending request");

        let (mut send, recv) = self.connection.connection.open_bi().await?;

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

    #[allow(clippy::missing_panics_doc)]
    pub async fn receive_and_handle(&mut self) -> Result<Index> {
        loop {
            let (_send, request) = self.receive().await?;
            match request {
                RequestType::Ping(_) => todo!(),
                RequestType::GetIndex(_) => todo!(),
                RequestType::Add(_) => todo!(),
                RequestType::Edit(_) => todo!(),
                RequestType::Rename(_) => todo!(),
                RequestType::Delete(_) => todo!(),
                RequestType::SetIndex(_) => todo!(),
                RequestType::Complete(request::Complete(index)) => return Ok(index),
            }
        }
    }

    async fn receive(&mut self) -> Result<(SendStream, RequestType)> {
        let quinn::NewConnection {
            ref mut bi_streams, ..
        } = self.connection;
        if let Some(stream) = bi_streams.next().await {
            let (send, recv) = match stream {
                Ok(s) => s,
                Err(quinn::ConnectionError::ApplicationClosed { .. }) => {
                    return Err(Error::PeerClosedConnection)
                }
                Err(e) => return Err(e.into()),
            };
            // TODO: Not large enough.
            let buffer = recv.read_to_end(1024).await?;
            let request = protocol::deserialize::<_, RequestType>(&buffer)?;

            Ok((send, request))
        } else {
            Err(Error::PeerClosedConnection)
        }
    }
}
