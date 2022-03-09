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

// TODO: Specialisation
// impl<T> From<T> for Error
// where
//     T: From<crate::Error>,
// {
//     fn from(_: T) -> Self {
//         // TODO
//         Error::Generic
//     }
// }
