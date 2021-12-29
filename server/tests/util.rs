use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    pin::Pin,
};

use lib::cs::key::PublicKey;
use tokio::io::{AsyncRead, AsyncWrite};

lazy_static::lazy_static! {
    pub static ref KEY_1: PublicKey = PublicKey::from_bytes(&[
        215, 90, 152, 1, 130, 177, 10, 183, 213, 75, 254, 211, 201, 100, 7, 58, 14, 225, 114, 243,
        218, 166, 35, 37, 175, 2, 26, 104, 247, 7, 81, 26,
    ])
    .unwrap();
    pub static ref ADDR_1: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4)), 8080);
}

pub struct MockRequest {
    input: Vec<u8>,
    output: Vec<u8>,
}

impl<T> From<T> for MockRequest
where
    T: serde::Serialize,
{
    fn from(r: T) -> Self {
        // TODO unwrap
        Self::new(bincode::serialize(&r).unwrap())
    }
}

impl MockRequest {
    pub fn new(input: Vec<u8>) -> Self {
        Self {
            input,
            output: Vec::new(),
        }
    }

    pub fn output(self) -> Vec<u8> {
        self.output
    }
}

impl AsyncWrite for MockRequest {
    fn poll_write(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        Pin::new(&mut self.output).poll_write(cx, buf)
    }

    fn poll_flush(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        Pin::new(&mut self.output).poll_flush(cx)
    }

    fn poll_shutdown(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        Pin::new(&mut self.output).poll_shutdown(cx)
    }
}

impl AsyncRead for MockRequest {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        Pin::new(&mut &self.input[..]).poll_read(cx, buf)
    }
}
