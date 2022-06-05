use crate::{
    crypto::Encrypted,
    fs::index::{Index, IndexDifference},
    net::{
        peer::{receive_from_stream, stream::encrypt_and_wide_copy, PeerConnection},
        protocol::{self, request, ENCRYPTED_FILE_FRAME_SIZE, FILE_FRAME_SIZE},
    },
    Result,
};

use tracing::debug;

impl<'a, 'b> PeerConnection<'a, 'b> {
    pub async fn send_data(
        &self,
        old_index: &Index,
        new_index: &Index,
        initiator: bool,
    ) -> Result<()> {
        let difference = new_index.difference(old_index);

        if difference.is_empty() {
            debug_assert_eq!(old_index, new_index);
            debug!("old index and new index identical");
        } else {
            for d in difference {
                self.send_difference(d).await?;
            }

            debug!("setting index on peer");
            self.send_request(&request::SetIndex {
                index: Encrypted::encrypt(&self.data.key_pair.private, new_index)?,
            })
            .await?;
        }

        if initiator {
            debug!("sending complete(continue) request");
            self.send_request(&request::Complete::Continue).await?;
        } else {
            debug!("sending complete(close) request");
            self.send_request(&request::Complete::Close).await?;
        }

        Ok(())
    }

    async fn send_difference(&self, diff: IndexDifference) -> Result<()> {
        debug!(difference=?diff, "sending difference");
        match diff {
            IndexDifference::Write(name) => {
                let path = self.config.backup_path.join(&name);
                debug!(?name, ?path, "writing file to peer");

                let contents_len = tokio::fs::metadata(&path).await?.len();
                let num_chunks = contents_len.div_ceil(FILE_FRAME_SIZE as u64);
                let encrypted_contents_len = contents_len
                    + num_chunks * (ENCRYPTED_FILE_FRAME_SIZE - FILE_FRAME_SIZE) as u64;

                debug!(
                    ?contents_len,
                    ?num_chunks,
                    ?encrypted_contents_len,
                    "sending write request"
                );

                let (mut send, mut recv) = self
                    .send_request_without_response(&request::Write {
                        contents_len: encrypted_contents_len,
                        name: name.as_path().into(),
                    })
                    .await?;

                encrypt_and_wide_copy(&mut send, &self.data.key_pair.private, &path, contents_len)
                    .await?;

                send.finish().await?;
                receive_from_stream::<protocol::Result<protocol::response::Write>>(&mut recv)
                    .await??;

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
}
