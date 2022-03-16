use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub(crate) async fn wide_copy<R, W>(mut reader: R, mut writer: W) -> crate::Result<usize>
where
    R: tokio::io::AsyncRead + std::marker::Unpin,
    W: tokio::io::AsyncWrite + std::marker::Unpin,
{
    let mut buffer = [0; 65536];
    let mut total = 0;

    loop {
        match reader.read(&mut buffer).await {
            Ok(0) => return Ok(total),
            Ok(n) => {
                writer.write_all(&buffer[..n]).await?;
                total += n;
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(e.into()),
        };
    }
}
