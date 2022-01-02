pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Connection(quinn::ConnectionError),
    Serde(soter_cs::serde::Error),
    Io(std::io::Error),
    Stun(soter_cert::Error),
}

impl From<quinn::ConnectionError> for Error {
    fn from(e: quinn::ConnectionError) -> Self {
        Self::Connection(e)
    }
}

impl From<soter_cs::serde::Error> for Error {
    fn from(e: soter_cs::serde::Error) -> Self {
        Self::Serde(e)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<soter_cert::Error> for Error {
    fn from(e: soter_cert::Error) -> Self {
        Self::Stun(e)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        #[allow(unreachable_patterns)]
        match self {
            Error::Connection(e) => Some(e),
            Error::Serde(e) => Some(e),
            Error::Io(e) => Some(e),
            _ => None,
        }
    }
}
