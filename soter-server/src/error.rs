pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("unknown connection error")]
    Connection(#[from] quinn::ConnectionError),
    #[error("error serializing or deserializing")]
    Serde(#[from] soter_cs::serde::Error),
    #[error("unknown I/O error")]
    Io(#[from] std::io::Error),
    #[error("error generating server config")]
    ServerConfig(#[from] soter_cert::Error),
    #[error("error sending response")]
    Write(#[from] quinn::WriteError),
}
