pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Connection(quinn::ConnectionError),
    Serde(soter_cs::serde::Error),
    Io(std::io::Error),
    CertificateGeneration(rcgen::RcgenError),
    TlsConfig(rustls::Error),
    Stun(soter_stun::Error),
    NoAddress,
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

impl From<rcgen::RcgenError> for Error {
    fn from(e: rcgen::RcgenError) -> Self {
        Self::CertificateGeneration(e)
    }
}

impl From<rustls::Error> for Error {
    fn from(e: rustls::Error) -> Self {
        Self::TlsConfig(e)
    }
}

impl From<soter_stun::Error> for Error {
    fn from(e: soter_stun::Error) -> Self {
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
            Error::CertificateGeneration(e) => Some(e),
            Error::TlsConfig(e) => Some(e),
            Error::Stun(e) => Some(e),
            _ => None,
        }
    }
}
