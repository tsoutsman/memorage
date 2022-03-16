pub fn hash<T>(reader: T) -> crate::Result<[u8; 32]>
where
    T: tokio::io::AsyncRead + std::marker::Unpin,
{
    let mut hasher = Hasher(blake3::Hasher::new());
    crate::util::wide_copy(reader, &mut hasher);
    Ok(hasher.0.finalize().into())
}

pub(crate) struct Hasher(blake3::Hasher);

impl tokio::io::AsyncWrite for Hasher {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        _: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        use std::io::Write;
        std::task::Poll::Ready(self.get_mut().0.write(buf))
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        _: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        _: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        std::task::Poll::Ready(Ok(()))
    }
}
