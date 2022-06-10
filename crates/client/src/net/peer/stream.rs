use crate::{
    crypto,
    net::protocol::{ENCRYPTED_FILE_FRAME_SIZE, FILE_FRAME_SIZE, NONCE_LENGTH, TAG_LENGTH},
    Error, Result,
};

use std::path::Path;

use memorage_core::PrivateKey;
use quinn::{RecvStream, SendStream};
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
};
use tracing::{debug, trace};

pub(crate) async fn encrypt_and_wide_copy(
    send: &mut SendStream,
    private_key: &PrivateKey,
    path: &Path,
    contents_len: u64,
) -> Result<()> {
    let mut buf = [0; ENCRYPTED_FILE_FRAME_SIZE];
    let mut contents_len_left = contents_len as usize;

    let mut file = File::open(path).await?;

    while contents_len_left != 0 {
        let data_read_len = std::cmp::min(FILE_FRAME_SIZE, contents_len_left as usize);

        debug!(?data_read_len);

        let buf_slice = &mut buf[..(NONCE_LENGTH + data_read_len + TAG_LENGTH)];
        let (nonce_slice, data_slice, tag_slice) = crypto::split_encrypted_buf(buf_slice);

        file.read_exact(data_slice).await?;
        let (nonce, tag) = crypto::encrypt_in_place(data_slice, private_key)?;
        nonce_slice.copy_from_slice(nonce.as_slice());
        tag_slice.copy_from_slice(tag.as_slice());

        send.write_all(buf_slice).await?;
        contents_len_left -= data_read_len;
    }
    Ok(())
}

pub(crate) async fn decrypt_and_wide_copy(
    recv: &mut RecvStream,
    private_key: &PrivateKey,
    path: &Path,
    contents_len: usize,
) -> Result<()> {
    let mut file = File::create(path).await?;

    let mut buf = [0; ENCRYPTED_FILE_FRAME_SIZE];
    let mut contents_len_left = contents_len;

    while contents_len_left != 0 {
        let read_len: usize = std::cmp::min(ENCRYPTED_FILE_FRAME_SIZE, contents_len_left);

        // Frame must contain at least one byte of data.
        if read_len < NONCE_LENGTH + 1 + TAG_LENGTH {
            return Err(Error::FrameTooShort);
        };

        trace!(?read_len, "reading frame");

        recv.read_exact(&mut buf[..read_len]).await?;

        let data = crypto::decrypt_in_place(private_key, &mut buf[..read_len])?;
        file.write_all(data).await?;

        contents_len_left -= read_len;
    }
    Ok(())
}
