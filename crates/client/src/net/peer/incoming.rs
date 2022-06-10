use crate::{
    crypto::Encrypted,
    fs::index::Index,
    net::{
        peer::{receive_packet, send_packet},
        protocol::{
            self,
            request::{self, RequestType},
            response,
        },
    },
    persistent::{config::Config, data::Data},
    Error, Result,
};

use futures_util::StreamExt;
use quinn::{IncomingBiStreams, RecvStream, SendStream};
use tokio::fs::File;
use tracing::{debug, trace};

#[derive(Debug)]
pub struct IncomingConnection<'a, 'b> {
    #[allow(dead_code)]
    pub(crate) data: &'a Data,
    pub(crate) config: &'b Config,
    pub(crate) bi_streams: IncomingBiStreams,
}

impl<'a, 'b> IncomingConnection<'a, 'b> {
    pub async fn handle(mut self) -> Result<()> {
        loop {
            let (mut send, mut recv) = self.accept_stream().await?;
            let request = receive_packet(&mut recv).await?;

            match request {
                RequestType::Ping(_) => send_packet(&mut send, &Ok(response::Ping)).await?,
                RequestType::GetIndex(_) => {
                    let response: crate::Result<_> = try {
                        response::GetIndex {
                            index: Encrypted::<Index>::from_disk(self.config.index_path()).await?,
                        }
                    };
                    send_packet(&mut send, &response.map_err(|e| e.into())).await?;
                }
                RequestType::GetFile(request::GetFile { name }) => {
                    let result: crate::Result<_> = try {
                        let path = self.config.peer_storage_path.file_path(name)?;
                        let len = match tokio::fs::metadata(&path).await {
                            Ok(meta) => Some(meta.len()),
                            Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => None,
                            Err(e) => Err(e)?,
                        };

                        (path, len)
                    };

                    match result {
                        Ok((path, len)) => {
                            let response = Ok(response::GetFile { len });
                            send_packet(&mut send, &response).await?;
                            trace!("sent get file response, starting wide copy");
                            // TODO: Communicate error to peer if it occurs during copying.
                            // TODO: Handle len == None
                            crate::util::async_wide_copy(File::open(path).await?, send).await?;
                            trace!("get file wide copy complete");
                        }
                        Err(e) => {
                            send_packet(
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
                        crate::util::async_wide_copy(recv, File::create(path).await?).await?;
                        response::Write
                    };
                    send_packet(&mut send, &response.map_err(|e| e.into())).await?;
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
                    send_packet(&mut send, &response.map_err(|e| e.into())).await?;
                }
                RequestType::Delete(request::Delete { name }) => {
                    let response: crate::Result<_> = try {
                        tokio::fs::remove_file(self.config.peer_storage_path.file_path(&name)?)
                            .await?;
                        response::Delete
                    };
                    send_packet(&mut send, &response.map_err(|e| e.into())).await?;
                }
                RequestType::SetIndex(request::SetIndex { index }) => {
                    let response: crate::Result<_> = try {
                        index.to_disk(self.config.index_path()).await?;
                        response::SetIndex
                    };
                    send_packet(&mut send, &response.map_err(|e| e.into())).await?;
                }
                RequestType::Complete(_) => {
                    debug!("sending complete response");
                    send_packet(&mut send, &Ok(response::Complete)).await?;
                    trace!("sleeping after sending complete response");
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    return Ok(());
                }
            }
            trace!("request handled");
        }
    }

    async fn accept_stream(&mut self) -> Result<(SendStream, RecvStream)> {
        if let Some(stream) = self.bi_streams.next().await {
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
