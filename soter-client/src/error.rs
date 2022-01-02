pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0} is not a directory")]
    NotDirectory(std::path::PathBuf),
    #[error("unknown I/O error")]
    Io(#[from] std::io::Error),
    #[error("error encrypting file")]
    Encryption,
    #[error("error decrypting file")]
    Decryption,
    #[error("UTF8 error")]
    Utf8(#[from] std::string::FromUtf8Error),
    /// An error that occurs during serialization or deserialization.
    #[error("error serializing or deserializing")]
    Serde(#[from] bincode::Error),
    #[error("server returned an error")]
    ServerConnection(#[from] soter_cs::Error),
    #[error("error generating server config")]
    ServerConfig(#[from] soter_cert::Error),
}
