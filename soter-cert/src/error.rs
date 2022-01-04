pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("unknown I/O error")]
    Io(#[from] std::io::Error),
    #[error("error generating certificate")]
    CertificateGeneration(#[from] rcgen::RcgenError),
    #[error("error generating server config")]
    TlsConfig(#[from] rustls::Error),
    #[error("invalid certificate")]
    InvalidCertificate,
    #[error("error obtaining certificate from connection data")]
    CertificateData,
}

impl From<x509_parser::error::X509Error> for Error {
    fn from(_: x509_parser::error::X509Error) -> Self {
        Self::InvalidCertificate
    }
}

impl From<x509_parser::nom::Err<x509_parser::error::X509Error>> for Error {
    fn from(_: x509_parser::nom::Err<x509_parser::error::X509Error>) -> Self {
        Self::InvalidCertificate
    }
}
