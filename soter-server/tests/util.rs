use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    pin::Pin,
};

use soter_core::Keypair;
use soter_server::setup::Channels;
use tokio::io::{AsyncRead, AsyncWrite};

lazy_static::lazy_static! {
    pub static ref KEYPAIR_1: Keypair = {
        let mut csprng = rand::rngs::OsRng;
        Keypair::generate(&mut csprng)
    };
    pub static ref KEYPAIR_2: Keypair = {
        let mut csprng = rand::rngs::OsRng;
        Keypair::generate(&mut csprng)
    };
    pub static ref ADDR_1: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4)), 1);
    pub static ref ADDR_2: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(2, 3, 4, 5)), 2);
}

pub async fn request<T>(
    request: T,
    addr: SocketAddr,
    channels: Channels,
) -> soter_cs::Result<T::Response>
where
    T: soter_cs::serde::SerializableRequestOrResponse + soter_cs::request::Request,
{
    let request = AsyncBuf(soter_cs::serialize(request).unwrap());
    let mut output = AsyncBuf(Vec::new());
    soter_server::handle_request((&mut output, request), addr, channels)
        .await
        .unwrap();
    soter_cs::deserialize(output.0).unwrap()
}

pub struct AsyncBuf(pub Vec<u8>);

impl AsyncWrite for AsyncBuf {
    fn poll_write(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        Pin::new(&mut self.0).poll_write(cx, buf)
    }

    fn poll_flush(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        Pin::new(&mut self.0).poll_flush(cx)
    }

    fn poll_shutdown(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        Pin::new(&mut self.0).poll_shutdown(cx)
    }
}

impl AsyncRead for AsyncBuf {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        Pin::new(&mut &self.0[..]).poll_read(cx, buf)
    }
}
