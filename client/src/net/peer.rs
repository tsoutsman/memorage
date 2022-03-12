use crate::{
    crypto::Encrypted,
    fs::{encrypt_file, read_bin, write_bin, Index, IndexDifference},
    net::protocol::{
        self,
        request::{self, RequestType},
        response,
    },
    persistent::{config::Config, data::Data},
    Error, Result,
};

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

    #[allow(clippy::missing_panics_doc)]
    pub async fn send_difference(&self, difference: Vec<IndexDifference>) -> Result<()> {
        // According to recv stream documentation, read_chunks cannot be used
        // for framing implying that new connections must be created for
        // framing.
        // TODO: Enforce unencrypted_paths containing path in type system.
        for d in difference {
            match d {
                IndexDifference::Add(name) => {
                    self.send(request::Add {
                        contents: encrypt_file(&self.data.key_pair.private, &name)?,
                        name: name.into(),
                    })
                    .await?;
                }
                IndexDifference::Edit(name) => {
                    self.send(request::Edit {
                        contents: encrypt_file(&self.data.key_pair.private, &name)?,
                        name: name.into(),
                    })
                    .await?;
                }
                IndexDifference::Rename { from, to } => {
                    self.send(request::Rename {
                        from: from.into(),

                        to: to.into(),
                    })
                    .await?;
                }
                IndexDifference::Delete(name) => {
                    self.send(request::Delete(name.into())).await?;
                }
            }
        }
        Ok(())
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

    pub async fn receive_and_handle(&mut self) -> Result<Index> {
        loop {
            let (send, request) = self.receive().await?;
            match request {
                RequestType::Ping(_) => {
                    send_with_stream(send, Ok(response::Ping)).await?;
                }
                RequestType::GetIndex(_) => {
                    let response: crate::Result<_> = try {
                        let index = match read_bin(self.config.index_path()) {
                            Ok(i) => i,
                            Err(e) => match e {
                                Error::Io(ref ee) => match ee.kind() {
                                    std::io::ErrorKind::NotFound => Index::new(),
                                    _ => return Err(e),
                                },
                                _ => return Err(e),
                            },
                        };
                        response::GetIndex(Encrypted::encrypt(&self.data.key_pair.private, &index)?)
                    };
                    send_with_stream(send, response.map_err(|e| e.into())).await?;
                }
                RequestType::Add(request::Add { name, contents }) => {
                    let response: crate::Result<_> = try {
                        write_bin(
                            self.config.peer_storage_path.file_path(&name)?,
                            &contents,
                            false,
                        )?;
                        response::Add
                    };
                    send_with_stream(send, response.map_err(|e| e.into())).await?;
                }
                RequestType::Edit(request::Edit { name, contents }) => {
                    let response: crate::Result<_> = try {
                        write_bin(
                            self.config.peer_storage_path.file_path(&name)?,
                            &contents,
                            true,
                        )?;
                        response::Edit
                    };
                    send_with_stream(send, response.map_err(|e| e.into())).await?;
                }
                RequestType::Rename(request::Rename { from, to }) => {
                    let response: crate::Result<_> = try {
                        std::fs::rename(
                            self.config.peer_storage_path.file_path(&from)?,
                            self.config.peer_storage_path.file_path(&to)?,
                        )?;
                        response::Rename
                    };
                    send_with_stream(send, response.map_err(|e| e.into())).await?;
                }
                RequestType::Delete(request::Delete(path)) => {
                    let response: crate::Result<_> = try {
                        std::fs::remove_file(self.config.peer_storage_path.file_path(&path)?)?;
                        response::Delete
                    };
                    send_with_stream(send, response.map_err(|e| e.into())).await?;
                }
                RequestType::SetIndex(request::SetIndex(index)) => {
                    let response: crate::Result<_> = try {
                        write_bin(self.config.index_path(), &index, true)?;
                        response::SetIndex
                    };
                    send_with_stream(send, response.map_err(|e| e.into())).await?;
                }
                RequestType::Complete(request::Complete(index)) => {
                    return index.decrypt(&self.data.key_pair.private)
                }
            }
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
