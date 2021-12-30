use crate::error::{Error, Result};

use lib::{bincode, cs::protocol::request::Request};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpStream, ToSocketAddrs},
};

pub struct Connection(TcpStream);

impl Connection {
    pub async fn try_to<A>(s: A) -> Result<Self>
    where
        A: ToSocketAddrs,
    {
        Ok(Self(TcpStream::connect(s).await?))
    }

    // TODO somehow remove generic
    pub async fn request<T>(&mut self, request: Request) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let encoded: Vec<u8> = bincode::serialize(&request)?;
        self.0.write_all(&encoded).await?;

        // TODO buffer len
        let mut buffer = vec![0u8; 1024];
        self.0.read(&mut buffer[..]).await?;

        let decoded: std::result::Result<T, lib::cs::protocol::error::Error> =
            bincode::deserialize(&buffer)?;
        decoded.map_err(Error::Server)
    }
}
