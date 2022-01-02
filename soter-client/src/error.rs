use std::{convert::From, path::PathBuf};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    NotDirectory(PathBuf),
    Io(std::io::Error),
    Encryption,
    Decryption,
    Utf8(std::string::FromUtf8Error),
    /// An error that occurs during serialization or deserialization.
    Serde(bincode::Error),
    ServerConnection(soter_cs::Error),
    Certificate(soter_cert::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Error::NotDirectory(p) => format!("{} is not a directory", p.to_string_lossy()),
            Error::Io(e) => format!("IO error: {}", e),
            Error::Encryption => todo!(),
            Error::Decryption => todo!(),
            Error::Utf8(_) => todo!(),
            Error::Serde(_) => todo!(),
            Error::ServerConnection(_) => todo!(),
            Error::Certificate(_) => todo!(),
        };

        write!(f, "{}", s)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(e) => Some(e),
            Error::Utf8(e) => Some(e),
            Error::Serde(e) => Some(e),
            Error::ServerConnection(e) => Some(e),
            Error::Certificate(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(e: std::string::FromUtf8Error) -> Self {
        Self::Utf8(e)
    }
}

impl From<bincode::Error> for Error {
    fn from(e: bincode::Error) -> Self {
        Self::Serde(e)
    }
}

impl From<soter_cs::Error> for Error {
    fn from(e: soter_cs::Error) -> Self {
        Self::ServerConnection(e)
    }
}

impl From<soter_cert::Error> for Error {
    fn from(e: soter_cert::Error) -> Self {
        Self::Certificate(e)
    }
}
