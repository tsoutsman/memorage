pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0} is not a directory")]
    NotDirectory(std::path::PathBuf),
    #[error("unknown I/O error")]
    Io { source: std::io::Error },
    #[error("entity not found")]
    NotFound { source: std::io::Error },
    #[error("entity already exists")]
    AlreadyExists { source: std::io::Error },
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
    Read(#[from] quinn::ReadError),
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
    #[error("peer didn't respond to connection request")]
    PeerNoResponse,
    #[error("unauthorised connection request")]
    UnauthorisedConnectionRequest,
    #[error("error occured while traversing directory")]
    Jwalk(#[from] jwalk::Error),
    #[error("peer encountered error")]
    Peer(#[from] crate::net::protocol::Error),
    #[error("peer closed connection")]
    PeerClosedConnection,
    #[error("failed to establish connection to peer")]
    FailedConnection,
    #[error("incorrect peer")]
    IncorrectPeer,
    #[error("peer sent malicious file name")]
    MaliciousFileName,
    #[error("missed peer synchronisation")]
    MissedSynchronisation,
    #[error("attempted retrieval of file that didn't exist on peer")]
    NotFoundOnPeer,
    #[error("end of stream reached prematurely")]
    UnexpectedEof,
    #[error("response too large")]
    TooLarge,
    #[error("frame too short")]
    FrameTooShort,
    #[error("join error")]
    Join(#[from] tokio::task::JoinError),
    #[error("mnemonic contains invalid words")]
    InvalidWords,
    #[error("user cancelled operation")]
    UserCancelled,
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        match e.kind() {
            std::io::ErrorKind::NotFound => Self::NotFound { source: e },
            std::io::ErrorKind::AlreadyExists => Self::AlreadyExists { source: e },
            std::io::ErrorKind::UnexpectedEof => Self::UnexpectedEof,
            _ => Self::Io { source: e },
        }
    }
}

impl From<quinn::ReadExactError> for Error {
    fn from(e: quinn::ReadExactError) -> Self {
        match e {
            quinn::ReadExactError::FinishedEarly => Self::UnexpectedEof,
            quinn::ReadExactError::ReadError(e) => Self::Read(e),
        }
    }
}

impl From<quinn::ReadToEndError> for Error {
    fn from(e: quinn::ReadToEndError) -> Self {
        match e {
            quinn::ReadToEndError::Read(e) => Self::Read(e),
            quinn::ReadToEndError::TooLong => Self::TooLarge,
        }
    }
}
