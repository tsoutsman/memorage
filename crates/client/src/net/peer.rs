use crate::{
    crypto::{self, Encrypted},
    fs::index::{Index, IndexDifference},
    net::protocol::{
        self,
        request::{self, RequestType},
        response, ENCRYPTED_FILE_FRAME_SIZE, FILE_FRAME_SIZE, NONCE_LENGTH, TAG_LENGTH,
    },
    persistent::{config::Config, data::Data},
    Error, Result,
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
    async fn send_request<T>(&self, request: &T) -> Result<(T::Response, (SendStream, RecvStream))>
    where
        T: protocol::Serialize + request::Request + std::fmt::Debug,
    {
        let (mut send, mut recv) = self.send_request_without_response(request).await?;
        send.finish().await?;
        let response = receive_from_stream::<protocol::Result<_>>(&mut recv).await??;
        debug!(?response, "received response");
        Ok((response, (send, recv)))
    }

    async fn send_request_without_response<T>(
        &self,
        request: &T,
    ) -> Result<(SendStream, RecvStream)>
    where
        T: protocol::Serialize + request::Request + std::fmt::Debug,
    {
        debug!(?request, "sending request");
        let (mut send, recv) = self.connection.connection.open_bi().await?;
        send_with_stream(&mut send, request).await?;
        Ok((send, recv))
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
                    info!(?name, "writing file to peer");

                    let contents_len = tokio::fs::metadata(&name).await?.len();

                    info!(?contents_len);

                    // TODO: peer will only send response::write after data has bee sentw
                    let (mut send, mut recv) = self
                        .send_request_without_response(&request::Write {
                            contents_len,
                            name: name.as_path().into(),
                        })
                        .await?;

                    let mut buf = [0; ENCRYPTED_FILE_FRAME_SIZE];
                    let mut contents_len_left = contents_len as usize;

                    let mut file = File::open(name).await?;

                    while contents_len_left != 0 {
                        let read_len = std::cmp::min(FILE_FRAME_SIZE, contents_len_left as usize);

                        debug!(?read_len);

                        file.read_exact(&mut buf[NONCE_LENGTH..(NONCE_LENGTH + read_len)])
                            .await?;
                        let (nonce, tag) =
                            crypto::encrypt_in_place(&self.data.key_pair.private, &mut buf)?;
                        buf[..NONCE_LENGTH].copy_from_slice(nonce.as_slice());
                        buf[(read_len + NONCE_LENGTH)..(read_len + NONCE_LENGTH + TAG_LENGTH)]
                            .copy_from_slice(tag.as_slice());
                        debug!("before send");
                        send.write_all(&buf).await?;
                        debug!("after send");

                        contents_len_left -= read_len;
                    }

                    send.finish().await?;
                    receive_from_stream::<protocol::Result<protocol::response::Write>>(&mut recv)
                        .await??;

                    info!("successfully wrote file to peer");
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
                Index::from_disk_encrypted(self.config.index_path()).await?
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
            info!(?request, "received request");

            match request {
                RequestType::Ping(_) => send_with_stream(&mut send, &Ok(response::Ping)).await?,
                RequestType::GetIndex(_) => {
                    let response: crate::Result<_> = try {
                        response::GetIndex {
                            index: Index::from_disk_encrypted(self.config.index_path()).await?,
                        }
                    };
                    send_with_stream(&mut send, &response.map_err(|e| e.into())).await?;
                }
                RequestType::GetFile(request::GetFile { name }) => {
                    let response: crate::Result<_> = try {
                        let path = self.config.peer_storage_path.file_path(name)?;
                        let contents_len = match tokio::fs::metadata(&path).await {
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
                            debug!("sent GetFile response, starting wide copy");
                            crate::util::wide_copy(File::open(path).await?, send).await?;
                            debug!("GetFile wide copy complete");
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
                        info!(?path, "writing to file");
                        debug!("received Write request, starting wide copy");
                        crate::util::wide_copy(recv, File::create(path).await?).await?;
                        debug!("Write wide copy complete");
                        response::Write
                    };
                    send_with_stream(&mut send, &response.map_err(|e| e.into())).await?;
                }
                RequestType::Rename(request::Rename { from, to }) => {
                    let response: crate::Result<_> = try {
                        tokio::fs::rename(
                            self.config.peer_storage_path.file_path(&from)?,
                            self.config.peer_storage_path.file_path(&to)?,
                        )
                        .await?;
                        response::Rename
                    };
                    send_with_stream(&mut send, &response.map_err(|e| e.into())).await?;
                }
                RequestType::Delete(request::Delete { name }) => {
                    let response: crate::Result<_> = try {
                        tokio::fs::remove_file(self.config.peer_storage_path.file_path(&name)?)
                            .await?;
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
            info!("request handled");
        }
    }

    pub async fn retrieve_backup_data<P>(&mut self, index: &Index, output: P) -> Result<()>
    where
        P: AsRef<std::path::Path>,
    {
        for (name, _) in index.into_iter() {
            info!(?name, "retrieving file");

            let hashed_name = name.as_path().into();
            let (response::GetFile { contents_len }, (_, mut recv)) = self
                .send_request(&request::GetFile { name: hashed_name })
                .await?;
            // TODO: Remove cast?
            let contents_len = contents_len.ok_or(Error::NotFoundOnPeer)? as usize;

            info!(?contents_len);

            let mut path = output.as_ref().to_owned();
            path.push(name);
            let mut file = File::create(path).await?;

            let mut buf = [0; ENCRYPTED_FILE_FRAME_SIZE];
            let mut contents_len_left = contents_len;

            while contents_len_left != 0 {
                let read_len: usize = std::cmp::min(ENCRYPTED_FILE_FRAME_SIZE, contents_len_left);

                if read_len < 24 + 16 {
                    return Err(Error::FrameTooShort);
                };

                debug!(?read_len);

                recv.read_exact(&mut buf[..read_len]).await?;
                let data = crypto::decrypt_in_place(&self.data.key_pair.private, &mut buf)?;
                file.write_all(data).await?;

                contents_len_left -= read_len;
            }

            info!("successfully retrieved file");
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
    debug!("encoded length: {}", encoded.len());
    send.write_u16(encoded.len() as u16).await?;
    send.write_all(&encoded).await?;
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
