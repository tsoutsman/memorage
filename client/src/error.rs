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
    Server(#[from] memorage_cs::Error),
    #[error("error generating server config")]
    ServerConfig(#[from] memorage_cert::Error),
    #[error("error determining public IP address")]
    Stun(#[from] memorage_stun::Error),
    #[error("unkown network read error")]
    Read(#[from] quinn::ReadToEndError),
    #[error("unknown network write error")]
    Write(#[from] quinn::WriteError),
    #[error("invalid connection configuration")]
    ConnectionConfig(#[from] quinn::ConnectError),
    #[error("unknown connection error")]
    Connection(#[from] quinn::ConnectionError),
    #[error("error reading config")]
    ConfigRead(#[from] toml::de::Error),
    #[error("error writing config")]
    ConfigWrite(#[from] toml::ser::Error),
    #[error("peer not set")]
    PeerNotSet,
    #[error("peer didn't respond to connection request")]
    PeerNoResponse,
    #[error("unauthorised connection request")]
    UnauthorisedConnectionRequest,
    #[error("error occured while traversing directory")]
    WalkDir(#[from] walkdir::Error),
}
