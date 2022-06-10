use crate::{
    crypto::Encrypted,
    fs::index::{Index, IndexDifference},
    net::{
        peer::{
            receive_packet, send_packet,
            stream::{decrypt_and_wide_copy, encrypt_and_wide_copy},
        },
        protocol::{self, request, response, ENCRYPTED_FILE_FRAME_SIZE, FILE_FRAME_SIZE},
    },
    persistent::{config::Config, data::Data},
    Error, Result,
};

use quinn::{Connection, RecvStream, SendStream};
use tracing::{debug, info};

#[derive(Debug)]
pub struct OutgoingConnection<'a, 'b> {
    pub(crate) data: &'a Data,
    pub(crate) config: &'b Config,
    pub(crate) connection: Connection,
}

impl<'a, 'b> OutgoingConnection<'a, 'b> {
    pub async fn ping(&self) -> Result<()> {
        self.send_request(&request::Ping).await.map(|_| ())
    }

    pub async fn backup(&self, new_index: &Index) -> Result<()> {
        let old_index = self.get_index().await?;
        let difference = new_index.difference(&old_index);

        if difference.is_empty() {
            debug_assert_eq!(old_index, *new_index);
            debug!("old index and new index identical");
        } else {
            for d in difference {
                self.send_difference(d).await?;
            }
            debug!("setting index on peer");
            self.send_request(&request::SetIndex {
                index: Encrypted::encrypt(new_index, &self.data.key_pair.private)?,
            })
            .await?;
        }

        self.send_request(&request::Complete).await?;
        Ok(())
    }

    pub async fn retrieve<P>(&mut self, output: P) -> Result<()>
    where
        P: AsRef<std::path::Path>,
    {
        let index = self.get_index().await?;
        for (name, _) in index.into_iter() {
            info!(?name, "retrieving file");

            let hashed_name = name.as_path().into();
            let (response::GetFile { len }, (_, mut recv)) = self
                .send_request(&request::GetFile { name: hashed_name })
                .await?;
            // TODO: Remove cast?
            let len = len.ok_or(Error::NotFoundOnPeer)? as usize;

            debug!(?name, ?len);

            let mut path = output.as_ref().to_owned();
            tokio::fs::create_dir_all(path.clone()).await?;
            path.push(name);

            debug!("writing decrypted file to {}", path.display());

            decrypt_and_wide_copy(&mut recv, &self.data.key_pair.private, &path, len).await?;
            info!(?name, "successfully retrieved file");
        }

        self.send_request(&request::Complete).await?;
        Ok(())
    }

    async fn get_index(&self) -> Result<Index> {
        Ok(match self.send_request(&request::GetIndex).await?.0.index {
            Some(i) => i.decrypt(&self.data.key_pair.private)?,
            None => Index::new(),
        })
    }

    async fn send_difference(&self, diff: IndexDifference) -> Result<()> {
        debug!(difference=?diff, "sending difference");
        match diff {
            IndexDifference::Write(name) => {
                let path = self.config.backup_path.join(&name);
                debug!(?name, ?path, "writing file to peer");

                let len = tokio::fs::metadata(&path).await?.len();
                let num_chunks = len.div_ceil(FILE_FRAME_SIZE as u64);
                let encrypted_len =
                    len + num_chunks * (ENCRYPTED_FILE_FRAME_SIZE - FILE_FRAME_SIZE) as u64;

                debug!(?len, ?num_chunks, ?encrypted_len, "sending write request");

                let (mut send, mut recv) = self
                    .send_request_without_response(&request::Write {
                        len: encrypted_len,
                        name: name.as_path().into(),
                    })
                    .await?;

                encrypt_and_wide_copy(&mut send, &self.data.key_pair.private, &path, len).await?;

                send.finish().await?;
                receive_packet::<protocol::Result<protocol::response::Write>>(&mut recv).await??;

                debug!("successfully wrote file to peer");
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
        Ok(())
    }

    async fn send_request<T>(&self, request: &T) -> Result<(T::Response, (SendStream, RecvStream))>
    where
        T: protocol::Serialize + request::Request + std::fmt::Debug,
    {
        let (mut send, mut recv) = self.send_request_without_response(request).await?;
        send.finish().await?;
        let response = receive_packet::<protocol::Result<_>>(&mut recv).await??;
        Ok((response, (send, recv)))
    }

    async fn send_request_without_response<T>(
        &self,
        request: &T,
    ) -> Result<(SendStream, RecvStream)>
    where
        T: protocol::Serialize + request::Request + std::fmt::Debug,
    {
        let (mut send, recv) = self.connection.open_bi().await?;
        send_packet(&mut send, request).await?;
        Ok((send, recv))
    }
}
