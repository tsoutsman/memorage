use crate::{
    crypto,
    fs::index::Index,
    net::{
        peer::PeerConnection,
        protocol::{request, response, ENCRYPTED_FILE_FRAME_SIZE, NONCE_LENGTH, TAG_LENGTH},
    },
    Error, Result,
};

use tokio::{fs::File, io::AsyncWriteExt};
use tracing::{debug, info};
impl<'a, 'b> PeerConnection<'a, 'b> {
    pub async fn retrieve_data<P>(&mut self, index: &Index, output: P) -> Result<()>
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

            debug!(?name, ?contents_len);

            let mut path = output.as_ref().to_owned();
            tokio::fs::create_dir_all(path.clone()).await?;
            path.push(name);

            debug!("writing decrypted file to {}", path.display());

            let mut file = File::create(path).await?;

            let mut buf = [0; ENCRYPTED_FILE_FRAME_SIZE];
            let mut contents_len_left = contents_len;

            while contents_len_left != 0 {
                let read_len: usize = std::cmp::min(ENCRYPTED_FILE_FRAME_SIZE, contents_len_left);

                if read_len < NONCE_LENGTH + 1 + TAG_LENGTH {
                    return Err(Error::FrameTooShort);
                };

                debug!(?read_len);

                recv.read_exact(&mut buf[..read_len]).await?;

                let data =
                    crypto::decrypt_in_place(&self.data.key_pair.private, &mut buf[..read_len])?;
                file.write_all(data).await?;

                contents_len_left -= read_len;
            }

            info!(?name, "successfully retrieved file");
        }

        self.send_request(&request::Complete::Close).await?;
        Ok(())
    }
}
