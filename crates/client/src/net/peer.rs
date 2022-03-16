use crate::{
    crypto::Encrypted,
    fs::index::{Index, IndexDifference},
    net::protocol::{
        self,
        request::{self, RequestType},
        response, FILE_FRAME_SIZE,
    },
    persistent::{config::Config, data::Data},
    Error, Result,
};

use chacha20poly1305::{
    aead::{AeadInPlace, NewAead},
    Tag, XChaCha20Poly1305, XNonce,
};
use quinn::{NewConnection, RecvStream, SendStream};
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
    net::UdpSocket,
};
use tracing::{debug, info};

#[derive(Debug)]
pub struct PeerConnection<'a, 'b> {
    pub(super) data: &'a Data,
    pub(super) config: &'b Config,
    pub(super) connection: NewConnection,
    #[allow(dead_code)]
    pub(super) socket: UdpSocket,
}

impl<'a, 'b> PeerConnection<'a, 'b> {
    async fn send_request<T>(&self, request: &T) -> Result<(T::Response, RecvStream)>
    where
        T: protocol::Serialize + request::Request + std::fmt::Debug,
    {
        debug!(?request, "sending request");
        let (mut send, mut recv) = self.connection.connection.open_bi().await?;
        send_with_stream(&mut send, request).await?;

        let response = receive_from_stream::<protocol::Result<_>>(&mut recv).await??;
        debug!(?response, "received response");
        Ok((response, recv))
    }

    async fn accept_stream(&mut self) -> Result<(SendStream, RecvStream)> {
        let quinn::NewConnection {
            ref mut bi_streams, ..
        } = self.connection;
        if let Some(stream) = bi_streams.next().await {
            let (send, recv) = match stream {
                Ok(s) => s,
                Err(quinn::ConnectionError::ApplicationClosed { .. }) => {
                    return Err(Error::PeerClosedConnection);
                }
                Err(e) => return Err(e.into()),
            };
            Ok((send, recv))
        } else {
            Err(Error::PeerClosedConnection)
        }
    }
}

impl<'a, 'b> PeerConnection<'a, 'b> {
    pub async fn get_index(&self) -> Result<Index> {
        Ok(match self.send_request(&request::GetIndex).await?.0.index {
            Some(i) => i.decrypt(&self.data.key_pair.private)?,
            None => Index::new(),
        })
    }

    pub async fn send_backup_data(
        &self,
        old_index: &Index,
        new_index: &Index,
        initiator: bool,
    ) -> Result<()> {
        let difference = new_index.difference(old_index);
        for d in difference {
            info!(diff=?d, "sending difference");
            match d {
                IndexDifference::Write(name) => {
                    self.send_request(&request::Write {
                        contents_len: std::fs::metadata(&name)?.len(),
                        name: name.into(),
                    })
                    .await?;
                    todo!();
                }
                IndexDifference::Rename { from, to } => {
                    self.send_request(&request::Rename {
                        from: from.into(),
                        to: to.into(),
                    })
                    .await?;
                }
                IndexDifference::Delete(name) => {
                    self.send_request(&request::Delete { name: name.into() })
                        .await?;
                }
            }
        }

        self.send_request(&request::SetIndex {
            index: Encrypted::encrypt(&self.data.key_pair.private, new_index)?,
        })
        .await?;

        self.send_request(&request::Complete {
            index: if initiator {
                Index::from_disk_encrypted(&self.data.key_pair.private, self.config.index_path())
                    .await?
            } else {
                // The index from request::Complete isn't used by the initiator
                // of the sync.
                None
            },
        })
        .await?;

        Ok(())
    }

    pub async fn receive_backup_data(&mut self) -> Result<Index> {
        loop {
            let (mut send, mut recv) = self.accept_stream().await?;

            let request = receive_from_stream(&mut recv).await?;

            match request {
                RequestType::Ping(_) => send_with_stream(&mut send, &Ok(response::Ping)).await?,
                RequestType::GetIndex(_) => {
                    let response: crate::Result<_> = try {
                        response::GetIndex {
                            index: Index::from_disk_encrypted(
                                &self.data.key_pair.private,
                                self.config.index_path(),
                            )
                            .await?,
                        }
                    };
                    send_with_stream(&mut send, &response.map_err(|e| e.into())).await?;
                }
                RequestType::GetFile(request::GetFile { name }) => {
                    let response: crate::Result<_> = try {
                        let path = self.config.peer_storage_path.file_path(name)?;
                        let contents_len = match std::fs::metadata(&path) {
                            Ok(meta) => Some(meta.len()),
                            Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => None,
                            Err(e) => Err(e)?,
                        };
                        (path, contents_len)
                    };
                    match response {
                        Ok((path, contents_len)) => {
                            let response = Ok(response::GetFile { contents_len });
                            send_with_stream(&mut send, &response).await?;
                            // TODO: Communicate error to peer if it occurs during copying.
                            crate::util::wide_copy(File::open(path).await?, send).await?;
                        }
                        Err(e) => {
                            send_with_stream(
                                &mut send,
                                &protocol::Result::<response::GetFile>::Err(e.into()),
                            )
                            .await?;
                        }
                    }
                }
                RequestType::Write(request::Write { name, .. }) => {
                    let response: crate::Result<_> = try {
                        let path = self.config.peer_storage_path.file_path(name)?;
                        crate::util::wide_copy(recv, File::create(path).await?).await?;
                        response::Write
                    };
                    send_with_stream(&mut send, &response.map_err(|e| e.into())).await?;
                }
                RequestType::Rename(request::Rename { from, to }) => {
                    let response: crate::Result<_> = try {
                        std::fs::rename(
                            self.config.peer_storage_path.file_path(&from)?,
                            self.config.peer_storage_path.file_path(&to)?,
                        )?;
                        response::Rename
                    };
                    send_with_stream(&mut send, &response.map_err(|e| e.into())).await?;
                }
                RequestType::Delete(request::Delete { name }) => {
                    let response: crate::Result<_> = try {
                        std::fs::remove_file(self.config.peer_storage_path.file_path(&name)?)?;
                        response::Delete
                    };
                    send_with_stream(&mut send, &response.map_err(|e| e.into())).await?;
                }
                RequestType::SetIndex(request::SetIndex { index }) => {
                    let response: crate::Result<_> = try {
                        let serialized = bincode::serialize(&index)?;
                        File::create(self.config.index_path())
                            .await?
                            .write_all(&serialized)
                            .await?;
                        response::SetIndex
                    };
                    send_with_stream(&mut send, &response.map_err(|e| e.into())).await?;
                }
                RequestType::Complete(request::Complete { index }) => {
                    send_with_stream(&mut send, &Ok(response::Complete)).await?;
                    return Ok(match index {
                        Some(index) => index.decrypt(&self.data.key_pair.private)?,
                        None => Index::new(),
                    });
                }
            }
        }
    }

    pub async fn retrieve_backup_data<P>(&mut self, index: &Index, output: P) -> Result<()>
    where
        P: AsRef<std::path::Path>,
    {
        let aed = XChaCha20Poly1305::new(chacha20poly1305::Key::from_slice(
            self.data.key_pair.private.as_ref(),
        ));
        for (name, _) in index.into_iter() {
            let hashed_name = name.as_path().into();
            let (response::GetFile { contents_len }, mut recv) = self
                .send_request(&request::GetFile { name: hashed_name })
                .await?;
            // TODO: Remove cast?
            let contents_len = contents_len.ok_or(Error::NotFoundOnPeer)? as usize;

            let mut path = output.as_ref().to_owned();
            path.push(name);
            let mut file = File::create(path).await?;

            let mut buffer = [0; FILE_FRAME_SIZE];
            let mut contents_len_left = contents_len;

            while contents_len_left != 0 {
                let read_len: usize = std::cmp::min(FILE_FRAME_SIZE, contents_len_left);

                if read_len < 24 + 16 {
                    return Err(Error::FrameTooShort);
                };

                recv.read_exact(&mut buffer[..read_len]).await?;
                file.write_all(&buffer).await?;

                let (nonce, data, tag) = {
                    let data_start = 24;
                    let tag_start = read_len - 16;

                    let (nonce, data_and_tag) = buffer[..read_len].split_at_mut(data_start);
                    let (data, tag) = data_and_tag.split_at_mut(tag_start);

                    let nonce = XNonce::from_slice(nonce);
                    let tag = Tag::from_slice(tag);

                    (nonce, data, tag)
                };

                match aed.decrypt_in_place_detached(nonce, &[], data, tag) {
                    Ok(_) => file.write_all(data).await?,
                    Err(_) => return Err(Error::Decryption),
                };

                contents_len_left -= read_len;
            }
        }

        self.send_request(&request::Complete { index: None })
            .await?;
        Ok(())
    }
}

async fn send_with_stream<T>(send: &mut SendStream, request: &T) -> Result<()>
where
    T: protocol::Serialize,
{
    let encoded = protocol::serialize(request)?;
    send.write_all(&encoded).await?;
    send.finish().await?;
    Ok(())
}

async fn receive_from_stream<T>(recv: &mut RecvStream) -> Result<T>
where
    T: protocol::Deserialize,
{
    // TODO: Make sure not too long.
    let request_length = usize::from(recv.read_u16().await?);
    let mut buf = vec![0; request_length];
    recv.read_exact(&mut buf).await?;
    protocol::deserialize::<_, T>(&buf).map_err(|e| e.into())
}
