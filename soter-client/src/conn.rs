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
        Ok(Self(TcpStream::connect(s).await?))
    }

    pub async fn request<T>(&mut self, request: T) -> Result<T::Response>
    where
        T: serde::Serialize + Request,
    {
        // TODO enforce to_enum in type system
        let encoded: Vec<u8> = bincode::serialize(&request.to_enum())?;
        self.0.write_all(&encoded).await?;

        // TODO buffer len
        let mut buffer = vec![0u8; 1024];
        self.0.read(&mut buffer[..]).await?;

        let decoded: std::result::Result<T::Response, soter_cs::Error> =
            bincode::deserialize(&buffer)?;
        decoded.map_err(|e| e.into())
    }
}
