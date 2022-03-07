pub type Result<T> = std::result::Result<T, Error>;

#[derive(Copy, Clone, thiserror::Error, Debug, serde::Serialize, serde::Deserialize)]
pub enum Error {
    #[error("generic error")]
    Generic,
}

impl From<crate::Error> for Error {
    fn from(_: crate::Error) -> Self {
        Error::Generic
    }
}
