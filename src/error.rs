use std::path::PathBuf;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0} is not a directory")]
    NotDirectory(PathBuf),
    #[error("I/O error")]
    IoError(#[from] std::io::Error),
}
