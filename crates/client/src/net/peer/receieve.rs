use crate::{
    fs::index::Index,
    net::{
        peer::{receive_from_stream, send_with_stream, PeerConnection},
        protocol::{
            self,
            request::{self, RequestType},
            response,
        },
    },
    Error, Result,
};

use futures_util::StreamExt;
use quinn::{RecvStream, SendStream};
use tokio::{fs::File, io::AsyncWriteExt};
use tracing::{debug, trace};

impl<'a, 'b> PeerConnection<'a, 'b> {
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
    pub async fn receive_commands(&mut self) -> Result<request::Complete> {
        debug!("receiving commands from peer");
        loop {
            let (mut send, mut recv) = self.accept_stream().await?;
            let request = receive_from_stream(&mut recv).await?;

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
                    let data_result: crate::Result<_> = try {
                        let path = self.config.peer_storage_path.file_path(name)?;

                        let contents_len = match tokio::fs::metadata(&path).await {
                            Ok(meta) => Some(meta.len()),
                            Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => None,
                            Err(e) => Err(e)?,
                        };

                        (path, contents_len)
                    };

                    match data_result {
                        Ok((path, contents_len)) => {
                            let response = Ok(response::GetFile { contents_len });
                            send_with_stream(&mut send, &response).await?;
                            trace!("sent get file response, starting wide copy");
                            // TODO: Communicate error to peer if it occurs during copying.
                            crate::util::async_wide_copy(File::open(path).await?, send).await?;
                            trace!("get file wide copy complete");
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
                        debug!(?path, "writing to file");
                        trace!("received write request, starting wide copy");
                        crate::util::async_wide_copy(recv, File::create(path).await?).await?;
                        trace!("write wide copy complete");
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
                RequestType::Complete(request) => {
                    debug!("sending complete response");
                    send_with_stream(&mut send, &Ok(response::Complete)).await?;
                    if request == request::Complete::Close {
                        trace!("sleeping after sending complete response");
                        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    }
                    return Ok(request);
                }
            }
            trace!("request handled");
        }
    }
}
