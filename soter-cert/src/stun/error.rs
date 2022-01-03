#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("UTF8 error")]
    InvalidUtf8(#[from] std::string::FromUtf8Error),
    #[error("invalid header")]
    InvalidHeader,
    #[error("invalid type")]
    InvalidType,
    #[error("invalid magic cookie")]
    InvalidMagicCookie,
    #[error("invalid attribute type")]
    InvalidAttributeType,
    #[error("incorrect attribute type")]
    IncorrectAttributeType,
    #[error("attribute {0} too large")]
    AttributeTooLarge(&'static str),
    #[error("incorrect attribute length")]
    IncorrectAttributeLength,
    #[error("incorrect message length")]
    IncorrectMessageLength,
    #[error("invalid address family")]
    InvalidAddressFamily,
    #[error("invalid address")]
    InvalidAddress,
    #[error("no xor-mapped address in message")]
    NoAddress,
}
