use crate::{
    fs::{
        index::{Index, IndexDifference},
        read_bin, write_bin, EncryptedFile, EncryptedPath,
    },
    net::protocol::{
        self,
        request::{self, RequestType},
        response,
    },
    persistent::{config::Config, data::Data},
    Error, Result,
};

use std::{collections::HashMap, path::PathBuf};

use quinn::{NewConnection, SendStream};
use tokio::net::UdpSocket;
use tracing::debug;

#[derive(Debug)]
pub struct PeerConnection<'a, 'b> {
    pub(super) data: &'a Data,
    pub(super) config: &'b Config,
    pub(super) connection: NewConnection,
    #[allow(dead_code)]
    pub(super) socket: UdpSocket,
}

impl<'a, 'b> PeerConnection<'a, 'b> {
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
                        contents: EncryptedFile::from_unencrypted(
                            &self.data.key_pair.private,
                            unencrypted_paths.get(&name).unwrap(),
                        )?,
                        name,
                    })
                    .await?;
                }
                IndexDifference::Edit(name) => {
                    self.send(request::Edit {
                        contents: EncryptedFile::from_unencrypted(
                            &self.data.key_pair.private,
                            unencrypted_paths.get(&name).unwrap(),
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
        let (send, recv) = self.connection.connection.open_bi().await?;
        send_with_stream(send, request).await?;

        // TODO: Not large enough.
        let buffer = recv.read_to_end(1024).await?;
        let response = protocol::deserialize::<_, protocol::Result<T::Response>>(&buffer)?
            .map_err(|e| e.into());
        debug!(?response, "received response");
        response
    }

    pub async fn receive_and_handle(&mut self) -> Result<Index> {
        loop {
            let (send, request) = self.receive().await?;
            match request {
                RequestType::Ping(_) => {
                    send_with_stream(send, Ok(response::Ping)).await?;
                }
                RequestType::GetIndex(_) => {
                    let response: crate::Result<_> = try {
                        let index = read_bin(self.config.index_path())?;
                        response::GetIndex(index)
                    };
                    send_with_stream(send, response.map_err(|e| e.into())).await?;
                }
                RequestType::Add(request::Add { name, contents }) => {
                    let response: crate::Result<_> = try {
                        write_bin(self.config.peer_file_path(&name), &contents)?;
                        response::Add
                    };
                    send_with_stream(send, response.map_err(|e| e.into())).await?;
                }
                RequestType::Edit(request::Edit { name, contents }) => {
                    let response: crate::Result<_> = try {
                        write_bin(self.config.peer_file_path(&name), &contents)?;
                        response::Edit
                    };
                    send_with_stream(send, response.map_err(|e| e.into())).await?;
                }
                RequestType::Rename(request::Rename { from, to }) => {
                    let response: crate::Result<_> = try {
                        std::fs::rename(
                            self.config.peer_file_path(&from),
                            self.config.peer_file_path(&to),
                        )?;
                        response::Rename
                    };
                    send_with_stream(send, response.map_err(|e| e.into())).await?;
                }
                RequestType::Delete(request::Delete(path)) => {
                    let response: crate::Result<_> = try {
                        std::fs::remove_file(self.config.peer_file_path(&path))?;
                        response::Delete
                    };
                    send_with_stream(send, response.map_err(|e| e.into())).await?;
                }
                RequestType::SetIndex(request::SetIndex(index)) => {
                    let response: crate::Result<_> = try {
                        write_bin(self.config.index_path(), &index)?;
                        response::SetIndex
                    };
                    send_with_stream(send, response.map_err(|e| e.into())).await?;
                }
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

async fn send_with_stream<T>(mut send: SendStream, request: T) -> Result<()>
where
    T: protocol::Serialize + std::fmt::Debug,
{
    debug!(?request, "sending request");

    let encoded = protocol::serialize(request)?;
    send.write_all(&encoded).await?;
    send.finish().await?;

    Ok(())
}
