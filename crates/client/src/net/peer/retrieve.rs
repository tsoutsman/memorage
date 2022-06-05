use crate::{
    fs::index::Index,
    net::{
        peer::{stream::decrypt_and_wide_copy, PeerConnection},
        protocol::{request, response},
    },
    Error, Result,
};

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

            decrypt_and_wide_copy(&mut recv, &self.data.key_pair.private, &path, contents_len)
                .await?;
            info!(?name, "successfully retrieved file");
        }

        let _ = self.send_request(&request::Complete::Close).await;
        Ok(())
    }
}
