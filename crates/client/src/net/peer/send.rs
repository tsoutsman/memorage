use crate::{
    crypto::{self, Encrypted},
    fs::index::{Index, IndexDifference},
    net::{
        peer::{receive_from_stream, PeerConnection},
        protocol::{
            self, request, ENCRYPTED_FILE_FRAME_SIZE, FILE_FRAME_SIZE, NONCE_LENGTH, TAG_LENGTH,
        },
    },
    Result,
};

use tokio::{fs::File, io::AsyncReadExt};
use tracing::{debug, info};

impl<'a, 'b> PeerConnection<'a, 'b> {
    pub async fn send_data(
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
                    let absolute_file_path = self.config.backup_path.join(&name);
                    info!(?name, ?absolute_file_path, "writing file to peer");

                    let contents_len = tokio::fs::metadata(&absolute_file_path).await?.len();
                    let num_chunks = contents_len.div_ceil(FILE_FRAME_SIZE as u64);
                    let encrypted_contents_len = contents_len
                        + num_chunks * (ENCRYPTED_FILE_FRAME_SIZE - FILE_FRAME_SIZE) as u64;

                    info!(?contents_len);

                    let (mut send, mut recv) = self
                        .send_request_without_response(&request::Write {
                            contents_len: encrypted_contents_len,
                            name: name.as_path().into(),
                        })
                        .await?;

                    let mut buf = [0; ENCRYPTED_FILE_FRAME_SIZE];
                    let mut contents_len_left = contents_len as usize;

                    let mut file = File::open(absolute_file_path).await?;

                    while contents_len_left != 0 {
                        let data_read_len =
                            std::cmp::min(FILE_FRAME_SIZE, contents_len_left as usize);

                        debug!(?data_read_len);

                        let buf_slice = &mut buf[..(NONCE_LENGTH + data_read_len + TAG_LENGTH)];
                        let (nonce_slice, data_slice, tag_slice) =
                            crypto::split_encrypted_buf(buf_slice);

                        file.read_exact(data_slice).await?;
                        let (nonce, tag) =
                            crypto::encrypt_in_place(&self.data.key_pair.private, data_slice)?;
                        nonce_slice.copy_from_slice(nonce.as_slice());
                        tag_slice.copy_from_slice(tag.as_slice());

                        send.write_all(buf_slice).await?;
                        contents_len_left -= data_read_len;
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

        info!("sending set_index request");

        self.send_request(&request::SetIndex {
            index: Encrypted::encrypt(&self.data.key_pair.private, new_index)?,
        })
        .await?;

        if initiator {
            info!("sending complete(continue) request");
            self.send_request(&request::Complete::Continue).await?;
        } else {
            info!("sending complete(close) request");
            self.send_request(&request::Complete::Close).await?;
        }

        Ok(())
    }
}
