use crate::Result;

use soter_cs::request::Request;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpStream, ToSocketAddrs},
};

#[derive(Debug)]
pub struct Connection(TcpStream);

impl Connection {
    pub async fn try_to<A>(s: A) -> Result<Self>
    where
        A: ToSocketAddrs,
    {
        // TODO use Quinn
        Ok(Self(TcpStream::connect(s).await?))
    }

    pub async fn request<T>(&mut self, request: T) -> Result<T::Response>
    where
        T: soter_cs::Serialize + Request,
    {
        let encoded: Vec<u8> = soter_cs::serialize(request)?;
        self.0.write_all(&encoded).await?;

        // TODO buffer len
        let mut buffer = vec![0u8; 1024];
        self.0.read(&mut buffer[..]).await?;

        let decoded: std::result::Result<T::Response, soter_cs::Error> =
            soter_cs::deserialize(&buffer)?;
        decoded.map_err(|e| e.into())
    }
}
