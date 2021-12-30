use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    pin::Pin,
};

use lib::cs::{
    key::Keypair,
    protocol::{error::Error, request::Request},
};
use server::setup::Channels;
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

pub async fn request<T>(request: Request, addr: SocketAddr, channels: Channels) -> Result<T, Error>
where
    T: serde::de::DeserializeOwned,
{
    let mut buffer = MockRequest::from(request);
    server::handle_request(&mut buffer, addr, channels).await;
    bincode::deserialize(&buffer.output()).unwrap()
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
