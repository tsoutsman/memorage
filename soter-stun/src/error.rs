pub type Result<T> = std::result::Result<T, Error>;

#[derive(Copy, Clone, Debug)]
pub enum Error {
    InvalidHeader,
    InvalidType,
    InvalidMagicCookie,
    InvalidAttributeType,
    IncorrectAttributeType,
    IncorrectAttributeLength,
    IncorrectMessageLength,
    InvalidAddressFamily,
    InvalidAddress,
    InvalidUtf8,
    AttributeTooLarge(&'static str),
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(_: std::string::FromUtf8Error) -> Self {
        Self::InvalidUtf8
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}
