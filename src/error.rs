use std::path::PathBuf;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("could not find {0}")]
    FileNotFound(PathBuf),
    #[error("denied permission to access {0}")]
    PermissionDenied(PathBuf),
    #[error("{0} is not a directory")]
    NotDirectory(PathBuf),
    #[error("I/O error")]
    Io(#[from] std::io::Error),
    #[error("error encrypting {0}")]
    Encryption(PathBuf),
    #[error("stun message length too large; length: {0}")]
    MessageTooLarge(usize),
}
