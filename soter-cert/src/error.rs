pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Stun(StunError),
    Io(std::io::Error),
    CertificateGeneration(rcgen::RcgenError),
    TlsConfig(rustls::Error),
}

impl From<StunError> for Error {
    fn from(e: StunError) -> Self {
        Self::Stun(e)
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

impl std::fmt::Display for Error {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Stun(e) => Some(e),
            Self::Io(e) => Some(e),
            Self::CertificateGeneration(e) => Some(e),
            Self::TlsConfig(e) => Some(e),
        }
    }
}

#[derive(Debug)]
pub enum StunError {
    InvalidUtf8(std::string::FromUtf8Error),
    InvalidHeader,
    InvalidType,
    InvalidMagicCookie,
    InvalidAttributeType,
    IncorrectAttributeType,
    AttributeTooLarge(&'static str),
    IncorrectAttributeLength,
    IncorrectMessageLength,
    InvalidAddressFamily,
    InvalidAddress,
    NoAddress,
}

impl From<std::string::FromUtf8Error> for StunError {
    fn from(e: std::string::FromUtf8Error) -> Self {
        Self::InvalidUtf8(e)
    }
}

impl std::fmt::Display for StunError {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl std::error::Error for StunError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidUtf8(e) => Some(e),
            _ => None,
        }
    }
}
